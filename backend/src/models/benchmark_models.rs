// models/benchmark_models.rs
// Database row structs for benchmark-specific tables

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// MMLU score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MMLUScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub category: String,
    pub score: f64,
    pub total_questions: i32,
    pub correct_answers: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// GSM8K score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GSM8KScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub problems_solved: i32,
    pub total_problems: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// HumanEval score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HumanEvalScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub pass_at_1: f64,
    pub pass_at_10: Option<f64>,
    pub pass_at_100: Option<f64>,
    pub total_problems: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// HellaSwag score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HellaSwagScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub accuracy: f64,
    pub total_questions: i32,
    pub correct_answers: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// TruthfulQA score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TruthfulQAScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub truthful_score: f64,
    pub helpful_score: Option<f64>,
    pub total_questions: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Generic benchmark score database table row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GenericBenchmarkScoreRow {
    pub id: Uuid,
    pub test_run_id: Uuid,
    pub benchmark_name: String,
    pub score: f64,
    pub total_questions: Option<i32>,
    pub correct_answers: Option<i32>,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Query result for aggregated benchmark scores across all types
#[derive(Debug, sqlx::FromRow)]
pub struct AggregatedBenchmarkScoreResult {
    pub test_run_id: Uuid,
    pub benchmark_type: String,
    pub overall_score: f64,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Query result for benchmark summary (used in comparison views)
#[derive(Debug, sqlx::FromRow)]
pub struct BenchmarkSummaryResult {
    pub test_run_id: Uuid,
    pub benchmark_name: String,
    pub overall_score: f64,
    pub has_subcategories: bool,
    pub category_count: Option<i64>,
}