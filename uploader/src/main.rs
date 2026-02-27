use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, NaiveDateTime};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use llm_benchmark_types::{*, benchmarks::{MMLUScore, MMLUCategoryScore}};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use arrow::array::{AsArray, Array};
use arrow::datatypes::{UInt64Type, Int64Type};
use std::fs::File;

/// LLM Performance Tool - Record and import LLM benchmark experiments
#[derive(Parser)]
#[command(name = "llm-perf")]
#[command(about = "Record configuration and import LLM benchmark experiments from SystemsLab", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Record system configuration and model info during experiment
    Record {
        /// Model path (will auto-detect model name and quantization)
        #[arg(short = 'p', long)]
        model_path: String,

        /// Backend name (e.g., vllm, llama.cpp)
        #[arg(short, long)]
        backend: Option<String>,

        /// Backend version (will try to auto-detect if not provided)
        #[arg(short = 'v', long)]
        backend_version: Option<String>,

        /// GPU power limit in watts (will try to auto-detect if not provided)
        #[arg(long)]
        power_limit: Option<i32>,

        /// Number of concurrent requests
        #[arg(short = 'c', long)]
        concurrent_requests: Option<i32>,

        /// Maximum context length / sequence length
        #[arg(short = 'm', long)]
        max_context_length: Option<i32>,

        /// Output file path (default: llm.json)
        #[arg(short, long, default_value = "llm.json")]
        output: PathBuf,
    },

    /// Import experiments or contexts from SystemsLab
    Import {
        /// SystemsLab experiment ID or context ID
        id: String,

        /// SystemsLab server URL (default: http://systemslab)
        #[arg(long, default_value = "http://systemslab")]
        systemslab_url: String,

        /// API server URL to upload to (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// Upload benchmark results with llm.json configuration
    Upload {
        /// Path to llm.json artifact
        #[arg(short = 'l', long)]
        llm_json: PathBuf,

        /// Path to results.json from benchmark
        #[arg(short = 'r', long)]
        results_json: PathBuf,

        /// API server URL to upload to (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
    },

    /// Upload MMLU-Pro evaluation results
    UploadMmlu {
        /// Path to MMLU report.txt file
        #[arg(short = 'f', long)]
        report_file: PathBuf,

        /// Model path (for auto-detecting model name and quantization)
        #[arg(short = 'p', long)]
        model_path: String,

        /// API server URL to upload to (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
    },
}

/// Benchmark artifact - captures system configuration and model info
#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkArtifact {
    // Hardware configuration
    gpu_model: String,
    gpu_count: i32,
    gpu_memory_gb: i32,
    cpu_model: String,
    cpu_arch: String,
    ram_gb: Option<i32>,

    // Model configuration
    model_name: String,
    model_path: String,
    quantization: String,

    // Runtime configuration
    gpu_power_limit_watts: Option<i32>,
    backend_name: Option<String>,
    backend_version: Option<String>,
    concurrent_requests: Option<i32>,
    max_context_length: Option<i32>,

    // Metadata
    hostname: String,
    timestamp: DateTime<Utc>,
    artifact_version: String,
}

/// Inference server benchmark output structure
#[derive(Debug, Deserialize)]
struct InferenceServerResult {
    timestamp: String,
    version: String,
    configuration: InferenceConfiguration,
    summary: InferenceSummary,
    throughput: InferenceThroughput,
    latency: InferenceLatency,
    errors: InferenceErrors,
    #[serde(default)]
    context_latency: std::collections::HashMap<String, ContextLatencyMetrics>,
    #[serde(default)]
    context_itl: std::collections::HashMap<String, ContextItlMetrics>,
    #[serde(default)]
    power_stats: Option<PowerStats>,
}

#[derive(Debug, Deserialize)]
struct PowerStats {
    min_watts: f64,
    max_watts: f64,
    avg_watts: f64,
    p50_watts: f64,
    p95_watts: f64,
    samples: usize,
}

#[derive(Debug, Deserialize)]
struct InferenceConfiguration {
    model: String,
    #[serde(default)]
    load_pattern: String,
    #[serde(default)]
    concurrent_requests: i32,
    #[serde(default)]
    duration_seconds: i32,
    #[serde(default)]
    prompt_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InferenceSummary {
    total_requests: i32,
    successful_requests: i32,
    failed_requests: i32,
    success_rate: f64,
}

#[derive(Debug, Deserialize)]
struct InferenceThroughput {
    requests_per_second: f64,
    input_tokens_per_second: f64,
    output_tokens_per_second: f64,
    total_input_tokens: i64,
    total_output_tokens: i64,
}

#[derive(Debug, Deserialize)]
struct InferenceLatency {
    ttft_mean_ms: f64,
    ttft_p50_ms: f64,
    ttft_p90_ms: f64,
    ttft_p95_ms: f64,
    ttft_p99_ms: f64,
    tpot_mean_ms: f64,
    tpot_p50_ms: f64,
    tpot_p90_ms: f64,
    tpot_p95_ms: f64,
    tpot_p99_ms: f64,
    itl_mean_ms: f64,
    itl_p50_ms: f64,
    itl_p90_ms: f64,
    itl_p95_ms: f64,
    itl_p99_ms: f64,
    request_mean_ms: f64,
    request_p50_ms: f64,
    request_p90_ms: f64,
    request_p95_ms: f64,
    request_p99_ms: f64,
}

#[derive(Debug, Deserialize)]
struct InferenceErrors {
    timeout_errors: i32,
    connection_errors: i32,
    http_4xx_errors: i32,
    http_5xx_errors: i32,
    other_errors: i32,
}

#[derive(Debug, Deserialize)]
struct ContextLatencyMetrics {
    ttft_p50_ms: f64,
    ttft_p90_ms: f64,
    ttft_p95_ms: f64,
    ttft_p99_ms: f64,
}

#[derive(Debug, Deserialize)]
struct ContextItlMetrics {
    itl_p50_ms: f64,
    itl_p90_ms: f64,
    itl_p95_ms: f64,
    itl_p99_ms: f64,
}

/// llama-bench output structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LlamaBenchResult {
    build_commit: String,
    build_number: i32,
    cpu_info: String,
    gpu_info: String,
    backends: String,
    model_filename: String,
    model_type: String,
    model_size: i64,
    model_n_params: i64,
    n_batch: i32,
    n_ubatch: i32,
    n_threads: i32,
    n_gpu_layers: i32,
    split_mode: String,
    main_gpu: i32,
    no_kv_offload: bool,
    flash_attn: bool,
    use_mmap: bool,
    embeddings: bool,
    n_prompt: i32,
    n_gen: i32,
    test_time: DateTime<Utc>,
    avg_ns: i64,
    stddev_ns: i64,
    avg_ts: f64,
    stddev_ts: f64,
    samples_ns: Vec<i64>,
    samples_ts: Vec<f64>,
}

/// Parsed model information from filename
#[derive(Debug)]
struct ModelInfo {
    name: String,
    quantization: String,
}

#[derive(Debug, Serialize)]
struct UploadRequest {
    experiment_run: ExperimentRun,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Record {
            model_path,
            backend,
            backend_version,
            power_limit,
            concurrent_requests,
            max_context_length,
            output,
        } => {
            record_config(
                model_path,
                backend,
                backend_version,
                power_limit,
                concurrent_requests,
                max_context_length,
                output,
            ).await?;
        }
        Commands::Import {
            id,
            systemslab_url,
            server,
        } => {
            import_from_systemslab(id, systemslab_url, server).await?;
        }
        Commands::Upload {
            llm_json,
            results_json,
            server,
        } => {
            upload_local_results(llm_json, results_json, server).await?;
        }
        Commands::UploadMmlu {
            report_file,
            model_path,
            server,
        } => {
            upload_mmlu_pro(
                report_file,
                None, // test_run_id
                Some(model_path),
                None, // model
                None, // quantization
                server,
                "llama.cpp".to_string(), // backend
                None, // notes
            ).await?;
        }
    }

    Ok(())
}

