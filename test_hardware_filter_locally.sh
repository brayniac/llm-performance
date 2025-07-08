#!/bin/bash

echo "Testing hardware category filtering locally..."

# Start backend if not running
echo "Make sure backend is running on localhost:3000"

# Test different filter combinations
echo -e "\n1. No filter (should show all):"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu" | jq '.models | length' || echo "Failed"

echo -e "\n2. Consumer GPU filter:"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu&hardware_categories=consumer_gpu" | jq '.models | length' || echo "Failed"

echo -e "\n3. Multiple categories (consumer_gpu,datacenter_gpu):"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu&hardware_categories=consumer_gpu,datacenter_gpu" | jq '.models | length' || echo "Failed"

echo -e "\n4. Show first result with consumer_gpu filter:"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu&hardware_categories=consumer_gpu" | jq '.models[0] | {model_name, best_quantization: .best_quantization.hardware_category}' || echo "Failed"