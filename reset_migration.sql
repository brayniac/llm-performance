-- Reset migration if needed (BE CAREFUL - this will delete migrated data!)

-- Drop v2 tables and start fresh
DROP TABLE IF EXISTS mmlu_scores_v2 CASCADE;
DROP TABLE IF EXISTS gsm8k_scores_v2 CASCADE;
DROP TABLE IF EXISTS humaneval_scores_v2 CASCADE;
DROP TABLE IF EXISTS hellaswag_scores_v2 CASCADE;
DROP TABLE IF EXISTS truthfulqa_scores_v2 CASCADE;
DROP TABLE IF EXISTS generic_benchmark_scores_v2 CASCADE;
DROP TABLE IF EXISTS model_variants CASCADE;

-- Now you can run the migration again from scratch