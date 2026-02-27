-- Migration to separate quality benchmarks from hardware-specific test runs
-- This preserves existing data while creating a cleaner architecture

-- Step 1: Create model_variants table
CREATE TABLE IF NOT EXISTS model_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    quantization VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_name, quantization)
);

-- Step 2: Create new benchmark tables that reference model_variants
CREATE TABLE IF NOT EXISTS mmlu_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    category VARCHAR(255) NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    total_questions INT,
    correct_answers INT,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id, category)
);

CREATE TABLE IF NOT EXISTS gsm8k_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    problems_solved INT NOT NULL,
    total_problems INT NOT NULL,
    accuracy DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id)
);

CREATE TABLE IF NOT EXISTS humaneval_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    pass_at_1 DOUBLE PRECISION NOT NULL,
    pass_at_10 DOUBLE PRECISION,
    pass_at_100 DOUBLE PRECISION,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id)
);

CREATE TABLE IF NOT EXISTS hellaswag_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    accuracy DOUBLE PRECISION NOT NULL,
    total_questions INT,
    correct_answers INT,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id)
);

CREATE TABLE IF NOT EXISTS truthfulqa_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    truthful_score DOUBLE PRECISION NOT NULL,
    truthful_and_informative_score DOUBLE PRECISION,
    total_questions INT,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id)
);

CREATE TABLE IF NOT EXISTS generic_benchmark_scores_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_variant_id UUID NOT NULL REFERENCES model_variants(id) ON DELETE CASCADE,
    benchmark_name VARCHAR(255) NOT NULL,
    overall_score DOUBLE PRECISION NOT NULL,
    sub_scores JSONB,
    timestamp TIMESTAMPTZ NOT NULL,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(model_variant_id, benchmark_name)
);

-- Step 3: Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_model_variants_model_name ON model_variants(model_name);
CREATE INDEX IF NOT EXISTS idx_model_variants_quantization ON model_variants(quantization);
CREATE INDEX IF NOT EXISTS idx_mmlu_scores_v2_model_variant ON mmlu_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_gsm8k_scores_v2_model_variant ON gsm8k_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_humaneval_scores_v2_model_variant ON humaneval_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_hellaswag_scores_v2_model_variant ON hellaswag_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_truthfulqa_scores_v2_model_variant ON truthfulqa_scores_v2(model_variant_id);
CREATE INDEX IF NOT EXISTS idx_generic_benchmark_scores_v2_model_variant ON generic_benchmark_scores_v2(model_variant_id);

-- Step 4: Migrate existing data
-- First, populate model_variants from existing test_runs
INSERT INTO model_variants (model_name, quantization)
SELECT DISTINCT model_name, quantization 
FROM test_runs
WHERE model_name IS NOT NULL AND quantization IS NOT NULL
ON CONFLICT (model_name, quantization) DO NOTHING;

-- Step 5: Migrate MMLU scores
-- Group by model/quant and take the latest score for each category
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

-- Step 6: Migrate GSM8K scores (take latest per model/quant)
WITH latest_gsm8k AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization)
        tr.model_name,
        tr.quantization,
        gs.problems_solved,
        gs.total_problems,
        (gs.problems_solved::DOUBLE PRECISION / gs.total_problems::DOUBLE PRECISION) as accuracy,
        gs.timestamp,
        gs.context
    FROM gsm8k_scores gs
    JOIN test_runs tr ON gs.test_run_id = tr.id
    ORDER BY tr.model_name, tr.quantization, gs.timestamp DESC
)
INSERT INTO gsm8k_scores_v2 (model_variant_id, problems_solved, total_problems, accuracy, timestamp, context)
SELECT
    mv.id,
    lg.problems_solved,
    lg.total_problems,
    lg.accuracy,
    lg.timestamp,
    lg.context
FROM latest_gsm8k lg
JOIN model_variants mv ON mv.model_name = lg.model_name AND mv.quantization = lg.quantization
ON CONFLICT (model_variant_id) DO NOTHING;

