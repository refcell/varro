use futures::lock::Mutex;
use std::{
    sync::Arc,
    time::Duration,
};

use tokio::{
    task::JoinHandle,
    time,
};
use tokio_stream::{
    wrappers::IntervalStream,
    StreamExt,
};

use ethers_core::types::{
    Address,
    H256,
};
use ethers_providers::Middleware;
use eyre::Result;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver,
    UnboundedSender,
};

use crate::{
    builder::VarroBuilder,
    metrics::Metrics,
    pool::TransactionPool,
    prelude::VarroBuilderError,
    rollup::{
        OutputResponse,
        RollupNode,
        SyncStatus,
    },
    L1Client,
    OutputOracleContract,
    SupportedOutputVersion,
};

/// Varro
///
/// This is the primary varro client, responsible for orchestrating proposal submission.
///
/// The [Varro] client should be constructed using the [crate::builder::VarroBuilder].
/// The builder provides an ergonomic way to construct the [Varro] client, with sensible defaults.
#[derive(Debug)]
pub struct Varro {
    /// An L1 [L1Client]
    l1_client: Arc<Mutex<L1Client>>,
    /// A [RollupNode] client
    rollup_node: Arc<Mutex<RollupNode>>,
    /// An output oracle contract
    output_oracle: Arc<Mutex<OutputOracleContract<L1Client>>>,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    allow_non_finalized: bool,
    /// The proposer
    #[allow(dead_code)]
    proposer: Address,
    /// The proposer's private key used to send the output transactions.
    output_private_key: H256,
    /// The polling interval
    polling_interval: Duration,
    /// An _optional_ metrics server
    metrics: Option<Metrics>,
    /// The proposal sender sends new [OutputResponse].
    proposal_sender: UnboundedSender<OutputResponse>,
    /// The proposal receiver listens for new [OutputResponse].
    proposal_receiver: UnboundedReceiver<OutputResponse>,
    /// The tx_pool sender sends new [OutputResponse].
    tx_pool_sender: UnboundedSender<OutputResponse>,
    /// The tx_pool receiver listens for new [OutputResponse].
    tx_pool_receiver: Option<UnboundedReceiver<OutputResponse>>,
    /// The transaction pool
    transaction_pool: Arc<Mutex<TransactionPool>>,
    /// The transaction pool task handle
    tx_pool_task: Option<JoinHandle<Result<()>>>,
    /// A handle for the metrics server
    metrics_handle: Option<JoinHandle<Result<()>>>,
    /// A list of output roots submitted to the [TransactionPool].
    roots: Vec<Vec<u8>>,
}

impl TryFrom<VarroBuilder> for Varro {
    type Error = VarroBuilderError;

    fn try_from(mut builder: VarroBuilder) -> std::result::Result<Self, Self::Error> {
        let l1_client = builder
            .l1_client
            .take()
            .ok_or(VarroBuilderError::MissingL1Client)?;
        let rollup_node = builder
            .rollup_node
            .take()
            .ok_or(VarroBuilderError::MissingRollupNode)?;
        let output_oracle = builder
            .output_oracle
            .take()
            .ok_or(VarroBuilderError::MissingOutputOracle)?;
        let proposer = builder
            .proposer
            .take()
            .ok_or(VarroBuilderError::MissingProposer)?;
        let output_private_key = builder
            .output_private_key
            .take()
            .ok_or(VarroBuilderError::MissingOutputPrivateKey)?;
        let polling_interval = builder
            .polling_interval
            .take()
            .ok_or(VarroBuilderError::MissingPollingInterval)?;
        let allow_non_finalized = match builder.allow_non_finalized {
            Some(allow_non_finalized) => allow_non_finalized,
            None => {
                tracing::warn!(target: "varro=client", "VarroBuilder was not provided 'allow_non_finalized', defaulting to false");
                false
            }
        };
        let metrics = builder.metrics.take();
        let (proposal_sender, proposal_receiver) = unbounded_channel();
        let (tx_pool_sender, tx_pool_receiver) = unbounded_channel();
        Ok(Varro {
            l1_client: Arc::new(Mutex::new(l1_client)),
            rollup_node: Arc::new(Mutex::new(rollup_node)),
            output_oracle: Arc::new(Mutex::new(output_oracle)),
            allow_non_finalized,
            proposer,
            output_private_key,
            polling_interval,
            metrics,
            proposal_sender,
            proposal_receiver,
            tx_pool_sender,
            tx_pool_receiver: Some(tx_pool_receiver),
            transaction_pool: Arc::new(Mutex::new(TransactionPool::new())),
            tx_pool_task: None,
            metrics_handle: None,
            roots: vec![],
        })
    }
}

