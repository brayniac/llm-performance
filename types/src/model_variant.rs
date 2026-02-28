use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A unique model variant (model + quantization combination)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariant {
    pub id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create or get a model variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariantRequest {
    pub model_name: String,
    pub quantization: String,
}

/// Response containing a model variant with its benchmark scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariantWithScores {
    pub variant: ModelVariant,
    pub mmlu_score: Option<f64>,
    pub gsm8k_score: Option<f64>,
    pub humaneval_score: Option<f64>,
    pub hellaswag_score: Option<f64>,
    pub truthfulqa_score: Option<f64>,
    pub benchmark_count: usize,
}

/// Request to upload benchmark scores for a model variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBenchmarkRequest {
    pub model_name: String,
    pub quantization: String,
    /// Optional LoRA adapter name. None/empty means base model.
    pub lora_adapter: Option<String>,
    pub benchmark_scores: Vec<crate::benchmarks::BenchmarkScoreType>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Response for benchmark upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBenchmarkResponse {
    pub success: bool,
    pub model_variant_id: Option<Uuid>,
    pub message: String,
    pub scores_uploaded: usize,
}