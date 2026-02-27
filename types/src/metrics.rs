// llm-benchmark-types/src/metrics.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single performance metric measurement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetric {
    /// Name of the metric (e.g., "tokens_per_second", "memory_usage_gb")
    pub metric_name: String,

    /// Measured value
    pub value: f64,

    /// Unit of measurement (e.g., "tok/s", "GB", "ms")
    pub unit: String,

    /// When the metric was measured
    pub timestamp: DateTime<Utc>,

    /// Optional context or metadata
    pub context: Option<serde_json::Value>,
}

/// A quality/benchmark score
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QualityScore {
    /// Name of the benchmark (e.g., "mmlu", "hellaswag")
    pub benchmark_name: String,

    /// Category within the benchmark (e.g., "science", "reasoning")
    pub category: String,

    /// Score value (typically 0.0 to 1.0 or percentage)
    pub score: f64,

    /// Total questions in this category (optional)
    pub total_questions: Option<i32>,

    /// Correct answers in this category (optional)
    pub correct_answers: Option<i32>,

    /// When the score was measured
    pub timestamp: DateTime<Utc>,

    /// Optional context or metadata
    pub context: Option<serde_json::Value>,
}

/// Constants for known metric names
pub mod metric_names {
    pub const TOKENS_PER_SECOND: &str = "tokens_per_second";
    pub const MEMORY_USAGE_GB: &str = "memory_usage_gb";
    pub const MODEL_LOADING_TIME: &str = "model_loading_time";
    pub const PROMPT_PROCESSING_SPEED: &str = "prompt_processing_speed";
    pub const FIRST_TOKEN_LATENCY_MS: &str = "first_token_latency_ms";
    pub const AVERAGE_TOKEN_LATENCY_MS: &str = "average_token_latency_ms";
    pub const THROUGHPUT_TOKENS_PER_SECOND: &str = "throughput_tokens_per_second";
    pub const PEAK_MEMORY_GB: &str = "peak_memory_gb";
    pub const MODEL_SIZE_GB: &str = "model_size_gb";
    pub const GPU_POWER_WATTS: &str = "gpu_power_watts";
}

/// Known metric names for validation
pub fn metric_names() -> Vec<&'static str> {
    vec![
        metric_names::TOKENS_PER_SECOND,
        metric_names::MEMORY_USAGE_GB, 
        metric_names::MODEL_LOADING_TIME,
        metric_names::PROMPT_PROCESSING_SPEED,
        metric_names::FIRST_TOKEN_LATENCY_MS,
        metric_names::AVERAGE_TOKEN_LATENCY_MS,
        metric_names::THROUGHPUT_TOKENS_PER_SECOND,
        metric_names::PEAK_MEMORY_GB,
        metric_names::MODEL_SIZE_GB,
    ]
}

/// Known benchmark names for validation
pub fn benchmark_names() -> Vec<&'static str> {
    vec![
        "mmlu",
        "hellaswag", 
        "winogrande",
        "truthfulqa",
        "gsm8k",
        "humaneval",
        "arc_challenge",
        "arc_easy",
        "commonsense_qa",
        "piqa",
    ]
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(metric_name: String, value: f64, unit: String) -> Self {
        Self {
            metric_name,
            value,
            unit,
            timestamp: Utc::now(),
            context: None,
        }
    }

    /// Create a metric with context
    pub fn with_context(
        metric_name: String,
        value: f64,
        unit: String,
        context: serde_json::Value,
    ) -> Self {
        Self {
            metric_name,
            value,
            unit,
            timestamp: Utc::now(),
            context: Some(context),
        }
    }

    /// Check if this is a valid known metric
    pub fn is_known_metric(&self) -> bool {
        metric_names().contains(&self.metric_name.as_str())
    }
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(benchmark_name: String, category: String, score: f64) -> Self {
        Self {
            benchmark_name,
            category,
            score,
            total_questions: None,
            correct_answers: None,
            timestamp: Utc::now(),
            context: None,
        }
    }

    /// Create a score with context and question counts
    pub fn with_details(
        benchmark_name: String,
        category: String,
        score: f64,
        total_questions: Option<i32>,
        correct_answers: Option<i32>,
        context: Option<serde_json::Value>,
    ) -> Self {
        Self {
            benchmark_name,
            category,
            score,
            total_questions,
            correct_answers,
            timestamp: Utc::now(),
            context,
        }
    }

    /// Check if this is a valid known benchmark
    pub fn is_known_benchmark(&self) -> bool {
        benchmark_names().contains(&self.benchmark_name.as_str())
    }

    /// Convert score to percentage (0-100)
    pub fn as_percentage(&self) -> f64 {
        if self.score <= 1.0 {
            self.score * 100.0
        } else {
            self.score
        }
    }
}