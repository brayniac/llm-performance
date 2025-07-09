// handlers/grouped_performance.rs
// Handler for grouped model performance view with quality-based filtering

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use sqlx::Row;
use std::collections::HashMap;

use llm_benchmark_types::{
    GroupedPerformanceRequest, GroupedPerformanceResponse, 
    ModelPerformanceGroup, QuantizationPerformance, ErrorResponse,
    hardware::HardwareCategory,
};

use crate::AppState;

/// Determine hardware category from GPU and CPU model strings
fn determine_hardware_category(gpu_model: &str, cpu_model: &str) -> HardwareCategory {
    // Check GPU first
    if gpu_model.contains("RTX") || gpu_model.contains("GTX") {
        HardwareCategory::ConsumerGpu
    } else if gpu_model.contains("A100") || gpu_model.contains("H100") 
        || gpu_model.contains("L4") || gpu_model.contains("L40")
        || gpu_model.contains("V100") || gpu_model.contains("T4") {
        HardwareCategory::DatacenterGpu
    } else if gpu_model == "CPU Only" || gpu_model == "N/A" || gpu_model.starts_with("CPU") {
        // CPU only - check CPU model
        if cpu_model.contains("Xeon") || cpu_model.contains("EPYC") {
            HardwareCategory::DatacenterCpu
        } else {
            HardwareCategory::ConsumerCpu
        }
    } else {
        // Unknown GPU, default to consumer
        HardwareCategory::ConsumerGpu
    }
}

/// Get grouped model performance with best quantization per model
pub async fn get_grouped_performance(
    Query(params): Query<GroupedPerformanceRequest>,
    State(state): State<AppState>,
) -> Result<Json<GroupedPerformanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Default to MMLU if no benchmark specified
    let benchmark = params.benchmark.as_deref().unwrap_or("mmlu");
    
    // Parse hardware categories from comma-separated string
    let filter_categories: Vec<HardwareCategory> = if let Some(ref categories_str) = params.hardware_categories {
        categories_str
            .split(',')
            .filter_map(|s| match s.trim() {
                "consumer_gpu" => Some(HardwareCategory::ConsumerGpu),
                "consumer_cpu" => Some(HardwareCategory::ConsumerCpu),
                "datacenter_gpu" => Some(HardwareCategory::DatacenterGpu),
                "datacenter_cpu" => Some(HardwareCategory::DatacenterCpu),
                _ => None,
            })
            .collect()
    } else {
        Vec::new()
    };
    
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
                CONCAT(hp.gpu_model, ' / ', hp.cpu_model) as hardware,
                hp.gpu_model,
                hp.cpu_arch,
                hp.cpu_model,
                CASE 
                    WHEN $1 = 'mmlu' THEN (
                        SELECT AVG(ms.score) 
                        FROM mmlu_scores_v2 ms
                        JOIN model_variants mv ON ms.model_variant_id = mv.id
                        WHERE mv.model_name = tr.model_name AND mv.quantization = tr.quantization
                    )
                    WHEN $1 = 'gsm8k' THEN (
                        SELECT gs.accuracy * 100
                        FROM gsm8k_scores_v2 gs
                        JOIN model_variants mv ON gs.model_variant_id = mv.id
                        WHERE mv.model_name = tr.model_name AND mv.quantization = tr.quantization
                        LIMIT 1
                    )
                    WHEN $1 = 'humaneval' THEN (
                        SELECT hs.pass_at_1
                        FROM humaneval_scores_v2 hs
                        JOIN model_variants mv ON hs.model_variant_id = mv.id
                        WHERE mv.model_name = tr.model_name AND mv.quantization = tr.quantization
                        LIMIT 1
                    )
                    WHEN $1 = 'hellaswag' THEN (
                        SELECT hs.accuracy
                        FROM hellaswag_scores_v2 hs
                        JOIN model_variants mv ON hs.model_variant_id = mv.id
                        WHERE mv.model_name = tr.model_name AND mv.quantization = tr.quantization
                        LIMIT 1
                    )
                    WHEN $1 = 'truthfulqa' THEN (
                        SELECT ts.truthful_score
                        FROM truthfulqa_scores_v2 ts
                        JOIN model_variants mv ON ts.model_variant_id = mv.id
                        WHERE mv.model_name = tr.model_name AND mv.quantization = tr.quantization
                        LIMIT 1
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
        let gpu_model: String = row.get("gpu_model");
        let cpu_arch: String = row.get("cpu_arch");
        let cpu_model: String = row.get("cpu_model");
        
        // Count total quantizations for this model (before filtering)
        *total_quants_by_model.entry(model_name.clone()).or_insert(0) += 1;
        
        // Skip entries without any performance data (benchmark-only entries)
        if tokens_per_second.is_none() && memory_gb.is_none() {
            continue;
        }
        
        // Also skip obvious generic entries
        if gpu_model.contains("Generic") || cpu_model.contains("Generic") || 
           gpu_model.contains("Benchmark Only") || cpu_model.contains("Benchmark Only") {
            continue;
        }
        
        // Determine hardware category
        let hardware_category = determine_hardware_category(&gpu_model, &cpu_model);
        
        // Apply hardware category filter
        if !filter_categories.is_empty() && !filter_categories.contains(&hardware_category) {
            continue;
        }
        
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
            hardware_category,
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
            
            // Sort by quality score first, then by speed if quality is equal
            quants.sort_by(|a, b| {
                match b.quality_score.partial_cmp(&a.quality_score).unwrap() {
                    std::cmp::Ordering::Equal => {
                        // If quality scores are equal, sort by speed (higher is better)
                        b.tokens_per_second.partial_cmp(&a.tokens_per_second).unwrap()
                    }
                    other => other
                }
            });
            
            let qualifying_count = quants.len();
            let best_quant = quants[0].clone();
            let all_quants = Some(quants);
            
            Some(ModelPerformanceGroup {
                model_name: model_name.clone(),
                best_quantization: best_quant,
                total_quantizations: *total_quants_by_model.get(&model_name).unwrap_or(&qualifying_count),
                qualifying_quantizations: qualifying_count,
                all_quantizations: all_quants,
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