// llm-benchmark-types/src/experiment.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{HardwareConfig, PerformanceMetric, BenchmarkScoreType, BenchmarkScore};

/// A complete experiment run containing all benchmark data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExperimentRun {
    /// Unique identifier for this experiment run
    /// For SystemsLab imports, this is the experiment ID
    /// For local uploads, generate a UUID v7 and persist it
    pub id: Uuid,

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

    /// Benchmark scores from various tests
    pub benchmark_scores: Vec<BenchmarkScoreType>,

    /// When the experiment was conducted
    pub timestamp: DateTime<Utc>,

    /// Optional notes about the experiment
    pub notes: Option<String>,

    /// Status of the experiment run
    #[serde(default = "default_status")]
    pub status: ExperimentStatus,

    /// Test configuration: number of concurrent requests
    #[serde(default)]
    pub concurrent_requests: Option<i32>,

    /// Test configuration: maximum context length in tokens
    #[serde(default)]
    pub max_context_length: Option<i32>,

    /// Test configuration: load pattern (e.g., "Concurrent", "QPS", "Burst")
    #[serde(default)]
    pub load_pattern: Option<String>,

    /// Test configuration: dataset name (e.g., "OpenOrca", "ShareGPT")
    #[serde(default)]
    pub dataset_name: Option<String>,

    /// GPU power limit in watts (e.g., 300 for limited RTX 4090)
    #[serde(default)]
    pub gpu_power_limit_watts: Option<i32>,
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
        id: Uuid,
        model_name: String,
        quantization: String,
        backend: String,
        backend_version: String,
        hardware_config: HardwareConfig,
    ) -> Self {
        Self {
            id,
            model_name,
            quantization,
            backend,
            backend_version,
            hardware_config,
            performance_metrics: Vec::new(),
            benchmark_scores: Vec::new(),
            timestamp: Utc::now(),
            notes: None,
            status: ExperimentStatus::Pending,
            concurrent_requests: None,
            max_context_length: None,
            load_pattern: None,
            dataset_name: None,
            gpu_power_limit_watts: None,
        }
    }

    /// Add a performance metric to this experiment
    pub fn add_performance_metric(&mut self, metric: PerformanceMetric) {
        self.performance_metrics.push(metric);
    }

    /// Add a benchmark score to this experiment
    pub fn add_benchmark_score(&mut self, score: BenchmarkScoreType) {
        self.benchmark_scores.push(score);
    }

    /// Get a specific performance metric by name
    pub fn get_performance_metric(&self, name: &str) -> Option<&PerformanceMetric> {
        self.performance_metrics.iter().find(|m| m.metric_name == name)
    }

    /// Get all benchmark scores for a specific benchmark
    pub fn get_benchmark_scores_for_benchmark(&self, benchmark_name: &str) -> Vec<&BenchmarkScoreType> {
        self.benchmark_scores
            .iter()
            .filter(|s| s.benchmark_name() == benchmark_name)
            .collect()
    }

    /// Calculate overall score across all benchmark scores
    pub fn calculate_overall_score(&self) -> Option<f64> {
        if self.benchmark_scores.is_empty() {
            return None;
        }

        let sum: f64 = self.benchmark_scores.iter().map(|s| s.overall_score()).sum();
        Some(sum / self.benchmark_scores.len() as f64)
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