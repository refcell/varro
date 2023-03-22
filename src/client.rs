use std::time::Duration;

use eyre::Result;
use ethers_core::types::{Address, H256};
use tokio::task::JoinHandle;

use crate::{metrics::Metrics, L1Client, rollup::{RollupNode, SyncStatus}};

/// Varro
///
/// This is the primary varro client, responsible for orchestrating proposal submission.
///
/// The [Varro] client should be constructed using the [crate::builder::VarroBuilder].
/// The builder provides an ergonomic way to construct the [Varro] client, with sensible defaults.
#[derive(Debug)]
pub struct Varro {
    /// An L1 [L1Client]
    l1_client: L1Client,
    /// A [RollupNode] client
    rollup_node: RollupNode,
    /// An output oracle contract
    output_oracle: OutputOracle,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    allow_non_finalized: bool,
    /// The proposer
    proposer: Address,
    /// The proposer's private key used to send the output transactions.
    output_private_key: H256,
    /// The polling interval
    polling_interval: Duration,
    /// An _optional_ metrics server
    metrics: Option<Metrics>,
}

impl Varro {
    /// Creates a new [Varro] client instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Derives the block number from a [RollupNode]'s [SyncStatus].
    pub fn get_block_number(&self, sync_status: SyncStatus) -> u64 {
        // If we're not allowed to use non-finalized data,
        // then we can only use the finalized l2 block number
        if !self.allow_non_finalized {
            return sync_status.finalized_l2;
        }

        // If "safe" blocks are allowed, use the l2 safe head
        sync_status.safe_l2
    }

    // TODO: spawn the "run" loop in a separate task
    // TODO: each `poll_interval` we should spawn a new task to create the next proposal.
    // TODO: these tasks feed back the proposals to the `Varro` client
    // TODO: then Varro checks to make sure the proposal is valid,
    // TODO: constructs an output proposal transaction (using a transaction manager or some other abstraction)
    // TODO: and dispatches it to the transaction pool


    /// Run the [Varro] client.
    pub async fn run(&self) -> Result<()> {
        // Get the L1 Chain ID
        let l1_chain_id = self.l1_client.get_chain_id().await?;

        // The main driver loop
        loop {
            // Get the next block number to use from the output oracle contract
            let next_block_number = self.output_oracle.get_next_block_number().await?;

            // Get the rollup node's sync status
            let sync_status = self.rollup_node.get_sync_status().await?;

            // Figure out which block number to use
            let block_number = self.get_block_number(sync_status);

            // We should not be submitting a block in the future
            if block_number < next_block_number {
                tracing::info!(target: "varro=client", "proposer submission interval has not elapsed", current_block_number = block_number, next_block_number = next_block_number);
                continue;
            }

            // Get the rollup node output at the given block number
            let output = self.rollup_node.get_output(next_block_number).await?;

            // Validate the output
            // if 
        }

        Ok(())
    }

    pub async fn construct_proposal() -> Result<JoinHandle<_>> {
        tokio::task::spawn(async || {
            
        })
    }

}

