use std::{
    sync::Arc,
    time::Duration,
};

use eyre::Result;
use futures::lock::Mutex;
use tokio::task::JoinHandle;

use ethers_core::types::{
    Address,
    H256,
};

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
    proposals::ProposalManager,
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
    /// The output oracle contract address
    output_oracle_address: Address,
    /// An output oracle contract
    output_oracle: OutputOracleContract<L1Client>,
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
    /// A handle for the spawned [ProposalManager] task
    proposal_manager_handle: Option<JoinHandle<Result<()>>>,
    /// A list of output roots submitted to the [TransactionPool].
    roots: Vec<Vec<u8>>,
    /// A backoff time for sending transaction to the [TransactionPool]
    tx_backoff: Duration,
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
        let tx_backoff = builder.tx_backoff.take().ok_or(VarroBuilderError::MissingTxBackoff)?;
        let output_oracle_address = output_oracle.address();
        let metrics = builder.metrics.take();

        let (proposal_sender, proposal_receiver) = unbounded_channel();
        let (tx_pool_sender, tx_pool_receiver) = unbounded_channel();

        Ok(Varro {
            l1_client: Arc::new(Mutex::new(l1_client)),
            rollup_node: Arc::new(Mutex::new(rollup_node)),
            output_oracle_address,
            output_oracle,
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
            proposal_manager_handle: None,
            roots: vec![],
            tx_backoff
        })
    }
}

impl Varro {
    /// Starts the [Varro] client.
    pub async fn start(&mut self) {
        // Get the L1 Chain ID
        // let l1_chain_id = self.l1_client.lock().await.get_chainid().await?;

        // Start a metrics server
        self.start_metrics_server().await;

        // Spawn the transaction pool task
        self.start_transaction_pool().await;

        // Start the proposal manager to listen for new proposals
        self.start_proposal_manager().await;

        // Listen for new proposals and submit them to the transaction pool
        // This will block until the proposal receiver channel is closed
        self.listen_for_proposals().await;
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

    /// Shuts down associated tasks.
    pub async fn shutdown(&mut self) {
        // Shutdown the transaction pool
        if let Some(tx_pool_task) = &self.tx_pool_task {
            tx_pool_task.abort()
        }

        // Shutdown the metrics server
        if let Some(metrics_handle) = self.metrics_handle.take() {
            metrics_handle.abort()
        }
    }

    /// Starts the [ProposalManager] task
    async fn start_proposal_manager(&mut self) {
        let l1_client = Arc::clone(&self.l1_client);
        let rollup_node = Arc::clone(&self.rollup_node);
        let output_oracle_address = self.output_oracle_address;
        let allow_non_finalized = self.allow_non_finalized;
        let polling_interval = self.polling_interval;
        let proposal_sender = self.proposal_sender.clone();
        let proposer = self.proposer.clone();
        let proposal_manager_handle = tokio::spawn(async move {
            ProposalManager::start(
                l1_client,
                rollup_node,
                output_oracle_address,
                allow_non_finalized,
                polling_interval,
                proposal_sender,
                proposer,
            )
            .await
        });
        self.proposal_manager_handle = Some(proposal_manager_handle);
    }

    /// Starts the [Metrics] server
    async fn start_metrics_server(&mut self) {
        // Spawn the metrics server
        if let Some(mut metrics) = self.metrics.take() {
            let metrics_handle = tokio::task::spawn(async move {
                tracing::info!(target: "varro=client", "starting metrics server...");
                metrics.serve().await
            });
            self.metrics_handle = Some(metrics_handle);
        }
    }

    /// Listens for new proposals and submits them to the transaction pool.
    pub async fn listen_for_proposals(&mut self) {
        while let Some(proposal) = self.proposal_receiver.recv().await {
            tracing::debug!(target: "varro=client", "received proposal: {:?}", proposal);
            self.submit_proposal(proposal).await;
        }
        tracing::warn!(target: "varro=client", "proposal receiver channel closed, varro client shutting down...");
    }

    /// Submits an [OutputResponse] to the [TransactionPool].
    pub async fn submit_proposal(&mut self, proposal: OutputResponse) {
        tracing::info!(target: "varro=client", "received proposal: {:?}", proposal);
        tracing::info!(target: "varro=client", "submitting to transaction pool...");

        // Check if the proposal is already being sent
        if self.roots.contains(&proposal.output_root) {
            tracing::warn!(target: "varro=client", "proposal already being sent, skipping...");
            return;
        }

        // Track that we already submitted this proposal
        self.roots.push(proposal.output_root.clone());

        // Send proposal to transaction pool with backoff retries
        // This is to gracefully handle when the transaction pool is full
        while self.tx_pool_sender.send(proposal.clone()).is_err() {
            tracing::warn!(target: "varro=client", "transaction pool is full, retrying...");
            tokio::time::sleep(self.tx_backoff).await;
        }
    }
}