impl Varro {
    /// Derives the block number from a [RollupNode]'s [SyncStatus].
    pub fn get_block_number(allow_non_finalized: bool, sync_status: SyncStatus) -> u64 {
        // If we're not allowed to use non-finalized data,
        // then we can only use the finalized l2 block number
        if !allow_non_finalized {
            return sync_status.finalized_l2
        }

        // If "safe" blocks are allowed, use the l2 safe head
        sync_status.safe_l2
    }

    /// Spawns the transaction pool task
    async fn start_transaction_pool(&mut self) {
        let output_private_key = self.output_private_key;
        let l1_client = Arc::clone(&self.l1_client);
        let tx_pool_receiver = self.tx_pool_receiver.take();
        let tx_pool = Arc::clone(&self.transaction_pool);
        let tx_pool_handle = tokio::spawn(async move {
            tx_pool
                .lock()
                .await
                .start(l1_client, output_private_key, tx_pool_receiver)
                .await
        });
        self.tx_pool_task = Some(tx_pool_handle);
    }

    /// Shuts down associated tasks
    pub async fn shutdown(&mut self) -> Result<()> {
        // Shutdown the transaction pool
        if let Some(tx_pool_task) = &self.tx_pool_task {
            tx_pool_task.abort()
        }

        // Shutdown the metrics server
        if let Some(metrics_handle) = self.metrics_handle.take() {
            metrics_handle.abort()
        }

        Ok(())
    }

