-- Create separate tables for different benchmark types
-- This replaces the monolithic quality_scores table with benchmark-specific tables

-- Create MMLU scores table
CREATE TABLE mmlu_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    category VARCHAR NOT NULL,
    score DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    total_questions INTEGER NOT NULL CHECK (total_questions > 0),
    correct_answers INTEGER NOT NULL CHECK (correct_answers >= 0 AND correct_answers <= total_questions),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create GSM8K scores table
CREATE TABLE gsm8k_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    problems_solved INTEGER NOT NULL CHECK (problems_solved >= 0),
    total_problems INTEGER NOT NULL CHECK (total_problems > 0 AND problems_solved <= total_problems),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create HumanEval scores table  
CREATE TABLE humaneval_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    pass_at_1 DOUBLE PRECISION NOT NULL CHECK (pass_at_1 >= 0 AND pass_at_1 <= 100),
    pass_at_10 DOUBLE PRECISION CHECK (pass_at_10 >= 0 AND pass_at_10 <= 100),
    pass_at_100 DOUBLE PRECISION CHECK (pass_at_100 >= 0 AND pass_at_100 <= 100),
    total_problems INTEGER NOT NULL CHECK (total_problems > 0),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create HellaSwag scores table
CREATE TABLE hellaswag_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    accuracy DOUBLE PRECISION NOT NULL CHECK (accuracy >= 0 AND accuracy <= 100),
    total_questions INTEGER NOT NULL CHECK (total_questions > 0),
    correct_answers INTEGER NOT NULL CHECK (correct_answers >= 0 AND correct_answers <= total_questions),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create TruthfulQA scores table
CREATE TABLE truthfulqa_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    truthful_score DOUBLE PRECISION NOT NULL CHECK (truthful_score >= 0 AND truthful_score <= 100),
    helpful_score DOUBLE PRECISION CHECK (helpful_score >= 0 AND helpful_score <= 100),
    total_questions INTEGER NOT NULL CHECK (total_questions > 0),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create generic benchmark scores table for unknown/simple benchmarks
CREATE TABLE generic_benchmark_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    benchmark_name VARCHAR NOT NULL,
    score DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    total_questions INTEGER CHECK (total_questions > 0),
    correct_answers INTEGER CHECK (correct_answers >= 0),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_correct_answers_vs_total CHECK (
        (total_questions IS NULL AND correct_answers IS NULL) OR 
        (total_questions IS NOT NULL AND correct_answers IS NOT NULL AND correct_answers <= total_questions)
    )
);

-- Create indexes for performance
CREATE INDEX idx_mmlu_scores_test_run_id ON mmlu_scores(test_run_id);
CREATE INDEX idx_mmlu_scores_category ON mmlu_scores(category);

CREATE INDEX idx_gsm8k_scores_test_run_id ON gsm8k_scores(test_run_id);

CREATE INDEX idx_humaneval_scores_test_run_id ON humaneval_scores(test_run_id);

CREATE INDEX idx_hellaswag_scores_test_run_id ON hellaswag_scores(test_run_id);

CREATE INDEX idx_truthfulqa_scores_test_run_id ON truthfulqa_scores(test_run_id);

CREATE INDEX idx_generic_benchmark_scores_test_run_id ON generic_benchmark_scores(test_run_id);
CREATE INDEX idx_generic_benchmark_scores_benchmark_name ON generic_benchmark_scores(benchmark_name);

-- Populate with fresh realistic sample data
-- Get existing test run IDs
DO $$
DECLARE
    llama_8b_id UUID;
    mistral_q8_id UUID;
    mistral_q4_id UUID;
