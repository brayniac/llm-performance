# LLM Benchmark Uploader Examples

## Consistent argument usage
All commands now use `-m/--model` and `-q/--quant` consistently.

## Upload llama-bench results
```bash
# Basic usage (will parse model and quant from filename)
llm-benchmark-uploader llama-bench -f results.json

# Specify model and quantization explicitly
llm-benchmark-uploader llama-bench -f results.json -m "meta-llama/Llama-3.1-70B-Instruct" -q "Q4_K_M"

# With server and notes
llm-benchmark-uploader llama-bench -f results.json -m "meta-llama/Llama-3.1-70B-Instruct" -q "Q4_K_M" -s http://10.3.0.50:3000 -n "Testing new flash attention"

# With benchmark scores file
llm-benchmark-uploader llama-bench -f results.json -m "meta-llama/Llama-3.1-70B-Instruct" -q "Q4_K_M" -b benchmark_scores.json
```

## Upload MMLU-Pro results
```bash
# Basic usage
llm-benchmark-uploader mmlu-pro -f report.txt -m "TheDrummer/Snowpiercer-15B-v1" -q "Q3_K_L"

# With custom server
llm-benchmark-uploader mmlu-pro -f report.txt -m "TheDrummer/Snowpiercer-15B-v1" -q "Q3_K_L" -s http://10.3.0.50:3000

# With backend and notes
llm-benchmark-uploader mmlu-pro -f report.txt -m "TheDrummer/Snowpiercer-15B-v1" -q "Q3_K_L" -b "llama.cpp" -n "5-shot test"
```

## Upload benchmark scores to existing test run
```bash
llm-benchmark-uploader benchmarks -t "YOUR_TEST_RUN_ID" -f benchmark_scores.json
```

## Upload custom experiment JSON
```bash
llm-benchmark-uploader custom -f experiment.json -s http://10.3.0.50:3000
```

## Help
```bash
# General help
llm-benchmark-uploader --help

# Command-specific help
llm-benchmark-uploader llama-bench --help
llm-benchmark-uploader mmlu-pro --help
```