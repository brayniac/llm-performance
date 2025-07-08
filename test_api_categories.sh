#!/bin/bash

# Test the API with hardware categories

echo "Testing API with hardware categories..."

# Test with no categories (should show all)
echo -e "\n1. No categories (show all):"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu" | jq '.models | length'

# Test with consumer_gpu
echo -e "\n2. Consumer GPU only:"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu&hardware_categories=consumer_gpu" | jq '.models | length'

# Test with multiple categories
echo -e "\n3. Multiple categories:"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu&hardware_categories=consumer_gpu&hardware_categories=datacenter_gpu" | jq '.models | length'

# Show the actual parameter format
echo -e "\n4. Debug - show first model's hardware info:"
curl -s "http://localhost:3000/api/grouped-performance?benchmark=mmlu" | jq '.models[0].best_quantization | {hardware, hardware_category}'