async fn upload_llama_bench(
    file: PathBuf,
    server: String,
    model_name: Option<String>,
    quantization: Option<String>,
    notes: Option<String>,
    benchmarks_file: Option<PathBuf>,
) -> Result<()> {
    // Read and parse llama-bench output
    let content = std::fs::read_to_string(&file)?;
    let results: Vec<LlamaBenchResult> = serde_json::from_str(&content)?;
    
    if results.is_empty() {
        return Err(anyhow!("No results found in llama-bench output"));
    }
    
    // Use the first result for hardware info (they should all be the same)
    let first_result = &results[0];
    
    // Parse model info from filename
    let model_info = parse_model_filename(&first_result.model_filename)?;
    
    // Use provided values or fall back to parsed values
    let model_name = model_name.unwrap_or(model_info.name);
    let quantization = quantization.unwrap_or(model_info.quantization);
    
    // Parse hardware info
    let hardware_config = parse_hardware_info(first_result)?;
    
    // Create performance metrics from all results
    let mut performance_metrics = Vec::new();
    
    // Find prompt processing result (n_prompt > 0, n_gen = 0)
    if let Some(prompt_result) = results.iter().find(|r| r.n_prompt > 0 && r.n_gen == 0) {
        performance_metrics.push(PerformanceMetric {
            metric_name: "prompt_processing_speed".to_string(),
            value: prompt_result.avg_ts,
            unit: "tokens/sec".to_string(),
            timestamp: prompt_result.test_time,
            context: Some(serde_json::json!({
                "n_prompt": prompt_result.n_prompt,
                "n_batch": prompt_result.n_batch,
                "n_ubatch": prompt_result.n_ubatch,
                "n_threads": prompt_result.n_threads,
                "n_gpu_layers": prompt_result.n_gpu_layers,
                "split_mode": prompt_result.split_mode,
                "flash_attn": prompt_result.flash_attn,
                "use_mmap": prompt_result.use_mmap,
            })),
        });
    }
    
    // Find text generation result (n_prompt = 0, n_gen > 0)
    if let Some(gen_result) = results.iter().find(|r| r.n_prompt == 0 && r.n_gen > 0) {
        performance_metrics.push(PerformanceMetric {
            metric_name: "tokens_per_second".to_string(),
            value: gen_result.avg_ts,
            unit: "tokens/sec".to_string(),
            timestamp: gen_result.test_time,
            context: Some(serde_json::json!({
                "n_gen": gen_result.n_gen,
                "n_batch": gen_result.n_batch,
                "n_ubatch": gen_result.n_ubatch,
                "n_threads": gen_result.n_threads,
                "n_gpu_layers": gen_result.n_gpu_layers,
                "split_mode": gen_result.split_mode,
                "flash_attn": gen_result.flash_attn,
                "use_mmap": gen_result.use_mmap,
            })),
        });
    }
    
    // Add model size metric
    performance_metrics.push(PerformanceMetric {
        metric_name: "model_size_gb".to_string(),
        value: first_result.model_size as f64 / (1024.0 * 1024.0 * 1024.0),
        unit: "GB".to_string(),
        timestamp: first_result.test_time,
        context: Some(serde_json::json!({
            "model_params": first_result.model_n_params,
            "model_type": first_result.model_type,
        })),
    });
    
    // Estimate memory usage (rough estimate based on model size + overhead)
    let memory_gb = (first_result.model_size as f64 / (1024.0 * 1024.0 * 1024.0)) * 1.2;
    performance_metrics.push(PerformanceMetric {
        metric_name: "memory_usage_gb".to_string(),
        value: memory_gb,
        unit: "GB".to_string(),
        timestamp: first_result.test_time,
        context: Some(serde_json::json!({
            "estimated": true,
            "model_params": first_result.model_n_params,
            "n_gpu_layers": first_result.n_gpu_layers,
        })),
    });
    
    // Load benchmark scores if provided
    let benchmark_scores = if let Some(benchmarks_file) = benchmarks_file {
        let content = std::fs::read_to_string(benchmarks_file)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    // Generate or load experiment ID
    let experiment_dir = file.parent()
        .ok_or_else(|| anyhow!("Could not determine parent directory of benchmark file"))?;
    let experiment_id_path = experiment_dir.join(".experiment-id");

    let exp_uuid = if experiment_id_path.exists() {
        let id_str = std::fs::read_to_string(&experiment_id_path)?;
        Uuid::parse_str(id_str.trim())
            .map_err(|e| anyhow!("Failed to parse existing experiment ID: {}", e))?
    } else {
        let new_id = Uuid::now_v7();
        std::fs::write(&experiment_id_path, new_id.to_string())?;
        println!("Generated new experiment ID: {}", new_id);
        new_id
    };

    // Create experiment run
    let experiment_run = ExperimentRun {
        id: exp_uuid,
        model_name,
        quantization,
        backend: "llama.cpp".to_string(),
        backend_version: format!("{}#{}", first_result.build_commit, first_result.build_number),
        hardware_config,
        performance_metrics,
        benchmark_scores,
        timestamp: first_result.test_time,
        status: ExperimentStatus::Completed,
        notes,
        concurrent_requests: None, // llama-bench doesn't provide this
        max_context_length: None,  // llama-bench doesn't provide this
        load_pattern: None,        // llama-bench doesn't provide this
        dataset_name: None,        // llama-bench doesn't provide this
        gpu_power_limit_watts: None, // llama-bench doesn't provide this
    };
    
    // Upload to server
    upload_experiment(experiment_run, &server).await?;
    
    Ok(())
}

async fn upload_inference_server(
    file: PathBuf,
    server: String,
    backend: String,
    backend_version: String,
    model_name: Option<String>,
    quantization: Option<String>,
    memory_gb: Option<f64>,
    gpu_power_limit_watts: Option<i32>,
    notes: Option<String>,
) -> Result<()> {
    // Read and parse inference benchmark output
    let content = std::fs::read_to_string(&file)?;
    let result: InferenceServerResult = serde_json::from_str(&content)?;

    // Extract model name and quantization from path
    let (extracted_model, extracted_quant) = extract_model_info_from_path(&result.configuration.model)?;

    let model_name = model_name.unwrap_or(extracted_model);
    let quantization = quantization.unwrap_or(extracted_quant);

    // Auto-detect hardware
    let hardware_config = detect_system_hardware()?;

    // Parse timestamp
    let timestamp = parse_timestamp(&result.timestamp)?;

    // Create performance metrics
    let mut performance_metrics = vec![
        // Primary throughput metrics
        PerformanceMetric {
            metric_name: "tokens_per_second".to_string(),
            value: result.throughput.output_tokens_per_second,
            unit: "tok/s".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "total_input_tokens": result.throughput.total_input_tokens,
                "total_output_tokens": result.throughput.total_output_tokens,
            })),
        },
        PerformanceMetric {
            metric_name: "prompt_processing_speed".to_string(),
            value: result.throughput.input_tokens_per_second,
            unit: "tok/s".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "requests_per_second".to_string(),
            value: result.throughput.requests_per_second,
            unit: "req/s".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "total_requests": result.summary.total_requests,
                "successful_requests": result.summary.successful_requests,
                "failed_requests": result.summary.failed_requests,
                "success_rate": result.summary.success_rate,
            })),
        },
        // TTFT metrics - store all percentiles separately
        PerformanceMetric {
            metric_name: "ttft_mean_ms".to_string(),
            value: result.latency.ttft_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p50_ms".to_string(),
            value: result.latency.ttft_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p90_ms".to_string(),
            value: result.latency.ttft_p90_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p95_ms".to_string(),
            value: result.latency.ttft_p95_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p99_ms".to_string(),
            value: result.latency.ttft_p99_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        // TPOT metrics - store all percentiles separately
        PerformanceMetric {
            metric_name: "tpot_mean_ms".to_string(),
            value: result.latency.tpot_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "tpot_p50_ms".to_string(),
            value: result.latency.tpot_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "tpot_p90_ms".to_string(),
            value: result.latency.tpot_p90_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "tpot_p95_ms".to_string(),
            value: result.latency.tpot_p95_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "tpot_p99_ms".to_string(),
            value: result.latency.tpot_p99_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        // ITL metrics - store all percentiles separately
        PerformanceMetric {
            metric_name: "itl_mean_ms".to_string(),
            value: result.latency.itl_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "itl_p50_ms".to_string(),
            value: result.latency.itl_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "itl_p90_ms".to_string(),
            value: result.latency.itl_p90_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "itl_p95_ms".to_string(),
            value: result.latency.itl_p95_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "itl_p99_ms".to_string(),
            value: result.latency.itl_p99_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        // Request latency - store all percentiles separately
        PerformanceMetric {
            metric_name: "request_mean_ms".to_string(),
            value: result.latency.request_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "request_p50_ms".to_string(),
            value: result.latency.request_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "request_p90_ms".to_string(),
            value: result.latency.request_p90_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "request_p95_ms".to_string(),
            value: result.latency.request_p95_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "request_p99_ms".to_string(),
            value: result.latency.request_p99_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        // Error metrics
        PerformanceMetric {
            metric_name: "error_rate".to_string(),
            value: 1.0 - result.summary.success_rate,
            unit: "ratio".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "timeout_errors": result.errors.timeout_errors,
                "connection_errors": result.errors.connection_errors,
                "http_4xx_errors": result.errors.http_4xx_errors,
                "http_5xx_errors": result.errors.http_5xx_errors,
                "other_errors": result.errors.other_errors,
                "total_errors": result.summary.failed_requests,
            })),
        },
    ];

    // Add context-aware TTFT metrics
    for (context_size, metrics) in &result.context_latency {
        performance_metrics.push(PerformanceMetric {
            metric_name: format!("ttft_p50_ms_{}", context_size),
            value: metrics.ttft_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "context_size": context_size,
                "p90": metrics.ttft_p90_ms,
                "p95": metrics.ttft_p95_ms,
                "p99": metrics.ttft_p99_ms,
            })),
        });
    }

    // Add context-aware ITL metrics
    for (context_size, metrics) in &result.context_itl {
        performance_metrics.push(PerformanceMetric {
            metric_name: format!("itl_p50_ms_{}", context_size),
            value: metrics.itl_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "context_size": context_size,
                "p90": metrics.itl_p90_ms,
                "p95": metrics.itl_p95_ms,
                "p99": metrics.itl_p99_ms,
            })),
        });
    }

    // Add memory usage if provided
    if let Some(memory) = memory_gb {
        performance_metrics.push(PerformanceMetric {
            metric_name: "memory_usage_gb".to_string(),
            value: memory,
            unit: "GB".to_string(),
            timestamp,
            context: None,
        });
    }

    // Build notes
    let notes_str = format!(
        "{} | Benchmark tool: {} | Duration: {}s | Requests: {} | Success: {:.2}%",
        notes.unwrap_or_default(),
        result.version,
        result.configuration.duration_seconds,
        result.summary.total_requests,
        result.summary.success_rate * 100.0
    );

    // Extract dataset name from prompt_file if available
    let dataset_name = if let Some(prompt_file) = result.configuration.prompt_file.as_ref() {
        std::path::Path::new(prompt_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Generate or load experiment ID
    let experiment_dir = file.parent()
        .ok_or_else(|| anyhow!("Could not determine parent directory of results file"))?;
    let experiment_id_path = experiment_dir.join(".experiment-id");

    let exp_uuid = if experiment_id_path.exists() {
        let id_str = std::fs::read_to_string(&experiment_id_path)?;
        Uuid::parse_str(id_str.trim())
            .map_err(|e| anyhow!("Failed to parse existing experiment ID: {}", e))?
    } else {
        let new_id = Uuid::now_v7();
        std::fs::write(&experiment_id_path, new_id.to_string())?;
        println!("Generated new experiment ID: {}", new_id);
        new_id
    };

    // Create experiment run
    let experiment_run = ExperimentRun {
        id: exp_uuid,
        model_name,
        quantization,
        backend,
        backend_version,
        hardware_config,
        performance_metrics,
        benchmark_scores: Vec::new(),
        timestamp,
        status: ExperimentStatus::Completed,
        notes: Some(notes_str),
        concurrent_requests: Some(result.configuration.concurrent_requests),
        max_context_length: None, // Not available in this benchmark format yet
        load_pattern: Some(result.configuration.load_pattern.clone()),
        dataset_name,
        gpu_power_limit_watts,
    };

    // Upload to server
    upload_experiment(experiment_run, &server).await?;

    Ok(())
}

fn detect_dtype_from_safetensors(model_path: &str) -> Result<Option<String>> {
    use std::path::Path;
    use std::fs::File;
    use memmap2::MmapOptions;
    use safetensors::SafeTensors;

    let model_dir = Path::new(model_path);

    // Look for safetensors files (try common patterns)
    let safetensor_patterns = vec![
        "model.safetensors",
        "model-00001-of-*.safetensors",
    ];

    // Try to find any .safetensors file
    let mut safetensor_file = None;
    for pattern in safetensor_patterns {
        let potential_path = model_dir.join(pattern);
        if potential_path.exists() {
            safetensor_file = Some(potential_path);
            break;
        }
    }

    // If no exact match, look for any .safetensors file
    if safetensor_file.is_none() {
        if let Ok(entries) = std::fs::read_dir(model_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".safetensors") {
                        safetensor_file = Some(entry.path());
                        break;
                    }
                }
            }
        }
    }

    // If we found a safetensors file, read the dtype
    if let Some(st_path) = safetensor_file {
        let file = File::open(&st_path)?;
        let buffer = unsafe { MmapOptions::new().map(&file)? };
        let tensors = SafeTensors::deserialize(&buffer)?;

        // Get dtype from first tensor and convert immediately to avoid lifetime issues
        let dtype_result = {
            if let Some((_name, tensor_view)) = tensors.iter().next() {
                use safetensors::Dtype;
                match tensor_view.dtype() {
                    Dtype::F16 => Some("FP16".to_string()),
                    Dtype::BF16 => Some("BF16".to_string()),
                    Dtype::F32 => Some("FP32".to_string()),
                    Dtype::F8_E4M3 | Dtype::F8_E5M2 => Some("FP8_DYNAMIC".to_string()),
                    _ => None, // Other dtypes - might be quantized or newer FP8 variants
                }
            } else {
                None
            }
        };

        return Ok(dtype_result);
    }

    Ok(None)
}

fn detect_quantization_from_config(model_path: &str) -> Result<Option<String>> {
    use std::path::Path;

    // Construct path to config.json
    let config_path = Path::new(model_path).join("config.json");

    // If config doesn't exist, try safetensors directly
    if !config_path.exists() {
        return detect_dtype_from_safetensors(model_path);
    }

    // Read and parse config.json
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Check for quantization_config first (for quantized models)
    if let Some(quant_config) = config.get("quantization_config") {
        // Check for FP8 quantization
        if let Some(quant_method) = quant_config.get("quant_method") {
            match quant_method.as_str() {
                Some("fp8") => {
                    // Check if dynamic or static
                    let is_dynamic = quant_config.get("dynamic")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if is_dynamic {
                        return Ok(Some("FP8_DYNAMIC".to_string()));
                    } else {
                        return Ok(Some("FP8".to_string()));
                    }
                }
                Some("awq") => {
                    // AWQ quantization - include method name for disambiguation
                    let bits = quant_config.get("bits")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(4);
                    return Ok(Some(format!("W{}A16-AWQ", bits)));
                }
                Some("gptq") => {
                    // GPTQ quantization - include method name for disambiguation
                    let bits = quant_config.get("bits")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(4);
                    return Ok(Some(format!("W{}A16-GPTQ", bits)));
                }
                Some("compressed-tensors") => {
                    // llmcompressor/compressed-tensors quantization
                    // Extract bits from config_groups
                    if let Some(config_groups) = quant_config.get("config_groups") {
                        if let Some(group_0) = config_groups.get("group_0") {
                            if let Some(weights) = group_0.get("weights") {
                                let bits = weights.get("num_bits")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(4);
                                return Ok(Some(format!("W{}A16-CT", bits)));
                            }
                        }
                    }
                    // Fallback if structure is different
                    return Ok(Some("W4A16-CT".to_string()));
                }
                _ => {}
            }
        }

        // Add more quantization patterns as needed
    }

    // Check torch_dtype for unquantized models
    if let Some(dtype) = config.get("torch_dtype") {
        match dtype.as_str() {
            Some("bfloat16") | Some("bf16") => return Ok(Some("BF16".to_string())),
            Some("float16") | Some("fp16") => return Ok(Some("FP16".to_string())),
            Some("float32") | Some("fp32") => return Ok(Some("FP32".to_string())),
            Some("auto") => {
                // For "auto", try to detect from safetensors weights
                if let Ok(Some(detected)) = detect_dtype_from_safetensors(model_path) {
                    return Ok(Some(detected));
                }
                // Fall back to path-based detection
                return Ok(None);
            }
            _ => {}
        }
    }

    // If dtype not found in config, try safetensors
    if config.get("torch_dtype").is_none() {
        if let Ok(Some(detected)) = detect_dtype_from_safetensors(model_path) {
            return Ok(Some(detected));
        }
    }

    // No recognizable quantization found, fall back to path-based
    Ok(None)
}

