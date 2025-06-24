// backend/src/handlers.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use std::collections::HashMap;

// Import from types crate
use llm_benchmark_types::{
    PerformanceGridRow, PerformanceGridRequest, ComparisonRequest, ComparisonData,
    DetailData, UploadExperimentRequest, UploadExperimentResponse, ErrorResponse,
    ExperimentSummary, ConfigurationListResponse, Validate
};

use crate::{models::*, AppState};

/// Get performance grid data with optional filtering
pub async fn get_performance_grid(
    Query(_params): Query<PerformanceGridRequest>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceGridRow>>, (StatusCode, Json<ErrorResponse>)> {
    // Build WHERE clause based on filters - fix unused variable warning
    let _where_conditions: Vec<String> = Vec::new();
    
    // For now, we'll use a basic query without dynamic parameters
    // You can enhance this later with proper parameter binding
    let query = r#"
        SELECT 
            tr.id as test_run_id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            hp.gpu_model,
            hp.cpu_arch,
            hp.virtualization_type,
            pm_speed.value as tokens_per_second,
            pm_memory.value as memory_gb,
            AVG(qs.score) as overall_score
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN performance_metrics pm_speed ON tr.id = pm_speed.test_run_id 
            AND pm_speed.metric_name = 'tokens_per_second'
        LEFT JOIN performance_metrics pm_memory ON tr.id = pm_memory.test_run_id 
            AND pm_memory.metric_name = 'memory_usage_gb'
        LEFT JOIN quality_scores qs ON tr.id = qs.test_run_id
        WHERE tr.status = 'completed'
        GROUP BY tr.id, tr.model_name, tr.quantization, tr.backend, 
                 hp.gpu_model, hp.cpu_arch, hp.virtualization_type,
                 pm_speed.value, pm_memory.value
        ORDER BY tr.model_name, tr.quantization
        "#;

    let rows = sqlx::query_as::<_, PerformanceGridQueryResult>(query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    let grid_rows: Vec<PerformanceGridRow> = rows.into_iter().map(Into::into).collect();

    Ok(Json(grid_rows))
}

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

/// Upload a new experiment run
pub async fn upload_experiment(
    State(state): State<AppState>,
    Json(request): Json<UploadExperimentRequest>,
) -> Result<Json<UploadExperimentResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate the experiment data
    if let Err(validation_error) = request.experiment_run.validate() {
        return Ok(Json(UploadExperimentResponse::failure(
            format!("Validation error: {}", validation_error)
        )));
    }

    // Get warnings
    let warnings = request.experiment_run.warnings();

    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to start transaction: {}", e))),
        )
    })?;

    // Insert or find hardware profile
    let hardware_profile_id = insert_or_find_hardware_profile(&mut tx, &request.experiment_run.hardware_config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Hardware profile error: {}", e))),
            )
        })?;

    // Insert test run
    let test_run_id = Uuid::new_v4();
    let status_str = match request.experiment_run.status {
        llm_benchmark_types::ExperimentStatus::Pending => "pending",
        llm_benchmark_types::ExperimentStatus::Running => "running", 
        llm_benchmark_types::ExperimentStatus::Completed => "completed",
        llm_benchmark_types::ExperimentStatus::Failed => "failed",
        llm_benchmark_types::ExperimentStatus::Cancelled => "cancelled",
    };

    sqlx::query!(
        r#"
        INSERT INTO test_runs (id, model_name, quantization, backend, backend_version, 
                              hardware_profile_id, timestamp, status, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        test_run_id,
        request.experiment_run.model_name,
        request.experiment_run.quantization,
        request.experiment_run.backend,
        request.experiment_run.backend_version,
        hardware_profile_id,
        request.experiment_run.timestamp,
        status_str,
        request.experiment_run.notes
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to insert test run: {}", e))),
        )
    })?;

    // Insert performance metrics
    for metric in &request.experiment_run.performance_metrics {
        sqlx::query!(
            r#"
            INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
            VALUES ($1, $2, $3, $4)
            "#,
            test_run_id,
            metric.metric_name,
            metric.value,
            metric.unit
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Failed to insert performance metric: {}", e))),
            )
        })?;
    }

    // Insert quality scores
    for score in &request.experiment_run.quality_scores {
        sqlx::query!(
            r#"
            INSERT INTO quality_scores (test_run_id, benchmark_name, category, score)
            VALUES ($1, $2, $3, $4)
            "#,
            test_run_id,
            score.benchmark_name,
            score.category,
            score.score
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Failed to insert quality score: {}", e))),
            )
        })?;
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to commit transaction: {}", e))),
        )
    })?;

    if warnings.is_empty() {
        Ok(Json(UploadExperimentResponse::success(test_run_id)))
    } else {
        Ok(Json(UploadExperimentResponse::success_with_warnings(test_run_id, warnings)))
    }
}

// Helper functions

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

async fn insert_or_find_hardware_profile(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    hardware_config: &llm_benchmark_types::HardwareConfig,
) -> Result<Uuid, sqlx::Error> {
    // Try to find existing hardware profile
    if let Ok(existing) = sqlx::query!(
        r#"
        SELECT id FROM hardware_profiles
        WHERE gpu_model = $1 AND cpu_model = $2 AND cpu_arch = $3 
              AND ram_gb = $4 AND ram_type = $5
        "#,
        hardware_config.gpu_model,
        hardware_config.cpu_model,
        hardware_config.cpu_arch,
        hardware_config.ram_gb,
        hardware_config.ram_type
    )
    .fetch_one(&mut **tx)
    .await
    {
        return Ok(existing.id);
    }

    // Create new hardware profile
    let hardware_profile_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO hardware_profiles 
        (id, gpu_model, gpu_memory_gb, cpu_model, cpu_arch, ram_gb, ram_type, 
         virtualization_type, optimizations)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        hardware_profile_id,
        hardware_config.gpu_model,
        hardware_config.gpu_memory_gb,
        hardware_config.cpu_model,
        hardware_config.cpu_arch,
        hardware_config.ram_gb,
        hardware_config.ram_type,
        hardware_config.virtualization_type,
        &hardware_config.optimizations
    )
    .execute(&mut **tx)
    .await?;

    Ok(hardware_profile_id)
}