// models/query_results.rs
// Complex query result types for database joins and aggregations

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Result type for performance grid queries with joined data
#[derive(Debug, sqlx::FromRow)]
pub struct PerformanceGridQueryResult {
    pub test_run_id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub gpu_model: String,
    pub cpu_arch: String,
    pub virtualization_type: Option<String>,
    pub tokens_per_second: Option<f64>,
    pub memory_gb: Option<f64>,
    pub overall_score: Option<f64>,
}

/// Result type for configuration data queries
#[derive(Debug, sqlx::FromRow)]
pub struct ConfigDataQueryResult {
    pub test_run_id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub backend_version: String,
    pub gpu_model: String,
    pub cpu_arch: String,
    pub timestamp: DateTime<Utc>,
}

/// Result type for performance metric queries
#[derive(Debug, sqlx::FromRow)]
pub struct PerformanceMetricQueryResult {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
}

/// Result type for quality score queries with optional question counts
#[derive(Debug, sqlx::FromRow)]
pub struct QualityScoreQueryResult {
    pub benchmark_name: String,
    pub category: String,
    pub score: f64,
    pub total_questions: Option<i32>,
    pub correct_answers: Option<i32>,
}