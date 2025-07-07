// handlers/list_test_runs.rs
// List test runs with filtering options

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTestRunsParams {
    pub model_name: Option<String>,
    pub quantization: Option<String>,
    pub benchmark_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TestRunInfo {
    pub id: Uuid,
    pub model_name: String,
    pub quantization: String,
    pub backend: String,
    pub timestamp: DateTime<Utc>,
    pub has_performance_metrics: bool,
    pub benchmarks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ListTestRunsResponse {
    pub test_runs: Vec<TestRunInfo>,
    pub total_count: usize,
}

pub async fn list_test_runs(
    Query(params): Query<ListTestRunsParams>,
    State(state): State<AppState>,
) -> Result<Json<ListTestRunsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let limit = params.limit.unwrap_or(50).min(200); // Max 200 results
    
    // Build the query
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            tr.id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            tr.timestamp,
            EXISTS(SELECT 1 FROM performance_metrics WHERE test_run_id = tr.id) as has_performance_metrics
        FROM test_runs tr
        WHERE tr.status = 'completed'
        "#
    );
    
    let mut conditions = Vec::new();
    
    if let Some(model) = &params.model_name {
        conditions.push(format!("tr.model_name ILIKE '%{}%'", model));
    }
    
    if let Some(quant) = &params.quantization {
        conditions.push(format!("tr.quantization = '{}'", quant));
    }
    
    if !conditions.is_empty() {
        query.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }
    
    query.push_str(&format!(" ORDER BY tr.timestamp DESC LIMIT {}", limit));
    
    // Execute main query
    let rows = sqlx::query_as::<_, TestRunRow>(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                })),
            )
        })?;
    
    // For each test run, get its benchmarks
    let mut test_runs = Vec::new();
    
    for row in rows {
        let benchmarks = get_benchmarks_for_test_run(&state.db, &row.id).await?;
        
        test_runs.push(TestRunInfo {
            id: row.id,
            model_name: row.model_name,
            quantization: row.quantization,
            backend: row.backend,
            timestamp: row.timestamp,
            has_performance_metrics: row.has_performance_metrics,
            benchmarks,
        });
    }
    
    let total_count = test_runs.len();
    
    Ok(Json(ListTestRunsResponse {
        test_runs,
        total_count,
    }))
}

#[derive(sqlx::FromRow)]
struct TestRunRow {
    id: Uuid,
    model_name: String,
    quantization: String,
    backend: String,
    timestamp: DateTime<Utc>,
    has_performance_metrics: bool,
}

async fn get_benchmarks_for_test_run(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<String>, (StatusCode, Json<serde_json::Value>)> {
    let mut benchmarks = Vec::new();
    
    // Check each benchmark table
    let tables = vec![
        ("mmlu_scores", "MMLU"),
        ("gsm8k_scores", "GSM8K"),
        ("humaneval_scores", "HumanEval"),
        ("hellaswag_scores", "HellaSwag"),
        ("truthfulqa_scores", "TruthfulQA"),
        ("generic_benchmark_scores", "Generic"),
    ];
    
    for (table, name) in tables {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE test_run_id = $1)", table);
        let exists: (bool,) = sqlx::query_as(&query)
            .bind(test_run_id)
            .fetch_one(db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to check {}: {}", table, e)
                    })),
                )
            })?;
            
        if exists.0 {
            benchmarks.push(name.to_string());
        }
    }
    
    Ok(benchmarks)
}