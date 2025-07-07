// handlers/grouped_performance.rs
// Handler for grouped model performance view with quality-based filtering

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use llm_benchmark_types::{
    GroupedPerformanceRequest, GroupedPerformanceResponse, 
    ModelPerformanceGroup, QuantizationPerformance, ErrorResponse,
};

use crate::AppState;

/// Get grouped model performance with best quantization per model
pub async fn get_grouped_performance(
    Query(params): Query<GroupedPerformanceRequest>,
    State(state): State<AppState>,
) -> Result<Json<GroupedPerformanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Default to MMLU if no benchmark specified
    let benchmark = params.benchmark.as_deref().unwrap_or("mmlu");
    
    // Get all test runs with their performance metrics and quality scores
    let query = r#"
        WITH test_run_data AS (
            SELECT 
                tr.id,
                tr.model_name,
                tr.quantization,
                tr.backend,
                pm_speed.value as tokens_per_second,
                pm_memory.value as memory_gb,
                CONCAT(hp.gpu_model, ' / ', hp.cpu_arch) as hardware,
                CASE 
                    WHEN $1 = 'mmlu' THEN (
                        SELECT AVG(score) FROM mmlu_scores WHERE test_run_id = tr.id
                    )
                    WHEN $1 = 'gsm8k' THEN (
                        SELECT (CAST(problems_solved AS FLOAT) / CAST(total_problems AS FLOAT)) * 100
                        FROM gsm8k_scores WHERE test_run_id = tr.id LIMIT 1
                    )
                    WHEN $1 = 'humaneval' THEN (
                        SELECT pass_at_1 FROM humaneval_scores WHERE test_run_id = tr.id LIMIT 1
                    )
                    WHEN $1 = 'hellaswag' THEN (
                        SELECT accuracy FROM hellaswag_scores WHERE test_run_id = tr.id LIMIT 1
                    )
                    WHEN $1 = 'truthfulqa' THEN (
                        SELECT truthful_score FROM truthfulqa_scores WHERE test_run_id = tr.id LIMIT 1
                    )
                    WHEN $1 = 'none' THEN NULL
                    ELSE NULL
                END as quality_score
            FROM test_runs tr
            JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
            LEFT JOIN performance_metrics pm_speed ON pm_speed.test_run_id = tr.id 
                AND pm_speed.metric_name = 'tokens_per_second'
            LEFT JOIN performance_metrics pm_memory ON pm_memory.test_run_id = tr.id 
                AND pm_memory.metric_name = 'memory_usage_gb'
            WHERE tr.status = 'completed'
        )
        SELECT * FROM test_run_data
        ORDER BY model_name, quality_score DESC NULLS LAST
    "#;

    let rows = sqlx::query(query)
        .bind(benchmark)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    // Group by model and apply filters
    let mut model_groups: HashMap<String, Vec<QuantizationPerformance>> = HashMap::new();
    let mut total_quants_by_model: HashMap<String, usize> = HashMap::new();
    
    for row in rows {
        let model_name: String = row.get("model_name");
        let tokens_per_second: Option<f64> = row.get("tokens_per_second");
        let memory_gb: Option<f64> = row.get("memory_gb");
        let quality_score: Option<f64> = row.get("quality_score");
        
        // Count total quantizations for this model
        *total_quants_by_model.entry(model_name.clone()).or_insert(0) += 1;
        
        // Apply filters
        if let Some(min_speed) = params.min_speed {
            match tokens_per_second {
                Some(speed) if speed < min_speed => continue,
                None => continue, // Skip if no speed data when filter is set
                _ => {}
            }
        }
        
        if let Some(max_memory) = params.max_memory_gb {
            match memory_gb {
                Some(memory) if memory > max_memory => continue,
                None => {} // Don't filter out if no memory data
                _ => {}
            }
        }
        
        if let Some(min_quality) = params.min_quality {
            if let Some(score) = quality_score {
                if score < min_quality {
                    continue;
                }
            } else {
                // No quality score, skip if min_quality filter is set
                continue;
            }
        }
        
        let quant_perf = QuantizationPerformance {
            id: row.get("id"),
            quantization: row.get("quantization"),
            quality_score: quality_score.unwrap_or(0.0), // Default to 0 if no score
            tokens_per_second: tokens_per_second.unwrap_or(0.0), // Default to 0 if no perf data
            memory_gb: memory_gb.unwrap_or(0.0), // Default to 0 if no memory data
            backend: row.get("backend"),
            hardware: row.get("hardware"),
        };
        
        model_groups.entry(model_name.clone())
            .or_insert_with(Vec::new)
            .push(quant_perf);
    }
    
    // Convert to response format - take best quantization per model
    let mut models: Vec<ModelPerformanceGroup> = model_groups
        .into_iter()
        .filter_map(|(model_name, mut quants)| {
            if quants.is_empty() {
                return None;
            }
            
            // Sort by quality score (already sorted by query, but just to be sure)
            quants.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
            
            let qualifying_count = quants.len();
            let best_quant = quants.into_iter().next().unwrap();
            
            Some(ModelPerformanceGroup {
                model_name: model_name.clone(),
                best_quantization: best_quant,
                total_quantizations: *total_quants_by_model.get(&model_name).unwrap_or(&qualifying_count),
                qualifying_quantizations: qualifying_count,
                all_quantizations: None, // Client can request this separately if needed
            })
        })
        .collect();
    
    // Apply sorting
    match params.sort_by.as_deref() {
        Some("quality") => {
            models.sort_by(|a, b| {
                let cmp = b.best_quantization.quality_score
                    .partial_cmp(&a.best_quantization.quality_score)
                    .unwrap();
                if params.sort_direction.as_deref() == Some("asc") {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
        Some("speed") => {
            models.sort_by(|a, b| {
                let cmp = b.best_quantization.tokens_per_second
                    .partial_cmp(&a.best_quantization.tokens_per_second)
                    .unwrap();
                if params.sort_direction.as_deref() == Some("asc") {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
        Some("memory") => {
            models.sort_by(|a, b| {
                let cmp = a.best_quantization.memory_gb
                    .partial_cmp(&b.best_quantization.memory_gb)
                    .unwrap();
                if params.sort_direction.as_deref() == Some("desc") {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
        Some("model_name") => {
            models.sort_by(|a, b| {
                let cmp = a.model_name.cmp(&b.model_name);
                if params.sort_direction.as_deref() == Some("desc") {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
        _ => {
            // Default: sort by quality score descending
            models.sort_by(|a, b| {
                b.best_quantization.quality_score
                    .partial_cmp(&a.best_quantization.quality_score)
                    .unwrap()
            });
        }
    }
    
    let total_count = models.len();
    
    Ok(Json(GroupedPerformanceResponse {
        models,
        total_count,
        benchmark_used: benchmark.to_string(),
    }))
}