use thiserror::Error;

/// [crate::client::Varro] Error
#[derive(Debug, Error)]
pub enum VarroError {
    /// Missing client
    #[error("missing client")]
    MissingClient,
}

/// A [crate::rollup::RollupNode] Error
#[derive(Debug, Error)]
pub enum RollupNodeError {
    /// Failed to create a new rollup node
    /// from the given rpc url.
    #[error("failed to create a new rollup node from the given rpc url: {0}")]
    RollupNodeInvalidUrl(String),
}

/// A [crate::config::Config] Error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// L1 Client URL is invalid
    #[error("l1 client url is invalid")]
    InvalidL1ClientUrl,
    /// Rollup Node RPC URL is invalid
    #[error("rollup node rpc url is invalid")]
    InvalidRollupNodeRpcUrl,
    /// Failed to create toml file on disk
    #[error("failed to create toml file at {0}")]
    TomlFileCreation(String),
    /// Failed to translate the [crate::config::Config] to a toml object
    #[error("failed to translate the config to a toml object")]
    ConfigTomlConversion,
    /// Failed to write the [crate::config::Config] to the toml file
    #[error("failed to write the config to the toml file")]
    TomlFileWrite,
    /// An invalid output oracle address was provided
    #[error("an invalid output oracle address was provided: {0}")]
    InvalidOutputOracleAddress(String),
    /// Failed to parse the given output private key as a 32 byte hex string.
    #[error("failed to parse the given output private key as a 32 byte hex string: {0}")]
    InvalidOutputPrivateKey(String),
}
