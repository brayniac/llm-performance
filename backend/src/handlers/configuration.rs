// handlers/configuration.rs
// Configuration listing and detail handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use std::collections::HashMap;

use llm_benchmark_types::{
    DetailData, ErrorResponse, ExperimentSummary, ConfigurationListResponse,
    BenchmarkScore  // Import trait for overall_score method
};

use crate::{
    models::{PerformanceMetricQueryResult, benchmark_queries},
    AppState
};

/// Get list of available configurations
pub async fn get_configurations(
    State(state): State<AppState>,
) -> Result<Json<ConfigurationListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let experiments = sqlx::query!(
        r#"
        SELECT 
            tr.id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            CONCAT(hp.gpu_model, ' / ', hp.cpu_arch) as hardware_summary,
            NULL as overall_score,
            tr.timestamp,
            tr.status
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        -- Benchmark scores now handled separately
        WHERE tr.status = 'completed'
        GROUP BY tr.id, tr.model_name, tr.quantization, tr.backend, 
                 hp.gpu_model, hp.cpu_arch, tr.timestamp, tr.status
        ORDER BY tr.timestamp DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let mut configurations = Vec::new();
    for row in experiments {
        // Get aggregated benchmark score for this test run
        let overall_score = benchmark_queries::get_aggregated_benchmark_scores_for_test_run(&state.db, &row.id)
            .await
            .ok();
        
        configurations.push(ExperimentSummary {
            id: row.id,
            model_name: row.model_name,
            quantization: row.quantization,
            backend: row.backend,
            hardware_summary: row.hardware_summary.unwrap_or_default(),
            overall_score,
            timestamp: row.timestamp.unwrap_or_else(|| chrono::Utc::now()),
            status: match row.status.as_str() {
                "pending" => llm_benchmark_types::ExperimentStatus::Pending,
                "running" => llm_benchmark_types::ExperimentStatus::Running,
                "completed" => llm_benchmark_types::ExperimentStatus::Completed,
                "failed" => llm_benchmark_types::ExperimentStatus::Failed,
                "cancelled" => llm_benchmark_types::ExperimentStatus::Cancelled,
                _ => llm_benchmark_types::ExperimentStatus::Completed,
            },
        });
    }

    let total_count = configurations.len();

    Ok(Json(ConfigurationListResponse {
        configurations,
        total_count,
    }))
}

/// Get detailed information about a specific test run
pub async fn get_detail(
    Path(test_run_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DetailData>, (StatusCode, Json<ErrorResponse>)> {
    // Get detailed config data
    let (config_detail, system_info) = get_detailed_config_data(&state.db, &test_run_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching config details: {}", e))),
            )
        })?;

    // Get category scores
    let categories = get_category_scores(&state.db, &test_run_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching categories: {}", e))),
            )
        })?;

    let detail_data = DetailData {
        config: config_detail,
        categories,
        system_info,
    };

    Ok(Json(detail_data))
}

