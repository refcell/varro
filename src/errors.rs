use thiserror::Error;

/// [crate::client::Varro] Error
#[derive(Debug, Error)]
pub enum VarroError {
    /// Missing client
    #[error("missing client")]
    MissingClient,
}

/// [crate::config::Config] Error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// L1 Client URL is invalid
    #[error("l1 client url is invalid")]
    InvalidL1ClientUrl,
    /// L2 Client URL is invalid
    #[error("l2 client url is invalid")]
    InvalidL2ClientUrl,
}
