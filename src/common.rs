
//! Common Types and Utilies for Varro

use ethers_providers::{Provider, Http};

// TODO: Can we just use the Middleware trait from ethers?

/// An L1Client is an [Http] [Provider] for Ethereum Mainnet.
pub type L1Client = Provider<Http>;

