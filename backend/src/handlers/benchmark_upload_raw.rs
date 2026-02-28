// handlers/benchmark_upload_raw.rs
// Temporary version using raw SQL queries until migration is run

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use sqlx::Row;

use llm_benchmark_types::{
    UploadBenchmarkRequest, UploadBenchmarkResponse, 
    benchmarks::BenchmarkScoreType,
};

use crate::AppState;

/// Upload benchmark scores for a model variant (raw SQL version)
pub async fn upload_benchmarks_raw(
    State(state): State<AppState>,
    Json(request): Json<UploadBenchmarkRequest>,
) -> Result<Json<UploadBenchmarkResponse>, (StatusCode, Json<UploadBenchmarkResponse>)> {
    // Check if v2 tables exist
    let tables_exist = sqlx::query(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'model_variants')"
    )
    .fetch_one(&state.db)
    .await
    .map(|row: sqlx::postgres::PgRow| row.get::<bool, _>(0))
    .unwrap_or(false);

    if !tables_exist {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(UploadBenchmarkResponse {
                success: false,
                model_variant_id: None,
                message: "Model variants tables not yet created. Please run migration 20250708000001_separate_benchmarks_from_hardware.sql".to_string(),
                scores_uploaded: 0,
            }),
        ));
    }

    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadBenchmarkResponse {
                success: false,
                model_variant_id: None,
                message: format!("Failed to start transaction: {}", e),
                scores_uploaded: 0,
            }),
        )
    })?;

    let lora_adapter = request.lora_adapter.as_deref().unwrap_or("");

    // Find or create model variant
    let model_variant_id = find_or_create_model_variant_raw(
        &mut tx,
        &request.model_name,
        &request.quantization,
        lora_adapter,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadBenchmarkResponse {
                success: false,
                model_variant_id: None,
                message: format!("Failed to create model variant: {}", e),
                scores_uploaded: 0,
            }),
        )
    })?;

    // Process benchmark scores
    let timestamp = request.timestamp.unwrap_or_else(chrono::Utc::now);
    let mut scores_uploaded = 0;

    for score in &request.benchmark_scores {
        match score {
            BenchmarkScoreType::MMLU(mmlu_score) => {
                // Delete existing MMLU scores
                sqlx::query("DELETE FROM mmlu_scores_v2 WHERE model_variant_id = $1")
                    .bind(model_variant_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UploadBenchmarkResponse {
                                success: false,
                                model_variant_id: Some(model_variant_id),
                                message: format!("Failed to delete existing MMLU scores: {}", e),
                                scores_uploaded,
                            }),
                        )
                    })?;

                // Insert new scores
                for category_score in &mmlu_score.categories {
                    sqlx::query(
                        r#"
                        INSERT INTO mmlu_scores_v2 
                        (model_variant_id, category, score, total_questions, correct_answers, timestamp, context)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        "#
                    )
                    .bind(model_variant_id)
                    .bind(&category_score.category)
                    .bind(category_score.score)
                    .bind(category_score.total_questions)
                    .bind(category_score.correct_answers)
                    .bind(timestamp)
                    .bind(&mmlu_score.context)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UploadBenchmarkResponse {
                                success: false,
                                model_variant_id: Some(model_variant_id),
                                message: format!("Failed to insert MMLU score: {}", e),
                                scores_uploaded,
                            }),
                        )
                    })?;
                }
                scores_uploaded += mmlu_score.categories.len();
            }
            _ => {
                // TODO: Implement other benchmark types
                return Err((
                    StatusCode::NOT_IMPLEMENTED,
                    Json(UploadBenchmarkResponse {
                        success: false,
                        model_variant_id: Some(model_variant_id),
                        message: "Only MMLU scores are currently supported in raw mode".to_string(),
                        scores_uploaded,
                    }),
                ));
            }
        }
    }

    // Update model variant timestamp
    sqlx::query("UPDATE model_variants SET updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(model_variant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadBenchmarkResponse {
                    success: false,
                    model_variant_id: Some(model_variant_id),
                    message: format!("Failed to update model variant: {}", e),
                    scores_uploaded,
                }),
            )
        })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadBenchmarkResponse {
                success: false,
                model_variant_id: Some(model_variant_id),
                message: format!("Failed to commit transaction: {}", e),
                scores_uploaded,
            }),
        )
    })?;

    Ok(Json(UploadBenchmarkResponse {
        success: true,
        model_variant_id: Some(model_variant_id),
        message: format!(
            "Successfully uploaded {} benchmark scores for {}/{}",
            scores_uploaded, request.model_name, request.quantization
        ),
        scores_uploaded,
    }))
}

async fn find_or_create_model_variant_raw(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    model_name: &str,
    quantization: &str,
    lora_adapter: &str,
) -> Result<Uuid, sqlx::Error> {
    // Try to find existing
    let existing = sqlx::query(
        "SELECT id FROM model_variants WHERE model_name = $1 AND quantization = $2 AND lora_adapter = $3"
    )
    .bind(model_name)
    .bind(quantization)
    .bind(lora_adapter)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(row) = existing {
        return Ok(row.get("id"));
    }

    // Create new
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO model_variants (id, model_name, quantization, lora_adapter) VALUES ($1, $2, $3, $4)"
    )
    .bind(id)
    .bind(model_name)
    .bind(quantization)
    .bind(lora_adapter)
    .execute(&mut **tx)
    .await?;

    Ok(id)
}