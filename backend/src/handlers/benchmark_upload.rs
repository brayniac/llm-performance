// handlers/benchmark_upload.rs
// Handler for uploading benchmark scores for model variants

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

use llm_benchmark_types::{
    UploadBenchmarkRequest, UploadBenchmarkResponse,
    ModelVariant, benchmarks::BenchmarkScoreType,
    normalize_quantization,
};

use crate::AppState;

/// Upload benchmark scores for a model variant
pub async fn upload_benchmarks(
    State(state): State<AppState>,
    Json(request): Json<UploadBenchmarkRequest>,
) -> Result<Json<UploadBenchmarkResponse>, (StatusCode, Json<UploadBenchmarkResponse>)> {
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

    // Normalize quantization (strip redundant -GGUF suffix, etc.)
    let quantization = normalize_quantization(&request.quantization);
    let lora_adapter = request.lora_adapter.as_deref().unwrap_or("");

    // Find or create model variant
    let model_variant_id = find_or_create_model_variant(
        &mut tx,
        &request.model_name,
        &quantization,
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

    // Insert benchmark scores
    let mut scores_uploaded = 0;
    let timestamp = request.timestamp.unwrap_or_else(chrono::Utc::now);

    for score in &request.benchmark_scores {
        match score {
            BenchmarkScoreType::MMLU(mmlu_score) => {
                // Delete existing MMLU scores for this variant
                sqlx::query!(
                    "DELETE FROM mmlu_scores_v2 WHERE model_variant_id = $1",
                    model_variant_id
                )
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

                // Insert new MMLU scores
                for category_score in &mmlu_score.category_scores {
                    sqlx::query!(
                        r#"
                        INSERT INTO mmlu_scores_v2 
                        (model_variant_id, category, score, total_questions, correct_answers, timestamp, context)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        "#,
                        model_variant_id,
                        category_score.category,
                        category_score.score,
                        category_score.total_questions.map(|v| v as i32),
                        category_score.correct_answers.map(|v| v as i32),
                        timestamp,
                        mmlu_score.context
                    )
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
                scores_uploaded += mmlu_score.category_scores.len();
            }
            BenchmarkScoreType::GSM8K(gsm8k_score) => {
                // Delete existing GSM8K score
                sqlx::query!(
                    "DELETE FROM gsm8k_scores_v2 WHERE model_variant_id = $1",
                    model_variant_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to delete existing GSM8K score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;

                // Insert new GSM8K score
                let accuracy = gsm8k_score.problems_solved as f64 / gsm8k_score.total_problems as f64;
                sqlx::query!(
                    r#"
                    INSERT INTO gsm8k_scores_v2 
                    (model_variant_id, problems_solved, total_problems, accuracy, timestamp, context)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    model_variant_id,
                    gsm8k_score.problems_solved as i32,
                    gsm8k_score.total_problems as i32,
                    accuracy,
                    timestamp,
                    gsm8k_score.context
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to insert GSM8K score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;
                scores_uploaded += 1;
            }
            BenchmarkScoreType::HumanEval(humaneval_score) => {
                // Delete existing HumanEval score
                sqlx::query!(
                    "DELETE FROM humaneval_scores_v2 WHERE model_variant_id = $1",
                    model_variant_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to delete existing HumanEval score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;

                // Insert new HumanEval score
                sqlx::query!(
                    r#"
                    INSERT INTO humaneval_scores_v2 
                    (model_variant_id, pass_at_1, pass_at_10, pass_at_100, timestamp, context)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    model_variant_id,
                    humaneval_score.pass_at_1,
                    humaneval_score.pass_at_10,
                    humaneval_score.pass_at_100,
                    timestamp,
                    humaneval_score.context
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to insert HumanEval score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;
                scores_uploaded += 1;
            }
            BenchmarkScoreType::HellaSwag(hellaswag_score) => {
                // Delete existing HellaSwag score
                sqlx::query!(
                    "DELETE FROM hellaswag_scores_v2 WHERE model_variant_id = $1",
                    model_variant_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to delete existing HellaSwag score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;

                // Insert new HellaSwag score
                sqlx::query!(
                    r#"
                    INSERT INTO hellaswag_scores_v2 
                    (model_variant_id, accuracy, total_questions, correct_answers, timestamp, context)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    model_variant_id,
                    hellaswag_score.accuracy,
                    hellaswag_score.total_questions.map(|v| v as i32),
                    hellaswag_score.correct_answers.map(|v| v as i32),
                    timestamp,
                    hellaswag_score.context
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to insert HellaSwag score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;
                scores_uploaded += 1;
            }
            BenchmarkScoreType::TruthfulQA(truthfulqa_score) => {
                // Delete existing TruthfulQA score
                sqlx::query!(
                    "DELETE FROM truthfulqa_scores_v2 WHERE model_variant_id = $1",
                    model_variant_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to delete existing TruthfulQA score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;

                // Insert new TruthfulQA score
                sqlx::query!(
                    r#"
                    INSERT INTO truthfulqa_scores_v2 
                    (model_variant_id, truthful_score, truthful_and_informative_score, 
                     total_questions, timestamp, context)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    model_variant_id,
                    truthfulqa_score.truthful_score,
                    truthfulqa_score.truthful_and_informative_score,
                    truthfulqa_score.total_questions.map(|v| v as i32),
                    timestamp,
                    truthfulqa_score.context
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to insert TruthfulQA score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;
                scores_uploaded += 1;
            }
            BenchmarkScoreType::Generic(generic_score) => {
                // Delete existing generic score with same name
                sqlx::query!(
                    "DELETE FROM generic_benchmark_scores_v2 WHERE model_variant_id = $1 AND benchmark_name = $2",
                    model_variant_id,
                    generic_score.benchmark_name
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to delete existing generic score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;

                // Insert new generic score
                sqlx::query!(
                    r#"
                    INSERT INTO generic_benchmark_scores_v2 
                    (model_variant_id, benchmark_name, overall_score, sub_scores, timestamp, context)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    model_variant_id,
                    generic_score.benchmark_name,
                    generic_score.overall_score,
                    generic_score.sub_scores,
                    timestamp,
                    generic_score.context
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UploadBenchmarkResponse {
                            success: false,
                            model_variant_id: Some(model_variant_id),
                            message: format!("Failed to insert generic score: {}", e),
                            scores_uploaded,
                        }),
                    )
                })?;
                scores_uploaded += 1;
            }
        }
    }

    // Update the model variant's updated_at timestamp
    sqlx::query!(
        "UPDATE model_variants SET updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        model_variant_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadBenchmarkResponse {
                success: false,
                model_variant_id: Some(model_variant_id),
                message: format!("Failed to update model variant timestamp: {}", e),
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

async fn find_or_create_model_variant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    model_name: &str,
    quantization: &str,
    lora_adapter: &str,
) -> Result<Uuid, sqlx::Error> {
    // Try to find existing model variant
    if let Ok(existing) = sqlx::query!(
        r#"
        SELECT id FROM model_variants
        WHERE model_name = $1 AND quantization = $2 AND lora_adapter = $3
        "#,
        model_name,
        quantization,
        lora_adapter
    )
    .fetch_one(&mut **tx)
    .await
    {
        return Ok(existing.id);
    }

    // Create new model variant
    let model_variant_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO model_variants (id, model_name, quantization, lora_adapter)
        VALUES ($1, $2, $3, $4)
        "#,
        model_variant_id,
        model_name,
        quantization,
        lora_adapter
    )
    .execute(&mut **tx)
    .await?;

    Ok(model_variant_id)
}