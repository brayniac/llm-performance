// handlers/model_hardware_analysis.rs
// Model + Hardware analysis endpoint for detailed visualizations

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use llm_benchmark_types::ErrorResponse;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AnalysisQueryParams {
    pub lora: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelHardwareAnalysis {
    pub model_name: String,
    pub gpu_model: String,
    pub total_configurations: usize,
    pub backends: Vec<BackendGroup>,
    pub quantizations: Vec<QuantizationSummary>,
    pub heatmap_data: HeatmapData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendGroup {
    pub backend: String,
    pub quantizations: Vec<QuantizationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationSummary {
    pub quantization: String,
    pub backend: String,
    pub best_speed: f64,
    pub best_ttft: Option<f64>,
    pub best_tokens_per_kwh: Option<f64>,
    pub quality_score: f64,
    pub configuration_count: usize,
    pub category_scores: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapData {
    pub quantizations: Vec<String>,
    pub power_limits: Vec<i32>,
    pub concurrent_requests: Vec<i32>,
    // Map: key -> power_limit -> concurrent_requests -> metric
    // Key is "backend||quantization" composite key
    pub speed_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>>,
    pub ttft_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>>,
    pub tpot_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>>,
    pub itl_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>>,
    pub efficiency_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>>,
}

/// Sort quantizations in a logical order (full precision first, then quantized)
fn quantization_sort_key(quant: &str) -> (u8, String) {
    let priority = match quant {
        // Full precision formats (highest priority)
        "FP32" => 0,
        "BF16" => 1,
        "FP16" => 2,
        "FP8_DYNAMIC" => 3,
        "FP8" => 4,
        // Weight-only quantization
        q if q.starts_with("W") && q.contains("A16") => 10,
        q if q.starts_with("W") && q.contains("A8") => 11,
        // Full quantization
        q if q.starts_with("W") => 20,
        // GGUF-style quantization
        q if q.starts_with("Q") => 30,
        // Everything else
        _ => 99,
    };
    (priority, quant.to_string())
}

/// Get model+hardware analysis data for visualizations
pub async fn get_model_hardware_analysis(
    Path((model_name, gpu_model_param)): Path<(String, String)>,
    Query(query_params): Query<AnalysisQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<ModelHardwareAnalysis>, (StatusCode, Json<ErrorResponse>)> {
    let lora_adapter = query_params.lora.as_deref().unwrap_or("");
    // Decode URL-encoded model name and gpu model
    let model_name = urlencoding::decode(&model_name)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Invalid model name encoding: {}", e))),
            )
        })?
        .to_string();

    let gpu_model = urlencoding::decode(&gpu_model_param)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Invalid GPU model encoding: {}", e))),
            )
        })?
        .to_string();

    // Aggregate metrics across all runs for each unique configuration (backend, quantization, power_limit, concurrent_requests)
    // Using GROUP BY instead of ROW_NUMBER to combine metrics from runs that may have different metrics available
    let test_runs = sqlx::query!(
        r#"
        SELECT
            tr.backend as "backend!",
            tr.quantization as "quantization!",
            tr.concurrent_requests as "concurrent_requests?",
            tr.gpu_power_limit_watts as "gpu_power_limit_watts?",
            MAX(pm_speed.value) as "tokens_per_second?",
            MIN(pm_ttft.value) as "ttft?",
            MIN(pm_tpot.value) as "tpot?",
            MIN(pm_itl.value) as "itl?",
            AVG(pm_power.value) as "gpu_power_watts?"
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        LEFT JOIN performance_metrics pm_speed
            ON tr.id = pm_speed.test_run_id AND pm_speed.metric_name = 'tokens_per_second'
        LEFT JOIN performance_metrics pm_ttft
            ON tr.id = pm_ttft.test_run_id AND pm_ttft.metric_name = 'ttft_p95_ms'
        LEFT JOIN performance_metrics pm_tpot
            ON tr.id = pm_tpot.test_run_id AND pm_tpot.metric_name = 'tpot_p95_ms'
        LEFT JOIN performance_metrics pm_itl
            ON tr.id = pm_itl.test_run_id AND pm_itl.metric_name = 'itl_p95_ms'
        LEFT JOIN performance_metrics pm_power
            ON tr.id = pm_power.test_run_id AND pm_power.metric_name = 'gpu_power_watts'
        WHERE tr.model_name = $1
            AND hp.gpu_model = $2
            AND tr.status = 'completed'
        GROUP BY tr.backend, tr.quantization, tr.concurrent_requests, tr.gpu_power_limit_watts
        "#,
        model_name,
        gpu_model
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    if test_runs.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "No test runs found for this model+hardware combination".to_string(),
            )),
        ));
    }

    // Aggregate data by (backend, quantization)
    // Tuple: (power_limit, concurrent, speed, ttft, tpot, itl, gpu_power, tokens_per_kwh)
    let mut quant_map: HashMap<(String, String), Vec<(i32, i32, f64, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>)>> = HashMap::new();
    let mut all_power_limits = std::collections::BTreeSet::new();
    let mut all_concurrent_requests = std::collections::BTreeSet::new();

    for run in test_runs.iter() {
        let backend = run.backend.clone();
        let quant = run.quantization.clone();
        let power_limit = run.gpu_power_limit_watts.unwrap_or(0);
        let concurrent = run.concurrent_requests.unwrap_or(1);
        let speed = run.tokens_per_second.unwrap_or(0.0);
        let ttft = run.ttft;
        let tpot = run.tpot;
        let itl = run.itl;
        let gpu_power = run.gpu_power_watts;

        // Calculate tokens/kWh: (tokens/second × 3,600,000) / watts
        let tokens_per_kwh = if let Some(power) = gpu_power {
            if power > 0.0 {
                Some((speed * 3_600_000.0) / power)
            } else {
                None
            }
        } else {
            None
        };

        all_power_limits.insert(power_limit);
        all_concurrent_requests.insert(concurrent);

        quant_map
            .entry((backend, quant))
            .or_insert_with(Vec::new)
            .push((power_limit, concurrent, speed, ttft, tpot, itl, gpu_power, tokens_per_kwh));
    }

    // Get quality scores for each quantization and build summaries
    let mut quantization_summaries = Vec::new();
    for ((backend, quant), runs) in quant_map.iter() {
        // Get category-level scores (filtered by LoRA adapter)
        let category_scores_rows = sqlx::query!(
            r#"
            SELECT ms.category, AVG(ms.score) as avg_score
            FROM mmlu_scores_v2 ms
            JOIN model_variants mv ON ms.model_variant_id = mv.id
            WHERE mv.model_name = $1 AND mv.quantization = $2 AND mv.lora_adapter = $3
            GROUP BY ms.category
            "#,
            model_name,
            quant,
            lora_adapter
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let mut category_scores = HashMap::new();
        let mut total_score = 0.0;
        let mut count = 0;

        for row in category_scores_rows {
            let score = row.avg_score.unwrap_or(0.0);
            category_scores.insert(row.category, score);
            total_score += score;
            count += 1;
        }

        let quality_score = if count > 0 { total_score / count as f64 } else { 0.0 };

        let best_speed = runs.iter().map(|(_, _, speed, _, _, _, _, _)| *speed).fold(0.0_f64, f64::max);
        let best_ttft = runs
            .iter()
            .filter_map(|(_, _, _, ttft, _, _, _, _)| *ttft)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let best_tokens_per_kwh = runs
            .iter()
            .filter_map(|(_, _, _, _, _, _, _, tokens_kwh)| *tokens_kwh)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        quantization_summaries.push(QuantizationSummary {
            quantization: quant.clone(),
            backend: backend.clone(),
            best_speed,
            best_ttft,
            best_tokens_per_kwh,
            quality_score,
            configuration_count: runs.len(),
            category_scores,
        });
    }

    // Sort by backend then by quantization logical order
    quantization_summaries.sort_by(|a, b| {
        a.backend.cmp(&b.backend)
            .then_with(|| quantization_sort_key(&a.quantization).cmp(&quantization_sort_key(&b.quantization)))
    });

    // Group into BackendGroups
    let mut backends: Vec<BackendGroup> = Vec::new();
    for summary in quantization_summaries.iter() {
        if let Some(last) = backends.last_mut() {
            if last.backend == summary.backend {
                last.quantizations.push(summary.clone());
                continue;
            }
        }
        backends.push(BackendGroup {
            backend: summary.backend.clone(),
            quantizations: vec![summary.clone()],
        });
    }

    // Build heatmap data using composite keys "backend||quantization"
    let mut speed_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>> = HashMap::new();
    let mut ttft_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>> = HashMap::new();
    let mut tpot_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>> = HashMap::new();
    let mut itl_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>> = HashMap::new();
    let mut efficiency_data: HashMap<String, HashMap<i32, HashMap<i32, f64>>> = HashMap::new();

    for ((backend, quant), runs) in quant_map.iter() {
        let composite_key = format!("{}||{}", backend, quant);
        let quant_speed_map = speed_data.entry(composite_key.clone()).or_insert_with(HashMap::new);
        let quant_ttft_map = ttft_data.entry(composite_key.clone()).or_insert_with(HashMap::new);
        let quant_tpot_map = tpot_data.entry(composite_key.clone()).or_insert_with(HashMap::new);
        let quant_itl_map = itl_data.entry(composite_key.clone()).or_insert_with(HashMap::new);
        let quant_efficiency_map = efficiency_data.entry(composite_key).or_insert_with(HashMap::new);

        for (power_limit, concurrent, speed, ttft, tpot, itl, _gpu_power, tokens_per_kwh) in runs {
            quant_speed_map
                .entry(*power_limit)
                .or_insert_with(HashMap::new)
                .insert(*concurrent, *speed);

            if let Some(ttft_val) = ttft {
                quant_ttft_map
                    .entry(*power_limit)
                    .or_insert_with(HashMap::new)
                    .insert(*concurrent, *ttft_val);
            }

            if let Some(tpot_val) = tpot {
                quant_tpot_map
                    .entry(*power_limit)
                    .or_insert_with(HashMap::new)
                    .insert(*concurrent, *tpot_val);
            }

            if let Some(itl_val) = itl {
                quant_itl_map
                    .entry(*power_limit)
                    .or_insert_with(HashMap::new)
                    .insert(*concurrent, *itl_val);
            }

            if let Some(efficiency_val) = tokens_per_kwh {
                quant_efficiency_map
                    .entry(*power_limit)
                    .or_insert_with(HashMap::new)
                    .insert(*concurrent, *efficiency_val);
            }
        }
    }

    // Collect and sort composite keys for heatmap quantizations list
    let mut heatmap_quantizations: Vec<String> = quant_map.keys()
        .map(|(backend, quant)| format!("{}||{}", backend, quant))
        .collect();
    heatmap_quantizations.sort();

    let heatmap_data = HeatmapData {
        quantizations: heatmap_quantizations,
        power_limits: all_power_limits.into_iter().collect(),
        concurrent_requests: all_concurrent_requests.into_iter().collect(),
        speed_data,
        ttft_data,
        tpot_data,
        itl_data,
        efficiency_data,
    };

    Ok(Json(ModelHardwareAnalysis {
        model_name: model_name.clone(),
        gpu_model,
        total_configurations: test_runs.len(),
        backends,
        quantizations: quantization_summaries,
        heatmap_data,
    }))
}
