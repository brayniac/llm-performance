#!/bin/bash

# Test delete API endpoints

echo "Testing delete API endpoints..."

# First, let's get a test_run_id to delete
echo -e "\n1. Getting list of test runs..."
curl -s http://localhost:3000/api/configurations | jq '.[0] | {id: .id, model: .model_name, quantization: .quantization}'

echo -e "\n2. Enter a test_run_id to delete (or press Enter to skip):"
read TEST_RUN_ID

if [ ! -z "$TEST_RUN_ID" ]; then
    echo -e "\n3. Deleting test run $TEST_RUN_ID..."
    curl -X DELETE http://localhost:3000/api/delete/$TEST_RUN_ID | jq '.'
fi

echo -e "\n4. Testing delete by model/quantization..."
echo "Enter model name (e.g., 'llama3.1:70b'):"
read MODEL_NAME
echo "Enter quantization (e.g., 'Q4_K_M'):"
read QUANTIZATION

if [ ! -z "$MODEL_NAME" ] && [ ! -z "$QUANTIZATION" ]; then
    echo -e "\n5. Deleting all test runs for $MODEL_NAME/$QUANTIZATION..."
    curl -X POST http://localhost:3000/api/delete-by-model \
        -H "Content-Type: application/json" \
        -d "{\"model_name\": \"$MODEL_NAME\", \"quantization\": \"$QUANTIZATION\"}" | jq '.'
fi

echo -e "\nDone!"