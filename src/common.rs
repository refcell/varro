#![allow(missing_docs)]
//! Common Types and Utilies for Varro

use ethers_contract::abigen;
use eyre::Result;

pub use ethers_providers::{
    Http,
    Middleware,
    Provider,
};

abigen!(OutputOracleContract, "contracts/L2OutputOracle.json");

pub use OutputOracleContract;

/// An enum of supported output versions for the [crate::rollup::OutputResponse] version.
#[derive(Debug, Clone)]
pub enum SupportedOutputVersion {
    /// Output version 1
    V1,
}

impl SupportedOutputVersion {
    /// Checks that the given output version is equal to the provided bytes version.
    pub fn equals(&self, other: &[u8]) -> bool {
        match (self, other.is_empty()) {
            // V1 is the only supported version for now - an empty Vec<u8>
            (Self::V1, true) => true,
            _ => false,
        }
    }
}

// TODO: Can we just use the Middleware trait from ethers?
/// An L1Client is an [Http] [Provider] for Ethereum Mainnet.
pub type L1Client = Provider<Http>;

// TODO: properly construct an OutputOracle Contract
/// An OutputOracle is a contract that provides the next block number to use for the next proposal.
#[derive(Debug, Default, Clone)]
pub struct OutputOracle {}

impl OutputOracle {
    /// Get the next block number to use for the next proposal.
    pub async fn get_next_block_number(&self) -> Result<u64> {
        Ok(0)
    }
}
