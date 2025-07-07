#!/bin/bash

# Script to help find test run IDs

echo "Finding test run IDs..."
echo "====================="

# Method 1: Get all configurations (includes test run IDs)
echo -e "\n1. Recent test runs from configurations endpoint:"
curl -s http://localhost:3000/api/configurations | jq -r '.[] | "\(.id) - \(.model_name) \(.quantization) (\(.timestamp))"' | head -20

# Method 2: Search for specific model/quant
echo -e "\n2. Search for specific model/quantization:"
echo "Enter model name (or press Enter to skip):"
read MODEL_SEARCH
echo "Enter quantization (or press Enter to skip):"
read QUANT_SEARCH

if [ ! -z "$MODEL_SEARCH" ] || [ ! -z "$QUANT_SEARCH" ]; then
    echo -e "\nSearching..."
    curl -s http://localhost:3000/api/configurations | jq -r --arg model "$MODEL_SEARCH" --arg quant "$QUANT_SEARCH" '.[] | select((.model_name | contains($model)) and (.quantization | contains($quant))) | "\(.id) - \(.model_name) \(.quantization) (\(.timestamp))"'
fi

# Method 3: Get from performance grid (if you know which row)
echo -e "\n3. From performance grid (shows test run IDs):"
curl -s http://localhost:3000/api/performance-grid | jq -r '.[] | "\(.id) - \(.model_name) \(.quantization)"' | head -10

echo -e "\n\nTip: You can also find test run IDs by:"
echo "- Looking at the URL when viewing details (e.g., /detail/72a32530-3305-40fe-86d5-7bf0bdb75717)"
echo "- Checking the response when you upload an experiment (returns test_run_id)"
echo "- Using the grouped performance view and clicking 'Details' on any result"