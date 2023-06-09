#![doc=include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

mod common;
pub use common::*;

pub mod telemetry;
pub use telemetry::*;

/// The core [Varro] client
pub mod client;

/// A Builder for the [Varro] client
pub mod builder;

/// Pool contains the logic to manage proposal transactions
pub mod pool;

/// Proposals holds the [ProposalManager] which
/// listens for new [OutputResponse] proposals
pub mod proposals;

/// Configuration
pub mod config;

/// CLI parsing
pub mod cli;

/// The Rollup Node
pub mod rollup;

/// Common Errors
pub mod errors;

/// The metrics server
pub mod metrics;

/// Common internal macros
pub(crate) mod macros;

/// Re-export Archon Types
pub mod prelude {
    pub use crate::{
        builder::*,
        client::*,
        common::*,
        config::*,
        errors::*,
        metrics::*,
        telemetry::*,
    };
}
