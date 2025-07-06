-- Clear all benchmark data from the database
-- This will delete all test runs and related data

-- First, delete all benchmark scores (they reference test runs)
DELETE FROM mmlu_scores;
DELETE FROM gsm8k_scores;
DELETE FROM humaneval_scores;
DELETE FROM hellaswag_scores;
DELETE FROM truthfulqa_scores;
DELETE FROM generic_benchmark_scores;

-- Delete performance metrics
DELETE FROM performance_metrics;

-- Delete test runs
DELETE FROM test_runs;

-- Optional: Delete hardware profiles that are no longer referenced
-- Note: This will only delete hardware profiles not used by any test runs
DELETE FROM hardware_profiles 
WHERE id NOT IN (SELECT DISTINCT hardware_profile_id FROM test_runs);

-- Reset any sequences if needed (PostgreSQL specific)
-- This is optional but can be useful for testing

-- Verify the cleanup
SELECT 'Test runs remaining: ' || COUNT(*) FROM test_runs
UNION ALL
SELECT 'Performance metrics remaining: ' || COUNT(*) FROM performance_metrics
UNION ALL
SELECT 'MMLU scores remaining: ' || COUNT(*) FROM mmlu_scores
UNION ALL
SELECT 'GSM8K scores remaining: ' || COUNT(*) FROM gsm8k_scores
UNION ALL
SELECT 'HumanEval scores remaining: ' || COUNT(*) FROM humaneval_scores
UNION ALL
SELECT 'HellaSwag scores remaining: ' || COUNT(*) FROM hellaswag_scores
UNION ALL
SELECT 'TruthfulQA scores remaining: ' || COUNT(*) FROM truthfulqa_scores
UNION ALL
SELECT 'Generic benchmark scores remaining: ' || COUNT(*) FROM generic_benchmark_scores
UNION ALL
SELECT 'Hardware profiles remaining: ' || COUNT(*) FROM hardware_profiles;