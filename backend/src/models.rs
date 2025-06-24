// backend/src/models.rs
// Migration to use types crate for shared types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Only re-export the types we actually use in handlers
pub use llm_benchmark_types::{
    PerformanceGridRow, ExperimentSummary, 
    CategoryScore, SystemInfo, ExperimentStatus,
    HardwareConfig, PerformanceMetric, QualityScore,
};

// Database-specific types that need sqlx derives
// These represent the actual database schema and need FromRow derives

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HardwareProfileRow {
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
pub struct PerformanceMetricRow {
    pub test_run_id: Uuid,
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QualityScoreRow {
    pub test_run_id: Uuid,
    pub benchmark_name: String,
    pub category: String,
    pub score: f64,
}

// Database query result types for complex joins
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

#[derive(Debug, sqlx::FromRow)]
pub struct PerformanceMetricQueryResult {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct QualityScoreQueryResult {
    pub benchmark_name: String,
    pub category: String,
    pub score: f64,
    pub total_questions: Option<i32>,
    pub correct_answers: Option<i32>,
}

// Conversion functions from database rows to API types
impl From<PerformanceGridQueryResult> for PerformanceGridRow {
    fn from(row: PerformanceGridQueryResult) -> Self {
        let hardware_type = if row.gpu_model.to_lowercase().contains("cpu") {
            "cpu_only".to_string()
        } else {
            "gpu".to_string()
        };

        Self {
            id: row.test_run_id,
            model_name: row.model_name,
            quantization: row.quantization,
            backend: row.backend,
            tokens_per_second: row.tokens_per_second.unwrap_or(0.0),
            memory_gb: row.memory_gb.unwrap_or(0.0),
            gpu_model: row.gpu_model,
            cpu_arch: row.cpu_arch,
            hardware_type,
            overall_score: row.overall_score,
        }
    }
}

impl From<QualityScoreQueryResult> for CategoryScore {
    fn from(row: QualityScoreQueryResult) -> Self {
        Self {
            name: row.category,
            score: row.score,
            total_questions: row.total_questions,
            correct_answers: row.correct_answers,
        }
    }
}

// Conversion from database rows to types crate structs
impl TestRunRow {
    pub fn to_experiment_summary(&self, hardware_summary: String, overall_score: Option<f64>) -> ExperimentSummary {
        let status = match self.status.as_str() {
            "pending" => ExperimentStatus::Pending,
            "running" => ExperimentStatus::Running,
            "completed" => ExperimentStatus::Completed,
            "failed" => ExperimentStatus::Failed,
            "cancelled" => ExperimentStatus::Cancelled,
            _ => ExperimentStatus::Completed,
        };

        ExperimentSummary {
            id: self.id,
            model_name: self.model_name.clone(),
            quantization: self.quantization.clone(),
            backend: self.backend.clone(),
            hardware_summary,
            overall_score,
            timestamp: self.timestamp,
            status,
        }
    }
}

impl HardwareProfileRow {
    pub fn to_hardware_config(&self) -> HardwareConfig {
        HardwareConfig {
            gpu_model: self.gpu_model.clone(),
            gpu_memory_gb: self.gpu_memory_gb,
            cpu_model: self.cpu_model.clone(),
            cpu_arch: self.cpu_arch.clone(),
            ram_gb: self.ram_gb,
            ram_type: self.ram_type.clone(),
            virtualization_type: self.virtualization_type.clone(),
            optimizations: self.optimizations.clone(),
        }
    }

    pub fn to_system_info(&self) -> SystemInfo {
        SystemInfo {
            gpu_model: self.gpu_model.clone(),
            gpu_memory_gb: self.gpu_memory_gb,
            cpu_model: self.cpu_model.clone(),
            cpu_arch: self.cpu_arch.clone(),
            ram_gb: self.ram_gb,
            ram_type: self.ram_type.clone(),
            virtualization_type: self.virtualization_type.clone(),
            optimizations: self.optimizations.clone(),
        }
    }
}

impl PerformanceMetricRow {
    pub fn to_performance_metric(&self) -> PerformanceMetric {
        PerformanceMetric {
            metric_name: self.metric_name.clone(),
            value: self.value,
            unit: self.unit.clone(),
            timestamp: chrono::Utc::now(),
            context: None,
        }
    }
}

impl QualityScoreRow {
    pub fn to_quality_score(&self) -> QualityScore {
        QualityScore {
            benchmark_name: self.benchmark_name.clone(),
            category: self.category.clone(),
            score: self.score,
            total_questions: None,
            correct_answers: None,
            timestamp: chrono::Utc::now(),
            context: None,
        }
    }
}