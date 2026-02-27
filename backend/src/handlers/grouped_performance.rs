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
    HardwarePlatformPerformance,
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
                tr.concurrent_requests,
                tr.max_context_length,
                tr.load_pattern,
                tr.dataset_name,
                tr.gpu_power_limit_watts,
                pm_speed.value as tokens_per_second,
                pm_memory.value as memory_gb,
                pm_power.value as gpu_power_watts,
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
            LEFT JOIN performance_metrics pm_power ON pm_power.test_run_id = tr.id
                AND pm_power.metric_name = 'gpu_power_watts'
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

    // Derive optimization goal from sort_by parameter
    let sort_by = params.sort_by.as_deref().unwrap_or("quality");

    // Group by model → hardware → configs
    let mut model_hardware_groups: HashMap<String, HashMap<String, Vec<QuantizationPerformance>>> = HashMap::new();
    let mut total_platforms_by_model: HashMap<String, usize> = HashMap::new();

    for row in rows {
        let model_name: String = row.get("model_name");
        let tokens_per_second: Option<f64> = row.get("tokens_per_second");
        let memory_gb: Option<f64> = row.get("memory_gb");
        let gpu_power_watts: Option<f64> = row.get("gpu_power_watts");
        let quality_score: Option<f64> = row.get("quality_score");
        let gpu_model: String = row.get("gpu_model");
        let _cpu_arch: String = row.get("cpu_arch");
        let cpu_model: String = row.get("cpu_model");
        let concurrent_requests: Option<i32> = row.get("concurrent_requests");
        let max_context_length: Option<i32> = row.get("max_context_length");
        let load_pattern: Option<String> = row.get("load_pattern");
        let dataset_name: Option<String> = row.get("dataset_name");
        let gpu_power_limit_watts: Option<i32> = row.get("gpu_power_limit_watts");
        let hardware: String = row.get("hardware");

        // Calculate tokens/kWh: (tokens/second × 3,600,000) / watts
        let tokens_per_kwh = if let (Some(speed), Some(power)) = (tokens_per_second, gpu_power_watts) {
            if power > 0.0 {
                Some((speed * 3_600_000.0) / power)
            } else {
                None
            }
        } else {
            None
        };
        
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
        
        let config = QuantizationPerformance {
            id: row.get("id"),
            quantization: row.get("quantization"),
            quality_score: quality_score.unwrap_or(0.0),
            tokens_per_second: tokens_per_second.unwrap_or(0.0),
            memory_gb: memory_gb.unwrap_or(0.0),
            backend: row.get("backend"),
            hardware: hardware.clone(),
            hardware_category,
            concurrent_requests,
            max_context_length,
            load_pattern,
            dataset_name,
            gpu_power_limit_watts,
            gpu_power_watts,
            tokens_per_kwh,
        };

        // Group by model → hardware
        model_hardware_groups
            .entry(model_name.clone())
            .or_insert_with(HashMap::new)
            .entry(hardware.clone())
            .or_insert_with(Vec::new)
            .push(config);
    }

    // Count total platforms per model (before filtering)
    for (model_name, hardware_map) in &model_hardware_groups {
        total_platforms_by_model.insert(model_name.clone(), hardware_map.len());
    }
    
    // Helper function to sort configs with tiebreakers based on sort_by
    let sort_configs = |configs: &mut Vec<QuantizationPerformance>| {
        configs.sort_by(|a, b| {
            use std::cmp::Ordering;

            match sort_by {
                "quality" => {
                    // Primary: quality (higher is better)
                    let quality_cmp = b.quality_score.partial_cmp(&a.quality_score).unwrap_or(Ordering::Equal);
                    if quality_cmp != Ordering::Equal {
                        return quality_cmp;
                    }
                    // Tiebreaker 1: speed (higher is better)
                    let speed_cmp = b.tokens_per_second.partial_cmp(&a.tokens_per_second).unwrap_or(Ordering::Equal);
                    if speed_cmp != Ordering::Equal {
                        return speed_cmp;
                    }
                    // Tiebreaker 2: efficiency (higher is better)
                    let eff_a = a.tokens_per_kwh.unwrap_or(0.0);
                    let eff_b = b.tokens_per_kwh.unwrap_or(0.0);
                    eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                }
                "speed" => {
                    // Primary: speed (higher is better)
                    let speed_cmp = b.tokens_per_second.partial_cmp(&a.tokens_per_second).unwrap_or(Ordering::Equal);
                    if speed_cmp != Ordering::Equal {
                        return speed_cmp;
                    }
                    // Tiebreaker 1: quality (higher is better)
                    let quality_cmp = b.quality_score.partial_cmp(&a.quality_score).unwrap_or(Ordering::Equal);
                    if quality_cmp != Ordering::Equal {
                        return quality_cmp;
                    }
                    // Tiebreaker 2: efficiency (higher is better)
                    let eff_a = a.tokens_per_kwh.unwrap_or(0.0);
                    let eff_b = b.tokens_per_kwh.unwrap_or(0.0);
                    eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                }
                "efficiency" => {
                    // Primary: efficiency (higher is better)
                    let eff_a = a.tokens_per_kwh.unwrap_or(0.0);
                    let eff_b = b.tokens_per_kwh.unwrap_or(0.0);
                    let eff_cmp = eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal);
                    if eff_cmp != Ordering::Equal {
                        return eff_cmp;
                    }
                    // Tiebreaker 1: quality (higher is better)
                    let quality_cmp = b.quality_score.partial_cmp(&a.quality_score).unwrap_or(Ordering::Equal);
                    if quality_cmp != Ordering::Equal {
                        return quality_cmp;
                    }
                    // Tiebreaker 2: speed (higher is better)
                    b.tokens_per_second.partial_cmp(&a.tokens_per_second).unwrap_or(Ordering::Equal)
                }
                _ => {
                    // Default to quality sorting
                    let quality_cmp = b.quality_score.partial_cmp(&a.quality_score).unwrap_or(Ordering::Equal);
                    if quality_cmp != Ordering::Equal {
                        return quality_cmp;
                    }
                    let speed_cmp = b.tokens_per_second.partial_cmp(&a.tokens_per_second).unwrap_or(Ordering::Equal);
                    if speed_cmp != Ordering::Equal {
                        return speed_cmp;
                    }
                    let eff_a = a.tokens_per_kwh.unwrap_or(0.0);
                    let eff_b = b.tokens_per_kwh.unwrap_or(0.0);
                    eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                }
            }
        });
    };

    // Convert to response format - build hardware platforms per model
    let mut models: Vec<ModelPerformanceGroup> = model_hardware_groups
        .into_iter()
        .filter_map(|(model_name, hardware_map)| {
            if hardware_map.is_empty() {
                return None;
            }

            // Build hardware platform list
            let mut hardware_platforms: Vec<HardwarePlatformPerformance> = hardware_map
                .into_iter()
                .map(|(hardware, mut configs)| {
                    let total_configs = configs.len();
                    sort_configs(&mut configs);
                    let best_config = configs[0].clone();

                    HardwarePlatformPerformance {
                        hardware,
                        hardware_category: best_config.hardware_category.clone(),
                        best_config,
                        total_configs,
                    }
                })
                .collect();

            if hardware_platforms.is_empty() {
                return None;
            }

            // Sort hardware platforms by the same criteria with tiebreakers
            hardware_platforms.sort_by(|a, b| {
                use std::cmp::Ordering;

                match sort_by {
                    "quality" => {
                        let quality_cmp = b.best_config.quality_score
                            .partial_cmp(&a.best_config.quality_score).unwrap_or(Ordering::Equal);
                        if quality_cmp != Ordering::Equal {
                            return quality_cmp;
                        }
                        let speed_cmp = b.best_config.tokens_per_second
                            .partial_cmp(&a.best_config.tokens_per_second).unwrap_or(Ordering::Equal);
                        if speed_cmp != Ordering::Equal {
                            return speed_cmp;
                        }
                        let eff_a = a.best_config.tokens_per_kwh.unwrap_or(0.0);
                        let eff_b = b.best_config.tokens_per_kwh.unwrap_or(0.0);
                        eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                    }
                    "speed" => {
                        let speed_cmp = b.best_config.tokens_per_second
                            .partial_cmp(&a.best_config.tokens_per_second).unwrap_or(Ordering::Equal);
                        if speed_cmp != Ordering::Equal {
                            return speed_cmp;
                        }
                        let quality_cmp = b.best_config.quality_score
                            .partial_cmp(&a.best_config.quality_score).unwrap_or(Ordering::Equal);
                        if quality_cmp != Ordering::Equal {
                            return quality_cmp;
                        }
                        let eff_a = a.best_config.tokens_per_kwh.unwrap_or(0.0);
                        let eff_b = b.best_config.tokens_per_kwh.unwrap_or(0.0);
                        eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                    }
                    "efficiency" => {
                        let eff_a = a.best_config.tokens_per_kwh.unwrap_or(0.0);
                        let eff_b = b.best_config.tokens_per_kwh.unwrap_or(0.0);
                        let eff_cmp = eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal);
                        if eff_cmp != Ordering::Equal {
                            return eff_cmp;
                        }
                        let quality_cmp = b.best_config.quality_score
                            .partial_cmp(&a.best_config.quality_score).unwrap_or(Ordering::Equal);
                        if quality_cmp != Ordering::Equal {
                            return quality_cmp;
                        }
                        b.best_config.tokens_per_second
                            .partial_cmp(&a.best_config.tokens_per_second).unwrap_or(Ordering::Equal)
                    }
                    _ => {
                        let quality_cmp = b.best_config.quality_score
                            .partial_cmp(&a.best_config.quality_score).unwrap_or(Ordering::Equal);
                        if quality_cmp != Ordering::Equal {
                            return quality_cmp;
                        }
                        let speed_cmp = b.best_config.tokens_per_second
                            .partial_cmp(&a.best_config.tokens_per_second).unwrap_or(Ordering::Equal);
                        if speed_cmp != Ordering::Equal {
                            return speed_cmp;
                        }
                        let eff_a = a.best_config.tokens_per_kwh.unwrap_or(0.0);
                        let eff_b = b.best_config.tokens_per_kwh.unwrap_or(0.0);
                        eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
                    }
                }
            });

            let qualifying_platforms = hardware_platforms.len();
            let best_hardware = hardware_platforms[0].clone();
            let all_platforms = Some(hardware_platforms);

            Some(ModelPerformanceGroup {
                model_name: model_name.clone(),
                best_hardware,
                total_hardware_platforms: *total_platforms_by_model.get(&model_name).unwrap_or(&qualifying_platforms),
                qualifying_platforms,
                all_hardware_platforms: all_platforms,
            })
        })
        .collect();
    
    // Apply sorting with tiebreakers
    use std::cmp::Ordering;

    match params.sort_by.as_deref() {
        Some("quality") => {
            models.sort_by(|a, b| {
                // Primary: quality
                let quality_cmp = b.best_hardware.best_config.quality_score
                    .partial_cmp(&a.best_hardware.best_config.quality_score)
                    .unwrap_or(Ordering::Equal);
                if quality_cmp != Ordering::Equal {
                    return if params.sort_direction.as_deref() == Some("asc") {
                        quality_cmp.reverse()
                    } else {
                        quality_cmp
                    };
                }
                // Tiebreaker 1: speed
                let speed_cmp = b.best_hardware.best_config.tokens_per_second
                    .partial_cmp(&a.best_hardware.best_config.tokens_per_second)
                    .unwrap_or(Ordering::Equal);
                if speed_cmp != Ordering::Equal {
                    return speed_cmp;
                }
                // Tiebreaker 2: efficiency
                let eff_a = a.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                let eff_b = b.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
            });
        }
        Some("speed") => {
            models.sort_by(|a, b| {
                // Primary: speed
                let speed_cmp = b.best_hardware.best_config.tokens_per_second
                    .partial_cmp(&a.best_hardware.best_config.tokens_per_second)
                    .unwrap_or(Ordering::Equal);
                if speed_cmp != Ordering::Equal {
                    return if params.sort_direction.as_deref() == Some("asc") {
                        speed_cmp.reverse()
                    } else {
                        speed_cmp
                    };
                }
                // Tiebreaker 1: quality
                let quality_cmp = b.best_hardware.best_config.quality_score
                    .partial_cmp(&a.best_hardware.best_config.quality_score)
                    .unwrap_or(Ordering::Equal);
                if quality_cmp != Ordering::Equal {
                    return quality_cmp;
                }
                // Tiebreaker 2: efficiency
                let eff_a = a.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                let eff_b = b.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
            });
        }
        Some("efficiency") => {
            models.sort_by(|a, b| {
                // Primary: efficiency
                let eff_a = a.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                let eff_b = b.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                let eff_cmp = eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal);
                if eff_cmp != Ordering::Equal {
                    return if params.sort_direction.as_deref() == Some("asc") {
                        eff_cmp.reverse()
                    } else {
                        eff_cmp
                    };
                }
                // Tiebreaker 1: quality
                let quality_cmp = b.best_hardware.best_config.quality_score
                    .partial_cmp(&a.best_hardware.best_config.quality_score)
                    .unwrap_or(Ordering::Equal);
                if quality_cmp != Ordering::Equal {
                    return quality_cmp;
                }
                // Tiebreaker 2: speed
                b.best_hardware.best_config.tokens_per_second
                    .partial_cmp(&a.best_hardware.best_config.tokens_per_second)
                    .unwrap_or(Ordering::Equal)
            });
        }
        Some("memory") => {
            models.sort_by(|a, b| {
                let cmp = a.best_hardware.best_config.memory_gb
                    .partial_cmp(&b.best_hardware.best_config.memory_gb)
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
            // Default: sort by quality score descending with tiebreakers
            models.sort_by(|a, b| {
                let quality_cmp = b.best_hardware.best_config.quality_score
                    .partial_cmp(&a.best_hardware.best_config.quality_score)
                    .unwrap_or(Ordering::Equal);
                if quality_cmp != Ordering::Equal {
                    return quality_cmp;
                }
                let speed_cmp = b.best_hardware.best_config.tokens_per_second
                    .partial_cmp(&a.best_hardware.best_config.tokens_per_second)
                    .unwrap_or(Ordering::Equal);
                if speed_cmp != Ordering::Equal {
                    return speed_cmp;
                }
                let eff_a = a.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                let eff_b = b.best_hardware.best_config.tokens_per_kwh.unwrap_or(0.0);
                eff_b.partial_cmp(&eff_a).unwrap_or(Ordering::Equal)
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