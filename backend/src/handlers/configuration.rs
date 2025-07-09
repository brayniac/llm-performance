// handlers/configuration.rs
// Configuration listing and detail handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use std::collections::HashMap;

use llm_benchmark_types::{
    DetailData, ErrorResponse, ExperimentSummary, ConfigurationListResponse
};

use crate::{
    models::PerformanceMetricQueryResult,
    AppState
};

/// Get list of available configurations
pub async fn get_configurations(
    State(state): State<AppState>,
) -> Result<Json<ConfigurationListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let experiments = sqlx::query!(
        r#"
        SELECT 
            tr.id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            CONCAT(hp.gpu_model, ' / ', hp.cpu_arch) as hardware_summary,
            NULL as overall_score,
            tr.timestamp,
            tr.status
        FROM test_runs tr
        JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
        -- Benchmark scores now handled separately
        WHERE tr.status = 'completed'
        GROUP BY tr.id, tr.model_name, tr.quantization, tr.backend, 
                 hp.gpu_model, hp.cpu_arch, tr.timestamp, tr.status
        ORDER BY tr.timestamp DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let mut configurations = Vec::new();
    for row in experiments {
        // Get aggregated benchmark score from v2 tables
        let overall_score = sqlx::query!(
            r#"
            SELECT AVG(ms.score) as avg_score
            FROM mmlu_scores_v2 ms
            JOIN model_variants mv ON ms.model_variant_id = mv.id
            WHERE mv.model_name = $1 AND mv.quantization = $2
            "#,
            row.model_name,
            row.quantization
        )
        .fetch_one(&state.db)
        .await
        .map(|r| r.avg_score)
        .ok()
        .flatten();
        
        configurations.push(ExperimentSummary {
            id: row.id,
            model_name: row.model_name,
            quantization: row.quantization,
            backend: row.backend,
            hardware_summary: row.hardware_summary.unwrap_or_default(),
            overall_score,
            timestamp: row.timestamp.unwrap_or_else(|| chrono::Utc::now()),
            status: match row.status.as_str() {
                "pending" => llm_benchmark_types::ExperimentStatus::Pending,
                "running" => llm_benchmark_types::ExperimentStatus::Running,
                "completed" => llm_benchmark_types::ExperimentStatus::Completed,
                "failed" => llm_benchmark_types::ExperimentStatus::Failed,
                "cancelled" => llm_benchmark_types::ExperimentStatus::Cancelled,
                _ => llm_benchmark_types::ExperimentStatus::Completed,
            },
        });
    }

    let total_count = configurations.len();

    Ok(Json(ConfigurationListResponse {
        configurations,
        total_count,
    }))
}

/// Get detailed information about a specific test run
pub async fn get_detail(
    Path(test_run_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DetailData>, (StatusCode, Json<ErrorResponse>)> {
    // Get detailed config data
    let (config_detail, system_info) = get_detailed_config_data(&state.db, &test_run_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching config details: {}", e))),
            )
        })?;

    // Get category scores
    let categories = get_category_scores(&state.db, &test_run_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Error fetching categories: {}", e))),
            )
        })?;

    let detail_data = DetailData {
        config: config_detail,
        categories,
        system_info,
    };

    Ok(Json(detail_data))
}

