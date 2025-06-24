// llm-benchmark-types/src/validation.rs

use crate::{
    benchmark_names, metric_names, ExperimentRun, HardwareConfig, PerformanceMetric, QualityScore,
    ValidationError, ValidationResult,
};

/// Validation trait for experiment data
pub trait Validate {
    /// Validate the data and return any errors
    fn validate(&self) -> ValidationResult<()>;

    /// Validate and return warnings (non-fatal issues)
    fn warnings(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Validate for ExperimentRun {
    fn validate(&self) -> ValidationResult<()> {
        // Validate model name
        if self.model_name.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "model_name".to_string(),
            });
        }

        // Validate quantization
        if self.quantization.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "quantization".to_string(),
            });
        }

        if !is_valid_quantization(&self.quantization) {
            return Err(ValidationError::InvalidField {
                field: "quantization".to_string(),
                message: format!("Unknown quantization scheme: {}", self.quantization),
            });
        }

        // Validate backend
        if self.backend.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "backend".to_string(),
            });
        }

        if !is_valid_backend(&self.backend) {
            return Err(ValidationError::InvalidField {
                field: "backend".to_string(),
                message: format!("Unknown backend: {}", self.backend),
            });
        }

        // Validate hardware config
        self.hardware_config.validate()?;

        // Validate performance metrics
        for (i, metric) in self.performance_metrics.iter().enumerate() {
            metric.validate().map_err(|e| match e {
                ValidationError::InvalidField { field, message } => ValidationError::InvalidField {
                    field: format!("performance_metrics[{}].{}", i, field),
                    message,
                },
                ValidationError::MissingField { field } => ValidationError::MissingField {
                    field: format!("performance_metrics[{}].{}", i, field),
                },
                other => other,
            })?;
        }

        // Validate quality scores
        for (i, score) in self.quality_scores.iter().enumerate() {
            score.validate().map_err(|e| match e {
                ValidationError::InvalidField { field, message } => ValidationError::InvalidField {
                    field: format!("quality_scores[{}].{}", i, field),
                    message,
                },
                ValidationError::MissingField { field } => ValidationError::MissingField {
                    field: format!("quality_scores[{}].{}", i, field),
                },
                other => other,
            })?;
        }

        Ok(())
    }

    fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for missing common metrics
        let metric_names: Vec<_> = self
            .performance_metrics
            .iter()
            .map(|m| m.name.as_str())
            .collect();

        if !metric_names.contains(&metric_names::TOKENS_PER_SECOND) {
            warnings.push("Missing tokens_per_second metric".to_string());
        }

        if !metric_names.contains(&metric_names::MEMORY_USAGE_GB) {
            warnings.push("Missing memory_usage_gb metric".to_string());
        }

        // Check for quality scores
        if self.quality_scores.is_empty() {
            warnings.push("No quality scores provided".to_string());
        }

        // Check for unrealistic values
        for metric in &self.performance_metrics {
            if metric.name == metric_names::TOKENS_PER_SECOND && metric.value > 1000.0 {
                warnings.push(format!(
                    "Unusually high tokens_per_second: {:.1} tok/s",
                    metric.value
                ));
            }

            if metric.name == metric_names::MEMORY_USAGE_GB && metric.value > 200.0 {
                warnings.push(format!(
                    "Unusually high memory usage: {:.1} GB",
                    metric.value
                ));
            }
        }

        warnings
    }
}

