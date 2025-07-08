// handlers/performance_v2.rs
// Updated performance grid handler that joins hardware-specific and model-specific data

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use llm_benchmark_types::{PerformanceGridRow, ErrorResponse};

use crate::AppState;

#[derive(Deserialize)]
pub struct GridFilters {
    backend: Option<String>,
    hardware: Option<String>,
    sort_by: Option<String>,
    sort_direction: Option<String>,
}

/// Get performance grid with model variants
pub async fn get_performance_grid_v2(
    Query(filters): Query<GridFilters>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceGridRow>>, (StatusCode, Json<ErrorResponse>)> {
    // Build the query
    let mut query = String::from(
        r#"
        SELECT 
            tr.id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            tr.backend_version,
            hp.gpu_model,
            hp.cpu_arch,
            pm_speed.value as tokens_per_second,
            pm_memory.value as memory_usage_gb,
            pm_size.value as model_size_gb,
            mv.id as model_variant_id,
            -- Get benchmark scores from v2 tables
            (SELECT AVG(score) FROM mmlu_scores_v2 WHERE model_variant_id = mv.id) as mmlu_score,
            (SELECT accuracy * 100 FROM gsm8k_scores_v2 WHERE model_variant_id = mv.id LIMIT 1) as gsm8k_score,
            (SELECT pass_at_1 FROM humaneval_scores_v2 WHERE model_variant_id = mv.id LIMIT 1) as humaneval_score,
            (SELECT accuracy FROM hellaswag_scores_v2 WHERE model_variant_id = mv.id LIMIT 1) as hellaswag_score,
            (SELECT truthful_score FROM truthfulqa_scores_v2 WHERE model_variant_id = mv.id LIMIT 1) as truthfulqa_score
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN model_variants mv ON mv.model_name = tr.model_name AND mv.quantization = tr.quantization
        LEFT JOIN performance_metrics pm_speed ON pm_speed.test_run_id = tr.id 
            AND pm_speed.metric_name = 'tokens_per_second'
        LEFT JOIN performance_metrics pm_memory ON pm_memory.test_run_id = tr.id 
            AND pm_memory.metric_name = 'memory_usage_gb'
        LEFT JOIN performance_metrics pm_size ON pm_size.test_run_id = tr.id 
            AND pm_size.metric_name = 'model_size_gb'
        WHERE tr.status = 'completed'
        "#
    );

    // Add filters
    let mut conditions = Vec::new();
    
    if let Some(backend) = &filters.backend {
        conditions.push(format!("tr.backend = '{}'", backend));
    }
    
    if let Some(hardware) = &filters.hardware {
        conditions.push(format!("(hp.gpu_model LIKE '%{}%' OR hp.cpu_arch LIKE '%{}%')", hardware, hardware));
    }
    
    if !conditions.is_empty() {
        query.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }

    // Add sorting
    let sort_column = match filters.sort_by.as_deref() {
        Some("model") => "tr.model_name",
        Some("speed") => "tokens_per_second",
        Some("memory") => "memory_usage_gb",
        Some("mmlu") => "mmlu_score",
        Some("gsm8k") => "gsm8k_score",
        _ => "tokens_per_second", // Default sort
    };
    
    let sort_direction = match filters.sort_direction.as_deref() {
        Some("asc") => "ASC NULLS LAST",
        _ => "DESC NULLS LAST",
    };
    
    query.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

    // Execute query
    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

    // Convert to response format
    let mut results = Vec::new();
    
    for row in rows {
        let id: Uuid = row.get("id");
        let model_name: String = row.get("model_name");
        let quantization: String = row.get("quantization");
        let backend: String = row.get("backend");
        let backend_version: String = row.get("backend_version");
        let gpu_model: String = row.get("gpu_model");
        let cpu_arch: String = row.get("cpu_arch");
        
        // Performance metrics (hardware-specific)
        let tokens_per_second: Option<f64> = row.get("tokens_per_second");
        let memory_usage_gb: Option<f64> = row.get("memory_usage_gb");
        let model_size_gb: Option<f64> = row.get("model_size_gb");
        
        // Benchmark scores (model-specific, same for all hardware)
        let mmlu_score: Option<f64> = row.get("mmlu_score");
        let gsm8k_score: Option<f64> = row.get("gsm8k_score");
        let humaneval_score: Option<f64> = row.get("humaneval_score");
        let hellaswag_score: Option<f64> = row.get("hellaswag_score");
        let truthfulqa_score: Option<f64> = row.get("truthfulqa_score");
        
        // Calculate overall quality score (average of available benchmarks)
        let mut quality_scores = Vec::new();
        if let Some(score) = mmlu_score { quality_scores.push(score); }
        if let Some(score) = gsm8k_score { quality_scores.push(score); }
        if let Some(score) = humaneval_score { quality_scores.push(score); }
        if let Some(score) = hellaswag_score { quality_scores.push(score); }
        if let Some(score) = truthfulqa_score { quality_scores.push(score); }
        
        let overall_score = if !quality_scores.is_empty() {
            Some(quality_scores.iter().sum::<f64>() / quality_scores.len() as f64)
        } else {
            None
        };
        
        // Hardware summary
        let hardware_summary = if gpu_model.starts_with("CPU") || gpu_model == "N/A" {
            format!("CPU: {}", cpu_arch)
        } else {
            format!("{} / {}", 
                gpu_model.replace("NVIDIA GeForce ", "").replace("NVIDIA ", ""),
                cpu_arch
            )
        };
        
        results.push(PerformanceGridRow {
            id,
            model_name: model_name.clone(),
            short_name: get_short_model_name(&model_name),
            quantization,
            backend,
            backend_version,
            hardware_summary,
            tokens_per_second: tokens_per_second.unwrap_or(0.0),
            memory_gb: memory_usage_gb,
            model_size_gb,
            hardware_type: if gpu_model.starts_with("CPU") || gpu_model == "N/A" { 
                "cpu_only".to_string() 
            } else { 
                "gpu".to_string() 
            },
            overall_score,
        });
    }

    Ok(Json(results))
}

fn get_short_model_name(full_name: &str) -> String {
    // Extract the last part of the model path
    let parts: Vec<&str> = full_name.split('/').collect();
    let model_part = parts.last().unwrap_or(&full_name);
    
    // Clean up common patterns
    model_part
        .replace("-GGUF", "")
        .replace("-gguf", "")
        .replace(".gguf", "")
        .split('-')
        .filter(|part| !part.starts_with('Q') && !part.starts_with('F'))
        .collect::<Vec<&str>>()
        .join("-")
}