async fn get_detailed_config_data(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<(llm_benchmark_types::ConfigDetail, llm_benchmark_types::SystemInfo), sqlx::Error> {
    // Get detailed test run and hardware info
    let result = sqlx::query!(
        r#"
        SELECT 
            tr.id as test_run_id,
            tr.model_name,
            tr.quantization,
            tr.backend,
            tr.backend_version,
            tr.timestamp,
            hp.gpu_model,
            hp.gpu_memory_gb,
            hp.cpu_model,
            hp.cpu_arch,
            hp.ram_gb,
            hp.ram_type,
            hp.virtualization_type,
            hp.optimizations
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
        result.test_run_id
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
        result.model_name,
        result.quantization
    )
    .fetch_one(db)
    .await
    .map(|row| row.avg_score.unwrap_or(0.0))
    .unwrap_or(0.0);

    let config_detail = llm_benchmark_types::ConfigDetail {
        name: format!("{} {}", result.model_name, result.quantization),
        model: result.model_name,
        quantization: result.quantization,
        backend: result.backend,
        backend_version: result.backend_version,
        overall_score,
        performance: llm_benchmark_types::PerformanceSummary {
            speed: perf_map.get("tokens_per_second").copied().unwrap_or(0.0),
            memory: perf_map.get("memory_usage_gb").copied().unwrap_or(0.0),
            loading_time: perf_map.get("model_loading_time").copied().unwrap_or(5.0),
            prompt_speed: perf_map.get("prompt_processing_speed").copied().unwrap_or(0.0),
        },
        test_run_date: result.timestamp.unwrap_or_else(|| chrono::Utc::now()).format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };

    let system_info = llm_benchmark_types::SystemInfo {
        gpu_model: result.gpu_model,
        gpu_memory_gb: result.gpu_memory_gb,
        cpu_model: result.cpu_model,
        cpu_arch: result.cpu_arch,
        ram_gb: result.ram_gb.unwrap_or(0),
        ram_type: result.ram_type.unwrap_or_else(|| "Unknown".to_string()),
        virtualization_type: result.virtualization_type,
        optimizations: result.optimizations.unwrap_or_default(),
    };

    Ok((config_detail, system_info))
}

async fn get_category_scores(
    db: &sqlx::PgPool,
    test_run_id: &Uuid,
) -> Result<Vec<llm_benchmark_types::CategoryScore>, sqlx::Error> {
    let mut categories = Vec::new();
    
    // First, get model variant info from test run
    let variant_info = sqlx::query!(
        r#"
        SELECT tr.model_name, tr.quantization
        FROM test_runs tr
        WHERE tr.id = $1
        "#,
        test_run_id
    )
    .fetch_one(db)
    .await?;
    
    // Get MMLU scores from v2 tables
    let mmlu_scores = sqlx::query!(
        r#"
        SELECT ms.category, ms.score, ms.total_questions, ms.correct_answers
        FROM mmlu_scores_v2 ms
        JOIN model_variants mv ON ms.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        ORDER BY ms.category
        "#,
        variant_info.model_name,
        variant_info.quantization
    )
    .fetch_all(db)
    .await?;
    
    for row in mmlu_scores {
        categories.push(llm_benchmark_types::CategoryScore {
            name: format!("MMLU - {}", row.category),
            score: row.score,
            total_questions: row.total_questions,
            correct_answers: row.correct_answers,
        });
    }
    
    // Get GSM8K scores from v2 tables
    let gsm8k_score = sqlx::query!(
        r#"
        SELECT gs.accuracy, gs.problems_solved, gs.total_problems
        FROM gsm8k_scores_v2 gs
        JOIN model_variants mv ON gs.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        variant_info.model_name,
        variant_info.quantization
    )
    .fetch_optional(db)
    .await?;
    
    if let Some(row) = gsm8k_score {
        categories.push(llm_benchmark_types::CategoryScore {
            name: "GSM8K".to_string(),
            score: row.accuracy * 100.0, // Convert to percentage
            total_questions: Some(row.total_problems),
            correct_answers: Some(row.problems_solved),
        });
    }
    
    // Get HumanEval scores from v2 tables
    let humaneval_score = sqlx::query!(
        r#"
        SELECT hs.pass_at_1
        FROM humaneval_scores_v2 hs
        JOIN model_variants mv ON hs.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        variant_info.model_name,
        variant_info.quantization
    )
    .fetch_optional(db)
    .await?;
    
    if let Some(row) = humaneval_score {
        categories.push(llm_benchmark_types::CategoryScore {
            name: "HumanEval".to_string(),
            score: row.pass_at_1,
            total_questions: None, // Not stored in v2 tables
            correct_answers: None,
        });
    }
    
    // Get HellaSwag scores from v2 tables
    let hellaswag_score = sqlx::query!(
        r#"
        SELECT hs.accuracy, hs.total_questions, hs.correct_answers
        FROM hellaswag_scores_v2 hs
        JOIN model_variants mv ON hs.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        variant_info.model_name,
        variant_info.quantization
    )
    .fetch_optional(db)
    .await?;
    
    if let Some(row) = hellaswag_score {
        categories.push(llm_benchmark_types::CategoryScore {
            name: "HellaSwag".to_string(),
            score: row.accuracy,
            total_questions: row.total_questions,
            correct_answers: row.correct_answers,
        });
    }
    
    // Get TruthfulQA scores from v2 tables
    let truthfulqa_score = sqlx::query!(
        r#"
        SELECT ts.truthful_score, ts.total_questions
        FROM truthfulqa_scores_v2 ts
        JOIN model_variants mv ON ts.model_variant_id = mv.id
        WHERE mv.model_name = $1 AND mv.quantization = $2
        LIMIT 1
        "#,
        variant_info.model_name,
        variant_info.quantization
    )
    .fetch_optional(db)
    .await?;
    
    if let Some(row) = truthfulqa_score {
        categories.push(llm_benchmark_types::CategoryScore {
            name: "TruthfulQA".to_string(),
            score: row.truthful_score,
            total_questions: row.total_questions,
            correct_answers: None,
        });
    }
    
    // Note: Generic benchmark scores would still come from v1 tables if needed
    // as they're tied to test runs, not model variants
    
    Ok(categories)
}