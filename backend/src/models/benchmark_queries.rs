// models/benchmark_queries.rs
// Query helper functions for benchmark data across multiple tables

use sqlx::PgPool;
use uuid::Uuid;
use llm_benchmark_types::{
    BenchmarkScoreType, BenchmarkScore
};

use super::benchmark_models::*;
use super::benchmark_conversions::{
    mmlu_score_to_insert_rows, gsm8k_score_to_insert_params, 
    humaneval_score_to_insert_params, hellaswag_score_to_insert_params,
    truthfulqa_score_to_insert_params, generic_score_to_insert_params,
    mmlu_rows_to_score, gsm8k_row_to_score, humaneval_row_to_score,
    hellaswag_row_to_score, truthfulqa_row_to_score, generic_row_to_score
};

/// Get all benchmark scores for a specific test run
pub async fn get_all_benchmark_scores_for_test_run(
    db: &PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<BenchmarkScoreType>, sqlx::Error> {
    let mut scores = Vec::new();

    // Get MMLU scores
    let mmlu_rows = sqlx::query_as!(
        MMLUScoreRow,
        "SELECT id, test_run_id, category, score, total_questions, correct_answers, timestamp, context, created_at 
         FROM mmlu_scores WHERE test_run_id = $1 ORDER BY category",
        test_run_id
    ).fetch_all(db).await?;

    if !mmlu_rows.is_empty() {
        let mmlu_score = mmlu_rows_to_score(mmlu_rows);
        scores.push(BenchmarkScoreType::MMLU(mmlu_score));
    }

    // Get GSM8K scores
    let gsm8k_rows = sqlx::query_as!(
        GSM8KScoreRow,
        "SELECT id, test_run_id, problems_solved, total_problems, timestamp, context, created_at 
         FROM gsm8k_scores WHERE test_run_id = $1",
        test_run_id
    ).fetch_all(db).await?;

    for row in gsm8k_rows {
        let gsm8k_score = gsm8k_row_to_score(row);
        scores.push(BenchmarkScoreType::GSM8K(gsm8k_score));
    }

    // Get HumanEval scores
    let humaneval_rows = sqlx::query_as!(
        HumanEvalScoreRow,
        "SELECT id, test_run_id, pass_at_1, pass_at_10, pass_at_100, total_problems, timestamp, context, created_at 
         FROM humaneval_scores WHERE test_run_id = $1",
        test_run_id
    ).fetch_all(db).await?;

    for row in humaneval_rows {
        let humaneval_score = humaneval_row_to_score(row);
        scores.push(BenchmarkScoreType::HumanEval(humaneval_score));
    }

    // Get HellaSwag scores
    let hellaswag_rows = sqlx::query_as!(
        HellaSwagScoreRow,
        "SELECT id, test_run_id, accuracy, total_questions, correct_answers, timestamp, context, created_at 
         FROM hellaswag_scores WHERE test_run_id = $1",
        test_run_id
    ).fetch_all(db).await?;

    for row in hellaswag_rows {
        let hellaswag_score = hellaswag_row_to_score(row);
        scores.push(BenchmarkScoreType::HellaSwag(hellaswag_score));
    }

    // Get TruthfulQA scores
    let truthfulqa_rows = sqlx::query_as!(
        TruthfulQAScoreRow,
        "SELECT id, test_run_id, truthful_score, helpful_score, total_questions, timestamp, context, created_at 
         FROM truthfulqa_scores WHERE test_run_id = $1",
        test_run_id
    ).fetch_all(db).await?;

    for row in truthfulqa_rows {
        let truthfulqa_score = truthfulqa_row_to_score(row);
        scores.push(BenchmarkScoreType::TruthfulQA(truthfulqa_score));
    }

    // Get Generic benchmark scores
    let generic_rows = sqlx::query_as!(
        GenericBenchmarkScoreRow,
        "SELECT id, test_run_id, benchmark_name, score, total_questions, correct_answers, timestamp, context, created_at 
         FROM generic_benchmark_scores WHERE test_run_id = $1",
        test_run_id
    ).fetch_all(db).await?;

    for row in generic_rows {
        let generic_score = generic_row_to_score(row);
        scores.push(BenchmarkScoreType::Generic(generic_score));
    }

    Ok(scores)
}

/// Get aggregated benchmark scores for performance grid (overall scores only)
pub async fn get_aggregated_benchmark_scores_for_test_run(
    db: &PgPool,
    test_run_id: &Uuid,
) -> Result<f64, sqlx::Error> {
    let scores = get_all_benchmark_scores_for_test_run(db, test_run_id).await?;
    
    if scores.is_empty() {
        return Ok(0.0);
    }

    let total_score: f64 = scores.iter().map(|s| s.overall_score()).sum();
    Ok(total_score / scores.len() as f64)
}

/// Get benchmark scores for a specific benchmark type
pub async fn get_benchmark_scores_by_type(
    db: &PgPool,
    test_run_id: &Uuid,
    benchmark_type: &str,
) -> Result<Option<BenchmarkScoreType>, sqlx::Error> {
    match benchmark_type.to_lowercase().as_str() {
        "mmlu" => {
            let rows = sqlx::query_as!(
                MMLUScoreRow,
                "SELECT id, test_run_id, category, score, total_questions, correct_answers, timestamp, context, created_at 
                 FROM mmlu_scores WHERE test_run_id = $1 ORDER BY category",
                test_run_id
            ).fetch_all(db).await?;

            if rows.is_empty() {
                Ok(None)
            } else {
                let mmlu_score = mmlu_rows_to_score(rows);
                Ok(Some(BenchmarkScoreType::MMLU(mmlu_score)))
            }
        }
        "gsm8k" => {
            let row = sqlx::query_as!(
                GSM8KScoreRow,
                "SELECT id, test_run_id, problems_solved, total_problems, timestamp, context, created_at 
                 FROM gsm8k_scores WHERE test_run_id = $1",
                test_run_id
            ).fetch_optional(db).await?;

            match row {
                Some(row) => {
                    let gsm8k_score = gsm8k_row_to_score(row);
                    Ok(Some(BenchmarkScoreType::GSM8K(gsm8k_score)))
                }
                None => Ok(None)
            }
        }
        "humaneval" => {
            let row = sqlx::query_as!(
                HumanEvalScoreRow,
                "SELECT id, test_run_id, pass_at_1, pass_at_10, pass_at_100, total_problems, timestamp, context, created_at 
                 FROM humaneval_scores WHERE test_run_id = $1",
                test_run_id
            ).fetch_optional(db).await?;

            match row {
                Some(row) => {
                    let humaneval_score = humaneval_row_to_score(row);
                    Ok(Some(BenchmarkScoreType::HumanEval(humaneval_score)))
                }
                None => Ok(None)
            }
        }
        "hellaswag" => {
            let row = sqlx::query_as!(
                HellaSwagScoreRow,
                "SELECT id, test_run_id, accuracy, total_questions, correct_answers, timestamp, context, created_at 
                 FROM hellaswag_scores WHERE test_run_id = $1",
                test_run_id
            ).fetch_optional(db).await?;

            match row {
                Some(row) => {
                    let hellaswag_score = hellaswag_row_to_score(row);
                    Ok(Some(BenchmarkScoreType::HellaSwag(hellaswag_score)))
                }
                None => Ok(None)
            }
        }
        "truthfulqa" => {
            let row = sqlx::query_as!(
                TruthfulQAScoreRow,
                "SELECT id, test_run_id, truthful_score, helpful_score, total_questions, timestamp, context, created_at 
                 FROM truthfulqa_scores WHERE test_run_id = $1",
                test_run_id
            ).fetch_optional(db).await?;

            match row {
                Some(row) => {
                    let truthfulqa_score = truthfulqa_row_to_score(row);
                    Ok(Some(BenchmarkScoreType::TruthfulQA(truthfulqa_score)))
                }
                None => Ok(None)
            }
        }
        _ => {
            // Try generic benchmark scores
            let row = sqlx::query_as!(
                GenericBenchmarkScoreRow,
                "SELECT id, test_run_id, benchmark_name, score, total_questions, correct_answers, timestamp, context, created_at 
                 FROM generic_benchmark_scores WHERE test_run_id = $1 AND benchmark_name = $2",
                test_run_id, benchmark_type
            ).fetch_optional(db).await?;

            match row {
                Some(row) => {
                    let generic_score = generic_row_to_score(row);
                    Ok(Some(BenchmarkScoreType::Generic(generic_score)))
                }
                None => Ok(None)
            }
        }
    }
}

/// Insert benchmark scores into appropriate tables
pub async fn insert_benchmark_score(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    test_run_id: &Uuid,
    score: &BenchmarkScoreType,
) -> Result<(), sqlx::Error> {
    match score {
        BenchmarkScoreType::MMLU(mmlu_score) => {
            let rows = mmlu_score_to_insert_rows(mmlu_score, *test_run_id);
            for (test_run_id, category, score, total_questions, correct_answers, timestamp, context) in rows {
                sqlx::query!(
                    "INSERT INTO mmlu_scores (test_run_id, category, score, total_questions, correct_answers, timestamp, context) 
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    test_run_id, category, score, total_questions, correct_answers, timestamp, context
                ).execute(&mut **tx).await?;
            }
        }
        BenchmarkScoreType::GSM8K(gsm8k_score) => {
            let (test_run_id, problems_solved, total_problems, timestamp, context) = gsm8k_score_to_insert_params(gsm8k_score, *test_run_id);
            sqlx::query!(
                "INSERT INTO gsm8k_scores (test_run_id, problems_solved, total_problems, timestamp, context) 
                 VALUES ($1, $2, $3, $4, $5)",
                test_run_id, problems_solved, total_problems, timestamp, context
            ).execute(&mut **tx).await?;
        }
        BenchmarkScoreType::HumanEval(humaneval_score) => {
            let (test_run_id, pass_at_1, pass_at_10, pass_at_100, total_problems, timestamp, context) = humaneval_score_to_insert_params(humaneval_score, *test_run_id);
            sqlx::query!(
                "INSERT INTO humaneval_scores (test_run_id, pass_at_1, pass_at_10, pass_at_100, total_problems, timestamp, context) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                test_run_id, pass_at_1, pass_at_10, pass_at_100, total_problems, timestamp, context
            ).execute(&mut **tx).await?;
        }
        BenchmarkScoreType::HellaSwag(hellaswag_score) => {
            let (test_run_id, accuracy, total_questions, correct_answers, timestamp, context) = hellaswag_score_to_insert_params(hellaswag_score, *test_run_id);
            sqlx::query!(
                "INSERT INTO hellaswag_scores (test_run_id, accuracy, total_questions, correct_answers, timestamp, context) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                test_run_id, accuracy, total_questions, correct_answers, timestamp, context
            ).execute(&mut **tx).await?;
        }
        BenchmarkScoreType::TruthfulQA(truthfulqa_score) => {
            let (test_run_id, truthful_score, helpful_score, total_questions, timestamp, context) = truthfulqa_score_to_insert_params(truthfulqa_score, *test_run_id);
            sqlx::query!(
                "INSERT INTO truthfulqa_scores (test_run_id, truthful_score, helpful_score, total_questions, timestamp, context) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                test_run_id, truthful_score, helpful_score, total_questions, timestamp, context
            ).execute(&mut **tx).await?;
        }
        BenchmarkScoreType::Generic(generic_score) => {
            let (test_run_id, benchmark_name, score, total_questions, correct_answers, timestamp, context) = generic_score_to_insert_params(generic_score, *test_run_id);
            sqlx::query!(
                "INSERT INTO generic_benchmark_scores (test_run_id, benchmark_name, score, total_questions, correct_answers, timestamp, context) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                test_run_id, benchmark_name, score, total_questions, correct_answers, timestamp, context
            ).execute(&mut **tx).await?;
        }
    }

    Ok(())
}