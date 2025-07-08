-- Check migration status

-- Check if v2 tables exist and have data
SELECT 'model_variants' as table_name, COUNT(*) as row_count FROM model_variants
UNION ALL
SELECT 'mmlu_scores_v2', COUNT(*) FROM mmlu_scores_v2
UNION ALL
SELECT 'gsm8k_scores_v2', COUNT(*) FROM gsm8k_scores_v2
UNION ALL
SELECT 'humaneval_scores_v2', COUNT(*) FROM humaneval_scores_v2
UNION ALL
SELECT 'hellaswag_scores_v2', COUNT(*) FROM hellaswag_scores_v2
UNION ALL
SELECT 'truthfulqa_scores_v2', COUNT(*) FROM truthfulqa_scores_v2
UNION ALL
SELECT 'generic_benchmark_scores_v2', COUNT(*) FROM generic_benchmark_scores_v2;

-- Check if old tables still have data
SELECT '---OLD TABLES---' as info;
SELECT 'mmlu_scores' as table_name, COUNT(*) as row_count FROM mmlu_scores
UNION ALL
SELECT 'gsm8k_scores', COUNT(*) FROM gsm8k_scores
UNION ALL
SELECT 'humaneval_scores', COUNT(*) FROM humaneval_scores
UNION ALL
SELECT 'hellaswag_scores', COUNT(*) FROM hellaswag_scores
UNION ALL
SELECT 'truthfulqa_scores', COUNT(*) FROM truthfulqa_scores
UNION ALL
SELECT 'generic_benchmark_scores', COUNT(*) FROM generic_benchmark_scores;

-- Check for any model variants without scores
SELECT '---MODEL VARIANTS WITHOUT SCORES---' as info;
SELECT mv.model_name, mv.quantization
FROM model_variants mv
WHERE NOT EXISTS (SELECT 1 FROM mmlu_scores_v2 WHERE model_variant_id = mv.id)
  AND NOT EXISTS (SELECT 1 FROM gsm8k_scores_v2 WHERE model_variant_id = mv.id)
  AND NOT EXISTS (SELECT 1 FROM humaneval_scores_v2 WHERE model_variant_id = mv.id)
  AND NOT EXISTS (SELECT 1 FROM hellaswag_scores_v2 WHERE model_variant_id = mv.id)
  AND NOT EXISTS (SELECT 1 FROM truthfulqa_scores_v2 WHERE model_variant_id = mv.id)
  AND NOT EXISTS (SELECT 1 FROM generic_benchmark_scores_v2 WHERE model_variant_id = mv.id);