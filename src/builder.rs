use std::time::Duration;

use eyre::Result;
use ethers_core::types::{Address, H256};

use crate::{
    metrics::Metrics, L1Client, rollup::RollupNode, errors::VarroBuilderError,
};

/// VarroBuilder
///
/// A builder for the [Varro] client.
#[derive(Debug, Default)]
pub struct VarroBuilder {
    /// An L1 [L1Client]
    l1_client: Option<L1Client>,
    /// A [RollupNode] client
    rollup_node: Option<RollupNode>,
    /// An output oracle contract
    output_oracle: Option<OutputOracle>,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    allow_non_finalized: Option<bool>,
    /// The proposer
    proposer: Option<Address>,
    /// The proposer's private key used to send the output transactions.
    output_private_key: Option<H256>,
    /// The polling interval
    polling_interval: Option<Duration>,
    /// An _optional_ metrics server
    metrics: Option<Metrics>,
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
    pub fn build(&mut self) -> Result<Varro> {
        let l1_client = self.l1_client.take().ok_or_else(|| VarroBuilderError::MissingL1Client)?;
        let rollup_node = self.rollup_node.take().ok_or_else(|| VarroBuilderError::MissingRollupNode)?;
        let output_oracle = self.output_oracle.take().ok_or_else(|| VarroBuilderError::MissingOutputOracle)?;
        let proposer = self.proposer.take().ok_or_else(|| VarroBuilderError::MissingProposer)?;
        let output_private_key = self.output_private_key.take().ok_or_else(|| VarroBuilderError::MissingOutputPrivateKey)?;
        let polling_interval = self.polling_interval.take().ok_or_else(|| VarroBuilderError::MissingPollingInterval)?;
        let allow_non_finalized = match self.allow_non_finalized {
            Some(allow_non_finalized) => allow_non_finalized,
            None => {
                tracing::warn!(target: "varro=builder", "VarroBuilder was not provided 'allow_non_finalized', defaulting to false");
                false
            }
        };
        let metrics = self.metrics.take();
        Ok(Varro {
            l1_client,
            rollup_node,
            output_oracle,
            allow_non_finalized,
            proposer,
            output_private_key,
            polling_interval,
            metrics
        })
    }
}
