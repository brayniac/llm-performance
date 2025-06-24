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

    /// Add a performance metric to this experiment
    pub fn add_performance_metric(&mut self, metric: PerformanceMetric) {
        self.performance_metrics.push(metric);
    }

    /// Add a quality score to this experiment
    pub fn add_quality_score(&mut self, score: QualityScore) {
        self.quality_scores.push(score);
    }

    /// Get a specific performance metric by name
    pub fn get_performance_metric(&self, name: &str) -> Option<&PerformanceMetric> {
        self.performance_metrics.iter().find(|m| m.metric_name == name)
    }

    /// Get all quality scores for a specific benchmark
    pub fn get_quality_scores_for_benchmark(&self, benchmark_name: &str) -> Vec<&QualityScore> {
        self.quality_scores
            .iter()
            .filter(|s| s.benchmark_name == benchmark_name)
            .collect()
    }

    /// Calculate overall score across all quality scores
    pub fn calculate_overall_score(&self) -> Option<f64> {
        if self.quality_scores.is_empty() {
            return None;
        }

        let sum: f64 = self.quality_scores.iter().map(|s| s.score).sum();
        Some(sum / self.quality_scores.len() as f64)
    }

    /// Mark the experiment as completed
    pub fn mark_completed(&mut self) {
        self.status = ExperimentStatus::Completed;
    }

    /// Mark the experiment as failed
    pub fn mark_failed(&mut self) {
        self.status = ExperimentStatus::Failed;
    }

    /// Check if the experiment has essential performance metrics
    pub fn has_essential_metrics(&self) -> bool {
        let has_speed = self.performance_metrics
            .iter()
            .any(|m| m.metric_name == crate::metric_names::TOKENS_PER_SECOND);
        
        let has_memory = self.performance_metrics
            .iter()
            .any(|m| m.metric_name == crate::metric_names::MEMORY_USAGE_GB);

        has_speed && has_memory
    }
}

impl ExperimentSummary {
    /// Create a summary from a full experiment run
    pub fn from_experiment_run(run: &ExperimentRun, id: Uuid) -> Self {
        let hardware_summary = format!(
            "{} / {}",
            run.hardware_config.gpu_model,
            run.hardware_config.cpu_arch
        );

        Self {
            id,
            model_name: run.model_name.clone(),
            quantization: run.quantization.clone(),
            backend: run.backend.clone(),
            hardware_summary,
            overall_score: run.calculate_overall_score(),
            timestamp: run.timestamp,
            status: run.status.clone(),
        }
    }
}