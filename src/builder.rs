use std::time::Duration;

use ethers_core::types::{
    Address,
    H256,
};
use eyre::Result;

use crate::{
    client::Varro,
    config::Config,
    metrics::Metrics,
    rollup::RollupNode,
    L1Client,
    OutputOracle,
};

/// VarroBuilder
///
/// A builder for the [Varro] client.
#[derive(Debug, Default, Clone)]
pub struct VarroBuilder {
    /// An L1 [L1Client]
    pub l1_client: Option<L1Client>,
    /// A [RollupNode] client
    pub rollup_node: Option<RollupNode>,
    /// An output oracle contract
    pub output_oracle: Option<OutputOracle>,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    pub allow_non_finalized: Option<bool>,
    /// The proposer
    pub proposer: Option<Address>,
    /// The proposer's private key used to send the output transactions.
    pub output_private_key: Option<H256>,
    /// The polling interval
    pub polling_interval: Option<Duration>,
    /// An _optional_ metrics server
    pub metrics: Option<Metrics>,
}

impl TryFrom<Config> for VarroBuilder {
    type Error = eyre::Report;

    fn try_from(conf: Config) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            l1_client: Some(conf.get_l1_client()?),
            rollup_node: Some(conf.get_rollup_node_client()?),
            output_oracle: Some(conf.get_output_oracle()?),
            allow_non_finalized: Some(conf.allow_non_finalized),
            proposer: Some(conf.output_oracle_address),
            output_private_key: Some(conf.get_output_private_key()?),
            polling_interval: Some(conf.polling_interval),
            metrics: None,
        })
    }
}

impl VarroBuilder {
    /// Creates a new [VarroBuilder].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the metrics server for the [Varro] client.
    pub fn with_metrics(&mut self, metrics: Metrics) -> &mut Self {
        self.metrics = Some(metrics);
        self
    }

    /// Sets the L1 client for the [Varro] client.
    pub fn with_l1_client(&mut self, l1_client: L1Client) -> &mut Self {
        self.l1_client = Some(l1_client);
        self
    }

    /// Sets the rollup node for the [Varro] client.
    pub fn with_rollup_node(&mut self, rollup_node: RollupNode) -> &mut Self {
        self.rollup_node = Some(rollup_node);
        self
    }

    /// Sets the output oracle for the [Varro] client.
    pub fn with_output_oracle(&mut self, output_oracle: OutputOracle) -> &mut Self {
        self.output_oracle = Some(output_oracle);
        self
    }

    /// Sets whether to use non-finalized L1 data to propose L2 blocks.
    pub fn with_allow_non_finalized(&mut self, allow_non_finalized: bool) -> &mut Self {
        self.allow_non_finalized = Some(allow_non_finalized);
        self
    }

    /// Sets the proposer for the [Varro] client.
    pub fn with_proposer(&mut self, proposer: Address) -> &mut Self {
        self.proposer = Some(proposer);
        self
    }

    /// Sets the proposer's private key used to send the output transactions.
    /// for the [Varro] client.
    pub fn with_output_private_key(&mut self, output_private_key: H256) -> &mut Self {
        self.output_private_key = Some(output_private_key);
        self
    }

    /// Sets the polling interval for the [Varro] client.
    pub fn with_polling_interval(&mut self, polling_interval: Duration) -> &mut Self {
        self.polling_interval = Some(polling_interval);
        self
    }

    /// Builds the [Varro] client.
    pub fn build(self) -> Result<Varro> {
        Ok(Varro::try_from(self)?)
    }
}
