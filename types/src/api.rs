// llm-benchmark-types/src/api.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ExperimentRun, ExperimentSummary};
use crate::hardware::HardwareCategory;

/// Request to upload a new experiment run
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadExperimentRequest {
    /// The experiment data to upload
    pub experiment_run: ExperimentRun,
}

/// Response from experiment upload
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadExperimentResponse {
    /// Whether the upload was successful
    pub success: bool,

    /// ID of the created test run (if successful)
    pub test_run_id: Option<Uuid>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
}

/// Request for performance grid data
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceGridRequest {
    /// Maximum memory usage filter (in GB)
    pub max_memory_gb: Option<f64>,

    /// Minimum speed filter (in tok/s)
    pub min_speed: Option<f64>,

    /// Backend filter
    pub backends: Option<Vec<String>>,

    /// Hardware type filter
    pub hardware_types: Option<Vec<String>>,

    /// Model name filter
    pub models: Option<Vec<String>>,
}

/// Request for grouped model performance data
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GroupedPerformanceRequest {
    /// Benchmark to use for quality scoring (e.g., "mmlu", "gsm8k")
    pub benchmark: Option<String>,
    
    /// Minimum quality score filter (0-100)
    pub min_quality: Option<f64>,
    
    /// Maximum memory usage filter (in GB)
    pub max_memory_gb: Option<f64>,

    /// Minimum speed filter (in tok/s)
    pub min_speed: Option<f64>,

    /// Sort field ("quality", "speed", "memory", "model_name")
    pub sort_by: Option<String>,
    
    /// Sort direction ("asc" or "desc")
    pub sort_direction: Option<String>,
    
    /// Hardware categories to include (comma-separated string)
    pub hardware_categories: Option<String>,
}

/// Row in the performance grid
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceGridRow {
    /// Test run ID
    pub id: Uuid,

    /// Model name
    pub model_name: String,

    /// Quantization scheme
    pub quantization: String,

    /// Backend used
    pub backend: String,

    /// Generation speed in tokens per second
    pub tokens_per_second: f64,

    /// Memory usage in GB
    pub memory_gb: f64,

    /// GPU model
    pub gpu_model: String,

    /// CPU architecture
    pub cpu_arch: String,

    /// Hardware type (gpu/cpu_only)
    pub hardware_type: String,

    /// Overall quality score (if available)
    pub overall_score: Option<f64>,
}

/// Response for grouped model performance
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedPerformanceResponse {
    /// List of models with their best qualifying quantization
    pub models: Vec<ModelPerformanceGroup>,
    
    /// Total number of models (before pagination)
    pub total_count: usize,
    
    /// Benchmark used for quality scoring
    pub benchmark_used: String,
}

/// A model with its best quantization that meets filters
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelPerformanceGroup {
    /// Model name/slug
    pub model_name: String,
    
    /// Best quantization that meets all filters
    pub best_quantization: QuantizationPerformance,
    
    /// Total number of quantizations tested
    pub total_quantizations: usize,
    
    /// Number of quantizations that meet current filters
    pub qualifying_quantizations: usize,
    
    /// All qualifying quantizations (for expansion)
    pub all_quantizations: Option<Vec<QuantizationPerformance>>,
}

/// Performance data for a specific quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationPerformance {
    /// Test run ID
    pub id: Uuid,
    
    /// Quantization scheme
    pub quantization: String,
    
    /// Quality score for the selected benchmark
    pub quality_score: f64,
    
    /// Generation speed in tokens per second
    pub tokens_per_second: f64,
    
    /// Memory usage in GB
    pub memory_gb: f64,
    
    /// Backend used
    pub backend: String,
    
    /// Hardware summary
    pub hardware: String,
    
    /// Hardware category
    pub hardware_category: HardwareCategory,
}

/// Request for comparison between two configurations
#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonRequest {
    /// First configuration ID
    pub config_a: Uuid,

    /// Second configuration ID
    pub config_b: Uuid,
}

/// Comparison data between two configurations
#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonData {
    /// First configuration summary
    pub config_a: ConfigSummary,

    /// Second configuration summary
    pub config_b: ConfigSummary,

    /// Category-by-category comparison
    pub categories: Vec<CategoryComparison>,
}

