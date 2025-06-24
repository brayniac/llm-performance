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
    ComparisonRequest, ComparisonData, ErrorResponse
};

use crate::{models::PerformanceMetricQueryResult, AppState};

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

    // Get overall score
    let overall_score_result = sqlx::query!(
        r#"
        SELECT AVG(score) as avg_score
        FROM quality_scores
        WHERE test_run_id = $1
        "#,
        test_run.test_run_id
    )
    .fetch_one(db)
    .await?;

    let overall_score = overall_score_result.avg_score.unwrap_or(0.0);

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
    let categories = sqlx::query!(
        r#"
        SELECT 
            a.category,
            a.score as score_a,
            b.score as score_b
        FROM quality_scores a
        JOIN quality_scores b ON a.category = b.category
        WHERE a.test_run_id = $1 AND b.test_run_id = $2
        ORDER BY a.category
        "#,
        run_a_id, run_b_id
    )
    .fetch_all(db)
    .await?;

    // Handle the case where we don't have matching categories
    if categories.is_empty() {
        // Get categories from run A only as fallback
        let categories_a = sqlx::query!(
            r#"
            SELECT category, score
            FROM quality_scores
            WHERE test_run_id = $1
            ORDER BY category
            "#,
            run_a_id
        )
        .fetch_all(db)
        .await?;

        return Ok(categories_a
            .into_iter()
            .map(|row| llm_benchmark_types::CategoryComparison {
                name: row.category,
                score_a: row.score,
                score_b: 0.0, // Default if no matching category
            })
            .collect());
    }

    Ok(categories
        .into_iter()
        .map(|row| llm_benchmark_types::CategoryComparison {
            name: row.category,
            score_a: row.score_a,
            score_b: row.score_b,
        })
        .collect())
}