BEGIN
    -- Get test run IDs
    SELECT id INTO llama_8b_id FROM test_runs WHERE model_name = 'Llama 3.1 8B' AND quantization = 'Q8_0' LIMIT 1;
    SELECT id INTO mistral_q8_id FROM test_runs WHERE model_name = 'Mistral Small 3.2 24B' AND quantization = 'Q8_0' LIMIT 1;
    SELECT id INTO mistral_q4_id FROM test_runs WHERE model_name = 'Mistral Small 3.2 24B' AND quantization = 'Q4_0' LIMIT 1;

    -- Populate MMLU scores for Llama 3.1 8B
    IF llama_8b_id IS NOT NULL THEN
        INSERT INTO mmlu_scores (test_run_id, category, score, total_questions, correct_answers) VALUES
        (llama_8b_id, 'Biology', 68.4, 71, 49),
        (llama_8b_id, 'Business', 52.6, 78, 41),
        (llama_8b_id, 'Chemistry', 71.8, 113, 81),
        (llama_8b_id, 'Computer Science', 79.6, 41, 33),
        (llama_8b_id, 'Economics', 66.3, 84, 56),
        (llama_8b_id, 'Engineering', 45.2, 96, 43),
        (llama_8b_id, 'Health', 58.7, 81, 48),
        (llama_8b_id, 'History', 69.7, 38, 26),
        (llama_8b_id, 'Law', 61.2, 110, 67),
        (llama_8b_id, 'Math', 74.2, 135, 100),
        (llama_8b_id, 'Philosophy', 63.8, 49, 31),
        (llama_8b_id, 'Physics', 68.2, 129, 88),
        (llama_8b_id, 'Psychology', 70.5, 79, 56),
        (llama_8b_id, 'Other', 42.1, 92, 39);
    END IF;

    -- Populate MMLU scores for Mistral Q8_0
    IF mistral_q8_id IS NOT NULL THEN
        INSERT INTO mmlu_scores (test_run_id, category, score, total_questions, correct_answers) VALUES
        (mistral_q8_id, 'Biology', 72.5, 71, 51),
        (mistral_q8_id, 'Business', 65.4, 78, 51),
        (mistral_q8_id, 'Chemistry', 78.8, 113, 89),
        (mistral_q8_id, 'Computer Science', 85.4, 41, 35),
        (mistral_q8_id, 'Economics', 73.8, 84, 62),
        (mistral_q8_id, 'Engineering', 52.1, 96, 50),
        (mistral_q8_id, 'Health', 66.7, 81, 54),
        (mistral_q8_id, 'History', 76.3, 38, 29),
        (mistral_q8_id, 'Law', 68.2, 110, 75),
        (mistral_q8_id, 'Math', 81.5, 135, 110),
        (mistral_q8_id, 'Philosophy', 71.4, 49, 35),
        (mistral_q8_id, 'Physics', 74.4, 129, 96),
        (mistral_q8_id, 'Psychology', 77.2, 79, 61),
        (mistral_q8_id, 'Other', 49.0, 92, 45);
    END IF;

    -- Populate MMLU scores for Mistral Q4_0 (slightly lower due to quantization)
    IF mistral_q4_id IS NOT NULL THEN
        INSERT INTO mmlu_scores (test_run_id, category, score, total_questions, correct_answers) VALUES
        (mistral_q4_id, 'Biology', 69.0, 71, 49),
        (mistral_q4_id, 'Business', 62.8, 78, 49),
        (mistral_q4_id, 'Chemistry', 75.2, 113, 85),
        (mistral_q4_id, 'Computer Science', 82.9, 41, 34),
        (mistral_q4_id, 'Economics', 70.2, 84, 59),
        (mistral_q4_id, 'Engineering', 48.9, 96, 47),
        (mistral_q4_id, 'Health', 63.0, 81, 51),
        (mistral_q4_id, 'History', 73.7, 38, 28),
        (mistral_q4_id, 'Law', 64.5, 110, 71),
        (mistral_q4_id, 'Math', 78.5, 135, 106),
        (mistral_q4_id, 'Philosophy', 67.3, 49, 33),
        (mistral_q4_id, 'Physics', 70.5, 129, 91),
        (mistral_q4_id, 'Psychology', 73.4, 79, 58),
        (mistral_q4_id, 'Other', 46.7, 92, 43);
    END IF;

    -- Add GSM8K scores
    IF llama_8b_id IS NOT NULL THEN
        INSERT INTO gsm8k_scores (test_run_id, problems_solved, total_problems) VALUES
        (llama_8b_id, 782, 1319);
    END IF;

    IF mistral_q8_id IS NOT NULL THEN
        INSERT INTO gsm8k_scores (test_run_id, problems_solved, total_problems) VALUES
        (mistral_q8_id, 956, 1319);
    END IF;

    IF mistral_q4_id IS NOT NULL THEN
        INSERT INTO gsm8k_scores (test_run_id, problems_solved, total_problems) VALUES
        (mistral_q4_id, 921, 1319);
    END IF;

    -- Add HumanEval scores
    IF llama_8b_id IS NOT NULL THEN
        INSERT INTO humaneval_scores (test_run_id, pass_at_1, pass_at_10, total_problems) VALUES
        (llama_8b_id, 56.1, 78.0, 164);
    END IF;

    IF mistral_q8_id IS NOT NULL THEN
        INSERT INTO humaneval_scores (test_run_id, pass_at_1, pass_at_10, total_problems) VALUES
        (mistral_q8_id, 73.2, 89.6, 164);
    END IF;

    IF mistral_q4_id IS NOT NULL THEN
        INSERT INTO humaneval_scores (test_run_id, pass_at_1, pass_at_10, total_problems) VALUES
        (mistral_q4_id, 69.5, 86.0, 164);
    END IF;

    -- Add HellaSwag scores
    IF llama_8b_id IS NOT NULL THEN
        INSERT INTO hellaswag_scores (test_run_id, accuracy, total_questions, correct_answers) VALUES
        (llama_8b_id, 82.4, 10042, 8275);
    END IF;

    IF mistral_q8_id IS NOT NULL THEN
        INSERT INTO hellaswag_scores (test_run_id, accuracy, total_questions, correct_answers) VALUES
        (mistral_q8_id, 87.3, 10042, 8767);
    END IF;

    IF mistral_q4_id IS NOT NULL THEN
        INSERT INTO hellaswag_scores (test_run_id, accuracy, total_questions, correct_answers) VALUES
        (mistral_q4_id, 85.8, 10042, 8616);
    END IF;

    -- Add TruthfulQA scores
    IF llama_8b_id IS NOT NULL THEN
        INSERT INTO truthfulqa_scores (test_run_id, truthful_score, helpful_score, total_questions) VALUES
        (llama_8b_id, 51.2, 68.7, 817);
    END IF;

    IF mistral_q8_id IS NOT NULL THEN
        INSERT INTO truthfulqa_scores (test_run_id, truthful_score, helpful_score, total_questions) VALUES
        (mistral_q8_id, 58.9, 74.3, 817);
    END IF;

    IF mistral_q4_id IS NOT NULL THEN
        INSERT INTO truthfulqa_scores (test_run_id, truthful_score, helpful_score, total_questions) VALUES
        (mistral_q4_id, 56.4, 71.8, 817);
    END IF;

END $$;

-- Drop the old quality_scores table (commented out for safety - uncomment when ready)
-- DROP TABLE IF EXISTS quality_scores;