/// Summary of a configuration for comparison
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Display name
    pub name: String,

    /// Model name
    pub model: String,

    /// Quantization scheme
    pub quantization: String,

    /// Backend used
    pub backend: String,

    /// Hardware summary
    pub hardware: String,

    /// Overall score across all categories
    pub overall_score: f64,

    /// Performance metrics summary
    pub performance: PerformanceSummary,
}

/// Performance metrics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Generation speed (tok/s)
    pub speed: f64,

    /// Memory usage (GB)
    pub memory: f64,

    /// Model loading time (seconds)
    pub loading_time: f64,

    /// Prompt processing speed (tok/s)
    pub prompt_speed: f64,
}

/// Comparison between two configurations for a specific category
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryComparison {
    /// Category name
    pub name: String,

    /// Score for first configuration
    pub score_a: f64,

    /// Score for second configuration
    pub score_b: f64,
}

/// Detailed view of a single configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailData {
    /// Detailed configuration information
    pub config: ConfigDetail,

    /// Individual category scores
    pub categories: Vec<CategoryScore>,

    /// System information
    pub system_info: SystemInfo,
}

/// Detailed configuration information
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDetail {
    /// Display name
    pub name: String,

    /// Model name
    pub model: String,

    /// Quantization scheme
    pub quantization: String,

    /// Backend used
    pub backend: String,

    /// Backend version
    pub backend_version: String,

    /// Overall score
    pub overall_score: f64,

    /// Performance metrics
    pub performance: PerformanceSummary,

    /// When the test was run
    pub test_run_date: String,
}

/// Individual category score with details
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryScore {
    /// Category name
    pub name: String,

    /// Score percentage
    pub score: f64,

    /// Total questions
    pub total_questions: Option<i32>,

    /// Correct answers
    pub correct_answers: Option<i32>,
}

/// System information for detailed view
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// GPU model
    pub gpu_model: String,

    /// GPU memory in GB
    pub gpu_memory_gb: i32,

    /// CPU model
    pub cpu_model: String,

    /// CPU architecture
    pub cpu_arch: String,

    /// RAM amount in GB
    pub ram_gb: i32,

    /// RAM type
    pub ram_type: String,

    /// Virtualization type
    pub virtualization_type: Option<String>,

    /// Applied optimizations
    pub optimizations: Vec<String>,
}

/// List of available configurations
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationListResponse {
    /// Available configurations
    pub configurations: Vec<ExperimentSummary>,

    /// Total count (for pagination)
    pub total_count: usize,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Current timestamp
    pub timestamp: DateTime<Utc>,

    /// Database connectivity
    pub database: bool,

    /// Version information
    pub version: Option<String>,
}

/// Error response for API failures
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,

    /// Error code
    pub code: Option<String>,

    /// Additional details
    pub details: Option<serde_json::Value>,

    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,
}

impl UploadExperimentResponse {
    /// Create a successful response
    pub fn success(test_run_id: Uuid) -> Self {
        Self {
            success: true,
            test_run_id: Some(test_run_id),
            error: None,
            warnings: Vec::new(),
        }
    }

    /// Create a successful response with warnings
    pub fn success_with_warnings(test_run_id: Uuid, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            test_run_id: Some(test_run_id),
            error: None,
            warnings,
        }
    }

    /// Create a failure response
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            test_run_id: None,
            error: Some(error),
            warnings: Vec::new(),
        }
    }
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: String) -> Self {
        Self {
            error,
            code: None,
            details: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response with code
    pub fn with_code(error: String, code: String) -> Self {
        Self {
            error,
            code: Some(code),
            details: None,
            timestamp: Utc::now(),
        }
    }
}

impl HealthResponse {
    /// Create a healthy response
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            database: true,
            version: None,
        }
    }

    /// Create an unhealthy response
    pub fn unhealthy(reason: &str) -> Self {
        Self {
            status: format!("unhealthy: {}", reason),
            timestamp: Utc::now(),
            database: false,
            version: None,
        }
    }
}