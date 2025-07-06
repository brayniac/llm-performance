#!/bin/bash

# Example of uploading MMLU-Pro results from report.txt

# Make sure the uploader is built
cargo build --release

# Upload MMLU-Pro results
# You need to provide:
# - The report.txt file path
# - The model slug (e.g., "TheDrummer/Snowpiercer-15B-v1")
# - The quantization format (e.g., "Q3_K_L")

./target/release/benchmark-uploader mmlu-pro \
  --file ../report.txt \
  --model "TheDrummer/Snowpiercer-15B-v1" \
  --quantization "Q3_K_L" \
  --server "http://localhost:3000" \
  --backend "llama.cpp" \
  --notes "MMLU-Pro benchmark run"

# Alternative with different model
# ./target/release/benchmark-uploader mmlu-pro \
#   --file ../report.txt \
#   --model "meta-llama/Meta-Llama-3-8B-Instruct" \
#   --quantization "Q4_K_M" \
#   --server "http://localhost:3000"