async fn get_detailed_config_data(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<(llm_benchmark_types::ConfigDetail, llm_benchmark_types::SystemInfo), sqlx::Error> {
    // Get detailed test run and hardware info
    let result = sqlx::query!(
        r#"
        SELECT 
            tr.id as test_run_id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            tr.backend_version,
            tr.timestamp,
            hp.gpu_model,
            hp.gpu_memory_gb,
            hp.cpu_model,
            hp.cpu_arch,
            hp.ram_gb,
            hp.ram_type,
            hp.virtualization_type,
            hp.optimizations
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
        result.test_run_id
    )
    .fetch_all(db)
    .await?;

    let perf_map: HashMap<String, f64> = performance_metrics
        .into_iter()
        .map(|row| (row.metric_name, row.value))
        .collect();

    // Get overall score from aggregated benchmark scores
    let overall_score = benchmark_queries::get_aggregated_benchmark_scores_for_test_run(db, &result.test_run_id)
        .await
        .unwrap_or(0.0);

    let config_detail = llm_benchmark_types::ConfigDetail {
        name: format!("{} {}", result.model_name, result.quantization),
        model: result.model_name,
        quantization: result.quantization,
        backend: result.backend,
        backend_version: result.backend_version,
        overall_score,
        performance: llm_benchmark_types::PerformanceSummary {
            speed: perf_map.get("tokens_per_second").copied().unwrap_or(0.0),
            memory: perf_map.get("memory_usage_gb").copied().unwrap_or(0.0),
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0),
            prompt_speed: perf_map.get("prompt_processing_speed").copied().unwrap_or(0.0),
        },
        test_run_date: result.timestamp.unwrap_or_else(|| chrono::Utc::now()).format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };

    let system_info = llm_benchmark_types::SystemInfo {
        gpu_model: result.gpu_model,
        gpu_memory_gb: result.gpu_memory_gb,
        cpu_model: result.cpu_model,
        cpu_arch: result.cpu_arch,
        ram_gb: result.ram_gb.unwrap_or(0),
        ram_type: result.ram_type.unwrap_or_else(|| "Unknown".to_string()),
        virtualization_type: result.virtualization_type,
        optimizations: result.optimizations.unwrap_or_default(),
    };

    Ok((config_detail, system_info))
}

async fn get_category_scores(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<llm_benchmark_types::CategoryScore>, sqlx::Error> {
    // Get all benchmark scores for this test run
    let benchmark_scores = benchmark_queries::get_all_benchmark_scores_for_test_run(db, test_run_id).await?;
    
    let mut categories = Vec::new();
    
    // Convert benchmark scores to category scores for display
    for score in benchmark_scores {
        match score {
            llm_benchmark_types::BenchmarkScoreType::MMLU(mmlu) => {
                for category in &mmlu.categories {
                    categories.push(llm_benchmark_types::CategoryScore {
                        name: format!("MMLU - {}", category.category),
                        score: category.score,
                        total_questions: Some(category.total_questions),
                        correct_answers: Some(category.correct_answers),
                    });
                }
            }
            llm_benchmark_types::BenchmarkScoreType::GSM8K(gsm8k) => {
                categories.push(llm_benchmark_types::CategoryScore {
                    name: "GSM8K".to_string(),
                    score: gsm8k.overall_score(),
                    total_questions: Some(gsm8k.total_problems),
                    correct_answers: Some(gsm8k.problems_solved),
                });
            }
            llm_benchmark_types::BenchmarkScoreType::HumanEval(humaneval) => {
                categories.push(llm_benchmark_types::CategoryScore {
                    name: "HumanEval".to_string(),
                    score: humaneval.pass_at_1,
                    total_questions: Some(humaneval.total_problems),
                    correct_answers: None,
                });
            }
            llm_benchmark_types::BenchmarkScoreType::HellaSwag(hellaswag) => {
                categories.push(llm_benchmark_types::CategoryScore {
                    name: "HellaSwag".to_string(),
                    score: hellaswag.accuracy,
                    total_questions: Some(hellaswag.total_questions),
                    correct_answers: Some(hellaswag.correct_answers),
                });
            }
            llm_benchmark_types::BenchmarkScoreType::TruthfulQA(truthfulqa) => {
                categories.push(llm_benchmark_types::CategoryScore {
                    name: "TruthfulQA".to_string(),
                    score: truthfulqa.truthful_score,
                    total_questions: Some(truthfulqa.total_questions),
                    correct_answers: None,
                });
            }
            llm_benchmark_types::BenchmarkScoreType::Generic(generic) => {
                categories.push(llm_benchmark_types::CategoryScore {
                    name: generic.benchmark_name.clone(),
                    score: generic.score,
                    total_questions: generic.total_questions,
                    correct_answers: generic.correct_answers,
                });
            }
        }
    }
    
    Ok(categories)
}