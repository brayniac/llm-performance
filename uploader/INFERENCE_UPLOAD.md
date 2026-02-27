# Inference Server Benchmark Upload Guide

This guide explains how to convert and upload inference server benchmark results to the LLM performance dashboard.

## What Gets Captured

The conversion script captures **18 performance metrics** from your inference benchmarks:

### Primary Metrics (shown in dashboard)
- **tokens_per_second**: Output token throughput (main speed metric)
- **prompt_processing_speed**: Input token throughput
- **memory_usage_gb**: Memory used during inference

### Latency Metrics (with percentiles)
- **first_token_latency_ms**: Time to first token (TTFT)
- **tpot_mean_ms**: Time per output token
- **itl_mean_ms**: Inter-token latency
- **request_mean_ms**: Overall request latency

Each latency metric includes p50, p90, p95, and p99 percentiles stored in the `context` field.

### Context-Aware Metrics
- **ttft_p50_ms_{size}**: TTFT by prompt size (small, medium, large, xlarge)
- **itl_p50_ms_{size}**: ITL by prompt size (small, medium, large, xlarge, xxlarge)

These show how performance scales with different input lengths.

### Error Tracking
- **error_rate**: Overall error ratio with breakdown by type
  - timeout_errors
  - connection_errors
  - http_4xx_errors
  - http_5xx_errors

## Usage

### Basic Usage

```bash
python3 uploader/convert_inference_results.py benchmark.json \
  --backend vllm \
  --backend-version v0.6.3 \
  --gpu-model "RTX 4090" \
  --gpu-memory-gb 24 \
  --cpu-model "AMD EPYC 4564P" \
  --cpu-arch Zen4 \
  --ram-gb 128 \
  --ram-type DDR5 \
  --memory-usage-gb 18.5
```

### Required Arguments

| Argument | Description | Example |
|----------|-------------|---------|
| `input_file` | Path to benchmark JSON file | `benchmark.json` |
| `--backend` | Inference backend name | `vllm`, `llama.cpp`, `tgi` |
| `--backend-version` | Backend version | `v0.6.3`, `b3312` |
| `--gpu-model` | GPU model or "CPU Only" | `RTX 4090`, `A100`, `CPU Only` |
| `--gpu-memory-gb` | GPU VRAM in GB (0 for CPU-only) | `24` |
| `--cpu-model` | CPU model name | `AMD EPYC 4564P` |
| `--cpu-arch` | CPU architecture | `Zen4`, `x86_64` |

### Optional Arguments

| Argument | Description | Example |
|----------|-------------|---------|
| `--ram-gb` | System RAM in GB | `128` |
| `--ram-type` | RAM type | `DDR5`, `DDR4` |
| `--memory-usage-gb` | Actual memory used | `18.5` |
| `--notes` | Additional notes | `"Testing with 8 concurrent requests"` |
| `--api-url` | API endpoint | `http://localhost:3000` |
| `--output` | Save to file instead of uploading | `converted.json` |
| `--dry-run` | Print JSON without uploading | |

## Model Name & Quantization Extraction

The script automatically extracts model name and quantization from the model path:

```
/mnt/llm-models/W4A16/Qwen/Qwen3-30B-A3B-Instruct-2507
                ↓      ↓
         quantization  model_name
```

Supported quantization patterns:
- `W4A16` (weight-4bit, activation-16bit)
- `Q4_K_M`, `Q8_0` (llama.cpp quantization)
- `FP16`, `BF16` (full/brain float)

## Workflow Examples

### 1. Test & Verify Before Upload

```bash
# Convert and review (no upload)
python3 uploader/convert_inference_results.py benchmark.json \
  --backend vllm \
  --backend-version v0.6.3 \
  --gpu-model "RTX 4090" \
  --gpu-memory-gb 24 \
  --cpu-model "AMD EPYC 4564P" \
  --cpu-arch Zen4 \
  --dry-run
```

### 2. Save Converted Format

```bash
# Save to file for manual upload later
python3 uploader/convert_inference_results.py benchmark.json \
  --backend vllm \
  --backend-version v0.6.3 \
  --gpu-model "RTX 4090" \
  --gpu-memory-gb 24 \
  --cpu-model "AMD EPYC 4564P" \
  --cpu-arch Zen4 \
  --output converted.json
```

### 3. Direct Upload

```bash
# Convert and upload in one step
python3 uploader/convert_inference_results.py benchmark.json \
  --backend vllm \
  --backend-version v0.6.3 \
  --gpu-model "RTX 4090" \
  --gpu-memory-gb 24 \
  --cpu-model "AMD EPYC 4564P" \
  --cpu-arch Zen4 \
  --ram-gb 128 \
  --ram-type DDR5 \
  --memory-usage-gb 18.5
```

### 4. Batch Upload Multiple Results

```bash
# Upload results from multiple models
for result in results/*.json; do
  echo "Uploading $result..."
  python3 uploader/convert_inference_results.py "$result" \
    --backend vllm \
    --backend-version v0.6.3 \
    --gpu-model "RTX 4090" \
    --gpu-memory-gb 24 \
    --cpu-model "AMD EPYC 4564P" \
    --cpu-arch Zen4 \
    --ram-gb 128 \
    --ram-type DDR5
done
```

## Getting Memory Usage

If your benchmark tool doesn't report memory usage, you can measure it separately:

```bash
# Using nvidia-smi during benchmark
nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits -l 1 > memory_log.txt

# Get peak usage (convert MB to GB)
awk 'BEGIN{max=0}{if($1>max)max=$1}END{print max/1024}' memory_log.txt
```

Then use `--memory-usage-gb $(awk 'BEGIN{max=0}{if($1>max)max=$1}END{print max/1024}' memory_log.txt)`

## Troubleshooting

### Model name not extracted correctly

The script tries to extract from the path. Override manually:
```bash
# Edit the script and modify model_name before building experiment_run
```

### Quantization not detected

Supported patterns: W4A16, Q4_K_M, FP16, etc. If your format is different, update the regex patterns in `extract_model_info()`.

### Upload fails

Check that the backend API is running:
```bash
curl http://localhost:3000/health
```

## Data Retention

All percentile data and context information is preserved in the database's `context` JSONB column, allowing for future detailed analysis even if not immediately displayed in the dashboard.
