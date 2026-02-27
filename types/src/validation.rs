// llm-benchmark-types/src/validation.rs

use crate::{
    ExperimentRun, HardwareConfig, PerformanceMetric, BenchmarkScore,
    QualityScore, ValidationError, ValidationResult, metric_names,
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

        // Validate benchmark scores
        for (i, score) in self.benchmark_scores.iter().enumerate() {
            score.validate().map_err(|e| match e {
                ValidationError::InvalidField { field, message } => ValidationError::InvalidField {
                    field: format!("benchmark_scores[{}].{}", i, field),
                    message,
                },
                ValidationError::MissingField { field } => ValidationError::MissingField {
                    field: format!("benchmark_scores[{}].{}", i, field),
                },
                other => other,
            })?;
        }

        Ok(())
    }

    fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for duplicate metric names
        let metric_names: Vec<&str> = self.performance_metrics
            .iter()
            .map(|m| m.metric_name.as_str())
            .collect();
        
        let unique_metrics: std::collections::HashSet<_> = metric_names.iter().collect();
        if metric_names.len() != unique_metrics.len() {
            warnings.push("Duplicate performance metrics detected".to_string());
        }

        // Check for missing essential metrics
        if !metric_names.contains(&metric_names::TOKENS_PER_SECOND) {
            warnings.push("Missing tokens_per_second metric".to_string());
        }
        
        if !metric_names.contains(&metric_names::MEMORY_USAGE_GB) {
            warnings.push("Missing memory_usage_gb metric".to_string());
        }

        // Check for unusual metric values
        for metric in &self.performance_metrics {
            match metric.metric_name.as_str() {
                metric_names::TOKENS_PER_SECOND => {
                    if metric.value > 1000.0 {
                        warnings.push(format!(
                            "Unusually high tokens_per_second: {}", 
                            metric.value
                        ));
                    }
                }
                metric_names::MEMORY_USAGE_GB => {
                    if metric.value > 200.0 {
                        warnings.push(format!(
                            "Unusually high memory_usage_gb: {}", 
                            metric.value
                        ));
                    }
                }
                _ => {}
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

        // Validate GPU memory (0 is allowed for CPU-only systems)
        if self.gpu_memory_gb < 0 {
            return Err(ValidationError::OutOfRange {
                field: "gpu_memory_gb".to_string(),
                value: self.gpu_memory_gb.to_string(),
                range: ">= 0".to_string(),
            });
        }
        
        // Check consistency: if GPU is "CPU Only", memory should be 0
        if (self.gpu_model == "CPU Only" || self.gpu_model == "N/A") && self.gpu_memory_gb > 0 {
            return Err(ValidationError::InvalidField {
                field: "gpu_memory_gb".to_string(),
                message: "GPU memory should be 0 for CPU-only systems".to_string(),
            });
        }

        // Validate CPU model
        if self.cpu_model.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "cpu_model".to_string(),
            });
        }

        // Validate CPU arch
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

        // Validate RAM (if provided)
        if let Some(ram_gb) = self.ram_gb {
            if ram_gb <= 0 {
                return Err(ValidationError::OutOfRange {
                    field: "ram_gb".to_string(),
                    value: ram_gb.to_string(),
                    range: "> 0".to_string(),
                });
            }
        }

        if let Some(ram_type) = &self.ram_type {
            if ram_type.trim().is_empty() {
                return Err(ValidationError::MissingField {
                    field: "ram_type".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Validate for PerformanceMetric {
    fn validate(&self) -> ValidationResult<()> {
        // Validate metric name
        if self.metric_name.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "metric_name".to_string(),
            });
        }

        // Validate unit
        if self.unit.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "unit".to_string(),
            });
        }

        // Validate value ranges for known metrics
        match self.metric_name.as_str() {
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
            _ => {
                // Allow any value for unknown metrics
            }
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

        // Validate score range (assuming 0-1 or 0-100)
        if self.score < 0.0 {
            return Err(ValidationError::OutOfRange {
                field: "score".to_string(),
                value: self.score.to_string(),
                range: "≥ 0".to_string(),
            });
        }

        if self.score > 100.0 {
            return Err(ValidationError::OutOfRange {
                field: "score".to_string(),
                value: self.score.to_string(),
                range: "≤ 100".to_string(),
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

            if let Some(total) = self.total_questions {
                if correct > total {
                    return Err(ValidationError::InvalidField {
                        field: "correct_answers".to_string(),
                        message: "Cannot be greater than total_questions".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if benchmark is known
        if !self.is_known_benchmark() {
            warnings.push(format!("Unknown benchmark: {}", self.benchmark_name));
        }

        // Check for unusual scores
        if self.score == 0.0 {
            warnings.push("Score is exactly 0 - verify this is correct".to_string());
        }

        if self.score == 100.0 {
            warnings.push("Perfect score (100) - verify this is correct".to_string());
        }

        warnings
    }
}

// Helper functions for validation

/// Normalize a quantization string by stripping redundant suffixes like `-GGUF`.
/// The GGUF format is already implied by the backend (llama.cpp), so `-GGUF`
/// suffixes on quant names like `Q8_0-GGUF` are redundant and cause mismatches.
pub fn normalize_quantization(quantization: &str) -> String {
    let stripped = quantization
        .strip_suffix("-GGUF")
        .or_else(|| quantization.strip_suffix("-gguf"))
        .unwrap_or(quantization);
    stripped.to_string()
}

fn is_valid_quantization(quantization: &str) -> bool {
    let normalized = normalize_quantization(quantization);
    let quant_upper = normalized.to_uppercase();

    // Check exact matches first
    let exact_match = matches!(
        quant_upper.as_str(),
        // Floating point formats
        "BF16" | "F16" | "FP16" | "F32" | "FP32" | "FP8" | "FP8_DYNAMIC" |
        // Standard quantization formats
        "Q8_0" | "Q6_K" | "Q5_K_M" | "Q5_K_S" | "Q5_1" | "Q5_0" |
        "Q4_K_M" | "Q4_K_S" | "Q4_1" | "Q4_0" |
        "Q3_K_L" | "Q3_K_M" | "Q3_K_S" | "Q2_K" |
        // IQ (Integer Quantization) formats
        "IQ4_XS" | "IQ4_NL" | "IQ3_M" |
        // Weight-Activation quantization formats (vLLM, TensorRT-LLM)
        "W8A8" | "W4A16" | "W4A8" | "W8A16" |
        // Other formats
        "INT8" | "INT4" | "GGUF" | "AWQ" | "GPTQ"
    );

    if exact_match {
        return true;
    }

    // Check for W*A* patterns with method suffixes (e.g., W4A16-CT, W4A16-AWQ, W4A16-GPTQ)
    if let Some(base) = quant_upper.strip_suffix("-CT")
        .or_else(|| quant_upper.strip_suffix("-AWQ"))
        .or_else(|| quant_upper.strip_suffix("-GPTQ"))
    {
        // Check if the base part matches W*A* pattern
        if base.starts_with('W') && base.contains("A") {
            return true;
        }
    }

    false
}

fn is_valid_backend(backend: &str) -> bool {
    matches!(
        backend.to_lowercase().as_str(),
        "llama.cpp" | "vllm" | "transformers" | "tgi" | "text-generation-inference" |
        "ctransformers" | "ggml" | "llamacpp" | "exllama" | "exllamav2" | "tensorrt-llm"
    )
}

fn is_valid_cpu_arch(cpu_arch: &str) -> bool {
    matches!(
        cpu_arch.to_lowercase().as_str(),
        "x86_64" | "x64" | "amd64" | "arm64" | "aarch64" | "armv7" | "armv8" |
        "zen2" | "zen3" | "zen4" | "haswell" | "skylake" | "icelake" | "alderlake" |
        "apple_m1" | "apple_m2" | "apple_m3" | "generic" | "unknown"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HardwareConfig, PerformanceMetric};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_valid_quantization() {
        // Test floating point formats
        assert!(is_valid_quantization("BF16"));
        assert!(is_valid_quantization("F16"));
        assert!(is_valid_quantization("FP16"));
        
        // Test standard Q formats
        assert!(is_valid_quantization("Q8_0"));
        assert!(is_valid_quantization("Q6_K"));
        assert!(is_valid_quantization("Q5_K_M"));
        assert!(is_valid_quantization("Q5_K_S"));
        assert!(is_valid_quantization("Q4_K_M"));
        assert!(is_valid_quantization("Q4_K_S"));
        assert!(is_valid_quantization("Q3_K_L"));
        assert!(is_valid_quantization("Q3_K_M"));
        assert!(is_valid_quantization("Q3_K_S"));
        assert!(is_valid_quantization("Q2_K"));
        
        // Test IQ formats
        assert!(is_valid_quantization("IQ4_XS"));
        assert!(is_valid_quantization("IQ4_NL"));
        assert!(is_valid_quantization("IQ3_M"));
        
        // Test case insensitivity
        assert!(is_valid_quantization("q4_k_m"));
        assert!(is_valid_quantization("iq4_xs"));
        
        // Test -GGUF suffix is accepted (normalized away)
        assert!(is_valid_quantization("Q8_0-GGUF"));
        assert!(is_valid_quantization("Q4_K_M-GGUF"));
        assert!(is_valid_quantization("FP16-GGUF"));
        assert!(is_valid_quantization("IQ4_XS-gguf"));

        // Test invalid formats
        assert!(!is_valid_quantization("INVALID"));
        assert!(!is_valid_quantization("Q3_K_XL"));
    }

    #[test]
    fn test_normalize_quantization() {
        assert_eq!(normalize_quantization("Q8_0-GGUF"), "Q8_0");
        assert_eq!(normalize_quantization("Q4_K_M-GGUF"), "Q4_K_M");
        assert_eq!(normalize_quantization("FP16-gguf"), "FP16");
        assert_eq!(normalize_quantization("Q8_0"), "Q8_0");
        assert_eq!(normalize_quantization("BF16"), "BF16");
        assert_eq!(normalize_quantization("W4A16-AWQ"), "W4A16-AWQ");
    }

    #[test]
    fn test_valid_backend() {
        assert!(is_valid_backend("llama.cpp"));
        assert!(is_valid_backend("VLLM")); // case insensitive
        assert!(!is_valid_backend("unknown_backend"));
    }

    #[test]
    fn test_performance_metric_validation() {
        let metric = PerformanceMetric {
            metric_name: "tokens_per_second".to_string(),
            value: 50.0,
            unit: "tok/s".to_string(),
            timestamp: Utc::now(),
            context: None,
        };
        assert!(metric.validate().is_ok());

        let invalid_metric = PerformanceMetric {
            metric_name: "".to_string(), // Empty name
            value: 50.0,
            unit: "tok/s".to_string(),
            timestamp: Utc::now(),
            context: None,
        };
        assert!(invalid_metric.validate().is_err());
    }

    #[test]
    fn test_experiment_run_warnings() {
        let hardware_config = HardwareConfig {
            gpu_model: "RTX 4090".to_string(),
            gpu_memory_gb: 24,
            cpu_model: "Intel i9".to_string(),
            cpu_arch: "x86_64".to_string(),
            ram_gb: Some(32),
            ram_type: Some("DDR4".to_string()),
            virtualization_type: None,
            optimizations: vec![],
        };

        let mut experiment = ExperimentRun::new(
            Uuid::new_v4(),
            "Test Model".to_string(),
            "FP16".to_string(),
            "llama.cpp".to_string(),
            "1.0".to_string(),
            hardware_config,
        );

        // Should warn about missing essential metrics
        let warnings = experiment.warnings();
        assert!(warnings.len() >= 2); // Should warn about missing speed and memory metrics

        // Add essential metrics
        experiment.add_performance_metric(PerformanceMetric::new(
            metric_names::TOKENS_PER_SECOND.to_string(),
            50.0,
            "tok/s".to_string(),
        ));

        experiment.add_performance_metric(PerformanceMetric::new(
            metric_names::MEMORY_USAGE_GB.to_string(),
            16.0,
            "GB".to_string(),
        ));

        let warnings_after = experiment.warnings();
        assert!(warnings_after.len() < warnings.len()); // Should have fewer warnings now
    }
}