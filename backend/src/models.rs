use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HardwareProfile {
    pub id: Uuid,
    pub gpu_model: String,
    pub gpu_memory_gb: i32,
    pub cpu_model: String,
    pub cpu_arch: String,
    pub ram_gb: i32,
    pub ram_type: String,
    pub virtualization_type: Option<String>,
    pub optimizations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestRun {
    pub id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub backend_version: String,
    pub hardware_profile_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceMetric {
    pub test_run_id: Uuid,
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QualityScore {
    pub test_run_id: Uuid,
    pub benchmark_name: String,
    pub category: String,
    pub score: f64,
}

// API Response Types
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceGridRow {
    pub id: String,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub tokens_per_second: f64,
    pub memory_gb: f64,
    pub gpu_model: String,
    pub cpu_arch: String,
    pub hardware_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonRequest {
    pub config_a: String,
    pub config_b: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonData {
    pub config_a: ConfigSummary,
    pub config_b: ConfigSummary,
    pub categories: Vec<CategoryComparison>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub name: String,
    pub model: String,
    pub quantization: String,
    pub backend: String,
    pub hardware: String,
    pub overall_score: f64,
    pub performance: PerformanceSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub speed: f64,
    pub memory: f64,
    pub loading_time: f64,
    pub prompt_speed: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryComparison {
    pub name: String,
    pub score_a: f64,
    pub score_b: f64,
}

// New types for detail view
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailData {
    pub config: ConfigDetail,
    pub categories: Vec<CategoryScore>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDetail {
    pub name: String,
    pub model: String,
    pub quantization: String,
    pub backend: String,
    pub backend_version: String,
    pub overall_score: f64,
    pub performance: PerformanceSummary,
    pub test_run_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryScore {
    pub name: String,
    pub score: f64,
    pub total_questions: Option<i32>,
    pub correct_answers: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub gpu_model: String,
    pub gpu_memory_gb: i32,
    pub cpu_model: String,
    pub cpu_arch: String,
    pub ram_gb: i32,
    pub ram_type: String,
    pub virtualization_type: Option<String>,
    pub optimizations: Vec<String>,
}

// Database query result types
#[derive(Debug, sqlx::FromRow)]
pub struct PerformanceGridQueryResult {
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub gpu_model: String,
    pub cpu_arch: String,
    pub virtualization_type: Option<String>,
    pub tokens_per_second: Option<f64>,
    pub memory_gb: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ConfigDataQueryResult {
    pub test_run_id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub gpu_model: String,
    pub cpu_arch: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PerformanceMetricQueryResult {
    pub metric_name: String,
    pub value: f64,
}