impl Validate for HardwareConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate GPU model
        if self.gpu_model.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "gpu_model".to_string(),
            });
        }

        // Validate GPU memory
        if self.gpu_memory_gb < 0 {
            return Err(ValidationError::OutOfRange {
                field: "gpu_memory_gb".to_string(),
                value: self.gpu_memory_gb.to_string(),
                range: "≥ 0".to_string(),
            });
        }

        // Check consistency: CPU Only should have 0 GPU memory
        if self.gpu_model == "CPU Only" && self.gpu_memory_gb != 0 {
            return Err(ValidationError::InvalidField {
                field: "gpu_memory_gb".to_string(),
                message: "CPU Only configuration should have 0 GPU memory".to_string(),
            });
        }

        // Validate CPU model
        if self.cpu_model.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "cpu_model".to_string(),
            });
        }

        // Validate CPU architecture
        if self.cpu_arch.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "cpu_arch".to_string(),
            });
        }

        if !is_valid_cpu_arch(&self.cpu_arch) {
            return Err(ValidationError::InvalidField {
                field: "cpu_arch".to_string(),
                message: format!("Unknown CPU architecture: {}", self.cpu_arch),
            });
        }

        // Validate RAM
        if self.ram_gb <= 0 {
            return Err(ValidationError::OutOfRange {
                field: "ram_gb".to_string(),
                value: self.ram_gb.to_string(),
                range: "> 0".to_string(),
            });
        }

        if self.ram_type.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "ram_type".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for PerformanceMetric {
    fn validate(&self) -> ValidationResult<()> {
        // Validate metric name
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "name".to_string(),
            });
        }

        // Validate unit
        if self.unit.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "unit".to_string(),
            });
        }

        // Validate value ranges for known metrics
        match self.name.as_str() {
            metric_names::TOKENS_PER_SECOND => {
                if self.value < 0.0 {
                    return Err(ValidationError::OutOfRange {
                        field: "value".to_string(),
                        value: self.value.to_string(),
                        range: "≥ 0".to_string(),
                    });
                }
            }
            metric_names::MEMORY_USAGE_GB => {
                if self.value < 0.0 {
                    return Err(ValidationError::OutOfRange {
                        field: "value".to_string(),
                        value: self.value.to_string(),
                        range: "≥ 0".to_string(),
                    });
                }
            }
            metric_names::MODEL_LOADING_TIME => {
                if self.value < 0.0 {
                    return Err(ValidationError::OutOfRange {
                        field: "value".to_string(),
                        value: self.value.to_string(),
                        range: "≥ 0".to_string(),
                    });
                }
            }
            _ => {} // Unknown metrics are allowed
        }

        // Check for NaN or infinite values
        if !self.value.is_finite() {
            return Err(ValidationError::InvalidField {
                field: "value".to_string(),
                message: "Value must be finite (not NaN or infinite)".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for QualityScore {
    fn validate(&self) -> ValidationResult<()> {
        // Validate benchmark name
        if self.benchmark_name.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "benchmark_name".to_string(),
            });
        }

        // Validate category
        if self.category.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "category".to_string(),
            });
        }

        // Validate score range
        if !(0.0..=100.0).contains(&self.score) {
            return Err(ValidationError::OutOfRange {
                field: "score".to_string(),
                value: self.score.to_string(),
                range: "0-100".to_string(),
            });
        }

        // Check for NaN
        if !self.score.is_finite() {
            return Err(ValidationError::InvalidField {
                field: "score".to_string(),
                message: "Score must be finite (not NaN or infinite)".to_string(),
            });
        }

        // Validate question counts if provided
        if let Some(total) = self.total_questions {
            if total <= 0 {
                return Err(ValidationError::OutOfRange {
                    field: "total_questions".to_string(),
                    value: total.to_string(),
                    range: "> 0".to_string(),
                });
            }
        }

        if let Some(correct) = self.correct_answers {
            if correct < 0 {
                return Err(ValidationError::OutOfRange {
                    field: "correct_answers".to_string(),
                    value: correct.to_string(),
                    range: "≥ 0".to_string(),
                });
            }

            // Check that correct <= total if both are provided
            if let Some(total) = self.total_questions {
                if correct > total {
                    return Err(ValidationError::InvalidField {
                        field: "correct_answers".to_string(),
                        message: format!(
                            "Correct answers ({}) cannot exceed total questions ({})",
                            correct, total
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Check if a quantization scheme is valid
fn is_valid_quantization(quant: &str) -> bool {
    matches!(
        quant,
        "FP32"
            | "FP16"
            | "BF16"
            | "Q8_0"
            | "Q6_K"
            | "Q5_K_M"
            | "Q5_K_S"
            | "Q4_K_M"
            | "Q4_K_S"
            | "Q4_0"
            | "Q3_K_M"
            | "Q3_K_S"
            | "Q2_K"
    )
}

/// Check if a backend is valid
fn is_valid_backend(backend: &str) -> bool {
    matches!(
        backend,
        "llama.cpp" | "llama_cpp" | "vllm" | "transformers" | "onnx" | "tvm" | "tensorrt"
    )
}

/// Check if a CPU architecture is valid
fn is_valid_cpu_arch(arch: &str) -> bool {
    matches!(
        arch,
        "x86_64"
            | "aarch64"
            | "arm64"
            | "Zen1"
            | "Zen2"
            | "Zen3"
            | "Zen4"
            | "Intel"
            | "AMD"
            | "Apple M1"
            | "Apple M2"
            | "Apple M3"
    )
}

/// Validation helper functions
pub mod validators {
    use super::*;

    /// Validate an entire experiment run and return both errors and warnings
    pub fn validate_experiment_run(
        experiment: &ExperimentRun,
    ) -> (ValidationResult<()>, Vec<String>) {
        let validation_result = experiment.validate();
        let warnings = experiment.warnings();
        (validation_result, warnings)
    }

    /// Check if performance metrics are reasonable
    pub fn check_performance_sanity(metrics: &[PerformanceMetric]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for common metric combinations
        let has_speed = metrics
            .iter()
            .any(|m| m.name == metric_names::TOKENS_PER_SECOND);
        let has_memory = metrics
            .iter()
            .any(|m| m.name == metric_names::MEMORY_USAGE_GB);

        if has_speed && has_memory {
            if let (Some(speed), Some(memory)) = (
                metrics
                    .iter()
                    .find(|m| m.name == metric_names::TOKENS_PER_SECOND),
                metrics
                    .iter()
                    .find(|m| m.name == metric_names::MEMORY_USAGE_GB),
            ) {
                // Very high speed with very low memory might indicate an error
                if speed.value > 100.0 && memory.value < 2.0 {
                    warnings
                        .push("High speed with very low memory usage seems unusual".to_string());
                }
            }
        }

        warnings
    }

    /// Validate that quality scores are consistent across categories
    pub fn check_quality_consistency(scores: &[QualityScore]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Group by benchmark
        let mut by_benchmark = std::collections::HashMap::new();
        for score in scores {
            by_benchmark
                .entry(&score.benchmark_name)
                .or_insert_with(Vec::new)
                .push(score);
        }

        for (benchmark, benchmark_scores) in by_benchmark {
            // Check for extreme outliers
            let scores_only: Vec<f64> = benchmark_scores.iter().map(|s| s.score).collect();
            if scores_only.len() > 2 {
                let mean = scores_only.iter().sum::<f64>() / scores_only.len() as f64;
                let variance = scores_only
                    .iter()
                    .map(|&score| (score - mean).powi(2))
                    .sum::<f64>()
                    / scores_only.len() as f64;
                let std_dev = variance.sqrt();

                // Flag scores that are more than 2 standard deviations from mean
                for score_obj in benchmark_scores {
                    if (score_obj.score - mean).abs() > 2.0 * std_dev && std_dev > 10.0 {
                        warnings.push(format!(
                            "{} score for {} ({:.1}%) is unusual compared to other categories",
                            benchmark, score_obj.category, score_obj.score
                        ));
                    }
                }
            }
        }

        warnings
    }
}

mod tests {
    use super::*;
    use crate::{ExperimentRun, HardwareConfig};

    #[test]
    fn test_hardware_validation() {
        let mut config = HardwareConfig::new(
            "RTX 4090".to_string(),
            24,
            "AMD Threadripper".to_string(),
            "Zen2".to_string(),
            64,
            "DDR4".to_string(),
        );

        assert!(config.validate().is_ok());

        // Test invalid GPU memory for CPU-only
        config.gpu_model = "CPU Only".to_string();
        config.gpu_memory_gb = 24; // Should be 0 for CPU Only
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metric_validation() {
        let metric = PerformanceMetric::tokens_per_second(45.2);
        assert!(metric.validate().is_ok());

        let invalid_metric = PerformanceMetric::new(
            metric_names::TOKENS_PER_SECOND.to_string(),
            -10.0, // Negative value invalid
            "tok/s".to_string(),
        );
        assert!(invalid_metric.validate().is_err());
    }

    #[test]
    fn test_score_validation() {
        let score = QualityScore::mmlu_pro("Math".to_string(), 75.0, 100, 75);
        assert!(score.validate().is_ok());

        let invalid_score = QualityScore::new(
            benchmark_names::MMLU_PRO.to_string(),
            "Math".to_string(),
            150.0, // Invalid score > 100
            Some(100),
            Some(75),
        );
        assert!(invalid_score.validate().is_err());
    }
}
