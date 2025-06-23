use axum::{
    extract::{Query, State},
    response::Json,
};
use crate::{AppState, models::*};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn get_performance_grid(
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceGridRow>>, (axum::http::StatusCode, String)> {
    let query_results = sqlx::query_as!(
        PerformanceGridQueryResult,
        r#"
        SELECT 
            tr.model_name,
            tr.quantization,
            tr.backend,
            hp.gpu_model,
            hp.cpu_arch,
            hp.virtualization_type,
            pm_speed.value as tokens_per_second,
            pm_memory.value as memory_gb
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN performance_metrics pm_speed ON tr.id = pm_speed.test_run_id 
            AND pm_speed.metric_name = 'tokens_per_second'
        LEFT JOIN performance_metrics pm_memory ON tr.id = pm_memory.test_run_id 
            AND pm_memory.metric_name = 'memory_usage_gb'
        WHERE tr.status = 'completed'
        ORDER BY tr.model_name, tr.quantization
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let rows: Vec<PerformanceGridRow> = query_results
        .into_iter()
        .map(|result| {
            let hardware_type = if result.virtualization_type.is_some() {
                "optimized_vm".to_string()
            } else {
                "bare_metal".to_string()
            };

            PerformanceGridRow {
                id: format!("{}-{}-{}-{}", 
                    result.model_name.replace(" ", "-").to_lowercase(),
                    result.quantization.to_lowercase(),
                    result.cpu_arch.to_lowercase(),
                    result.gpu_model.replace(" ", "-").to_lowercase()
                ),
                model_name: result.model_name,
                quantization: result.quantization,
                backend: result.backend,
                tokens_per_second: result.tokens_per_second.unwrap_or(0.0),
                memory_gb: result.memory_gb.unwrap_or(0.0),
                gpu_model: result.gpu_model,
                cpu_arch: result.cpu_arch,
                hardware_type,
            }
        })
        .collect();

    Ok(Json(rows))
}

pub async fn get_comparison(
    Query(params): Query<ComparisonRequest>,
    State(state): State<AppState>,
) -> Result<Json<ComparisonData>, (axum::http::StatusCode, String)> {
    // Parse config IDs (format: model-quantization-cpu_arch-gpu_model)
    let config_a_parts: Vec<&str> = params.config_a.split('-').collect();
    let config_b_parts: Vec<&str> = params.config_b.split('-').collect();
    
    if config_a_parts.len() < 4 || config_b_parts.len() < 4 {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Invalid config format".to_string()));
    }

    // Get test run data for both configs
    let config_a_data = get_config_data(&state.db, &config_a_parts).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching config A: {}", e)))?;
    
    let config_b_data = get_config_data(&state.db, &config_b_parts).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching config B: {}", e)))?;

    // Get MMLU-Pro category scores
    let categories = get_category_comparison(&state.db, &config_a_data.0, &config_b_data.0).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching categories: {}", e)))?;

    let comparison = ComparisonData {
        config_a: config_a_data.1,
        config_b: config_b_data.1,
        categories,
    };

    Ok(Json(comparison))
}

async fn get_config_data(
    db: &sqlx::PgPool,
    config_parts: &[&str],
) -> Result<(Uuid, ConfigSummary), sqlx::Error> {
    // Convert parts back to original format
    let model_pattern = config_parts[0].replace("-", " ");
    let quantization = config_parts[1].to_uppercase();
    let cpu_arch = config_parts[2];
    let gpu_pattern = config_parts[3].replace("-", " ");

    // Find the test run
    let test_run = sqlx::query_as!(
        ConfigDataQueryResult,
        r#"
        SELECT tr.id as test_run_id, tr.model_name, tr.quantization, tr.backend, hp.gpu_model, hp.cpu_arch
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        WHERE tr.model_name ILIKE $1 AND tr.quantization = $2 
        AND hp.cpu_arch ILIKE $3 AND hp.gpu_model ILIKE $4
        AND tr.status = 'completed'
        ORDER BY tr.timestamp DESC
        LIMIT 1
        "#,
        format!("%{}%", model_pattern),
        quantization,
        format!("%{}%", cpu_arch),
        format!("%{}%", gpu_pattern)
    )
    .fetch_one(db)
    .await?;

    // Get performance metrics
    let performance_metrics = sqlx::query_as!(
        PerformanceMetricQueryResult,
        r#"
        SELECT metric_name, value
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

    // Get overall MMLU-Pro score
    let overall_score_result = sqlx::query!(
        r#"
        SELECT AVG(score) as avg_score
        FROM quality_scores
        WHERE test_run_id = $1 AND benchmark_name = 'mmlu_pro'
        "#,
        test_run.test_run_id
    )
    .fetch_one(db)
    .await?;

    let overall_score = overall_score_result.avg_score.unwrap_or(0.0);

    let config_summary = ConfigSummary {
        name: format!("{} {}", test_run.model_name, test_run.quantization),
        model: test_run.model_name,
        quantization: test_run.quantization,
        backend: test_run.backend,
        hardware: format!("{}/{}", test_run.gpu_model, test_run.cpu_arch),
        overall_score,
        performance: PerformanceSummary {
            speed: perf_map.get("tokens_per_second").copied().unwrap_or(0.0),
            memory: perf_map.get("memory_usage_gb").copied().unwrap_or(0.0),
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0), // Default loading time
        },
    };

    Ok((test_run.test_run_id, config_summary))
}

async fn get_category_comparison(
    db: &sqlx::PgPool,
    run_a_id: &Uuid,
    run_b_id: &Uuid,
) -> Result<Vec<CategoryComparison>, sqlx::Error> {
    let categories = sqlx::query!(
        r#"
        SELECT 
            a.category,
            a.score as score_a,
            b.score as score_b
        FROM quality_scores a
        JOIN quality_scores b ON a.category = b.category
        WHERE a.test_run_id = $1 AND b.test_run_id = $2
        AND a.benchmark_name = 'mmlu_pro' AND b.benchmark_name = 'mmlu_pro'
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
            WHERE test_run_id = $1 AND benchmark_name = 'mmlu_pro'
            ORDER BY category
            "#,
            run_a_id
        )
        .fetch_all(db)
        .await?;

        return Ok(categories_a
            .into_iter()
            .map(|row| CategoryComparison {
                name: row.category,
                score_a: row.score,
                score_b: 0.0, // Default if no matching category
            })
            .collect());
    }

    Ok(categories
        .into_iter()
        .map(|row| CategoryComparison {
            name: row.category,
            score_a: row.score_a,
            score_b: row.score_b,
        })
        .collect())
}

pub async fn get_configurations(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, (axum::http::StatusCode, String)> {
    let configs = sqlx::query!(
        r#"
        SELECT DISTINCT 
            CONCAT(
                LOWER(REPLACE(tr.model_name, ' ', '-')), '-',
                LOWER(tr.quantization), '-',
                LOWER(hp.cpu_arch), '-',
                LOWER(REPLACE(hp.gpu_model, ' ', '-'))
            ) as config_id
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        WHERE tr.status = 'completed'
        ORDER BY config_id
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let config_ids: Vec<String> = configs
        .into_iter()
        .filter_map(|row| row.config_id)
        .collect();
    Ok(Json(config_ids))
}