// handlers/mod.rs
// Module declarations for split handler modules

pub mod performance;
pub mod comparison;
pub mod configuration;
pub mod experiment;

// Re-export public handler functions for use in main.rs
pub use performance::get_performance_grid;
pub use comparison::get_comparison;
pub use configuration::{get_configurations, get_detail};
pub use experiment::upload_experiment;