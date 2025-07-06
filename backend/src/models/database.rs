// models/database.rs
// Database row structs that map directly to database tables

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Test run database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestRunRow {
    pub id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub backend_version: String,
    pub hardware_profile_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
}

/// Hardware profile database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HardwareProfileRow {
    pub id: Uuid,
    pub gpu_model: String,
    pub gpu_memory_gb: i32,
    pub cpu_model: String,
    pub cpu_arch: String,
    pub ram_gb: Option<i32>,
    pub ram_type: Option<String>,
    pub virtualization_type: Option<String>,
    pub optimizations: Vec<String>,
}

/// Performance metric database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceMetricRow {
    pub test_run_id: Uuid,
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
}

