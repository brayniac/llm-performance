// handlers/experiment.rs
// Experiment upload related handlers

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

use llm_benchmark_types::{
    UploadExperimentRequest, UploadExperimentResponse, ErrorResponse, Validate
};

use crate::AppState;

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

    // Use provided experiment ID
    let test_run_id = request.experiment_run.id;
    let status_str = match request.experiment_run.status {
        llm_benchmark_types::ExperimentStatus::Pending => "pending",
        llm_benchmark_types::ExperimentStatus::Running => "running",
        llm_benchmark_types::ExperimentStatus::Completed => "completed",
        llm_benchmark_types::ExperimentStatus::Failed => "failed",
        llm_benchmark_types::ExperimentStatus::Cancelled => "cancelled",
    };

    // Insert or update test run (UPSERT)
    sqlx::query!(
        r#"
        INSERT INTO test_runs (id, model_name, quantization, backend, backend_version,
                              hardware_profile_id, timestamp, status, notes,
                              concurrent_requests, max_context_length, load_pattern,
                              dataset_name, gpu_power_limit_watts)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (id) DO UPDATE SET
            model_name = EXCLUDED.model_name,
            quantization = EXCLUDED.quantization,
            backend = EXCLUDED.backend,
            backend_version = EXCLUDED.backend_version,
            hardware_profile_id = EXCLUDED.hardware_profile_id,
            timestamp = EXCLUDED.timestamp,
            status = EXCLUDED.status,
            notes = EXCLUDED.notes,
            concurrent_requests = EXCLUDED.concurrent_requests,
            max_context_length = EXCLUDED.max_context_length,
            load_pattern = EXCLUDED.load_pattern,
            dataset_name = EXCLUDED.dataset_name,
            gpu_power_limit_watts = EXCLUDED.gpu_power_limit_watts
        "#,
        test_run_id,
        request.experiment_run.model_name,
        request.experiment_run.quantization,
        request.experiment_run.backend,
        request.experiment_run.backend_version,
        hardware_profile_id,
        request.experiment_run.timestamp,
        status_str,
        request.experiment_run.notes,
        request.experiment_run.concurrent_requests,
        request.experiment_run.max_context_length,
        request.experiment_run.load_pattern,
        request.experiment_run.dataset_name,
        request.experiment_run.gpu_power_limit_watts
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to insert test run: {}", e))),
        )
    })?;

    // Delete existing performance metrics for this test run to allow re-upload
    sqlx::query!(
        r#"
        DELETE FROM performance_metrics WHERE test_run_id = $1
        "#,
        test_run_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to delete old performance metrics: {}", e))),
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

    // Delete existing benchmark scores for this test run to allow re-upload
    sqlx::query!("DELETE FROM mmlu_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old MMLU scores: {}", e)))))?;
    sqlx::query!("DELETE FROM gsm8k_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old GSM8K scores: {}", e)))))?;
    sqlx::query!("DELETE FROM humaneval_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old HumanEval scores: {}", e)))))?;
    sqlx::query!("DELETE FROM hellaswag_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old HellaSwag scores: {}", e)))))?;
    sqlx::query!("DELETE FROM truthfulqa_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old TruthfulQA scores: {}", e)))))?;
    sqlx::query!("DELETE FROM generic_benchmark_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(format!("Failed to delete old generic scores: {}", e)))))?;

    // Insert benchmark scores
    for score in &request.experiment_run.benchmark_scores {
        crate::models::benchmark_queries::insert_benchmark_score(&mut tx, &test_run_id, score)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Failed to insert benchmark score: {}", e))),
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

async fn insert_or_find_hardware_profile(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    hardware_config: &llm_benchmark_types::HardwareConfig,
) -> Result<Uuid, sqlx::Error> {
    // Try to find existing hardware profile
    if let Ok(existing) = sqlx::query!(
        r#"
        SELECT id FROM hardware_profiles
        WHERE gpu_model = $1 AND cpu_model = $2 AND cpu_arch = $3 
              AND ((ram_gb IS NULL AND $4::INT IS NULL) OR ram_gb = $4)
              AND ((ram_type IS NULL AND $5::TEXT IS NULL) OR ram_type = $5)
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