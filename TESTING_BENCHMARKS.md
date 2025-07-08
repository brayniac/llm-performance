# Testing Benchmark Upload Functionality

This guide explains how to test the new model variant and benchmark upload system.

## Background

The system now separates:
- **Model-specific benchmarks** (MMLU, GSM8K, etc.) - stored per model variant
- **Hardware-specific metrics** (tokens/s, memory) - stored per test run

This means MMLU-Pro scores uploaded for a model/quantization combo will appear for ALL hardware configurations of that combo.

## Testing Steps

### 1. Run the test script
```bash
./test_benchmark_upload.sh
```

This script will:
1. Upload performance data for a model/quant
2. Upload MMLU-Pro scores for the same model/quant
3. Upload another hardware config
4. Verify MMLU scores appear for both

### 2. Manual verification

Check the database:
```bash
psql $DATABASE_URL < verify_model_variants.sql
```

### 3. Frontend verification

1. Go to http://localhost:3000
2. Look for your test model in the performance grid
3. Change the benchmark filter to "MMLU"
4. Expand the model - all quantizations should show MMLU scores
5. Click on different hardware configs - they should all have the same MMLU score

## Expected Behavior

- ✅ MMLU scores are the same across all hardware for a model/quant
- ✅ Performance metrics (tokens/s, memory) differ by hardware
- ✅ Uploading new MMLU scores updates all hardware configs
- ✅ Uploading new hardware doesn't affect MMLU scores

## Troubleshooting

If MMLU scores don't appear:
1. Check that the migration has been run
2. Verify model_variants table exists
3. Check uploader output for errors
4. Look at backend logs for SQL errors