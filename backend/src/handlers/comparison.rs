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

use crate::{
    models::PerformanceMetricQueryResult,
    AppState
};

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

    // Get overall score from v2 benchmark scores
    let overall_score = sqlx::query!(
        r#"
        SELECT AVG(ms.score) as avg_score
        FROM mmlu_scores_v2 ms
        JOIN model_variants mv ON ms.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        "#,
        test_run.model_name,
        test_run.quantization
    )
    .fetch_one(db)
    .await
    .map(|row| row.avg_score.unwrap_or(0.0))
    .unwrap_or(0.0);

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
    use std::collections::HashMap;
    
    // Get model variants for both test runs
    let run_a = sqlx::query!(
        "SELECT model_name, quantization FROM test_runs WHERE id = $1",
        run_a_id
    )
    .fetch_one(db)
    .await?;
    
    let run_b = sqlx::query!(
        "SELECT model_name, quantization FROM test_runs WHERE id = $1",
        run_b_id
    )
    .fetch_one(db)
    .await?;
    
    
    // Build a map of category names to scores for easier comparison
    let mut scores_map: HashMap<String, (Option<f64>, Option<f64>)> = HashMap::new();
    
    // Get MMLU scores from v2 tables for both configs
    let mmlu_scores_a = sqlx::query!(
        r#"
        SELECT ms.category, ms.score
        FROM mmlu_scores_v2 ms
        JOIN model_variants mv ON ms.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        ORDER BY ms.category
        "#,
        run_a.model_name,
        run_a.quantization
    )
    .fetch_all(db)
    .await?;
    
    for row in mmlu_scores_a {
        scores_map.insert(
            format!("MMLU - {}", row.category),
            (Some(row.score), None)
        );
    }
    
    let mmlu_scores_b = sqlx::query!(
        r#"
        SELECT ms.category, ms.score
        FROM mmlu_scores_v2 ms
        JOIN model_variants mv ON ms.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        ORDER BY ms.category
        "#,
        run_b.model_name,
        run_b.quantization
    )
    .fetch_all(db)
    .await?;
    
    for row in mmlu_scores_b {
        let entry = scores_map.entry(format!("MMLU - {}", row.category))
            .or_insert((None, None));
        entry.1 = Some(row.score);
    }
    
    // Get other benchmark scores from v2 tables
    // GSM8K
    if let Ok(gsm8k_a) = sqlx::query!(
        r#"
        SELECT gs.accuracy
        FROM gsm8k_scores_v2 gs
        JOIN model_variants mv ON gs.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        run_a.model_name,
        run_a.quantization
    ).fetch_optional(db).await {
        if let Some(row) = gsm8k_a {
            scores_map.insert("GSM8K".to_string(), (Some(row.accuracy * 100.0), None));
        }
    }
    
    if let Ok(gsm8k_b) = sqlx::query!(
        r#"
        SELECT gs.accuracy
        FROM gsm8k_scores_v2 gs
        JOIN model_variants mv ON gs.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        run_b.model_name,
        run_b.quantization
    ).fetch_optional(db).await {
        if let Some(row) = gsm8k_b {
            let entry = scores_map.entry("GSM8K".to_string()).or_insert((None, None));
            entry.1 = Some(row.accuracy * 100.0);
        }
    }
    
    // Convert map to comparison vector
    let mut comparisons: Vec<llm_benchmark_types::CategoryComparison> = scores_map
        .into_iter()
        .map(|(name, (score_a, score_b))| {
            llm_benchmark_types::CategoryComparison {
                name,
                score_a: score_a.unwrap_or(0.0),
                score_b: score_b.unwrap_or(0.0),
            }
        })
        .collect();
    
    // Sort by name for consistent ordering
    comparisons.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(comparisons)
}