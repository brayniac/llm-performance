// llm-benchmark-types/src/lib.rs

//! Shared types for LLM benchmark data structures
//!
//! This crate provides common data types used across the LLM benchmark
//! ecosystem, including experiment results, hardware configurations,
//! and API request/response types.

pub mod api;
pub mod experiment;
pub mod hardware;
pub mod metrics;
pub mod validation;

// Re-export commonly used types
pub use api::*;
pub use experiment::*;
pub use hardware::*;
pub use metrics::*;
pub use validation::*;

/// Result type for validation errors
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Common validation error type
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid field '{field}': {message}")]
    InvalidField { field: String, message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Value out of range for '{field}': {value} (expected {range})")]
    OutOfRange {
        field: String,
        value: String,
        range: String,
    },
}
