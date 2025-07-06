// models/benchmark_conversions.rs
// Conversion functions between database rows and types crate benchmark scores

use chrono::Utc;
use llm_benchmark_types::{
    MMLUScore, MMLUCategoryScore, GSM8KScore, HumanEvalScore, 
    HellaSwagScore, TruthfulQAScore, GenericBenchmarkScore, BenchmarkScoreType
};

use super::benchmark_models::*;

// Helper functions for converting database rows to types crate structs

pub fn mmlu_rows_to_score(rows: Vec<MMLUScoreRow>) -> MMLUScore {
    let categories = rows.into_iter().map(|row| {
        MMLUCategoryScore {
            category: row.category,
            score: row.score,
            total_questions: row.total_questions,
            correct_answers: row.correct_answers,
        }
    }).collect();

    MMLUScore {
        categories,
        timestamp: Utc::now(), // Use current time as aggregate timestamp
        context: None,
    }
}

pub fn gsm8k_row_to_score(row: GSM8KScoreRow) -> GSM8KScore {
    GSM8KScore {
        problems_solved: row.problems_solved,
        total_problems: row.total_problems,
        timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
        context: row.context,
    }
}

pub fn humaneval_row_to_score(row: HumanEvalScoreRow) -> HumanEvalScore {
    HumanEvalScore {
        pass_at_1: row.pass_at_1,
        pass_at_10: row.pass_at_10,
        pass_at_100: row.pass_at_100,
        total_problems: row.total_problems,
        timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
        context: row.context,
    }
}

pub fn hellaswag_row_to_score(row: HellaSwagScoreRow) -> HellaSwagScore {
    HellaSwagScore {
        accuracy: row.accuracy,
        total_questions: row.total_questions,
        correct_answers: row.correct_answers,
        timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
        context: row.context,
    }
}

pub fn truthfulqa_row_to_score(row: TruthfulQAScoreRow) -> TruthfulQAScore {
    TruthfulQAScore {
        truthful_score: row.truthful_score,
        helpful_score: row.helpful_score,
        total_questions: row.total_questions,
        timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
        context: row.context,
    }
}

pub fn generic_row_to_score(row: GenericBenchmarkScoreRow) -> GenericBenchmarkScore {
    GenericBenchmarkScore {
        benchmark_name: row.benchmark_name,
        score: row.score,
        total_questions: row.total_questions,
        correct_answers: row.correct_answers,
        timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
        context: row.context,
    }
}

// Helper functions for converting benchmark scores to database insert parameters

pub fn mmlu_score_to_insert_rows(
    score: &MMLUScore, 
    test_run_id: uuid::Uuid
) -> Vec<(uuid::Uuid, String, f64, i32, i32, chrono::DateTime<Utc>, Option<serde_json::Value>)> {
    score.categories.iter().map(|category| {
        (
            test_run_id,
            category.category.clone(),
            category.score,
            category.total_questions,
            category.correct_answers,
            score.timestamp,
            score.context.clone(),
        )
    }).collect()
}

pub fn gsm8k_score_to_insert_params(
    score: &GSM8KScore, 
    test_run_id: uuid::Uuid
) -> (uuid::Uuid, i32, i32, chrono::DateTime<Utc>, Option<serde_json::Value>) {
    (
        test_run_id,
        score.problems_solved,
        score.total_problems,
        score.timestamp,
        score.context.clone(),
    )
}

pub fn humaneval_score_to_insert_params(
    score: &HumanEvalScore, 
    test_run_id: uuid::Uuid
) -> (uuid::Uuid, f64, Option<f64>, Option<f64>, i32, chrono::DateTime<Utc>, Option<serde_json::Value>) {
    (
        test_run_id,
        score.pass_at_1,
        score.pass_at_10,
        score.pass_at_100,
        score.total_problems,
        score.timestamp,
        score.context.clone(),
    )
}

pub fn hellaswag_score_to_insert_params(
    score: &HellaSwagScore, 
    test_run_id: uuid::Uuid
) -> (uuid::Uuid, f64, i32, i32, chrono::DateTime<Utc>, Option<serde_json::Value>) {
    (
        test_run_id,
        score.accuracy,
        score.total_questions,
        score.correct_answers,
        score.timestamp,
        score.context.clone(),
    )
}

pub fn truthfulqa_score_to_insert_params(
    score: &TruthfulQAScore, 
    test_run_id: uuid::Uuid
) -> (uuid::Uuid, f64, Option<f64>, i32, chrono::DateTime<Utc>, Option<serde_json::Value>) {
    (
        test_run_id,
        score.truthful_score,
        score.helpful_score,
        score.total_questions,
        score.timestamp,
        score.context.clone(),
    )
}

pub fn generic_score_to_insert_params(
    score: &GenericBenchmarkScore, 
    test_run_id: uuid::Uuid
) -> (uuid::Uuid, String, f64, Option<i32>, Option<i32>, chrono::DateTime<Utc>, Option<serde_json::Value>) {
    (
        test_run_id,
        score.benchmark_name.clone(),
        score.score,
        score.total_questions,
        score.correct_answers,
        score.timestamp,
        score.context.clone(),
    )
}

// Helper function to determine benchmark type from BenchmarkScoreType
pub fn get_benchmark_type_name(score: &BenchmarkScoreType) -> String {
    match score {
        BenchmarkScoreType::MMLU(_) => "mmlu".to_string(),
        BenchmarkScoreType::GSM8K(_) => "gsm8k".to_string(),
        BenchmarkScoreType::HumanEval(_) => "humaneval".to_string(),
        BenchmarkScoreType::HellaSwag(_) => "hellaswag".to_string(),
        BenchmarkScoreType::TruthfulQA(_) => "truthfulqa".to_string(),
        BenchmarkScoreType::Generic(score) => score.benchmark_name.clone(),
    }
}