// handlers/mod.rs
// Module declarations for split handler modules

pub mod performance;
pub mod comparison;
pub mod configuration;
pub mod experiment;
pub mod grouped_performance;
pub mod delete;
// pub mod list_test_runs; // Disabled until migration is run
// pub mod benchmark_upload; // Disabled until migration is run
pub mod benchmark_upload_raw;
// pub mod performance_v2; // Disabled until migration is run

// Re-export public handler functions for use in main.rs
pub use performance::get_performance_grid;
pub use comparison::get_comparison;
pub use configuration::{get_configurations, get_detail};
pub use experiment::upload_experiment;
pub use grouped_performance::get_grouped_performance;
pub use delete::{delete_test_run, delete_by_model_quant, delete_benchmark_scores};
// pub use list_test_runs::list_test_runs; // Disabled until migration is run
// pub use benchmark_upload::upload_benchmarks; // Disabled until migration is run
pub use benchmark_upload_raw::upload_benchmarks_raw;
// pub use performance_v2::get_performance_grid_v2; // Disabled until migration is run