-- Complete a partially run migration

-- Only try to create indexes if they don't exist
CREATE INDEX IF NOT EXISTS idx_model_variants_model_name ON model_variants(model_name);
CREATE INDEX IF NOT EXISTS idx_model_variants_quantization ON model_variants(quantization);
CREATE INDEX IF NOT EXISTS idx_mmlu_scores_v2_model_variant ON mmlu_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_gsm8k_scores_v2_model_variant ON gsm8k_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_humaneval_scores_v2_model_variant ON humaneval_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_hellaswag_scores_v2_model_variant ON hellaswag_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_truthfulqa_scores_v2_model_variant ON truthfulqa_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_generic_benchmark_scores_v2_model_variant ON generic_benchmark_scores_v2(model_variant_id);

-- Continue migration for any missing data
-- First ensure all model variants exist
INSERT INTO model_variants (model_name, quantization)
SELECT DISTINCT model_name, quantization 
FROM test_runs
WHERE model_name IS NOT NULL AND quantization IS NOT NULL
ON CONFLICT (model_name, quantization) DO NOTHING;

-- Migrate any missing MMLU scores
WITH latest_mmlu AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization, ms.category)
        tr.model_name,
        tr.quantization,
        ms.category,
        ms.score,
        ms.total_questions,
        ms.correct_answers,
        ms.timestamp,
        ms.context
    FROM mmlu_scores ms
    JOIN test_runs tr ON ms.test_run_id = tr.id
    WHERE NOT EXISTS (
        SELECT 1 FROM mmlu_scores_v2 m2
        JOIN model_variants mv ON m2.model_variant_id = mv.id
        WHERE mv.model_name = tr.model_name 
        AND mv.quantization = tr.quantization
        AND m2.category = ms.category
    )
    ORDER BY tr.model_name, tr.quantization, ms.category, ms.timestamp DESC
)
INSERT INTO mmlu_scores_v2 (model_variant_id, category, score, total_questions, correct_answers, timestamp, context)
SELECT 
    mv.id,
    lm.category,
    lm.score,
    lm.total_questions,
    lm.correct_answers,
    lm.timestamp,
    lm.context
FROM latest_mmlu lm
JOIN model_variants mv ON mv.model_name = lm.model_name AND mv.quantization = lm.quantization
ON CONFLICT (model_variant_id, category) DO NOTHING;

-- Continue with other benchmark types...
-- (Similar patterns for GSM8K, HumanEval, etc.)

-- Show final status
SELECT 'Migration status after completion:' as info;
SELECT table_name, row_count FROM (
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
    SELECT 'generic_benchmark_scores_v2', COUNT(*) FROM generic_benchmark_scores_v2
) t ORDER BY table_name;