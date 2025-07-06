// handlers/performance.rs
// Performance grid related handlers

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};

use llm_benchmark_types::{
    PerformanceGridRow, PerformanceGridRequest, ErrorResponse
};

use crate::{
    models::{PerformanceGridQueryResult, benchmark_queries},
    AppState
};

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
            NULL as overall_score
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN performance_metrics pm_speed ON tr.id = pm_speed.test_run_id 
            AND pm_speed.metric_name = 'tokens_per_second'
        LEFT JOIN performance_metrics pm_memory ON tr.id = pm_memory.test_run_id 
            AND pm_memory.metric_name = 'memory_usage_gb'
        -- Benchmark scores now handled separately
        WHERE tr.status = 'completed'
        -- No GROUP BY needed without aggregation
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

    // Get benchmark scores for each row
    let mut grid_rows = Vec::new();
    for row in rows {
        let overall_score = benchmark_queries::get_aggregated_benchmark_scores_for_test_run(&state.db, &row.test_run_id)
            .await
            .ok();
        
        let mut grid_row: PerformanceGridRow = row.into();
        grid_row.overall_score = overall_score;
        grid_rows.push(grid_row);
    }

    Ok(Json(grid_rows))
}