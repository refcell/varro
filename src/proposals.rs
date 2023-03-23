
use std::time::Duration;

use eyre::Result;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::IntervalStream;

/// ProposalManager
///
/// Queries for [OutputResponse] proposals on a periodic basis,
/// sending them back to a receiver channel.
#[derive(Debug)]
pub struct ProposalManager {
    /// The interval at which to query for new proposals.
    polling_interval: Duration,
    /// 
}

impl ProposalManager {
    pub fn new() -> Self {
        Self {
            polling_interval: Duration::from_secs(5),
        }
    }

    pub async fn start(&self) -> JoinHandle<Result<()>> {
        // TODO: pull variables out of state
        let interval = self.polling_interval;

        tokio::spawn(async move {
            tracing::info!(target: "varro=proposals", "starting proposer with polling interval: {:?}", interval);
            let mut stream = IntervalStream::new(tokio::time::interval(interval));
            while let Some(ts) = stream.next().await {
                tracing::debug!(target: "varro=proposals", "polling interval triggered at {:?}", ts);
                tracing::debug!(target: "varr=proposals", "spawning proposal task in new thread for chain ID: {}", l1_chain_id.as_u64());
                let _handle = ProposalManager::spawn_proposal_task(
                    // l1_chain_id.as_u64(),
                    allow_non_finalized,
                    Arc::clone(&rollup_node),
                    l1_client,
                    output_oracle_address,
                    proposal_sender.clone(),
                );
            }
            Ok(())
        })
    }

    /// Spawns a new proposal task
    pub fn spawn_proposal_task(
        allow_non_finalized: bool,
        rollup_node: Arc<Mutex<RollupNode>>,
        l1_client: Arc<Mutex<L1Client>>,
        output_oracle_address: Address,
        proposal_sender: UnboundedSender<OutputResponse>,
    ) -> JoinHandle<Result<()>> {
        // Spawn a new task to create the next proposal
        let l1_client = l1_client.clone();
        tokio::task::spawn(async move {
            let next_block_number = {
                let locked = l1_client.lock().await.to_owned().clone();
                let output_oracle: OutputOracleContract<L1Client> = OutputOracleContract::new(
                    output_oracle_address,
                    locked.into(),
                );
                // Get the next block number to use from the output oracle contract
                let next_block_number: U256 = output_oracle.next_block_number().await?.clone();
                next_block_number
            };
            // let locked_output_oracle = cloned_output_oracle.lock().map_err(|_| eyre::eyre!("Failed to lock output oracle"))?;
            // let pinned: OutputOracleContract<L1Client> = locked_output_oracle.to_owned();
            // pinned
            // let next_block_number = {
            // };

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