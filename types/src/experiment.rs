// llm-benchmark-types/src/experiment.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{HardwareConfig, PerformanceMetric, QualityScore};

/// A complete experiment run containing all benchmark data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExperimentRun {
    /// Name of the model being tested (e.g., "Mistral Small 3.2 24B")
    pub model_name: String,

    /// Quantization scheme (e.g., "Q8_0", "Q4_0", "FP16")
    pub quantization: String,

    /// Backend used for inference (e.g., "llama.cpp", "vllm")
    pub backend: String,

    /// Version of the backend
    pub backend_version: String,

    /// Hardware configuration used for the test
    pub hardware_config: HardwareConfig,

    /// Performance metrics collected during the run
    pub performance_metrics: Vec<PerformanceMetric>,

    /// Quality scores from various benchmarks
    pub quality_scores: Vec<QualityScore>,

    /// When the experiment was conducted
    pub timestamp: DateTime<Utc>,

    /// Optional notes about the experiment
    pub notes: Option<String>,

    /// Status of the experiment run
    #[serde(default = "default_status")]
    pub status: ExperimentStatus,
}

/// Status of an experiment run
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

fn default_status() -> ExperimentStatus {
    ExperimentStatus::Completed
}

/// Metadata about an experiment run (without full data)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExperimentSummary {
    pub id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub hardware_summary: String, // e.g., "RTX 4090 / Zen2"
    pub overall_score: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub status: ExperimentStatus,
}

impl ExperimentRun {
    /// Create a new experiment run with the current timestamp
    pub fn new(
        model_name: String,
        quantization: String,
        backend: String,
        backend_version: String,
        hardware_config: HardwareConfig,
    ) -> Self {
        Self {
            model_name,
            quantization,
            backend,
            backend_version,
            hardware_config,
            performance_metrics: Vec::new(),
            quality_scores: Vec::new(),
            timestamp: Utc::now(),
            notes: None,
            status: ExperimentStatus::Pending,
        }
    }

    /// Add a performance metric to the experiment
    pub fn add_metric(&mut self, metric: PerformanceMetric) {
        self.performance_metrics.push(metric);
    }

    /// Add a quality score to the experiment
    pub fn add_quality_score(&mut self, score: QualityScore) {
        self.quality_scores.push(score);
    }

    /// Get the overall MMLU-Pro score if available
    pub fn mmlu_pro_score(&self) -> Option<f64> {
        let mmlu_scores: Vec<_> = self
            .quality_scores
            .iter()
            .filter(|s| s.benchmark_name == "mmlu_pro")
            .collect();

        if mmlu_scores.is_empty() {
            return None;
        }

        let sum: f64 = mmlu_scores.iter().map(|s| s.score).sum();
        Some(sum / mmlu_scores.len() as f64)
    }

    /// Get a specific performance metric by name
    pub fn get_metric(&self, name: &str) -> Option<&PerformanceMetric> {
        self.performance_metrics.iter().find(|m| m.name == name)
    }

    /// Generate a summary for this experiment
    pub fn to_summary(&self, id: Uuid) -> ExperimentSummary {
        ExperimentSummary {
            id,
            model_name: self.model_name.clone(),
            quantization: self.quantization.clone(),
            backend: self.backend.clone(),
            hardware_summary: format!(
                "{} / {}",
                self.hardware_config.gpu_model, self.hardware_config.cpu_arch
            ),
            overall_score: self.mmlu_pro_score(),
            timestamp: self.timestamp,
            status: self.status.clone(),
        }
    }
}
