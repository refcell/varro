
//! Common Types and Utilies for Varro

use eyre::Result;
// use ethers_core::types::Address;
pub use ethers_providers::{Provider, Http, Middleware};

// TODO: Can we just use the Middleware trait from ethers?

/// An L1Client is an [Http] [Provider] for Ethereum Mainnet.
pub type L1Client = Provider<Http>;

// TODO: properly construct an OutputOracle Contract
/// An OutputOracle is a contract that provides the next block number to use for the next proposal.
#[derive(Debug, Clone)]
pub struct OutputOracle {}

impl OutputOracle {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the next block number to use for the next proposal.
    pub async fn get_next_block_number(&self) -> Result<u64> {
        Ok(0)
    }
}