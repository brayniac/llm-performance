// handlers/comparison.rs
// Comparison related handlers

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use std::collections::HashMap;

use llm_benchmark_types::{
    ComparisonRequest, ComparisonData, ErrorResponse,
    BenchmarkScore  // Import trait for overall_score method
};

use crate::{
    models::{PerformanceMetricQueryResult, benchmark_queries},
    AppState
};

/// Compare two configurations
pub async fn get_comparison(
    Query(params): Query<ComparisonRequest>,
    State(state): State<AppState>,
) -> Result<Json<ComparisonData>, (StatusCode, Json<ErrorResponse>)> {
    let uuid_a = params.config_a;
    let uuid_b = params.config_b;

    // Get test run data for both configs using UUIDs
    let config_a_data = get_config_data_by_uuid(&state.db, &uuid_a).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching config A: {}", e))),
            )
        })?;
    
    let config_b_data = get_config_data_by_uuid(&state.db, &uuid_b).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching config B: {}", e))),
            )
        })?;

    // Get category comparison
    let categories = get_category_comparison(&state.db, &uuid_a, &uuid_b).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching categories: {}", e))),
            )
        })?;

    let comparison = ComparisonData {
        config_a: config_a_data,
        config_b: config_b_data,
        categories,
    };

    Ok(Json(comparison))
}

async fn get_config_data_by_uuid(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<llm_benchmark_types::ConfigSummary, sqlx::Error> {
    // Get test run data by UUID
    let test_run = sqlx::query!(
        r#"
        SELECT 
            tr.id as test_run_id, 
            tr.model_name, 
            tr.quantization, 
            tr.backend,
            tr.backend_version,
            hp.gpu_model, 
            hp.cpu_arch
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        WHERE tr.id = $1 AND tr.status = 'completed'
        "#,
        test_run_id
    )
    .fetch_one(db)
    .await?;

    // Get performance metrics
    let performance_metrics = sqlx::query_as!(
        PerformanceMetricQueryResult,
        r#"
        SELECT metric_name, value, unit
        FROM performance_metrics
        WHERE test_run_id = $1
        "#,
        test_run.test_run_id
    )
    .fetch_all(db)
    .await?;

    let perf_map: HashMap<String, f64> = performance_metrics
        .into_iter()
        .map(|row| (row.metric_name, row.value))
        .collect();

    // Get overall score from aggregated benchmark scores
    let overall_score = benchmark_queries::get_aggregated_benchmark_scores_for_test_run(db, &test_run.test_run_id)
        .await
        .unwrap_or(0.0);

    let config_summary = llm_benchmark_types::ConfigSummary {
        name: format!("{} {}", test_run.model_name, test_run.quantization),
        model: test_run.model_name,
        quantization: test_run.quantization,
        backend: test_run.backend,
        hardware: format!("{}/{}", test_run.gpu_model, test_run.cpu_arch),
        overall_score,
        performance: llm_benchmark_types::PerformanceSummary {
            speed: perf_map.get("tokens_per_second").copied().unwrap_or(0.0),
            memory: perf_map.get("memory_usage_gb").copied().unwrap_or(0.0),
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0),
            prompt_speed: perf_map.get("prompt_processing_speed").copied().unwrap_or(0.0),
        },
    };

    Ok(config_summary)
}

async fn get_category_comparison(
    db: &sqlx::PgPool,
    run_a_id: &Uuid,
    run_b_id: &Uuid,
) -> Result<Vec<llm_benchmark_types::CategoryComparison>, sqlx::Error> {
    use llm_benchmark_types::BenchmarkScoreType;
    use std::collections::HashMap;
    
    // Get benchmark scores for both test runs
    let scores_a = benchmark_queries::get_all_benchmark_scores_for_test_run(db, run_a_id).await?;
    let scores_b = benchmark_queries::get_all_benchmark_scores_for_test_run(db, run_b_id).await?;
    
    // Build a map of category names to scores for easier comparison
    let mut scores_map: HashMap<String, (Option<f64>, Option<f64>)> = HashMap::new();
    
    // Helper to add scores to the map
    let mut add_score = |name: String, score: f64, is_a: bool| {
        let entry = scores_map.entry(name).or_insert((None, None));
        if is_a {
            entry.0 = Some(score);
        } else {
            entry.1 = Some(score);
        }
    };
    
    // Process scores from test run A
    for score in scores_a {
        match score {
            BenchmarkScoreType::MMLU(mmlu) => {
                for category in &mmlu.categories {
                    add_score(format!("MMLU - {}", category.category), category.score, true);
                }
            }
            BenchmarkScoreType::GSM8K(gsm8k) => {
                add_score("GSM8K".to_string(), gsm8k.overall_score(), true);
            }
            BenchmarkScoreType::HumanEval(humaneval) => {
                add_score("HumanEval".to_string(), humaneval.pass_at_1, true);
            }
            BenchmarkScoreType::HellaSwag(hellaswag) => {
                add_score("HellaSwag".to_string(), hellaswag.accuracy, true);
            }
            BenchmarkScoreType::TruthfulQA(truthfulqa) => {
                add_score("TruthfulQA".to_string(), truthfulqa.truthful_score, true);
            }
            BenchmarkScoreType::Generic(generic) => {
                add_score(generic.benchmark_name.clone(), generic.score, true);
            }
        }
    }
    
    // Process scores from test run B
    for score in scores_b {
        match score {
            BenchmarkScoreType::MMLU(mmlu) => {
                for category in &mmlu.categories {
                    add_score(format!("MMLU - {}", category.category), category.score, false);
                }
            }
            BenchmarkScoreType::GSM8K(gsm8k) => {
                add_score("GSM8K".to_string(), gsm8k.overall_score(), false);
            }
            BenchmarkScoreType::HumanEval(humaneval) => {
                add_score("HumanEval".to_string(), humaneval.pass_at_1, false);
            }
            BenchmarkScoreType::HellaSwag(hellaswag) => {
                add_score("HellaSwag".to_string(), hellaswag.accuracy, false);
            }
            BenchmarkScoreType::TruthfulQA(truthfulqa) => {
                add_score("TruthfulQA".to_string(), truthfulqa.truthful_score, false);
            }
            BenchmarkScoreType::Generic(generic) => {
                add_score(generic.benchmark_name.clone(), generic.score, false);
            }
        }
    }
    
    // Convert map to comparison vector
    let mut comparisons: Vec<llm_benchmark_types::CategoryComparison> = scores_map
        .into_iter()
        .map(|(name, (score_a, score_b))| {
            llm_benchmark_types::CategoryComparison {
                name,
                score_a: score_a.unwrap_or(0.0),
                score_b: score_b.unwrap_or(0.0),
            }
        })
        .collect();
    
    // Sort by name for consistent ordering
    comparisons.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(comparisons)
}