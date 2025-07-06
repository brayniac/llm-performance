use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, NaiveDateTime};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use llm_benchmark_types::{*, benchmarks::{MMLUScore, MMLUCategoryScore}};

/// LLM Benchmark Uploader - Parse and upload benchmark results to the database
#[derive(Parser)]
#[command(name = "benchmark-uploader")]
#[command(about = "Parse and upload LLM benchmark results", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and upload llama-bench results
    LlamaBench {
        /// Path to llama-bench.json output file
        #[arg(short, long)]
        file: PathBuf,
        
        /// API server URL (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
        
        /// Model name (will be extracted from filename if not provided)
        #[arg(short, long)]
        model_name: Option<String>,
        
        /// Quantization format (will be extracted from filename if not provided)
        #[arg(short = 'q', long)]
        quantization: Option<String>,
        
        /// Optional notes about this run
        #[arg(short, long)]
        notes: Option<String>,
        
        /// Benchmark scores JSON file (optional)
        #[arg(short, long)]
        benchmarks: Option<PathBuf>,
    },
    
    /// Upload benchmark scores for an existing test run
    Benchmarks {
        /// Test run ID to attach benchmarks to
        #[arg(short, long)]
        test_run_id: String,
        
        /// Path to benchmark scores JSON file
        #[arg(short, long)]
        file: PathBuf,
        
        /// API server URL (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
    },
    
    /// Parse and upload MMLU-Pro results from report.txt
    MmluPro {
        /// Path to report.txt file
        #[arg(short, long)]
        file: PathBuf,
        
        /// Model slug (e.g., "TheDrummer/Snowpiercer-15B-v1")
        #[arg(short, long)]
        model: String,
        
        /// Quantization format (e.g., "Q3_K_L")
        #[arg(short = 'q', long)]
        quantization: String,
        
        /// API server URL (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
        
        /// Backend name (default: llama.cpp)
        #[arg(short, long, default_value = "llama.cpp")]
        backend: String,
        
        /// Optional notes about this run
        #[arg(short, long)]
        notes: Option<String>,
    },
    
    /// Parse and upload custom experiment results
    Custom {
        /// Path to experiment JSON file
        #[arg(short, long)]
        file: PathBuf,
        
        /// API server URL (default: http://localhost:3000)
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
    },
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
        Commands::LlamaBench { 
            file, 
            server, 
            model_name, 
            quantization, 
            notes,
            benchmarks,
        } => {
            upload_llama_bench(file, server, model_name, quantization, notes, benchmarks).await?;
        }
        Commands::Benchmarks { test_run_id, file, server } => {
            upload_benchmarks_only(test_run_id, file, server).await?;
        }
        Commands::MmluPro { file, model, quantization, server, backend, notes } => {
            upload_mmlu_pro(file, model, quantization, server, backend, notes).await?;
        }
        Commands::Custom { file, server } => {
            upload_custom(file, server).await?;
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
    
    // Create experiment run
    let experiment_run = ExperimentRun {
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
    };
    
    // Upload to server
    upload_experiment(experiment_run, &server).await?;
    
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
    model: String,
    quantization: String,
    server: String,
    backend: String,
    notes: Option<String>,
) -> Result<()> {
    
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
                
                // Parse individual category scores
                for (i, category_name) in category_names.iter().enumerate() {
                    if let Ok(score) = parts[i + 1].parse::<f64>() {
                        // Since we don't have question counts, estimate based on standard MMLU-Pro
                        // Typically MMLU-Pro has ~100 questions per category
                        let estimated_questions = 100;
                        let estimated_correct = (score / 100.0 * estimated_questions as f64).round() as i32;
                        
                        categories.push(MMLUCategoryScore {
                            category: category_name.to_string(),
                            score,
                            total_questions: estimated_questions,
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
    
    // Create generic hardware config for benchmark-only upload
    // MMLU-Pro scores are hardware-independent - they depend only on model+quantization
    let hardware_config = HardwareConfig {
        gpu_model: "Generic (Benchmark Only)".to_string(),
        gpu_memory_gb: 1, // Minimal valid value
        cpu_model: "Generic (Benchmark Only)".to_string(),
        cpu_arch: "generic".to_string(),
        ram_gb: None,
        ram_type: None,
        virtualization_type: None,
        optimizations: vec![],
    };
    
    // Create experiment run with just the MMLU benchmark score
    let experiment_run = ExperimentRun {
        model_name: model,
        quantization,
        backend,
        backend_version: "unknown".to_string(),
        hardware_config,
        performance_metrics: vec![], // No performance metrics from MMLU-Pro
        benchmark_scores: vec![BenchmarkScoreType::MMLU(mmlu_score)],
        timestamp: test_timestamp,
        status: ExperimentStatus::Completed,
        notes,
    };
    
    // Upload to server
    upload_experiment(experiment_run, &server).await?;
    
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
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("✅ Upload successful!");
        if let Some(test_run_id) = result.get("test_run_id").and_then(|v| v.as_str()) {
            println!("Test run ID: {}", test_run_id);
        } else {
            println!("Response: {}", serde_json::to_string_pretty(&result)?);
        }
        if let Some(warnings) = result.get("warnings").and_then(|w| w.as_array()) {
            if !warnings.is_empty() {
                println!("⚠️  Warnings:");
                for warning in warnings {
                    println!("  - {}", warning);
                }
            }
        }
    } else {
        let error_text = response.text().await?;
        return Err(anyhow!("Upload failed: {}", error_text));
    }
    
    Ok(())
}