    /// Starts the [Varro] client.
    pub async fn start(&mut self) -> Result<()> {
        // Get the L1 Chain ID
        let l1_chain_id = self.l1_client.lock().await.get_chainid().await?;

        // Spawn the metrics server
        if let Some(mut metrics) = self.metrics.take() {
            let metrics_handle = tokio::task::spawn(async move {
                tracing::info!(target: "varro=client", "starting metrics server...");
                metrics.serve().await
            });
            self.metrics_handle = Some(metrics_handle);
        }

        // Spawn the transaction pool task
        self.start_transaction_pool().await;

        // Interval Stream every polling interval
        let interval = self.polling_interval;
        let allow_non_finalized = self.allow_non_finalized;
        let rollup_node = Arc::clone(&self.rollup_node);
        let output_oracle = Arc::clone(&self.output_oracle);
        let proposal_sender = self.proposal_sender.clone();
        let _proposing_spawn_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            tracing::info!(target: "varro=client", "starting proposer with polling interval: {:?}", interval);
            let mut stream = IntervalStream::new(time::interval(interval));
            while let Some(ts) = stream.next().await {
                tracing::debug!(target: "varro=client", "polling interval triggered at {:?}", ts);
                tracing::debug!(target: "varr=client", "spawning proposal task in new thread for chain ID: {}", l1_chain_id.as_u64());
                let _handle = Varro::spawn_proposal_task(
                    // l1_chain_id.as_u64(),
                    allow_non_finalized,
                    Arc::clone(&rollup_node),
                    Arc::clone(&output_oracle),
                    proposal_sender.clone(),
                );
            }
            Ok(())
        });

        // Listen for new proposals and submit them to the transaction pool
        // This will block until the proposal receiver channel is closed
        self.listen_for_proposals().await?;
        Ok(())
    }

    /// Listens for new proposals and submits them to the transaction pool.
    pub async fn listen_for_proposals(&mut self) -> Result<()> {
        while let Some(proposal) = self.proposal_receiver.recv().await {
            tracing::debug!(target: "varro=client", "received proposal: {:?}", proposal);
            self.submit_proposal(proposal).await?;
        }
        tracing::warn!(target: "varro=client", "proposal receiver channel closed, varro client shutting down...");
        Ok(())
    }

    /// Submits an [OutputResponse] to the [TransactionPool].
    pub async fn submit_proposal(&mut self, proposal: OutputResponse) -> Result<()> {
        tracing::info!(target: "varro=client", "received proposal: {:?}", proposal);
        tracing::info!(target: "varro=client", "submitting to transaction pool...");

        // Check if the proposal is already being sent
        if self.roots.contains(&proposal.output_root) {
            tracing::warn!(target: "varro=client", "proposal already being sent, skipping...");
            return Ok(());
        }

        // Track that we already submitted this proposal
        self.roots.push(proposal.output_root.clone());

        // Send proposal to transaction pool with backoff retries
        // This is to gracefully handle when the transaction pool is full
        while self.tx_pool_sender.send(proposal.clone()).is_err() {
            tracing::warn!(target: "varro=client", "transaction pool is full, retrying...");
            // TODO: make this backoff time configurable
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    /// Spawns a new proposal task
    pub fn spawn_proposal_task(
        allow_non_finalized: bool,
        rollup_node: Arc<Mutex<RollupNode>>,
        output_oracle: Arc<Mutex<OutputOracleContract<L1Client>>>,
        proposal_sender: UnboundedSender<OutputResponse>,
    ) -> JoinHandle<Result<()>> {
        // Spawn a new task to create the next proposal
        tokio::task::spawn(async move {
            // Get the next block number to use from the output oracle contract
            let next_block_number = {
                let locked_output_oracle = Arc::clone(&output_oracle).lock().await;
                let pinned = locked_output_oracle.to_owned().next_block_number();
                pinned.await?
            };

            // Get the rollup node's sync status
            let sync_status = {
                let locked_rollup_node = rollup_node.lock().await;
                locked_rollup_node.sync_status().await?
            };

            // Figure out which block number to use
            let block_number = Varro::get_block_number(allow_non_finalized, sync_status);

            // We should not be submitting a block in the future
            if block_number < next_block_number.as_u64() {
                tracing::info!(target: "varro=client", "proposer submission interval has not elapsed, current block number: {}, next block number: {}", block_number, next_block_number);
                return Ok(())
            }

            // Get the rollup node output at the given block number
            let output = {
                let locked_rollup_node = rollup_node.lock().await;
                locked_rollup_node
                    .output_at_block(next_block_number.as_u64())
                    .await?
            };

            // Validate the output version
            if SupportedOutputVersion::V1.equals(&output.version) {
                tracing::warn!(target: "varro=client", "output version is not not supported, skipping proposal on L2 block {}", output.block_ref.number);
                return Ok(())
            }

            // Validate that the block number is correct
            if output.block_ref.number != next_block_number.as_u64() {
                tracing::warn!(target: "varro=client", "output block number does not match expected block number, skipping proposal on L2 block {}", output.block_ref.number);
                return Ok(())
            }

            // Only allow proposals for finalized blocks and safe, if allowed
            let is_finalized = output.block_ref.number <= output.sync_status.finalized_l2;
            let is_safe_and_accepted = allow_non_finalized
                && output.block_ref.number <= output.sync_status.safe_l2;
            if !is_finalized && !is_safe_and_accepted {
                tracing::warn!(target: "varro=client", "output block is not finalized or safe, skipping proposal on L2 block {}", output.block_ref.number);
                return Ok(())
            }

            // Reaching this point means we can bubble up the valid proposal through the channel
            Ok(proposal_sender.send(output)?)
        })
    }
}
