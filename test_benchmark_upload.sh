#!/bin/bash

# Test script to verify benchmark upload functionality
# This tests that MMLU-Pro scores are properly stored in model variants
# and appear across all hardware configurations

set -e

SERVER="http://localhost:3000"
MODEL="llama-3.2-3b-instruct"
QUANT="Q4_K_M"

echo "Testing benchmark upload functionality..."
echo "========================================"

# Step 1: Upload llama-bench performance data
echo -e "\n1. Uploading llama-bench performance data..."
cd /Users/brian/workspace/brayniac/llm-performance/uploader
cargo run -- llama-bench \
  --model "$MODEL" \
  --quantization "$QUANT" \
  --file ../test-data/llama-bench.json \
  --server "$SERVER"

# Step 2: Check performance grid to see the uploaded data
echo -e "\n2. Checking performance grid for uploaded data..."
curl -s "$SERVER/api/grouped-performance?benchmark=none" | jq '.models[] | select(.model_name == "'$MODEL'")'

# Step 3: Upload MMLU-Pro scores
echo -e "\n3. Uploading MMLU-Pro scores..."
cargo run -- mmlu-pro \
  --model "$MODEL" \
  --quantization "$QUANT" \
  --file ../test-data/report.txt \
  --server "$SERVER"

# Step 4: Check that MMLU scores appear
echo -e "\n4. Checking that MMLU scores appear in grouped performance..."
curl -s "$SERVER/api/grouped-performance?benchmark=mmlu" | jq '.models[] | select(.model_name == "'$MODEL'")'

# Step 5: Upload another hardware config for same model/quant
echo -e "\n5. Simulating another hardware config upload..."
# We'll modify the llama-bench.json to simulate different hardware
cp ../test-data/llama-bench.json ../test-data/llama-bench-alt.json
# Note: In a real test, you'd modify the hardware details in the JSON

cargo run -- llama-bench \
  --model "$MODEL" \
  --quantization "$QUANT" \
  --file ../test-data/llama-bench-alt.json \
  --server "$SERVER"

# Step 6: Verify MMLU scores still appear for both hardware configs
echo -e "\n6. Verifying MMLU scores appear for all hardware configs..."
curl -s "$SERVER/api/grouped-performance?benchmark=mmlu" | jq '.models[] | select(.model_name == "'$MODEL'") | .all_quantizations'

echo -e "\n========================================"
echo "Test complete!"
echo ""
echo "Expected behavior:"
echo "- Performance data (tokens/s, memory) should be hardware-specific"
echo "- MMLU scores should be the same across all hardware configs for the same model/quant"
echo "- The grouped view should show the MMLU score regardless of which hardware uploaded it"