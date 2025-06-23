use axum::{
    extract::{Query, State, Path},
    response::Json,
};
use crate::{AppState, models::*};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn get_performance_grid(
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceGridRow>>, (axum::http::StatusCode, String)> {
    let query_results = sqlx::query!(
        r#"
        SELECT 
            tr.id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            hp.gpu_model,
            hp.cpu_arch,
            hp.virtualization_type,
            pm_speed.value as "tokens_per_second: Option<f64>",
            pm_memory.value as "memory_gb: Option<f64>"
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
                id: result.id.to_string(), // Use UUID as string
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
    // Parse UUIDs from strings
    let uuid_a = Uuid::parse_str(&params.config_a)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "Invalid UUID format for config_a".to_string()))?;
    
    let uuid_b = Uuid::parse_str(&params.config_b)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "Invalid UUID format for config_b".to_string()))?;

    // Get test run data for both configs using UUIDs
    let config_a_data = get_config_data_by_uuid(&state.db, &uuid_a).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching config A: {}", e)))?;
    
    let config_b_data = get_config_data_by_uuid(&state.db, &uuid_b).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching config B: {}", e)))?;

    // Get MMLU-Pro category scores
    let categories = get_category_comparison(&state.db, &uuid_a, &uuid_b).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching categories: {}", e)))?;

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
) -> Result<ConfigSummary, sqlx::Error> {
    // Get test run data by UUID
    let test_run = sqlx::query!(
        r#"
        SELECT tr.id as test_run_id, tr.model_name, tr.quantization, tr.backend, hp.gpu_model, hp.cpu_arch
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
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0),
        },
    };

    Ok(config_summary)
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

pub async fn get_detail(
    Path(test_run_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DetailData>, (axum::http::StatusCode, String)> {
    // Parse UUID from string
    let uuid = Uuid::parse_str(&test_run_id)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "Invalid UUID format".to_string()))?;

    // Get detailed config data
    let (config_detail, system_info) = get_detailed_config_data(&state.db, &uuid).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching config details: {}", e)))?;

    // Get category scores
    let categories = get_category_scores(&state.db, &uuid).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching categories: {}", e)))?;

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
) -> Result<(ConfigDetail, SystemInfo), sqlx::Error> {
    // Get detailed test run and hardware info by UUID - much simpler!
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
        SELECT metric_name, value
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

    // Get overall MMLU-Pro score
    let overall_score_result = sqlx::query!(
        r#"
        SELECT AVG(score) as avg_score
        FROM quality_scores
        WHERE test_run_id = $1 AND benchmark_name = 'mmlu_pro'
        "#,
        result.test_run_id
    )
    .fetch_one(db)
    .await?;

    let overall_score = overall_score_result.avg_score.unwrap_or(0.0);

    let config_detail = ConfigDetail {
        name: format!("{} {}", result.model_name, result.quantization),
        model: result.model_name,
        quantization: result.quantization,
        backend: result.backend,
        backend_version: result.backend_version,
        overall_score,
        performance: PerformanceSummary {
            speed: perf_map.get("tokens_per_second").copied().unwrap_or(0.0),
            memory: perf_map.get("memory_usage_gb").copied().unwrap_or(0.0),
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0),
        },
        test_run_date: result.timestamp
            .map(|ts| ts.format("%Y-%m-%d %H:%M UTC").to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
    };

    let system_info = SystemInfo {
        gpu_model: result.gpu_model,
        gpu_memory_gb: result.gpu_memory_gb,
        cpu_model: result.cpu_model,
        cpu_arch: result.cpu_arch,
        ram_gb: result.ram_gb,
        ram_type: result.ram_type,
        virtualization_type: result.virtualization_type,
        optimizations: result.optimizations.unwrap_or_else(|| vec![]),
    };

    Ok((config_detail, system_info))
}

async fn get_category_scores(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<CategoryScore>, sqlx::Error> {
    let categories = sqlx::query!(
        r#"
        SELECT category, score, total_questions, correct_answers
        FROM quality_scores
        WHERE test_run_id = $1 AND benchmark_name = 'mmlu_pro'
        ORDER BY category
        "#,
        test_run_id
    )
    .fetch_all(db)
    .await?;

    Ok(categories
        .into_iter()
        .map(|row| CategoryScore {
            name: row.category,
            score: row.score,
            total_questions: row.total_questions,
            correct_answers: row.correct_answers,
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