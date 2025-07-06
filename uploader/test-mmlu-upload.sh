#!/bin/bash

# Test MMLU-Pro upload with debugging

cd /Users/brian/workspace/brayniac/llm-performance/uploader

echo "Testing MMLU-Pro upload..."

cargo run -- mmlu-pro \
  --file ../report.txt \
  --model "test-model/v1" \
  --quantization "Q4_K_M" \
  --server "http://localhost:3000" \
  --backend "llama.cpp" \
  --notes "Test MMLU-Pro upload"