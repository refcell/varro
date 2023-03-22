#![doc=include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

/// Telemetry
pub mod telemetry;

/// The core client
pub mod client;

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
        config::*,
        errors::*,
        telemetry::*,
        metrics::*,
        client::*,
    };
}
