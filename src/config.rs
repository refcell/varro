use std::{
    str::FromStr,
    time::Duration,
};

use ethers_core::types::{
    Address,
    H256,
};
use ethers_providers::{
    Http,
    Provider,
};
use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    errors::ConfigError,
    extract_env,
    rollup::RollupNode,
    OutputOracleContract,
};

/// A system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The L1 client rpc url.
    pub l1_client_rpc_url: String,
    /// The L2 client rollup node rpc url.
    /// Note: This is not explicitly the same as an "L2 client rpc url".
    /// The Rollup Node may expose a [RollupRPC] interface on a different port.
    /// This is the url that should be used for this field.
    pub rollup_node_rpc_url: String,
    /// The L2OutputOracle contract address.
    pub output_oracle_address: Address,
    /// The delay between querying L2 for more blocks and creating a new batch.
    pub polling_interval: Duration,
    /// The number of confirmations which to wait after appending new batches.
    pub num_confirmation: u64,
    /// The number of [VarroError::NonceTooLow] errors required to give up on a
    /// tx at a particular nonce without receiving confirmation.
    pub safe_abort_nonce_too_low: u64,
    /// The time to wait before resubmitting a transaction.
    pub resubmission_timeout: Duration,
    /// The HD seed used to derive the wallet private keys for both
    /// the sequencer and proposer. Must be used in conjunction with
    /// SequencerHDPath and ProposerHDPath.
    ///
    /// If not provided, a new mnemonic will be generated.
    pub mnemonic: String,
    /// The private key used for the L2 Output transactions.
    pub output_private_key: String,
    /// The HD path used to derive the wallet private key for the sequencer.
    pub output_hd_path: String,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    pub allow_non_finalized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            l1_client_rpc_url: extract_env!("L1_CLIENT_RPC_URL"),
            rollup_node_rpc_url: extract_env!("ROLLUP_NODE_RPC_URL"),
            output_oracle_address: Address::from_str(&extract_env!(
                "OUTPUT_ORACLE_ADDRESS"
            ))
            .expect("invalid output oracle address"),
            polling_interval: Duration::from_secs(5),
            num_confirmation: 1,
            safe_abort_nonce_too_low: 3,
            resubmission_timeout: Duration::from_secs(60),
            mnemonic: extract_env!("MNEMONIC"),
            output_private_key: extract_env!("OUTPUT_PRIVATE_KEY"),
            output_hd_path: String::from("m/44'/60'/0'/0/0"),
            allow_non_finalized: false,
        }
    }
}

impl Config {
    /// Parses the output private key string into a 32-byte hash
    pub fn get_output_private_key(&self) -> Result<H256> {
        Ok(H256::from_str(&self.output_private_key).map_err(|_| {
            ConfigError::InvalidOutputPrivateKey(self.output_private_key.clone())
        })?)
    }

    /// Returns the output oracle address
    pub fn get_output_oracle_address(&self) -> Address {
        self.output_oracle_address
    }

    /// Constructs an L1 [Provider<Http>] using the configured L1 client rpc url.
    pub fn get_l1_client(&self) -> Result<Provider<Http>> {
        Ok(Provider::<Http>::try_from(&self.l1_client_rpc_url)
            .map_err(|_| ConfigError::InvalidL1ClientUrl)?)
    }

    /// Constructs a Rollup Node [RollupNode] using the configured rollup node rpc url.
    pub fn get_rollup_node_client(&self) -> Result<RollupNode> {
        Ok(RollupNode::try_from(self.rollup_node_rpc_url.clone())
            .map_err(|_| ConfigError::InvalidRollupNodeRpcUrl)?)
    }

    /// Get the output oracle contract
    pub fn get_output_oracle_contract(&self) -> Result<OutputOracleContract<Provider<Http>>> {
        let l1_client = self.get_l1_client()?;
        let output_oracle = OutputOracleContract::new(
            self.get_output_oracle_address(),
            l1_client.into(),
        );
        Ok(output_oracle)
    }
}
