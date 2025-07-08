# Model Variants Migration Guide

## Overview

This migration separates quality benchmarks (MMLU, GSM8K, etc.) from hardware-specific performance metrics. Quality benchmarks are now stored per model/quantization combination, not per test run.

## Why This Change?

- **Quality benchmarks** (MMLU, GSM8K, etc.) measure the model's knowledge and capabilities - they're the same regardless of hardware
- **Performance metrics** (tokens/sec, memory usage) are hardware-specific
- Previously, uploading MMLU-Pro separately would create a new test run with no performance data
- Now, benchmarks are stored once per model/quantization and automatically shown with all hardware configurations

## Migration Steps

### 1. Backup Your Database
```bash
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d_%H%M%S).sql
```

### 2. Run the Migration
```bash
# Using the provided script
./run_migration.sh

# Or manually
psql $DATABASE_URL -f backend/migrations/20250708000001_separate_benchmarks_from_hardware.sql
```

### 3. Update Your Code
```bash
git pull
cargo build --release -p llm-benchmark-api
cargo build --release -p llm-benchmark-uploader
```

### 4. Deploy the New Backend
Deploy the new backend with the updated endpoints.

## New Workflow

### Uploading Performance Data (llama-bench)
```bash
# Same as before - uploads hardware-specific performance
llm-benchmark-uploader llama-bench -f results.json -m "model/name" -q "Q4_K_M"
```

### Uploading Quality Benchmarks (MMLU-Pro)
```bash
# Now uploads to model variant, not a specific test run
llm-benchmark-uploader mmlu-pro -f report.txt -m "model/name" -q "Q4_K_M"
```

The MMLU scores will automatically appear for ALL hardware configurations of that model/quantization combination.

## API Changes

### New Endpoints
- `POST /api/benchmarks/upload` - Upload benchmark scores for a model/quantization
- `GET /api/model-variants` - List all model/quantization combinations

### Modified Behavior
- Performance grid now JOINs hardware-specific data with model-specific benchmarks
- Benchmark scores are no longer duplicated across test runs

## Database Changes

### New Tables
- `model_variants` - Unique model/quantization combinations
- `mmlu_scores_v2`, `gsm8k_scores_v2`, etc. - Benchmark scores linked to model variants

### Old Tables (kept for rollback)
- `mmlu_scores`, `gsm8k_scores`, etc. - Can be dropped after verification

## Rollback Plan

If needed, the old tables are preserved. To rollback:
1. Update code to previous version
2. Deploy old backend
3. Drop the v2 tables

## Verification

After migration, verify:
1. All benchmark scores migrated correctly
2. Performance grid shows benchmarks for all hardware configs
3. New uploads work correctly

## Cleanup

Once verified, drop old tables:
```sql
DROP TABLE mmlu_scores CASCADE;
DROP TABLE gsm8k_scores CASCADE;
DROP TABLE humaneval_scores CASCADE;
DROP TABLE hellaswag_scores CASCADE;
DROP TABLE truthfulqa_scores CASCADE;
DROP TABLE generic_benchmark_scores CASCADE;
```