-- Step 7: Migrate HumanEval scores (take latest per model/quant)
WITH latest_humaneval AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization)
        tr.model_name,
        tr.quantization,
        he.pass_at_1,
        he.pass_at_10,
        he.pass_at_100,
        he.timestamp,
        he.context
    FROM humaneval_scores he
    JOIN test_runs tr ON he.test_run_id = tr.id
    ORDER BY tr.model_name, tr.quantization, he.timestamp DESC
)
INSERT INTO humaneval_scores_v2 (model_variant_id, pass_at_1, pass_at_10, pass_at_100, timestamp, context)
SELECT
    mv.id,
    lh.pass_at_1,
    lh.pass_at_10,
    lh.pass_at_100,
    lh.timestamp,
    lh.context
FROM latest_humaneval lh
JOIN model_variants mv ON mv.model_name = lh.model_name AND mv.quantization = lh.quantization
ON CONFLICT (model_variant_id) DO NOTHING;

-- Step 8: Migrate HellaSwag scores (take latest per model/quant)
WITH latest_hellaswag AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization)
        tr.model_name,
        tr.quantization,
        hs.accuracy,
        hs.total_questions,
        hs.correct_answers,
        hs.timestamp,
        hs.context
    FROM hellaswag_scores hs
    JOIN test_runs tr ON hs.test_run_id = tr.id
    ORDER BY tr.model_name, tr.quantization, hs.timestamp DESC
)
INSERT INTO hellaswag_scores_v2 (model_variant_id, accuracy, total_questions, correct_answers, timestamp, context)
SELECT
    mv.id,
    lh.accuracy,
    lh.total_questions,
    lh.correct_answers,
    lh.timestamp,
    lh.context
FROM latest_hellaswag lh
JOIN model_variants mv ON mv.model_name = lh.model_name AND mv.quantization = lh.quantization
ON CONFLICT (model_variant_id) DO NOTHING;

-- Step 9: Migrate TruthfulQA scores (take latest per model/quant)
WITH latest_truthfulqa AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization)
        tr.model_name,
        tr.quantization,
        tq.truthful_score,
        tq.helpful_score as truthful_and_informative_score,
        tq.total_questions,
        tq.timestamp,
        tq.context
    FROM truthfulqa_scores tq
    JOIN test_runs tr ON tq.test_run_id = tr.id
    ORDER BY tr.model_name, tr.quantization, tq.timestamp DESC
)
INSERT INTO truthfulqa_scores_v2 (model_variant_id, truthful_score, truthful_and_informative_score, total_questions, timestamp, context)
SELECT
    mv.id,
    lt.truthful_score,
    lt.truthful_and_informative_score,
    lt.total_questions,
    lt.timestamp,
    lt.context
FROM latest_truthfulqa lt
JOIN model_variants mv ON mv.model_name = lt.model_name AND mv.quantization = lt.quantization
ON CONFLICT (model_variant_id) DO NOTHING;

-- Step 10: Migrate generic benchmark scores (take latest per model/quant/benchmark)
WITH latest_generic AS (
    SELECT DISTINCT ON (tr.model_name, tr.quantization, gb.benchmark_name)
        tr.model_name,
        tr.quantization,
        gb.benchmark_name,
        gb.score as overall_score,
        jsonb_build_object(
            'score', gb.score,
            'total_questions', gb.total_questions,
            'correct_answers', gb.correct_answers
        ) as sub_scores,
        gb.timestamp,
        gb.context
    FROM generic_benchmark_scores gb
    JOIN test_runs tr ON gb.test_run_id = tr.id
    ORDER BY tr.model_name, tr.quantization, gb.benchmark_name, gb.timestamp DESC
)
INSERT INTO generic_benchmark_scores_v2 (model_variant_id, benchmark_name, overall_score, sub_scores, timestamp, context)
SELECT
    mv.id,
    lg.benchmark_name,
    lg.overall_score,
    lg.sub_scores,
    lg.timestamp,
    lg.context
FROM latest_generic lg
JOIN model_variants mv ON mv.model_name = lg.model_name AND mv.quantization = lg.quantization
ON CONFLICT (model_variant_id, benchmark_name) DO NOTHING;

-- The old tables are kept for rollback purposes
-- Once you verify the migration worked, you can drop them with:
-- DROP TABLE mmlu_scores CASCADE;
-- DROP TABLE gsm8k_scores CASCADE;
-- DROP TABLE humaneval_scores CASCADE;
-- DROP TABLE hellaswag_scores CASCADE;
-- DROP TABLE truthfulqa_scores CASCADE;
-- DROP TABLE generic_benchmark_scores CASCADE;