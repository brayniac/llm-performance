// models/conversions.rs
// Conversion implementations between database types and API types

use llm_benchmark_types::{
    PerformanceGridRow, CategoryScore, ExperimentSummary, ExperimentStatus,
    HardwareConfig, SystemInfo, PerformanceMetric, QualityScore
};

use super::{
    database::{TestRunRow, HardwareProfileRow, PerformanceMetricRow, QualityScoreRow},
    query_results::{PerformanceGridQueryResult, QualityScoreQueryResult}
};

// Conversion from query results to API types
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

// Conversion methods for database rows
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