fn extract_model_info_from_path(model_path: &str) -> Result<(String, String)> {
    use regex::Regex;

    let path = std::path::Path::new(model_path);

    // Check if this is a GGUF file
    if model_path.ends_with(".gguf") {
        let filename = path.file_stem()
            .ok_or_else(|| anyhow!("Invalid GGUF filename"))?
            .to_string_lossy();

        // Extract quantization from filename (e.g., Q4_0 from "Qwen3-4B.Q4_0")
        let gguf_quant_patterns = vec![
            Regex::new(r"\.([A-Z][\dA-Z_]+)$").unwrap(),  // Matches .Q4_0, .Q4_K_M, .FP16, .MXFP4, .IQ4_XS, etc.
        ];

        let mut quantization = "F16-GGUF".to_string();
        for pattern in &gguf_quant_patterns {
            if let Some(caps) = pattern.captures(&filename) {
                quantization = format!("{}-GGUF", caps.get(1).unwrap().as_str());
                break;
            }
        }

        // Extract base model name from filename (remove quantization suffix)
        let base_name = if let Some(caps) = Regex::new(r"^(.+?)\.([QF][\dA-Z_]+)$").unwrap().captures(&filename) {
            caps.get(1).unwrap().as_str()
        } else {
            filename.as_ref()
        };

        // Build HuggingFace-style model slug (org/model)
        // For path like: /Volumes/Models/GGUF/Qwen/Qwen3-4B/Qwen3-4B.Q4_0.gguf
        // We want: Qwen/Qwen3-4B (components[-3]/components[-2] without the file)
        let components: Vec<_> = path.components().collect();
        let model_name = if components.len() >= 3 {
            let len = components.len();
            // Get grandparent directory (organization) and parent directory (model)
            format!(
                "{}/{}",
                components[len - 3].as_os_str().to_string_lossy(),
                components[len - 2].as_os_str().to_string_lossy()
            )
        } else {
            base_name.to_string()
        };

        return Ok((model_name, quantization));
    }

    // For non-GGUF files, use the original path-based detection
    let quant_patterns = vec![
        Regex::new(r"/(W\d+A\d+)/").unwrap(),        // W4A16 style
        Regex::new(r"/(Q\d+_[KM]_[SM])/").unwrap(),  // Q4_K_M style
        Regex::new(r"/(Q\d+_\d+)/").unwrap(),        // Q4_0 style
        Regex::new(r"/(FP\d+)/").unwrap(),           // FP16 style
        Regex::new(r"/(BF\d+)/").unwrap(),           // BF16 style
        Regex::new(r"/(FP8_DYNAMIC)/").unwrap(),     // FP8_DYNAMIC style
    ];

    let mut path_based_quant: Option<String> = None;
    for pattern in &quant_patterns {
        if let Some(caps) = pattern.captures(model_path) {
            path_based_quant = Some(caps.get(1).unwrap().as_str().to_string());
            break;
        }
    }

    // If path-based detection found something, use it (most reliable)
    let quantization = if let Some(mut quant) = path_based_quant {
        // Append -CT suffix to W4A16 patterns from path (all existing data is from llmcompressor)
        if quant.starts_with("W") && quant.contains("A16") && !quant.contains("-") {
            quant = format!("{}-CT", quant);
        }
        quant
    } else {
        // Fall back to config.json detection for models without explicit path markers
        match detect_quantization_from_config(model_path) {
            Ok(Some(quant)) => quant,
            Ok(None) | Err(_) => "FP16".to_string(), // Default to FP16
        }
    };

    // Extract model name from path (last two components)
    let components: Vec<_> = path.components().collect();
    let model_name = if components.len() >= 2 {
        let len = components.len();
        format!(
            "{}/{}",
            components[len - 2].as_os_str().to_string_lossy(),
            components[len - 1].as_os_str().to_string_lossy()
        )
    } else if !components.is_empty() {
        components.last().unwrap().as_os_str().to_string_lossy().to_string()
    } else {
        "unknown".to_string()
    };

    Ok((model_name, quantization))
}

fn parse_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
    // Handle ISO8601 with 'Z' suffix
    let timestamp = if timestamp_str.ends_with('Z') {
        DateTime::parse_from_rfc3339(&timestamp_str.replace('Z', "+00:00"))?
            .with_timezone(&Utc)
    } else {
        DateTime::parse_from_rfc3339(timestamp_str)?
            .with_timezone(&Utc)
    };
    Ok(timestamp)
}

fn detect_system_hardware() -> Result<HardwareConfig> {
    use std::process::Command;

    // Detect CPU info
    let cpu_info = if cfg!(target_os = "linux") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("grep -m 1 'model name' /proc/cpuinfo | cut -d ':' -f2 | xargs")
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "Unknown CPU".to_string(),
        }
    } else if cfg!(target_os = "macos") {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "Unknown CPU".to_string(),
        }
    } else {
        "Unknown CPU".to_string()
    };

    let cpu_arch = detect_cpu_arch(&cpu_info);

    // Detect GPU info
    let (gpu_memory_gb, gpu_model) = detect_gpu_info()?;

    Ok(HardwareConfig {
        gpu_model,
        gpu_memory_gb,
        cpu_model: cpu_info,
        cpu_arch: cpu_arch.to_string(),
        ram_gb: detect_ram_gb(),
        ram_type: None,
        virtualization_type: None,
        optimizations: Vec::new(),
    })
}

fn detect_gpu_info() -> Result<(i32, String)> {
    use std::process::Command;

    // Try nvidia-smi first
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=name,memory.total")
        .arg("--format=csv,noheader,nounits")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() >= 2 {
                let gpu_name = parts[0].trim()
                    .replace("NVIDIA GeForce ", "")
                    .replace("NVIDIA ", "");
                let memory_mb: i32 = parts[1].trim().parse().unwrap_or(0);
                let memory_gb = memory_mb / 1024;
                return Ok((memory_gb, gpu_name));
            }
        }
    }

    // Try Apple Silicon GPU detection on macOS
    if cfg!(target_os = "macos") {
        let chip_output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output();

        if let Ok(output) = chip_output {
            let chip_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Check if it's an Apple Silicon chip (M1, M2, M3, M4, etc.)
            if chip_name.contains("Apple M") {
                // Extract the chip model (M1, M2, M3, M4)
                let gpu_name = if chip_name.contains("Max") {
                    chip_name.split_whitespace()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(" ")
                } else if chip_name.contains("Pro") || chip_name.contains("Ultra") {
                    chip_name.split_whitespace()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    chip_name.split_whitespace()
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                // Get unified memory size
                let mem_output = Command::new("sysctl")
                    .arg("-n")
                    .arg("hw.memsize")
                    .output();

                let memory_gb = if let Ok(mem_out) = mem_output {
                    let mem_bytes: i64 = String::from_utf8_lossy(&mem_out.stdout)
                        .trim()
                        .parse()
                        .unwrap_or(0);
                    (mem_bytes / 1024 / 1024 / 1024) as i32
                } else {
                    0
                };

                return Ok((memory_gb, gpu_name));
            }
        }
    }

    // Fallback to CPU-only
    Ok((0, "CPU Only".to_string()))
}

fn detect_ram_gb() -> Option<i32> {
    use std::process::Command;

    if cfg!(target_os = "linux") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("grep MemTotal /proc/meminfo | awk '{print $2}'")
            .output();

        if let Ok(output) = output {
            let mem_kb: i64 = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            return Some((mem_kb / 1024 / 1024) as i32);
        }
    } else if cfg!(target_os = "macos") {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output();

        if let Ok(output) = output {
            let mem_bytes: i64 = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            return Some((mem_bytes / 1024 / 1024 / 1024) as i32);
        }
    }

    None
}

async fn upload_benchmark_scores(request: llm_benchmark_types::UploadBenchmarkRequest, server: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/benchmarks/upload", server);
    
    println!("Uploading benchmark scores to {}...", url);
    println!("Model: {}/{}", request.model_name, request.quantization);
    println!("Benchmarks: {} scores", request.benchmark_scores.len());
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: llm_benchmark_types::UploadBenchmarkResponse = response.json().await?;
        if result.success {
            println!("✅ Upload successful!");
            println!("{}", result.message);
            if let Some(variant_id) = result.model_variant_id {
                println!("Model variant ID: {}", variant_id);
            }
        } else {
            return Err(anyhow!("Upload failed: {}", result.message));
        }
    } else {
        let error_text = response.text().await?;
        return Err(anyhow!("Upload failed: {}", error_text));
    }
    
    Ok(())
}

async fn upload_custom(file: PathBuf, server: String) -> Result<()> {
    let content = std::fs::read_to_string(&file)?;
    let experiment_run: ExperimentRun = serde_json::from_str(&content)?;
    
    upload_experiment(experiment_run, &server).await?;
    
    Ok(())
}

fn parse_hardware_info(result: &LlamaBenchResult) -> Result<HardwareConfig> {
    // Parse CPU architecture from CPU info string
    let cpu_arch = detect_cpu_arch(&result.cpu_info);
    
    // Determine GPU memory from GPU info
    let (gpu_memory_gb, gpu_model) = parse_gpu_info(&result.gpu_info);
    
    // Extract optimizations from backends
    let mut optimizations = Vec::new();
    if result.backends.contains("CUDA") {
        optimizations.push("CUDA".to_string());
    }
    if result.backends.contains("ROCM") {
        optimizations.push("ROCm".to_string());
    }
    if result.backends.contains("Metal") {
        optimizations.push("Metal".to_string());
    }
    if result.backends.contains("AVX") {
        optimizations.push("AVX".to_string());
    }
    if result.backends.contains("AVX2") {
        optimizations.push("AVX2".to_string());
    }
    if result.flash_attn {
        optimizations.push("FlashAttention".to_string());
    }
    
    Ok(HardwareConfig {
        gpu_model,
        gpu_memory_gb,
        cpu_model: result.cpu_info.clone(),
        cpu_arch: cpu_arch.to_string(),
        ram_gb: None, // Not available in llama-bench output
        ram_type: None, // Not available in llama-bench output
        virtualization_type: None,
        optimizations,
    })
}

fn detect_cpu_arch(cpu_info: &str) -> &'static str {
    let cpu_lower = cpu_info.to_lowercase();
    
    if cpu_lower.contains("threadripper 3") || cpu_lower.contains("ryzen 3") {
        "zen2"
    } else if cpu_lower.contains("threadripper 5") || cpu_lower.contains("ryzen 5") {
        "zen3"
    } else if cpu_lower.contains("threadripper 7") || cpu_lower.contains("ryzen 7") || cpu_lower.contains("ryzen 9") {
        "zen4"
    } else if cpu_lower.contains("intel") && cpu_lower.contains("12th") {
        "alderlake"
    } else if cpu_lower.contains("intel") && cpu_lower.contains("13th") {
        "raptorlake"
    } else if cpu_lower.contains("apple m1") {
        "apple_m1"
    } else if cpu_lower.contains("apple m2") {
        "apple_m2"  
    } else if cpu_lower.contains("apple m3") {
        "apple_m3"
    } else {
        "unknown"
    }
}

fn parse_gpu_info(gpu_info: &str) -> (i32, String) {
    // Handle CPU-only systems
    if gpu_info.is_empty() || gpu_info == "" {
        return (0, "CPU Only".to_string());
    }
    
    let gpu_lower = gpu_info.to_lowercase();
    
    // Clean up the GPU name - remove common prefixes
    let clean_name = gpu_info
        .replace("NVIDIA GeForce ", "")
        .replace("NVIDIA ", "")
        .replace("AMD Radeon ", "")
        .replace("Intel Arc ", "Arc ")
        .trim()
        .to_string();
    
    // Determine memory based on GPU model
    let memory_gb = if gpu_lower.contains("rtx 4090") {
        24
    } else if gpu_lower.contains("rtx 4080") {
        16
    } else if gpu_lower.contains("rtx 4070 ti") {
        12
    } else if gpu_lower.contains("rtx 4070") {
        12
    } else if gpu_lower.contains("rtx 3090") {
        24
    } else if gpu_lower.contains("rtx 3080") {
        10
    } else if gpu_lower.contains("a100") && gpu_lower.contains("80") {
        80
    } else if gpu_lower.contains("a100") && gpu_lower.contains("40") {
        40
    } else if gpu_lower.contains("a100") {
        40 // Default A100 size
    } else if gpu_lower.contains("h100") {
        80
    } else if gpu_lower.contains("7900") {
        24
    } else {
        0 // Unknown or CPU-only
    };
    
    (memory_gb, clean_name)
}

