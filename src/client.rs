use std::time::Duration;

use eyre::Result;
use ethers_core::types::{Address, H256};

use crate::{metrics::Metrics, L1Client, rollup::RollupNode};

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
    // TODO: Ok we built the thing, now what?
}

