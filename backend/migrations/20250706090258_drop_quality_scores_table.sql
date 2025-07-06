-- Drop the old quality_scores table now that all data has been migrated
-- to the new benchmark-specific tables

-- Safety check: This migration should only run after the benchmark tables migration
-- Make sure all data has been migrated before running this!

DROP TABLE IF EXISTS quality_scores;

-- Also drop the related indexes if they exist
DROP INDEX IF EXISTS idx_quality_scores_test_run_id;
DROP INDEX IF EXISTS idx_quality_scores_benchmark_name;
DROP INDEX IF EXISTS idx_quality_scores_category;