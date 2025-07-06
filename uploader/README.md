# LLM Benchmark Uploader

A CLI tool for parsing and uploading LLM benchmark results to the performance tracking database.

## Features

- Parse llama-bench JSON output files
- Automatically extract model name and quantization from filenames
- Automatically extract hardware information from llama-bench results
- Upload llama-bench results and benchmark scores separately or together
- Support for all benchmark types (MMLU, GSM8K, HumanEval, HellaSwag, TruthfulQA)
- Upload custom experiment JSON files
- Capture detailed performance context (batch size, threads, GPU layers, etc.)

## Installation

```bash
cd uploader
cargo build --release
```

The binary will be available at `target/release/benchmark-uploader`.

## Usage

### Upload llama-bench Results

Basic usage (model name and quantization extracted from filename):
```bash
benchmark-uploader llama-bench \
  --file /path/to/llama-bench.json
```

With explicit model name and quantization:
```bash
benchmark-uploader llama-bench \
  --file /path/to/llama-bench.json \
  --model-name "TheDrummer/Snowpiercer-15B-v1" \
  --quantization Q3_K_L
```

With benchmark scores:
```bash
benchmark-uploader llama-bench \
  --file /path/to/llama-bench.json \
  --benchmarks /path/to/benchmark-scores.json \
  --notes "Testing new quantization method"
```

Full options:
```bash
benchmark-uploader llama-bench \
  --file /path/to/llama-bench.json \
  --server http://localhost:3000 \
  --model-name "meta-llama/Llama-3-70B" \
  --quantization Q4_K_M \
  --benchmarks benchmark-scores.json \
  --notes "Production run with optimized settings"
```

### Upload Benchmark Scores to Existing Test Run

```bash
benchmark-uploader benchmarks \
  --test-run-id "84b61d9c-bdde-411d-b449-f5ab68a1df08" \
  --file /path/to/benchmark-scores.json
```

### Upload Custom Experiment

```bash
benchmark-uploader custom \
  --file /path/to/experiment.json \
  --server http://localhost:3000
```

## File Formats

### llama-bench Output

The tool expects standard llama-bench JSON output and extracts:
- Hardware information (CPU model/arch, GPU model/memory)
- Model details (size, parameters, filename)
- Performance metrics with context:
  - Prompt processing speed (with n_prompt, batch, threads, etc.)
  - Generation speed (with n_gen, batch, threads, etc.)
  - Model size and estimated memory usage
- Build information (commit, build number)
- Optimization flags (CUDA, FlashAttention, etc.)

### Benchmark Scores Format

See `example-benchmarks.json` for the expected format. Each benchmark type has specific fields:

- **MMLU**: Categories with scores, total questions, and correct answers
- **GSM8K**: Problems solved and total problems
- **HumanEval**: Pass@1, Pass@10, Pass@100 rates
- **HellaSwag**: Accuracy, total questions, correct answers
- **TruthfulQA**: Truthful and helpful scores

### Custom Experiment Format

Custom experiments should follow the `ExperimentRun` structure from the types crate, including:
- Model information (name, quantization, backend)
- Hardware configuration
- Performance metrics
- Benchmark scores (optional)
- Status and notes

## Examples

### Example 1: Upload llama-bench results (auto-detect model info)

```bash
# For a file at: /models/GGUF/TheDrummer/Snowpiercer-15B-v1/Snowpiercer-15B-v1.Q3_K_L.gguf
# This will extract:
#   - Model: TheDrummer/Snowpiercer-15B-v1
#   - Quantization: Q3_K_L

benchmark-uploader llama-bench \
  --file results/llama-bench.json
```

### Example 2: Upload llama-bench with benchmark scores

```bash
benchmark-uploader llama-bench \
  --file results/llama-bench.json \
  --benchmarks results/benchmarks.json \
  --notes "Full evaluation run"
```

### Example 3: Upload benchmark scores separately

```bash
# First upload llama-bench results
benchmark-uploader llama-bench \
  --file results/llama-bench.json
# Output: Test run ID: "84b61d9c-bdde-411d-b449-f5ab68a1df08"

# Later, upload benchmark scores
benchmark-uploader benchmarks \
  --test-run-id "84b61d9c-bdde-411d-b449-f5ab68a1df08" \
  --file results/benchmarks.json
```

### Example 4: Override detected values

```bash
benchmark-uploader llama-bench \
  --file results/mistral-bench.json \
  --model-name "mistralai/Mistral-7B-v0.2" \
  --quantization Q8_0 \
  --notes "Testing with flash attention enabled"
```

## Automatic Detection

### Model Information (from filename)
The tool parses model filenames to extract:
- **Model name**: From directory structure (e.g., `TheDrummer/Snowpiercer-15B-v1`)
- **Quantization**: From filename (e.g., `Q3_K_L`, `Q4_0`, `FP16`)

### Hardware Information (from llama-bench)
- **CPU architecture**: zen2, zen3, zen4, alderlake, apple_m1, etc.
- **GPU model**: Full name from gpu_info field
- **GPU memory**: Detected from known GPU models
- **Optimization flags**: CUDA, ROCm, Metal, AVX2, FlashAttention

### Performance Context
Each metric includes relevant context:
- Batch size (`n_batch`)
- Micro-batch size (`n_ubatch`) 
- Thread count (`n_threads`)
- GPU layers (`n_gpu_layers`)
- Split mode and other settings

## Error Handling

The tool will report errors for:
- Missing or invalid JSON files
- Network connection issues
- Server validation errors
- Missing required fields

Check the error messages for specific details about what went wrong.