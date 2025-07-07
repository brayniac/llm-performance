// handlers/delete.rs
// Handler for deleting test runs and related data

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
    pub deleted_id: Option<Uuid>,
}

/// Delete a test run and all related data
pub async fn delete_test_run(
    Path(test_run_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<DeleteResponse>)> {
    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Failed to start transaction: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    // Check if the test run exists
    let exists = sqlx::query!(
        "SELECT id FROM test_runs WHERE id = $1",
        test_run_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Database error: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(DeleteResponse {
                success: false,
                message: format!("Test run {} not found", test_run_id),
                deleted_id: None,
            }),
        ));
    }

    // Delete benchmark scores (they reference test_run_id)
    // Due to CASCADE, these should be deleted automatically, but let's be explicit
    sqlx::query!("DELETE FROM mmlu_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete MMLU scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    sqlx::query!("DELETE FROM gsm8k_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete GSM8K scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    sqlx::query!("DELETE FROM humaneval_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete HumanEval scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    sqlx::query!("DELETE FROM hellaswag_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete HellaSwag scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    sqlx::query!("DELETE FROM truthfulqa_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete TruthfulQA scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    sqlx::query!("DELETE FROM generic_benchmark_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete generic benchmark scores: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    // Delete performance metrics
    sqlx::query!("DELETE FROM performance_metrics WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete performance metrics: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    // Finally, delete the test run itself
    sqlx::query!("DELETE FROM test_runs WHERE id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete test run: {}", e),
                    deleted_id: None,
                }),
            )
        })?;

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Failed to commit transaction: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("Successfully deleted test run {}", test_run_id),
        deleted_id: Some(test_run_id),
    }))
}

/// Delete all test runs for a specific model and quantization
pub async fn delete_by_model_quant(
    State(state): State<AppState>,
    Json(request): Json<DeleteByModelQuantRequest>,
) -> Result<Json<DeleteMultipleResponse>, (StatusCode, Json<DeleteMultipleResponse>)> {
    // Find all matching test runs
    let test_runs = sqlx::query!(
        "SELECT id FROM test_runs WHERE model_name = $1 AND quantization = $2",
        request.model_name,
        request.quantization
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteMultipleResponse {
                success: false,
                message: format!("Failed to find test runs: {}", e),
                deleted_count: 0,
                deleted_ids: vec![],
            }),
        )
    })?;

    let mut deleted_ids = Vec::new();
    let mut errors = Vec::new();

    // Delete each test run
    for row in test_runs {
        match delete_test_run_internal(&state.db, row.id).await {
            Ok(_) => deleted_ids.push(row.id),
            Err(e) => errors.push(format!("Failed to delete {}: {}", row.id, e)),
        }
    }

    if !errors.is_empty() {
        return Err((
            StatusCode::PARTIAL_CONTENT,
            Json(DeleteMultipleResponse {
                success: false,
                message: format!("Some deletions failed: {}", errors.join(", ")),
                deleted_count: deleted_ids.len(),
                deleted_ids,
            }),
        ));
    }

    Ok(Json(DeleteMultipleResponse {
        success: true,
        message: format!(
            "Successfully deleted {} test runs for {}/{}",
            deleted_ids.len(),
            request.model_name,
            request.quantization
        ),
        deleted_count: deleted_ids.len(),
        deleted_ids,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DeleteByModelQuantRequest {
    pub model_name: String,
    pub quantization: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteMultipleResponse {
    pub success: bool,
    pub message: String,
    pub deleted_count: usize,
    pub deleted_ids: Vec<Uuid>,
}

// Internal helper function to delete a test run
async fn delete_test_run_internal(db: &sqlx::PgPool, test_run_id: Uuid) -> Result<(), String> {
    let mut tx = db.begin().await.map_err(|e| e.to_string())?;

    // Delete all related data
    sqlx::query!("DELETE FROM mmlu_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM gsm8k_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM humaneval_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM hellaswag_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM truthfulqa_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM generic_benchmark_scores WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM performance_metrics WHERE test_run_id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query!("DELETE FROM test_runs WHERE id = $1", test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete specific benchmark scores from a test run
pub async fn delete_benchmark_scores(
    Path(test_run_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<DeleteBenchmarkRequest>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<DeleteResponse>)> {
    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Failed to start transaction: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    // Check if the test run exists
    let exists = sqlx::query!(
        "SELECT id FROM test_runs WHERE id = $1",
        test_run_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Database error: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(DeleteResponse {
                success: false,
                message: format!("Test run {} not found", test_run_id),
                deleted_id: None,
            }),
        ));
    }

    // Delete the specific benchmark scores
    let table_name = match request.benchmark_type.to_lowercase().as_str() {
        "mmlu" => "mmlu_scores",
        "gsm8k" => "gsm8k_scores",
        "humaneval" => "humaneval_scores",
        "hellaswag" => "hellaswag_scores",
        "truthfulqa" => "truthfulqa_scores",
        "generic" => "generic_benchmark_scores",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Invalid benchmark type: {}", request.benchmark_type),
                    deleted_id: None,
                }),
            ));
        }
    };

    // Execute the delete query
    let query = format!("DELETE FROM {} WHERE test_run_id = $1", table_name);
    let rows_affected = sqlx::query(&query)
        .bind(test_run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    message: format!("Failed to delete {} scores: {}", request.benchmark_type, e),
                    deleted_id: None,
                }),
            )
        })?
        .rows_affected();

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                message: format!("Failed to commit transaction: {}", e),
                deleted_id: None,
            }),
        )
    })?;

    Ok(Json(DeleteResponse {
        success: true,
        message: format!(
            "Successfully deleted {} {} scores from test run {}",
            rows_affected, request.benchmark_type, test_run_id
        ),
        deleted_id: Some(test_run_id),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DeleteBenchmarkRequest {
    pub benchmark_type: String,
}