fn parse_model_filename(filename: &str) -> Result<ModelInfo> {
    // Example: /mnt/llm-models/GGUF/TheDrummer/Snowpiercer-15B-v1/Snowpiercer-15B-v1.Q3_K_L.gguf
    let path = std::path::Path::new(filename);
    let file_name = path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid filename"))?;
    
    // Extract parent directories for model name
    let mut path_components = Vec::new();
    let mut current_path = path.parent();
    
    while let Some(parent) = current_path {
        if let Some(name) = parent.file_name().and_then(|s| s.to_str()) {
            if name != "GGUF" && name != "models" && name != "llm-models" {
                path_components.push(name);
            }
        }
        current_path = parent.parent();
    }
    
    // Build model name from path components
    path_components.reverse();
    let model_name = if path_components.len() >= 2 {
        // Likely format: owner/model-name
        format!("{}/{}", path_components[path_components.len()-2], path_components[path_components.len()-1])
    } else if !path_components.is_empty() {
        path_components.last().unwrap().to_string()
    } else {
        // Fall back to parsing from filename
        file_name.split('.').next().unwrap_or("unknown").to_string()
    };
    
    // Extract quantization - it's typically the last part before .gguf
    let parts: Vec<&str> = file_name.split('.').collect();
    let quantization = if parts.len() >= 2 {
        // Look for quantization patterns
        let potential_quant = parts[parts.len() - 1];
        if potential_quant.starts_with('Q') || potential_quant.starts_with('F') {
            potential_quant.to_string()
        } else if parts.len() >= 3 {
            parts[parts.len() - 2].to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };
    
    Ok(ModelInfo {
        name: model_name,
        quantization,
    })
}

async fn upload_benchmarks_only(test_run_id: String, file: PathBuf, _server: String) -> Result<()> {
    println!("Uploading benchmark scores for test run {}...", test_run_id);
    
    // Read benchmark scores
    let content = std::fs::read_to_string(&file)?;
    let benchmark_scores: Vec<BenchmarkScoreType> = serde_json::from_str(&content)?;
    
    // TODO: Implement API endpoint for adding benchmarks to existing test run
    // For now, this is a placeholder
    println!("⚠️  Note: The API endpoint for adding benchmarks to existing test runs is not yet implemented.");
    println!("Benchmark scores loaded: {} benchmarks", benchmark_scores.len());
    
    Ok(())
}

async fn upload_mmlu_pro(
    file: PathBuf,
    test_run_id: Option<String>,
    model_path: Option<String>,
    model: Option<String>,
    quantization: Option<String>,
    server: String,
    _backend: String,
    _notes: Option<String>,
) -> Result<()> {

    // Auto-detect model name and quantization from model_path if provided
    let (model, quantization) = if let Some(path) = model_path {
        let (detected_model, detected_quant) = extract_model_info_from_path(&path)?;
        (Some(model.unwrap_or(detected_model)), Some(quantization.unwrap_or(detected_quant)))
    } else {
        (model, quantization)
    };

    // Read and parse the report.txt file
    let content = std::fs::read_to_string(&file)?;
    let mut categories = Vec::new();
    let mut overall_score = 0.0;
    let mut test_timestamp = chrono::Utc::now();
    
    // Parse lines looking for category scores
    let lines: Vec<&str> = content.lines().collect();
    
    for line in lines {
        // Look for timestamp at the beginning (only if line looks like a timestamp)
        if line.len() > 20 && line.chars().nth(4) == Some('-') && line.chars().nth(7) == Some('-') {
            // Parse without timezone first, then convert to UTC
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(line.trim(), "%Y-%m-%d %H:%M:%S%.f") {
                test_timestamp = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
            }
        }
        
        // Skip individual category parsing since we'll use the markdown table
        // The report format has individual categories but we'll parse from the summary table
        
        // Look for the markdown table with all scores
        if line.starts_with("| overall") {
            // Skip the header and separator lines
            continue;
        } else if line.starts_with("| ") && line.contains(" | ") && !line.contains("---") {
            // This is the data row with all scores
            let parts: Vec<&str> = line.split(" | ").map(|s| s.trim_matches('|').trim()).collect();
            if parts.len() >= 15 {
                // Parse overall score
                overall_score = parts[0].parse::<f64>().unwrap_or(0.0);
                
                // Category names in order from the header
                let category_names = vec![
                    "biology", "business", "chemistry", "computer science", "economics",
                    "engineering", "health", "history", "law", "math",
                    "philosophy", "physics", "psychology", "other"
                ];
                
                // Actual MMLU-Pro question counts per category
                let question_counts: Vec<i32> = vec![
                    71,   // biology
                    78,   // business
                    113,  // chemistry
                    41,   // computer science
                    84,   // economics
                    96,   // engineering
                    81,   // health
                    38,   // history
                    110,  // law
                    135,  // math
                    49,   // philosophy
                    129,  // physics
                    79,   // psychology
                    92    // other
                ];
                
                // Parse individual category scores
                for (i, category_name) in category_names.iter().enumerate() {
                    if let Ok(score) = parts[i + 1].parse::<f64>() {
                        // Use actual MMLU-Pro question counts
                        let total_questions = question_counts[i];
                        let estimated_correct = (score / 100.0 * total_questions as f64).round() as i32;
                        
                        categories.push(MMLUCategoryScore {
                            category: category_name.to_string(),
                            score,
                            total_questions,
                            correct_answers: estimated_correct,
                        });
                    }
                }
            }
        }
    }
    
    // Create MMLU score
    let mmlu_score = MMLUScore {
        categories,
        timestamp: test_timestamp,
        context: Some(serde_json::json!({
            "source": "mmlu-pro",
            "report_file": file.to_string_lossy(),
            "overall_score": overall_score,
            "note": "Question counts are estimated as report.txt doesn't include them"
        })),
    };
    
    // Check if we're uploading to an existing test run or creating benchmark scores
    if let Some(_test_id) = test_run_id {
        // TODO: Implement adding benchmarks to existing test run
        // For now, this is not supported
        return Err(anyhow!("Adding benchmarks to existing test runs is not yet implemented"));
    }
    
    // Ensure we have model and quantization
    let model = model.ok_or_else(|| anyhow!("Model name is required when not specifying test run ID"))?;
    let quantization = quantization.ok_or_else(|| anyhow!("Quantization is required when not specifying test run ID"))?;
    
    // Upload benchmark scores to the new endpoint
    let upload_request = llm_benchmark_types::UploadBenchmarkRequest {
        model_name: model,
        quantization,
        benchmark_scores: vec![BenchmarkScoreType::MMLU(mmlu_score)],
        timestamp: Some(test_timestamp),
    };
    
    upload_benchmark_scores(upload_request, &server).await?;
    
    Ok(())
}

async fn upload_experiment(experiment_run: ExperimentRun, server: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/upload-experiment", server);

    let request = UploadRequest { experiment_run };

    println!("Uploading experiment to {}...", url);

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;

    let result: llm_benchmark_types::UploadExperimentResponse = response.json().await?;

    if result.success {
        println!("✅ Upload successful!");
        if let Some(test_run_id) = result.test_run_id {
            println!("Test run ID: {}", test_run_id);
        }
        if !result.warnings.is_empty() {
            println!("⚠️  Warnings:");
            for warning in &result.warnings {
                println!("  - {}", warning);
            }
        }
    } else {
        let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
        return Err(anyhow!("Upload failed: {}", error_msg));
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabGraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<SystemslabGraphQLError>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabGraphQLError {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabExperimentData {
    #[serde(rename = "experimentById")]
    experiment_by_id: Option<SystemslabExperiment>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabContextData {
    #[serde(rename = "contextById")]
    context_by_id: SystemslabContext,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabContext {
    id: String,
    name: String,
    state: String,
    experiments: Vec<SystemslabContextExperiment>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabContextExperiment {
    experiment: Option<SystemslabNestedExperiment>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabNestedExperiment {
    id: String,
    name: String,
    state: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabExperiment {
    id: String,
    name: String,
    state: String,
    artifact: Vec<SystemslabArtifact>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemslabArtifact {
    id: String,
    name: String,
    #[serde(rename = "experimentId")]
    experiment_id: Option<String>,
    #[serde(rename = "jobId")]
    job_id: Option<String>,
    #[serde(rename = "runId")]
    run_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SystemInfo {
    hwinfo: HwInfo,
}

#[derive(Debug, Deserialize)]
struct HwInfo {
    cpus: Vec<CpuInfo>,
    memory: MemoryInfo,
}

#[derive(Debug, Deserialize)]
struct CpuInfo {
    model_name: String,
}

#[derive(Debug, Deserialize)]
struct MemoryInfo {
    total_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct ExperimentJson {
    name: String,
    jobs: std::collections::HashMap<String, ExperimentJob>,
}

#[derive(Debug, Deserialize)]
struct ExperimentJob {
    steps: Vec<ExperimentStep>,
}

#[derive(Debug, Deserialize)]
struct ExperimentStep {
    uses: String,
    with: serde_json::Value,
    #[serde(default)]
    background: bool,
}

/// Upload local benchmark results with llm.json configuration
async fn upload_local_results(
    llm_json_path: PathBuf,
    results_json_path: PathBuf,
    server: String,
) -> Result<()> {
    println!("📤 Uploading local benchmark results...");

    // Read llm.json
    println!("Reading llm.json from: {}", llm_json_path.display());
    let llm_json_content = std::fs::read_to_string(&llm_json_path)?;
    let mut artifact: BenchmarkArtifact = serde_json::from_str(&llm_json_content)?;

    // Fix quantization naming: append -CT to W*A16 patterns (all existing data is from llmcompressor)
    if artifact.quantization.starts_with("W") && artifact.quantization.contains("A16") && !artifact.quantization.contains("-") {
        artifact.quantization = format!("{}-CT", artifact.quantization);
    }

    println!("Parsed llm.json:");
    println!("  GPU: {} x {}", artifact.gpu_count, artifact.gpu_model);
    if let Some(pl) = artifact.gpu_power_limit_watts {
        println!("  Power Limit: {}W", pl);
    }
    println!("  CPU: {} ({})", artifact.cpu_model, artifact.cpu_arch);
    println!("  Model: {}", artifact.model_name);
    println!("  Quantization: {}", artifact.quantization);
    if let Some(backend) = &artifact.backend_name {
        if let Some(version) = &artifact.backend_version {
            println!("  Backend: {} v{}", backend, version);
        } else {
            println!("  Backend: {}", backend);
        }
    }
    if let Some(concurrency) = artifact.concurrent_requests {
        println!("  Concurrent Requests: {}", concurrency);
    }
    if let Some(ctx_len) = artifact.max_context_length {
        println!("  Max Context Length: {}", ctx_len);
    }

    // Read results.json
    println!("\nReading results.json from: {}", results_json_path.display());
    let results_content = std::fs::read_to_string(&results_json_path)?;
    let result: InferenceServerResult = serde_json::from_str(&results_content)?;

    println!("Parsed results successfully");
    println!("Throughput: {} tok/s", result.throughput.output_tokens_per_second);

    // Build hardware config from artifact
    let hardware_config = HardwareConfig {
        gpu_model: artifact.gpu_model.clone(),
        gpu_memory_gb: artifact.gpu_memory_gb,
        cpu_model: artifact.cpu_model.clone(),
        cpu_arch: artifact.cpu_arch.clone(),
        ram_gb: artifact.ram_gb,
        ram_type: None,
        virtualization_type: None,
        optimizations: Vec::new(),
    };

    // Parse timestamp
    let timestamp = parse_timestamp(&result.timestamp)?;

    // Create performance metrics
    let performance_metrics = build_performance_metrics(&result, timestamp);

    // Build notes
    let notes_str = format!(
        "Uploaded via llm-perf | Benchmark tool: {} | Duration: {}s",
        result.version,
        result.configuration.duration_seconds
    );

    // Extract dataset name
    let dataset_name = result.configuration.prompt_file.as_ref()
        .and_then(|p| std::path::Path::new(p).file_stem())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Generate or load experiment ID
    let experiment_dir = llm_json_path.parent()
        .ok_or_else(|| anyhow!("Could not determine parent directory of llm.json"))?;
    let experiment_id_path = experiment_dir.join(".experiment-id");

    let exp_uuid = if experiment_id_path.exists() {
        // Load existing ID
        let id_str = std::fs::read_to_string(&experiment_id_path)?;
        Uuid::parse_str(id_str.trim())
            .map_err(|e| anyhow!("Failed to parse existing experiment ID: {}", e))?
    } else {
        // Generate new UUID v7
        let new_id = Uuid::now_v7();
        std::fs::write(&experiment_id_path, new_id.to_string())?;
        println!("Generated new experiment ID: {}", new_id);
        new_id
    };

    // Create experiment run
    let experiment_run = ExperimentRun {
        id: exp_uuid,
        model_name: artifact.model_name.clone(),
        quantization: artifact.quantization.clone(),
        backend: artifact.backend_name.clone().unwrap_or_else(|| "Unknown".to_string()),
        backend_version: artifact.backend_version.clone().unwrap_or_else(|| result.version.clone()),
        hardware_config,
        performance_metrics,
        benchmark_scores: Vec::new(),
        timestamp,
        status: ExperimentStatus::Completed,
        notes: Some(notes_str),
        concurrent_requests: artifact.concurrent_requests.or(Some(result.configuration.concurrent_requests)),
        max_context_length: artifact.max_context_length,
        load_pattern: Some(result.configuration.load_pattern.clone()),
        dataset_name,
        gpu_power_limit_watts: artifact.gpu_power_limit_watts,
    };

    // Upload to server
    println!("\n📡 Uploading to server: {}", server);
    upload_experiment(experiment_run, &server).await?;

    println!("✅ Upload successful!");

    Ok(())
}

/// Import experiments or contexts from SystemsLab (unified function)
async fn import_from_systemslab(
    id: String,
    systemslab_url: String,
    server: String,
) -> Result<()> {
    let client = reqwest::Client::new();

    // Try to fetch as an experiment first
    let experiment_query = serde_json::json!({
        "query": format!(
            r#"query {{ experimentById(id: "{}") {{ id name state artifact {{ id name experimentId jobId runId }} }} }}"#,
            id
        )
    });

    let graphql_url = format!("{}/api/graphql", systemslab_url);
    let response = client
        .post(&graphql_url)
        .json(&experiment_query)
        .send()
        .await?;

    let experiment_result: SystemslabGraphQLResponse<SystemslabExperimentData> = response.json().await?;

    // Check if we got an experiment
    if experiment_result.errors.is_none()
        && experiment_result.data.is_some()
        && experiment_result.data.as_ref().unwrap().experiment_by_id.is_some() {
        println!("Detected as experiment ID");
        return import_single_experiment(id, systemslab_url, server, &client).await;
    }

    // Try as a context
    println!("Not an experiment, trying as context ID...");
    let context_query = serde_json::json!({
        "query": format!(
            r#"query {{ contextById(id: "{}") {{ id name state experiments {{ experiment {{ id name state }} }} }} }}"#,
            id
        )
    });

    let response = client
        .post(&graphql_url)
        .json(&context_query)
        .send()
        .await?;

    let context_result: SystemslabGraphQLResponse<SystemslabContextData> = response.json().await?;

    if let Some(errors) = context_result.errors {
        return Err(anyhow!("ID not found as experiment or context: {:?}", errors));
    }

    let context_data = context_result.data
        .ok_or_else(|| anyhow!("No data returned for context"))?;

    let context = context_data.context_by_id;

    println!("Detected as context ID");
    println!("Found context: {}", context.name);
    println!("State: {}", context.state);
    println!("Experiments: {}", context.experiments.len());
    println!();

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut skipped_count = 0;

    for (idx, context_exp) in context.experiments.iter().enumerate() {
        // Skip experiments that are still running (experiment is None)
        let Some(experiment) = &context_exp.experiment else {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("[{}/{}] Skipping (experiment still running or null)", idx + 1, context.experiments.len());
            skipped_count += 1;
            println!();
            continue;
        };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("[{}/{}] Importing: {}", idx + 1, context.experiments.len(), experiment.name);
        println!("Experiment ID: {}", experiment.id);
        println!("State: {}", experiment.state);

        // Skip experiments that aren't successful
        if experiment.state != "success" {
            println!("⊗ Skipping (state: {})", experiment.state);
            skipped_count += 1;
            println!();
            continue;
        }

        // Import the experiment
        match import_single_experiment(
            experiment.id.clone(),
            systemslab_url.clone(),
            server.clone(),
            &client,
        ).await {
            Ok(_) => {
                success_count += 1;
                println!("✅ Success");
            }
            Err(e) => {
                failure_count += 1;
                println!("❌ Failed: {}", e);

                println!("⚠️  Continuing to next experiment...");
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Import Summary:");
    println!("  ✅ Successful: {}", success_count);
    println!("  ❌ Failed: {}", failure_count);
    println!("  ⊗ Skipped: {}", skipped_count);
    println!("  📊 Total: {}", context.experiments.len());

    Ok(())
}

/// Import a single experiment from SystemsLab
async fn import_single_experiment(
    experiment_id: String,
    systemslab_url: String,
    server: String,
    client: &reqwest::Client,
) -> Result<()> {
    println!("Fetching experiment: {}", experiment_id);

    // Query GraphQL API for experiment metadata
    let graphql_query = serde_json::json!({
        "query": format!(
            r#"query {{ experimentById(id: "{}") {{ id name state artifact {{ id name experimentId jobId runId }} }} }}"#,
            experiment_id
        )
    });

    let graphql_url = format!("{}/api/graphql", systemslab_url);
    let response = client
        .post(&graphql_url)
        .json(&graphql_query)
        .send()
        .await?;

    let graphql_result: SystemslabGraphQLResponse<SystemslabExperimentData> = response.json().await?;

    if let Some(errors) = graphql_result.errors {
        return Err(anyhow!("GraphQL errors: {:?}", errors));
    }

    let experiment_data = graphql_result.data
        .ok_or_else(|| anyhow!("No data returned from GraphQL"))?;

    let experiment = experiment_data.experiment_by_id
        .ok_or_else(|| anyhow!("Experiment not found"))?;

    println!("Found experiment: {}", experiment.name);
    println!("State: {}", experiment.state);

    // Check for llm.json (required!)
    let llm_artifact = experiment.artifact.iter()
        .find(|a| a.name == "llm.json")
        .ok_or_else(|| anyhow!("llm.json artifact not found - please re-run experiment with 'llm-perf record' step"))?;

    println!("Found llm.json artifact");

    // Check what type of experiment this is
    let has_results = experiment.artifact.iter().any(|a| a.name == "results.json");
    let has_mmlu = experiment.artifact.iter().any(|a| a.name == "mmlu-results.json" || a.name == "report.txt");

    if has_results {
        println!("Detected as performance experiment (results.json found)");
        upload_performance_from_systemslab(
            experiment_id,
            systemslab_url,
            server,
            client,
            llm_artifact.id.clone(),
            &experiment.artifact,
        ).await
    } else if has_mmlu {
        println!("Detected as MMLU experiment (MMLU artifact found)");
        upload_mmlu_from_systemslab(
            experiment_id,
            systemslab_url,
            server,
            client,
            llm_artifact.id.clone(),
            &experiment.artifact,
        ).await
    } else {
        Err(anyhow!("No benchmark results found - experiment must have either results.json or mmlu-results.json/report.txt"))
    }
}

/// Upload performance experiment from SystemsLab using llm.json + results.json
async fn upload_performance_from_systemslab(
    experiment_id: String,
    systemslab_url: String,
    server: String,
    client: &reqwest::Client,
    llm_artifact_id: String,
    all_artifacts: &[SystemslabArtifact],
) -> Result<()> {
    // Download llm.json
    println!("Downloading llm.json...");
    let artifact_url = format!("{}/api/v1/artifact/{}", systemslab_url, llm_artifact_id);

    let response = client.get(&artifact_url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("Failed to download llm.json: HTTP {}", response.status()));
    }

    let artifact_content = response.text().await?;
    let mut artifact: BenchmarkArtifact = serde_json::from_str(&artifact_content)?;

    // Fix quantization naming: append -CT to W*A16 patterns (all existing data is from llmcompressor)
    if artifact.quantization.starts_with("W") && artifact.quantization.contains("A16") && !artifact.quantization.contains("-") {
        artifact.quantization = format!("{}-CT", artifact.quantization);
    }

    println!("Parsed benchmark artifact:");
    println!("  GPU: {} x {}", artifact.gpu_count, artifact.gpu_model);
    if let Some(pl) = artifact.gpu_power_limit_watts {
        println!("  Power Limit: {}W", pl);
    }
    println!("  CPU: {} ({})", artifact.cpu_model, artifact.cpu_arch);
    println!("  Model: {}", artifact.model_name);
    println!("  Quantization: {}", artifact.quantization);
    if let Some(backend) = &artifact.backend_name {
        if let Some(version) = &artifact.backend_version {
            println!("  Backend: {} v{}", backend, version);
        } else {
            println!("  Backend: {}", backend);
        }
    }

    // Get results.json artifact
    let results_artifact = all_artifacts.iter()
        .find(|a| a.name == "results.json")
        .ok_or_else(|| anyhow!("results.json artifact not found"))?;

    println!("Downloading results.json...");
    let results_url = format!("{}/api/v1/artifact/{}", systemslab_url, results_artifact.id);

    let results_response = client.get(&results_url).send().await?;
    if !results_response.status().is_success() {
        return Err(anyhow!("Failed to download results.json: HTTP {}", results_response.status()));
    }

    let results_content = results_response.text().await?;
    let result: InferenceServerResult = serde_json::from_str(&results_content)?;

    println!("Parsed results successfully");
    println!("Throughput: {} tok/s", result.throughput.output_tokens_per_second);

    // Build hardware config from artifact
    let hardware_config = HardwareConfig {
        gpu_model: artifact.gpu_model.clone(),
        gpu_memory_gb: artifact.gpu_memory_gb,
        cpu_model: artifact.cpu_model.clone(),
        cpu_arch: artifact.cpu_arch.clone(),
        ram_gb: artifact.ram_gb,
        ram_type: None,
        virtualization_type: None,
        optimizations: Vec::new(),
    };

    // Parse timestamp
    let timestamp = parse_timestamp(&result.timestamp)?;

    // Create performance metrics
    let mut performance_metrics = build_performance_metrics(&result, timestamp);

    // Try to get GPU power consumption from metrics.parquet
    if let Some(metrics_artifact) = all_artifacts.iter().find(|a| a.name == "metrics.parquet") {
        if let Ok(Some(gpu_power)) = calculate_gpu_power_from_parquet(client, &systemslab_url, &metrics_artifact.id).await {
            performance_metrics.push(PerformanceMetric {
                metric_name: "gpu_power_watts".to_string(),
                value: gpu_power,
                unit: "W".to_string(),
                timestamp,
                context: None,
            });
        }
    }

    // Build notes
    let notes_str = format!(
        "Imported from SystemsLab | Experiment: {} | Benchmark tool: {} | Duration: {}s",
        experiment_id,
        result.version,
        result.configuration.duration_seconds
    );

    // Extract dataset name
    let dataset_name = result.configuration.prompt_file.as_ref()
        .and_then(|p| std::path::Path::new(p).file_stem())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Parse experiment ID as UUID
    let exp_uuid = Uuid::parse_str(&experiment_id)
        .map_err(|e| anyhow!("Failed to parse experiment ID as UUID: {}", e))?;

    // Create experiment run
    let experiment_run = ExperimentRun {
        id: exp_uuid,
        model_name: artifact.model_name.clone(),
        quantization: artifact.quantization.clone(),
        backend: artifact.backend_name.clone().unwrap_or_else(|| "vLLM".to_string()),
        backend_version: artifact.backend_version.clone().unwrap_or_else(|| result.version.clone()),
        hardware_config,
        performance_metrics,
        benchmark_scores: Vec::new(),
        timestamp,
        status: ExperimentStatus::Completed,
        notes: Some(notes_str),
        concurrent_requests: artifact.concurrent_requests.or(Some(result.configuration.concurrent_requests)),
        max_context_length: artifact.max_context_length,
        load_pattern: Some(result.configuration.load_pattern.clone()),
        dataset_name,
        gpu_power_limit_watts: artifact.gpu_power_limit_watts,
    };

    // Upload to server
    upload_experiment(experiment_run, &server).await?;

    Ok(())
}

/// Upload MMLU experiment from SystemsLab using llm.json + MMLU results
async fn upload_mmlu_from_systemslab(
    _experiment_id: String,
    systemslab_url: String,
    server: String,
    client: &reqwest::Client,
    llm_artifact_id: String,
    all_artifacts: &[SystemslabArtifact],
) -> Result<()> {
    // Download llm.json
    println!("Downloading llm.json...");
    let artifact_url = format!("{}/api/v1/artifact/{}", systemslab_url, llm_artifact_id);

    let response = client.get(&artifact_url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("Failed to download llm.json: HTTP {}", response.status()));
    }

    let artifact_content = response.text().await?;
    let mut artifact: BenchmarkArtifact = serde_json::from_str(&artifact_content)?;

    // Fix quantization naming: append -CT to W*A16 patterns (all existing data is from llmcompressor)
    if artifact.quantization.starts_with("W") && artifact.quantization.contains("A16") && !artifact.quantization.contains("-") {
        artifact.quantization = format!("{}-CT", artifact.quantization);
    }

    println!("Parsed llm.json:");
    println!("  Model: {}", artifact.model_name);
    println!("  Quantization: {}", artifact.quantization);

    // Find MMLU artifact (prefer mmlu-results.json, fall back to report.txt)
    let mmlu_artifact = all_artifacts.iter()
        .find(|a| a.name == "mmlu-results.json")
        .or_else(|| all_artifacts.iter().find(|a| a.name == "report.txt"))
        .ok_or_else(|| anyhow!("No MMLU artifact found"))?;

    println!("Downloading MMLU results: {}", mmlu_artifact.name);
    let mmlu_url = format!("{}/api/v1/artifact/{}", systemslab_url, mmlu_artifact.id);

    let mmlu_response = client.get(&mmlu_url).send().await?;
    if !mmlu_response.status().is_success() {
        return Err(anyhow!("Failed to download MMLU artifact: HTTP {}", mmlu_response.status()));
    }

    let mmlu_content = mmlu_response.text().await?;

    // Parse MMLU results based on format
    let mmlu_score = if mmlu_artifact.name == "mmlu-results.json" {
        // Parse JSON format
        serde_json::from_str::<MMLUScore>(&mmlu_content)?
    } else {
        // Parse report.txt format (MMLU-Pro)
        parse_mmlu_pro_report(&mmlu_content)?
    };

    println!("Parsed MMLU results: {:.2}% overall", mmlu_score.overall_score());

    // Create benchmark upload request
    let request = UploadBenchmarkRequest {
        model_name: artifact.model_name.clone(),
        quantization: artifact.quantization.clone(),
        benchmark_scores: vec![benchmarks::BenchmarkScoreType::MMLU(mmlu_score)],
        timestamp: Some(Utc::now()),
    };

    // Upload to server
    println!("📡 Uploading MMLU scores to server...");
    upload_benchmark_scores(request, &server).await?;

    println!("✅ MMLU scores uploaded successfully!");

    Ok(())
}

/// Parse MMLU-Pro report.txt format
fn parse_mmlu_pro_report(content: &str) -> Result<MMLUScore> {
    let mut categories = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        // Look for the markdown table with all scores
        if line.starts_with("| ") && line.contains(" | ") && !line.contains("---") && !line.starts_with("| overall") {
            // This is the data row with all scores
            let parts: Vec<&str> = line.split(" | ").map(|s| s.trim_matches('|').trim()).collect();
            if parts.len() >= 15 {
                // Category names in order from the header
                let category_names = vec![
                    "biology", "business", "chemistry", "computer science", "economics",
                    "engineering", "health", "history", "law", "math",
                    "philosophy", "physics", "psychology", "other"
                ];

                // Actual MMLU-Pro question counts per category
                let question_counts: Vec<i32> = vec![
                    71, 78, 113, 41, 84, 96, 81, 38, 110, 135, 49, 129, 79, 92
                ];

                // Parse individual category scores
                for (i, category_name) in category_names.iter().enumerate() {
                    if let Ok(score) = parts[i + 1].parse::<f64>() {
                        let total_questions = question_counts[i];
                        let estimated_correct = (score / 100.0 * total_questions as f64).round() as i32;

                        categories.push(MMLUCategoryScore {
                            category: category_name.to_string(),
                            score,
                            total_questions,
                            correct_answers: estimated_correct,
                        });
                    }
                }
                break;
            }
        }
    }

    if categories.is_empty() {
        return Err(anyhow!("Failed to parse MMLU-Pro report - no scores found"));
    }

    Ok(MMLUScore {
        categories,
        timestamp: Utc::now(),
        context: Some(serde_json::json!({
            "source": "mmlu-pro",
            "format": "report.txt"
        })),
    })
}

async fn upload_from_systemslab(
    experiment_id: String,
    systemslab_url: String,
    server: String,
    artifact_file: Option<PathBuf>,
) -> Result<()> {
    println!("Fetching experiment from SystemsLab: {}", experiment_id);

    let client = reqwest::Client::new();

    // Query GraphQL API for experiment metadata
    let graphql_query = serde_json::json!({
        "query": format!(
            r#"query {{ experimentById(id: "{}") {{ id name state artifact {{ id name experimentId jobId runId }} }} }}"#,
            experiment_id
        )
    });

    let graphql_url = format!("{}/api/graphql", systemslab_url);
    let response = client
        .post(&graphql_url)
        .json(&graphql_query)
        .send()
        .await?;

    let graphql_result: SystemslabGraphQLResponse<SystemslabExperimentData> = response.json().await?;

    if let Some(errors) = graphql_result.errors {
        return Err(anyhow!("GraphQL errors: {:?}", errors));
    }

    let experiment_data = graphql_result.data
        .ok_or_else(|| anyhow!("No data returned from GraphQL"))?;

    let experiment = experiment_data.experiment_by_id
        .ok_or_else(|| anyhow!("Experiment not found"))?;

    println!("Found experiment: {}", experiment.name);
    println!("State: {}", experiment.state);

    // Check for llm.json (preferred, contains all config)
    let benchmark_artifact = experiment.artifact.iter()
        .find(|a| a.name == "llm.json");

    if let Some(_artifact) = benchmark_artifact {
        // This path is no longer used - import_from_systemslab handles this now
        return Err(anyhow!("This import path is deprecated - use the unified import command"));
    }

    println!("No llm.json found - using legacy detection");

    // Get results.json artifact
    let results_artifact = experiment.artifact.iter()
        .find(|a| a.name == "results.json")
        .ok_or_else(|| anyhow!("results.json artifact not found"))?;

    println!("Found results.json artifact (ID: {})", results_artifact.id);

    // Get systeminfo.json artifact
    let systeminfo_artifact = experiment.artifact.iter()
        .find(|a| a.name == "systeminfo.json")
        .ok_or_else(|| anyhow!("systeminfo.json artifact not found"))?;

    println!("Found systeminfo.json artifact (ID: {})", systeminfo_artifact.id);

    // Get experiment.json artifact
    let experiment_artifact = experiment.artifact.iter()
        .find(|a| a.name == "experiment.json")
        .ok_or_else(|| anyhow!("experiment.json artifact not found"))?;

    println!("Found experiment.json artifact (ID: {})", experiment_artifact.id);

    // Download results.json
    let artifact_content = if let Some(file_path) = artifact_file {
        println!("Reading results from file: {}", file_path.display());
        std::fs::read_to_string(file_path)?
    } else {
        println!("Downloading results.json...");
        let artifact_url = format!("{}/api/v1/artifact/{}", systemslab_url, results_artifact.id);

        let response = client.get(&artifact_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download results.json: HTTP {}", response.status()));
        }

        let content = response.text().await?;
        println!("Downloaded {} bytes", content.len());
        content
    };

    // Download systeminfo.json
    println!("Downloading systeminfo.json...");
    let systeminfo_url = format!("{}/api/v1/artifact/{}", systemslab_url, systeminfo_artifact.id);

    let systeminfo_response = client.get(&systeminfo_url).send().await?;
    if !systeminfo_response.status().is_success() {
        return Err(anyhow!("Failed to download systeminfo.json: HTTP {}", systeminfo_response.status()));
    }

    let systeminfo_content = systeminfo_response.text().await?;
    let systeminfo: SystemInfo = serde_json::from_str(&systeminfo_content)?;

    println!("Parsed system info:");
    println!("  CPU: {}", systeminfo.hwinfo.cpus[0].model_name);
    println!("  RAM: {} GB", systeminfo.hwinfo.memory.total_bytes / 1024 / 1024 / 1024);

    // Download experiment.json
    println!("Downloading experiment.json...");
    let experiment_json_url = format!("{}/api/v1/artifact/{}", systemslab_url, experiment_artifact.id);

    let experiment_json_response = client.get(&experiment_json_url).send().await?;
    if !experiment_json_response.status().is_success() {
        return Err(anyhow!("Failed to download experiment.json: HTTP {}", experiment_json_response.status()));
    }

    let experiment_json_content = experiment_json_response.text().await?;
    let experiment_json: ExperimentJson = serde_json::from_str(&experiment_json_content)?;

    println!("Parsed experiment.json");

    // Extract vllm serve command and power limit from experiment.json
    let (model_path, gpu_power_limit) = extract_experiment_info(&experiment_json)?;

    println!("Extracted from experiment.json:");
    println!("  Model path: {}", model_path);
    if let Some(power) = gpu_power_limit {
        println!("  GPU power limit: {}W", power);
    }

    // Use the existing model detection logic
    let (model_name, quantization) = extract_model_info_from_path(&model_path)?;

    println!("Detected from path:");
    println!("  Model: {}", model_name);
    println!("  Quantization: {}", quantization);

    // Parse as inference server result
    let result: InferenceServerResult = serde_json::from_str(&artifact_content)?;

    println!("Parsed results successfully");
    println!("Throughput: {} tok/s", result.throughput.output_tokens_per_second);

    // Backend is vLLM (from the vllm serve command)
    let backend = "vLLM".to_string();
    let backend_version = result.version.clone();

    // Build hardware config from systeminfo.json
    let cpu_model = systeminfo.hwinfo.cpus[0].model_name.clone();
    let cpu_arch = detect_cpu_arch(&cpu_model);
    let ram_gb = Some((systeminfo.hwinfo.memory.total_bytes / 1024 / 1024 / 1024) as i32);

    // Try to detect GPU from local system as fallback
    // (SystemsLab doesn't store GPU info in systeminfo.json)
    let (gpu_memory_gb, gpu_model) = detect_gpu_info().unwrap_or((0, "Unknown GPU".to_string()));

    let hardware_config = HardwareConfig {
        gpu_model,
        gpu_memory_gb,
        cpu_model,
        cpu_arch: cpu_arch.to_string(),
        ram_gb,
        ram_type: None,
        virtualization_type: None,
        optimizations: Vec::new(),
    };

    // Parse timestamp
    let timestamp = parse_timestamp(&result.timestamp)?;

    // Create performance metrics (reusing existing logic)
    let performance_metrics = build_performance_metrics(&result, timestamp);

    // Build notes
    let notes_str = format!(
        "Imported from SystemsLab | Experiment: {} | Benchmark tool: {} | Duration: {}s",
        experiment_id,
        result.version,
        result.configuration.duration_seconds
    );

    // Extract dataset name
    let dataset_name = result.configuration.prompt_file.as_ref()
        .and_then(|p| std::path::Path::new(p).file_stem())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Parse experiment ID as UUID
    let exp_uuid = Uuid::parse_str(&experiment_id)
        .map_err(|e| anyhow!("Failed to parse experiment ID as UUID: {}", e))?;

    // Create experiment run
    let experiment_run = ExperimentRun {
        id: exp_uuid,
        model_name,
        quantization,
        backend,
        backend_version,
        hardware_config,
        performance_metrics,
        benchmark_scores: Vec::new(),
        timestamp,
        status: ExperimentStatus::Completed,
        notes: Some(notes_str),
        concurrent_requests: Some(result.configuration.concurrent_requests),
        max_context_length: None,
        load_pattern: Some(result.configuration.load_pattern.clone()),
        dataset_name,
        gpu_power_limit_watts: gpu_power_limit,
    };

    // Upload to server
    upload_experiment(experiment_run, &server).await?;

    Ok(())
}

async fn upload_from_systemslab_context(
    context_id: String,
    systemslab_url: String,
    server: String,
    skip_failures: bool,
) -> Result<()> {
    println!("Fetching context from SystemsLab: {}", context_id);

    let client = reqwest::Client::new();

    // Query GraphQL API for context metadata
    let graphql_query = serde_json::json!({
        "query": format!(
            r#"query {{ contextById(id: "{}") {{ id name state experiments {{ experiment {{ id name state }} }} }} }}"#,
            context_id
        )
    });

    let graphql_url = format!("{}/api/graphql", systemslab_url);
    let response = client
        .post(&graphql_url)
        .json(&graphql_query)
        .send()
        .await?;

    let graphql_result: SystemslabGraphQLResponse<SystemslabContextData> = response.json().await?;

    if let Some(errors) = graphql_result.errors {
        return Err(anyhow!("GraphQL errors: {:?}", errors));
    }

    let context_data = graphql_result.data
        .ok_or_else(|| anyhow!("No data returned from GraphQL"))?;

    let context = context_data.context_by_id;

    println!("Found context: {}", context.name);
    println!("State: {}", context.state);
    println!("Experiments: {}", context.experiments.len());
    println!();

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut skipped_count = 0;

    for (idx, context_exp) in context.experiments.iter().enumerate() {
        // Skip experiments that are still running (experiment is None)
        let Some(experiment) = &context_exp.experiment else {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("[{}/{}] Skipping (experiment still running or null)", idx + 1, context.experiments.len());
            skipped_count += 1;
            println!();
            continue;
        };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("[{}/{}] Importing: {}", idx + 1, context.experiments.len(), experiment.name);
        println!("Experiment ID: {}", experiment.id);
        println!("State: {}", experiment.state);

        // Skip experiments that aren't successful
        if experiment.state != "success" {
            println!("⊗ Skipping (state: {})", experiment.state);
            skipped_count += 1;
            println!();
            continue;
        }

        // Import the experiment
        match upload_from_systemslab(
            experiment.id.clone(),
            systemslab_url.clone(),
            server.clone(),
            None, // No artifact file for context imports
        ).await {
            Ok(_) => {
                success_count += 1;
                println!("✅ Success");
            }
            Err(e) => {
                failure_count += 1;
                println!("❌ Failed: {}", e);

                println!("⚠️  Continuing to next experiment...");
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Import Summary:");
    println!("  ✅ Successful: {}", success_count);
    println!("  ❌ Failed: {}", failure_count);
    println!("  ⊗ Skipped: {}", skipped_count);
    println!("  📊 Total: {}", context.experiments.len());

    Ok(())
}

fn extract_experiment_info(experiment_json: &ExperimentJson) -> Result<(String, Option<i32>)> {
    // Look through all job steps to find:
    // 1. vllm serve command with model path
    // 2. nvidia-smi -pl command with power limit

    let mut model_path: Option<String> = None;
    let mut power_limit: Option<i32> = None;

    for (_job_name, job) in &experiment_json.jobs {
        for step in &job.steps {
            if step.uses == "systemslab/shell" {
                if let Some(run_cmd) = step.with.get("run").and_then(|v| v.as_str()) {
                    // Look for vllm serve command
                    if run_cmd.contains("vllm serve") {
                        // Extract model path from: vllm serve /path/to/model [other args]
                        if let Some(serve_idx) = run_cmd.find("vllm serve") {
                            let after_serve = &run_cmd[serve_idx + "vllm serve".len()..];
                            let path = after_serve
                                .trim()
                                .split_whitespace()
                                .next()
                                .ok_or_else(|| anyhow!("Could not extract model path from vllm command"))?;
                            model_path = Some(path.to_string());
                        }
                    }

                    // Look for nvidia-smi -pl command
                    if run_cmd.contains("nvidia-smi") && run_cmd.contains("-pl") {
                        // Extract power limit from: nvidia-smi -pl 450
                        if let Some(pl_idx) = run_cmd.find("-pl") {
                            let after_pl = &run_cmd[pl_idx + 3..];
                            if let Some(power_str) = after_pl.trim().split_whitespace().next() {
                                if let Ok(power) = power_str.parse::<i32>() {
                                    power_limit = Some(power);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let model_path = model_path.ok_or_else(|| anyhow!("Could not find vllm serve command in experiment.json"))?;

    Ok((model_path, power_limit))
}

/// Extract GPU power from gpu_power_usage column (already in milliwatts) using Arrow reader
fn extract_gpu_power_usage_arrow(file_path: &std::path::Path) -> Result<Option<f64>> {
    let file = File::open(file_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

    // Check if gpu_power_usage/0 column exists
    let schema = builder.schema();
    let column_name = "gpu_power_usage/0";

    if schema.column_with_name(column_name).is_none() {
        return Ok(None);
    }

    // Build reader and collect all values
    let mut reader = builder.build()?;
    let mut all_values: Vec<u64> = Vec::new();

    while let Some(batch) = reader.next() {
        let batch = batch?;

        if let Some(column) = batch.column_by_name(column_name) {
            // Try as Int64 first (metriken uses signed integers)
            if let Some(int_array) = column.as_primitive_opt::<Int64Type>() {
                for i in 0..int_array.len() {
                    if !int_array.is_null(i) {
                        let val = int_array.value(i);
                        if val >= 0 {
                            all_values.push(val as u64);
                        }
                    }
                }
            }
            // Fallback to UInt64
            else if let Some(uint_array) = column.as_primitive_opt::<UInt64Type>() {
                for i in 0..uint_array.len() {
                    if !uint_array.is_null(i) {
                        all_values.push(uint_array.value(i));
                    }
                }
            }
        }
    }

    if all_values.is_empty() {
        return Ok(None);
    }

    // Calculate P95 (95th percentile)
    all_values.sort();
    let p95_index = ((all_values.len() as f64 * 0.95) as usize).min(all_values.len() - 1);
    let p95_milliwatts = all_values[p95_index] as f64;

    // Convert from milliwatts to watts
    Ok(Some(p95_milliwatts / 1000.0))
}

/// Calculate GPU power consumption from Rezolus metrics.parquet file
/// Returns average power in watts - tries gpu_power_usage first, then falls back to calculating from gpu_energy_consumption
async fn calculate_gpu_power_from_parquet(
    client: &reqwest::Client,
    systemslab_url: &str,
    metrics_artifact_id: &str,
) -> Result<Option<f64>> {
    println!("Downloading metrics.parquet...");
    let metrics_url = format!("{}/api/v1/artifact/{}", systemslab_url, metrics_artifact_id);

    let response = client.get(&metrics_url).send().await?;
    if !response.status().is_success() {
        println!("Warning: Failed to download metrics.parquet: HTTP {}", response.status());
        return Ok(None);
    }

    let bytes = response.bytes().await?;

    // Write to temporary file for parquet reading
    let temp_path = std::env::temp_dir().join(format!("metrics-{}.parquet", metrics_artifact_id));
    std::fs::write(&temp_path, &bytes)?;

    // Try to use gpu_power_usage column first (already in milliwatts) using Arrow reader
    match extract_gpu_power_usage_arrow(&temp_path) {
        Ok(Some(power)) => {
            let _ = std::fs::remove_file(&temp_path);
            println!("  GPU Power: {:.2}W P95 (from gpu_power_usage)", power);
            return Ok(Some(power));
        }
        Ok(None) | Err(_) => {
            // gpu_power_usage not available, try energy calculation
        }
    }

    // If Arrow approach didn't work, try with SerializedFileReader for energy calculation
    let file = File::open(&temp_path)?;
    let reader = SerializedFileReader::new(file)?;

    // Fallback: calculate from energy deltas
    let mut first_energy: Option<i64> = None;
    let mut last_energy: Option<i64> = None;
    let mut first_timestamp: Option<i64> = None;
    let mut last_timestamp: Option<i64> = None;

    let num_row_groups = reader.metadata().num_row_groups();

    // Get first row from first row group
    if num_row_groups > 0 {
        let first_row_group = reader.get_row_group(0)?;
        let first_metadata = reader.metadata().row_group(0);

        for col_idx in 0..first_metadata.num_columns() {
            let col_metadata = first_metadata.column(col_idx);
            let col_name = col_metadata.column_descr().name();

            // Look for GPU energy (with or without /0 suffix for GPU index)
            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" || col_name == "timestamp" {
                println!("    Found column: {}", col_name);
                let mut column_reader = first_row_group.get_column_reader(col_idx)?;

                // Try Int64 first
                if let parquet::column::reader::ColumnReader::Int64ColumnReader(ref mut reader) = column_reader {
                    println!("      Using Int64ColumnReader");
                    // Try reading without definition levels (for non-nullable columns)
                    let mut values = vec![0i64; 5];

                    if let Ok((values_read, _)) = reader.read_batch(5, None, None, &mut values) {
                        println!("      Read {} values (no def levels): {:?}", values_read, &values[..values_read.min(5)]);
                        if values_read > 0 {
                            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" {
                                first_energy = Some(values[0]);
                            } else {
                                first_timestamp = Some(values[0]);
                            }
                        }
                    } else {
                        println!("      Failed to read without def levels, trying with def levels...");
                        // Read more values to find non-NULL ones
                        let mut values = vec![0i64; 100];
                        let mut def_levels = vec![0i16; 100];

                        if let Ok((values_read, defs_read)) = reader.read_batch(100, Some(&mut def_levels), None, &mut values) {
                            println!("      Read {} values, {} defs", values_read, defs_read);
                            // Find first non-NULL value (def_level > 0)
                            let mut found_value = None;
                            for i in 0..defs_read {
                                if def_levels[i] > 0 {
                                    println!("      First non-NULL at index {}: value={}", i, values[i]);
                                    found_value = Some(values[i]);
                                    break;
                                }
                            }
                            if let Some(val) = found_value {
                                if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" {
                                    first_energy = Some(val);
                                } else {
                                    first_timestamp = Some(val);
                                }
                            } else {
                                println!("      All {} values were NULL (def_level=0)", defs_read);
                            }
                        }
                    }
                }
                // Try DoubleColumnReader for floating point values
                else if let parquet::column::reader::ColumnReader::DoubleColumnReader(ref mut reader) = column_reader {
                    println!("      Using DoubleColumnReader");
                    let mut values = vec![0.0f64; 1];
                    let mut def_levels = vec![0i16; 1];

                    if let Ok((values_read, _)) = reader.read_batch(1, Some(&mut def_levels), None, &mut values) {
                        println!("      Read {} values, first value: {}", values_read, if values_read > 0 { values[0] } else { 0.0 });
                        if values_read > 0 {
                            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" {
                                first_energy = Some(values[0] as i64);
                            } else {
                                first_timestamp = Some(values[0] as i64);
                            }
                        }
                    }
                } else {
                    println!("      Column type not Int64 or Double");
                }
            }
        }
    }

    // Get last row from last row group
    if num_row_groups > 0 {
        let last_row_group_idx = num_row_groups - 1;
        let last_row_group = reader.get_row_group(last_row_group_idx)?;
        let last_metadata = reader.metadata().row_group(last_row_group_idx);
        let num_rows_in_last_group = last_metadata.num_rows() as usize;

        for col_idx in 0..last_metadata.num_columns() {
            let col_metadata = last_metadata.column(col_idx);
            let col_name = col_metadata.column_descr().name();

            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" || col_name == "timestamp" {
                let mut column_reader = last_row_group.get_column_reader(col_idx)?;

                // Try Int64 first
                if let parquet::column::reader::ColumnReader::Int64ColumnReader(ref mut reader) = column_reader {
                    // Skip to the last row by reading all values
                    let mut values = vec![0i64; num_rows_in_last_group];
                    let mut def_levels = vec![0i16; num_rows_in_last_group];

                    if let Ok((values_read, _)) = reader.read_batch(
                        num_rows_in_last_group,
                        Some(&mut def_levels),
                        None,
                        &mut values,
                    ) {
                        if values_read > 0 {
                            let last_value = values[values_read - 1];
                            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" {
                                last_energy = Some(last_value);
                            } else {
                                last_timestamp = Some(last_value);
                            }
                        }
                    }
                }
                // Try DoubleColumnReader for floating point values
                else if let parquet::column::reader::ColumnReader::DoubleColumnReader(ref mut reader) = column_reader {
                    let mut values = vec![0.0f64; num_rows_in_last_group];
                    let mut def_levels = vec![0i16; num_rows_in_last_group];

                    if let Ok((values_read, _)) = reader.read_batch(
                        num_rows_in_last_group,
                        Some(&mut def_levels),
                        None,
                        &mut values,
                    ) {
                        if values_read > 0 {
                            let last_value = values[values_read - 1];
                            if col_name == "gpu_energy_consumption" || col_name == "gpu_energy_consumption/0" {
                                last_energy = Some(last_value as i64);
                            } else {
                                last_timestamp = Some(last_value as i64);
                            }
                        }
                    }
                }
            }
        }
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    println!("  Debug: first_energy={:?}, last_energy={:?}, first_timestamp={:?}, last_timestamp={:?}",
        first_energy, last_energy, first_timestamp, last_timestamp);

    if let (Some(first), Some(last), Some(t_first), Some(t_last)) =
        (first_energy, last_energy, first_timestamp, last_timestamp) {
        // Energy is in milliJoules, convert to Joules
        let energy_joules = (last - first) as f64 / 1000.0;

        // Timestamps are in nanoseconds, convert to seconds
        let duration_seconds = (t_last - t_first) as f64 / 1_000_000_000.0;

        if duration_seconds > 0.0 {
            let average_power_watts = energy_joules / duration_seconds;
            println!("  GPU Power: {:.2}W (from {:.2}J over {:.2}s)",
                average_power_watts, energy_joules, duration_seconds);
            return Ok(Some(average_power_watts));
        }
    }

    println!("Warning: Could not calculate GPU power from metrics.parquet");
    Ok(None)
}

fn build_performance_metrics(result: &InferenceServerResult, timestamp: DateTime<Utc>) -> Vec<PerformanceMetric> {
    let mut performance_metrics = vec![
        // Primary throughput metrics
        PerformanceMetric {
            metric_name: "tokens_per_second".to_string(),
            value: result.throughput.output_tokens_per_second,
            unit: "tok/s".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "total_input_tokens": result.throughput.total_input_tokens,
                "total_output_tokens": result.throughput.total_output_tokens,
            })),
        },
        PerformanceMetric {
            metric_name: "prompt_processing_speed".to_string(),
            value: result.throughput.input_tokens_per_second,
            unit: "tok/s".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "requests_per_second".to_string(),
            value: result.throughput.requests_per_second,
            unit: "req/s".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "total_requests": result.summary.total_requests,
                "successful_requests": result.summary.successful_requests,
                "failed_requests": result.summary.failed_requests,
                "success_rate": result.summary.success_rate,
            })),
        },
    ];

    // Add all TTFT percentiles
    performance_metrics.extend(vec![
        PerformanceMetric {
            metric_name: "ttft_mean_ms".to_string(),
            value: result.latency.ttft_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p50_ms".to_string(),
            value: result.latency.ttft_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p90_ms".to_string(),
            value: result.latency.ttft_p90_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p95_ms".to_string(),
            value: result.latency.ttft_p95_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "ttft_p99_ms".to_string(),
            value: result.latency.ttft_p99_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
    ]);

    // Add all TPOT, ITL, request latency metrics
    performance_metrics.extend(vec![
        PerformanceMetric {
            metric_name: "tpot_mean_ms".to_string(),
            value: result.latency.tpot_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "itl_mean_ms".to_string(),
            value: result.latency.itl_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
        PerformanceMetric {
            metric_name: "request_mean_ms".to_string(),
            value: result.latency.request_mean_ms,
            unit: "ms".to_string(),
            timestamp,
            context: None,
        },
    ]);

    // Add error metrics
    performance_metrics.push(PerformanceMetric {
        metric_name: "error_rate".to_string(),
        value: 1.0 - result.summary.success_rate,
        unit: "ratio".to_string(),
        timestamp,
        context: Some(serde_json::json!({
            "timeout_errors": result.errors.timeout_errors,
            "connection_errors": result.errors.connection_errors,
            "http_4xx_errors": result.errors.http_4xx_errors,
            "http_5xx_errors": result.errors.http_5xx_errors,
            "other_errors": result.errors.other_errors,
            "total_errors": result.summary.failed_requests,
        })),
    });

    // Add GPU power metrics if available
    if let Some(power) = &result.power_stats {
        performance_metrics.push(PerformanceMetric {
            metric_name: "gpu_power_watts".to_string(),
            value: power.p95_watts,
            unit: "W".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "min_watts": power.min_watts,
                "max_watts": power.max_watts,
                "avg_watts": power.avg_watts,
                "p50_watts": power.p50_watts,
                "p95_watts": power.p95_watts,
                "samples": power.samples,
            })),
        });
    }

    // Add context-aware metrics
    for (context_size, metrics) in &result.context_latency {
        performance_metrics.push(PerformanceMetric {
            metric_name: format!("ttft_p50_ms_{}", context_size),
            value: metrics.ttft_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "context_size": context_size,
                "p90": metrics.ttft_p90_ms,
                "p95": metrics.ttft_p95_ms,
                "p99": metrics.ttft_p99_ms,
            })),
        });
    }

    for (context_size, metrics) in &result.context_itl {
        performance_metrics.push(PerformanceMetric {
            metric_name: format!("itl_p50_ms_{}", context_size),
            value: metrics.itl_p50_ms,
            unit: "ms".to_string(),
            timestamp,
            context: Some(serde_json::json!({
                "context_size": context_size,
                "p90": metrics.itl_p90_ms,
                "p95": metrics.itl_p95_ms,
                "p99": metrics.itl_p99_ms,
            })),
        });
    }

    performance_metrics
}

/// Record system configuration and model info
async fn record_config(
    model_path: String,
    backend: Option<String>,
    backend_version: Option<String>,
    power_limit: Option<i32>,
    concurrent_requests: Option<i32>,
    max_context_length: Option<i32>,
    output: PathBuf,
) -> Result<()> {
    println!("🔍 Recording system configuration...");

    // Detect hardware
    println!("  Detecting hardware...");
    let hardware = detect_system_hardware()?;
    let gpu_count = detect_gpu_count()?;

    // Extract model info
    println!("  Analyzing model path: {}", model_path);
    let (model_name, quantization) = extract_model_info_from_path(&model_path)?;
    println!("    Model: {}", model_name);
    println!("    Quantization: {}", quantization);

    // Detect or use provided power limit
    let detected_power_limit = if let Some(pl) = power_limit {
        println!("  Using provided power limit: {}W", pl);
        Some(pl)
    } else {
        match detect_power_limit() {
            Ok(Some(pl)) => {
                println!("  Detected power limit: {}W", pl);
                Some(pl)
            }
            Ok(None) => {
                println!("  No power limit detected");
                None
            }
            Err(e) => {
                println!("  Warning: Failed to detect power limit: {}", e);
                None
            }
        }
    };

    // Detect or use provided backend version
    let detected_backend_version = if let Some(ver) = backend_version {
        println!("  Using provided backend version: {}", ver);
        Some(ver)
    } else {
        match detect_backend_version(backend.as_deref()) {
            Ok(Some(ver)) => {
                println!("  Detected backend version: {}", ver);
                Some(ver)
            }
            Ok(None) => {
                println!("  No backend version detected");
                None
            }
            Err(e) => {
                println!("  Warning: Failed to detect backend version: {}", e);
                None
            }
        }
    };

    // Get hostname
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| {
            std::process::Command::new("hostname")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|_| "unknown".to_string());

    // Create artifact
    let artifact = BenchmarkArtifact {
        // Hardware
        gpu_model: hardware.gpu_model.clone(),
        gpu_count,
        gpu_memory_gb: hardware.gpu_memory_gb,
        cpu_model: hardware.cpu_model.clone(),
        cpu_arch: hardware.cpu_arch.clone(),
        ram_gb: hardware.ram_gb,

        // Model
        model_name: model_name.clone(),
        model_path: model_path.clone(),
        quantization: quantization.clone(),

        // Runtime
        gpu_power_limit_watts: detected_power_limit,
        backend_name: backend.clone(),
        backend_version: detected_backend_version,
        concurrent_requests,
        max_context_length,

        // Metadata
        hostname,
        timestamp: Utc::now(),
        artifact_version: "1.0".to_string(),
    };

    // Write to file
    let json = serde_json::to_string_pretty(&artifact)?;
    std::fs::write(&output, json)?;

    println!("\n✅ Configuration captured successfully!");
    println!("📄 Artifact saved to: {}", output.display());
    println!("\n📊 Summary:");
    println!("  GPU: {} x {}", artifact.gpu_count, artifact.gpu_model);
    if let Some(pl) = artifact.gpu_power_limit_watts {
        println!("  Power Limit: {}W", pl);
    }
    println!("  CPU: {} ({})", artifact.cpu_model, artifact.cpu_arch);
    if let Some(ram) = artifact.ram_gb {
        println!("  RAM: {}GB", ram);
    }
    println!("  Model: {}", artifact.model_name);
    println!("  Quantization: {}", artifact.quantization);
    if let Some(backend_name) = &artifact.backend_name {
        print!("  Backend: {}", backend_name);
        if let Some(version) = &artifact.backend_version {
            println!(" v{}", version);
        } else {
            println!();
        }
    }
    if let Some(concurrency) = artifact.concurrent_requests {
        println!("  Concurrent Requests: {}", concurrency);
    }
    if let Some(ctx_len) = artifact.max_context_length {
        println!("  Max Context Length: {}", ctx_len);
    }

    Ok(())
}

/// Detect number of GPUs
fn detect_gpu_count() -> Result<i32> {
    use std::process::Command;

    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=count")
        .arg("--format=csv,noheader")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            return Ok(lines.len() as i32);
        }
    }

    // Check for Apple Silicon on macOS
    if cfg!(target_os = "macos") {
        let chip_output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output();

        if let Ok(output) = chip_output {
            let chip_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if chip_name.contains("Apple M") {
                return Ok(1);
            }
        }
    }

    // Fallback to 0 for CPU-only systems
    Ok(0)
}

/// Detect current GPU power limit
fn detect_power_limit() -> Result<Option<i32>> {
    use std::process::Command;

    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=power.limit")
        .arg("--format=csv,noheader,nounits")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout);
            let power_str = line.trim().split('\n').next().unwrap_or("").trim();
            if let Ok(power) = power_str.parse::<f64>() {
                return Ok(Some(power.round() as i32));
            }
        }
    }

    Ok(None)
}

/// Detect backend version (vLLM, llama.cpp, etc.)
fn detect_backend_version(backend: Option<&str>) -> Result<Option<String>> {
    use std::process::Command;

    let backend_name = backend.unwrap_or("vllm");

    match backend_name {
        "vllm" => {
            // Try vllm --version
            let output = Command::new("vllm")
                .arg("--version")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    // vllm outputs something like "vllm 0.5.0"
                    if let Some(version) = version_str.split_whitespace().nth(1) {
                        return Ok(Some(version.trim().to_string()));
                    }
                }
            }

            // Try python -c "import vllm; print(vllm.__version__)"
            let output = Command::new("python3")
                .arg("-c")
                .arg("import vllm; print(vllm.__version__)")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !version.is_empty() {
                        return Ok(Some(version));
                    }
                }
            }
        }
        "llama.cpp" => {
            // Try to get version from llama-cli or main
            for binary in &["llama-cli", "main"] {
                let output = Command::new(binary)
                    .arg("--version")
                    .output();

                if let Ok(output) = output {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !version.is_empty() {
                            return Ok(Some(version));
                        }
                    }
                }
            }
        }
        _ => {
            // Generic: try <backend> --version
            let output = Command::new(backend_name)
                .arg("--version")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !version.is_empty() {
                        return Ok(Some(version));
                    }
                }
            }
        }
    }

    Ok(None)
}