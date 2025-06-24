// llm-benchmark-types/src/metrics.rs

use serde::{Deserialize, Serialize};

/// A performance metric measurement
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PerformanceMetric {
    /// Name of the metric (e.g., "tokens_per_second", "memory_usage_gb")
    pub name: String,

    /// Measured value
    pub value: f64,

    /// Unit of measurement (e.g., "tok/s", "GB", "ms")
    pub unit: String,
}

/// A quality/accuracy score from a benchmark
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct QualityScore {
    /// Name of the benchmark (e.g., "mmlu_pro", "hellaswag")
    pub benchmark_name: String,

    /// Category within the benchmark (e.g., "Math", "Physics")
    pub category: String,

    /// Score as a percentage (0-100)
    pub score: f64,

    /// Total number of questions in this category
    pub total_questions: Option<i32>,

    /// Number of correct answers
    pub correct_answers: Option<i32>,
}

/// Common performance metric names
pub mod metric_names {
    pub const TOKENS_PER_SECOND: &str = "tokens_per_second";
    pub const MEMORY_USAGE_GB: &str = "memory_usage_gb";
    pub const MODEL_LOADING_TIME: &str = "model_loading_time";
    pub const PROMPT_PROCESSING_SPEED: &str = "prompt_processing_speed";
    pub const FIRST_TOKEN_LATENCY: &str = "first_token_latency";
    pub const THROUGHPUT: &str = "throughput";
    pub const BATCH_SIZE: &str = "batch_size";
}

/// Common benchmark names
pub mod benchmark_names {
    pub const MMLU_PRO: &str = "mmlu_pro";
    pub const HELLASWAG: &str = "hellaswag";
    pub const ARC_CHALLENGE: &str = "arc_challenge";
    pub const TRUTHFUL_QA: &str = "truthful_qa";
    pub const WINOGRANDE: &str = "winogrande";
}

/// MMLU-Pro categories
pub mod mmlu_categories {
    pub const BIOLOGY: &str = "Biology";
    pub const BUSINESS: &str = "Business";
    pub const CHEMISTRY: &str = "Chemistry";
    pub const COMPUTER_SCIENCE: &str = "Computer Science";
    pub const ECONOMICS: &str = "Economics";
    pub const ENGINEERING: &str = "Engineering";
    pub const HEALTH: &str = "Health";
    pub const HISTORY: &str = "History";
    pub const LAW: &str = "Law";
    pub const MATH: &str = "Math";
    pub const PHILOSOPHY: &str = "Philosophy";
    pub const PHYSICS: &str = "Physics";
    pub const PSYCHOLOGY: &str = "Psychology";
    pub const OTHER: &str = "Other";
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(name: String, value: f64, unit: String) -> Self {
        Self { name, value, unit }
    }

    /// Create a tokens per second metric
    pub fn tokens_per_second(value: f64) -> Self {
        Self::new(
            metric_names::TOKENS_PER_SECOND.to_string(),
            value,
            "tok/s".to_string(),
        )
    }

    /// Create a memory usage metric
    pub fn memory_usage_gb(value: f64) -> Self {
        Self::new(
            metric_names::MEMORY_USAGE_GB.to_string(),
            value,
            "GB".to_string(),
        )
    }

    /// Create a model loading time metric
    pub fn model_loading_time_seconds(value: f64) -> Self {
        Self::new(
            metric_names::MODEL_LOADING_TIME.to_string(),
            value,
            "s".to_string(),
        )
    }

    /// Create a prompt processing speed metric
    pub fn prompt_processing_speed(value: f64) -> Self {
        Self::new(
            metric_names::PROMPT_PROCESSING_SPEED.to_string(),
            value,
            "tok/s".to_string(),
        )
    }

    /// Format the metric for display
    pub fn display(&self) -> String {
        format!("{}: {:.2} {}", self.name, self.value, self.unit)
    }

    /// Check if this is a "higher is better" metric
    pub fn higher_is_better(&self) -> bool {
        matches!(
            self.name.as_str(),
            metric_names::TOKENS_PER_SECOND
                | metric_names::PROMPT_PROCESSING_SPEED
                | metric_names::THROUGHPUT
        )
    }

    /// Check if this is a "lower is better" metric
    pub fn lower_is_better(&self) -> bool {
        matches!(
            self.name.as_str(),
            metric_names::MEMORY_USAGE_GB
                | metric_names::MODEL_LOADING_TIME
                | metric_names::FIRST_TOKEN_LATENCY
        )
    }
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(
        benchmark_name: String,
        category: String,
        score: f64,
        total_questions: Option<i32>,
        correct_answers: Option<i32>,
    ) -> Self {
        Self {
            benchmark_name,
            category,
            score,
            total_questions,
            correct_answers,
        }
    }

    /// Create an MMLU-Pro score
    pub fn mmlu_pro(category: String, score: f64, total: i32, correct: i32) -> Self {
        Self::new(
            benchmark_names::MMLU_PRO.to_string(),
            category,
            score,
            Some(total),
            Some(correct),
        )
    }

    /// Calculate accuracy if raw counts are available
    pub fn accuracy(&self) -> Option<f64> {
        match (self.total_questions, self.correct_answers) {
            (Some(total), Some(correct)) if total > 0 => {
                Some((correct as f64 / total as f64) * 100.0)
            }
            _ => None,
        }
    }

    /// Format the score for display
    pub fn display(&self) -> String {
        match (self.total_questions, self.correct_answers) {
            (Some(total), Some(correct)) => {
                format!(
                    "{}: {:.1}% ({}/{})",
                    self.category, self.score, correct, total
                )
            }
            _ => {
                format!("{}: {:.1}%", self.category, self.score)
            }
        }
    }

    /// Get performance tier based on score
    pub fn performance_tier(&self) -> PerformanceTier {
        match self.score {
            s if s >= 80.0 => PerformanceTier::Excellent,
            s if s >= 70.0 => PerformanceTier::Good,
            s if s >= 60.0 => PerformanceTier::Fair,
            _ => PerformanceTier::Poor,
        }
    }
}

/// Performance tier for categorizing scores
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceTier {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl PerformanceTier {
    /// Get a color associated with this tier (for UI)
    pub fn color(&self) -> &'static str {
        match self {
            PerformanceTier::Excellent => "#28a745", // Green
            PerformanceTier::Good => "#17a2b8",      // Blue
            PerformanceTier::Fair => "#ffc107",      // Yellow
            PerformanceTier::Poor => "#dc3545",      // Red
        }
    }

    /// Get a display name for this tier
    pub fn display_name(&self) -> &'static str {
        match self {
            PerformanceTier::Excellent => "Excellent",
            PerformanceTier::Good => "Good",
            PerformanceTier::Fair => "Fair",
            PerformanceTier::Poor => "Poor",
        }
    }
}

impl std::fmt::Display for PerformanceTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metric_creation() {
        let metric = PerformanceMetric::tokens_per_second(45.2);
        assert_eq!(metric.name, "tokens_per_second");
        assert_eq!(metric.value, 45.2);
        assert_eq!(metric.unit, "tok/s");
        assert!(metric.higher_is_better());
    }

    #[test]
    fn test_quality_score_accuracy() {
        let score = QualityScore::mmlu_pro("Math".to_string(), 75.0, 100, 75);
        assert_eq!(score.accuracy(), Some(75.0));
        assert_eq!(score.performance_tier(), PerformanceTier::Good);
    }

    #[test]
    fn test_performance_tiers() {
        assert_eq!(PerformanceTier::Excellent.color(), "#28a745");
        assert_eq!(PerformanceTier::Poor.display_name(), "Poor");
    }
}
