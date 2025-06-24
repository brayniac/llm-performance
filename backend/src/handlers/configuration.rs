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
    DetailData, ErrorResponse, ExperimentSummary, ConfigurationListResponse
};

use crate::{models::{PerformanceMetricQueryResult, QualityScoreQueryResult}, AppState};

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
            AVG(qs.score) as overall_score,
            tr.timestamp,
            tr.status
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN quality_scores qs ON tr.id = qs.test_run_id
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

    let configurations: Vec<ExperimentSummary> = experiments
        .into_iter()
        .map(|row| ExperimentSummary {
            id: row.id,
            model_name: row.model_name,
            quantization: row.quantization,
            backend: row.backend,
            hardware_summary: row.hardware_summary.unwrap_or_default(),
            overall_score: row.overall_score,
            timestamp: row.timestamp.unwrap_or_else(|| chrono::Utc::now()),
            status: match row.status.as_str() {
                "pending" => llm_benchmark_types::ExperimentStatus::Pending,
                "running" => llm_benchmark_types::ExperimentStatus::Running,
                "completed" => llm_benchmark_types::ExperimentStatus::Completed,
                "failed" => llm_benchmark_types::ExperimentStatus::Failed,
                "cancelled" => llm_benchmark_types::ExperimentStatus::Cancelled,
                _ => llm_benchmark_types::ExperimentStatus::Completed,
            },
        })
        .collect();

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

    // Get overall score
    let overall_score_result = sqlx::query!(
        r#"
        SELECT AVG(score) as avg_score
        FROM quality_scores
        WHERE test_run_id = $1
        "#,
        result.test_run_id
    )
    .fetch_one(db)
    .await?;

    let overall_score = overall_score_result.avg_score.unwrap_or(0.0);

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
        ram_gb: result.ram_gb,
        ram_type: result.ram_type,
        virtualization_type: result.virtualization_type,
        optimizations: result.optimizations.unwrap_or_default(),
    };

    Ok((config_detail, system_info))
}

async fn get_category_scores(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<llm_benchmark_types::CategoryScore>, sqlx::Error> {
    let categories = sqlx::query_as!(
        QualityScoreQueryResult,
        r#"
        SELECT 
            benchmark_name,
            category,
            score,
            NULL::int as total_questions,
            NULL::int as correct_answers
        FROM quality_scores
        WHERE test_run_id = $1
        ORDER BY category
        "#,
        test_run_id
    )
    .fetch_all(db)
    .await?;

    Ok(categories.into_iter().map(Into::into).collect())
}