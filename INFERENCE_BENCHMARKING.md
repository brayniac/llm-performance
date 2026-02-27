# Inference Server Benchmarking Guide

## Simple Upload Command

After running your inference benchmarks, upload results with:

```bash
benchmark-uploader inference-server \
  --file benchmark_results.json \
  --backend vllm \
  --backend-version v0.6.3 \
  --server http://10.3.0.50:3000
```

That's it! The uploader automatically:
- ✅ Detects your hardware (CPU, GPU, RAM)
- ✅ Extracts model name from path (`Qwen/Qwen3-30B-A3B-Instruct-2507`)
- ✅ Extracts quantization from path (`W4A16`)
- ✅ Captures all 18+ performance metrics with percentiles

## What Gets Uploaded

### Primary Metrics (shown in dashboard)
- `tokens_per_second`: 577.2 tok/s
- `prompt_processing_speed`: 1129.9 tok/s
- `memory_usage_gb`: (if provided)

### Latency Metrics (with p50/p90/p95/p99)
- Time to First Token (TTFT)
- Time Per Output Token (TPOT)
- Inter-Token Latency (ITL)
- Request latency

### Context-Aware Metrics
- TTFT by prompt size (small/medium/large/xlarge)
- ITL by prompt size (small/medium/large/xlarge/xxlarge)

### Error Tracking
- Success rate, error breakdown by type

## Optional Arguments

```bash
--model "Qwen/Qwen2.5-72B-Instruct"    # Override auto-detection
--quantization "W4A16"                  # Override auto-detection
--memory-gb 45.2                        # Actual memory used
--notes "Testing with flash attention" # Custom notes
```

## Example Workflow

```bash
# 1. Run your inference benchmark
./my_benchmark_tool --model /mnt/llm-models/W4A16/Qwen/Qwen3-30B > results.json

# 2. Upload (one command!)
benchmark-uploader inference-server \
  -f results.json \
  -b vllm \
  -v v0.6.3 \
  -s http://10.3.0.50:3000
```

## Integrate with Test Runners

```bash
# In your test script:
benchmark-uploader inference-server \
  --file "output_$(date +%Y%m%d_%H%M%S).json" \
  --backend vllm \
  --backend-version v0.6.3 \
  --server http://10.3.0.50:3000 \
  --notes "Nightly run - $(hostname)"
```

## Supported Quantization Formats

The system now recognizes:
- **Weight-Activation**: W4A16, W8A8, W4A8, W8A16
- **GGUF**: Q4_K_M, Q8_0, Q5_K_S, etc.
- **Float**: FP16, BF16, FP32
- **Other**: AWQ, GPTQ, INT8, INT4

## Troubleshooting

### "Unknown quantization scheme" error
The backend server needs to be restarted to pick up new quantization formats. Run:
```bash
cd backend && cargo build --release && ./target/release/llm-benchmark-api
```

### Model name not detected
Override with `--model "owner/model-name"` and `--quantization "W4A16"`

### GPU not detected
The uploader uses `nvidia-smi`. For AMD GPUs, you'll need to manually specify hardware (not yet supported).
