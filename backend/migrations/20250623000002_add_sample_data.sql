-- Add more sample test runs and performance data

-- Add model_loading_time metric for existing test run
INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    'model_loading_time',
    5.2,
    's'
FROM test_runs tr 
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q8_0'
LIMIT 1;

-- Add a second test run with Q4_0 quantization
INSERT INTO test_runs (model_name, quantization, backend, backend_version, hardware_profile_id, status) 
SELECT 
    'Mistral Small 3.2 24B', 
    'Q4_0', 
    'llama_cpp', 
    'b3312', 
    id, 
    'completed'
FROM hardware_profiles 
WHERE cpu_arch = 'Zen2' 
LIMIT 1;

-- Add performance metrics for Q4_0 version
INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    metric_name,
    value,
    unit
FROM test_runs tr,
(VALUES 
    ('tokens_per_second', 78.1, 'tok/s'),
    ('memory_usage_gb', 12.3, 'GB'),
    ('model_loading_time', 3.8, 's')
) AS metrics(metric_name, value, unit)
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q4_0';

-- Add MMLU-Pro scores for Q4_0 version
INSERT INTO quality_scores (test_run_id, benchmark_name, category, score, total_questions, correct_answers)
SELECT 
    tr.id,
    'mmlu_pro',
    category,
    score,
    100,
    (score::integer)
FROM test_runs tr,
(VALUES 
    ('Math', 71.8),
    ('Physics', 79.3),
    ('Chemistry', 67.1),
    ('Biology', 65.2),
    ('Computer Science', 82.1),
    ('Economics', 69.5),
    ('History', 62.8),
    ('Law', 57.3),
    ('Philosophy', 55.6),
    ('Psychology', 68.9)
) AS categories(category, score)
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q4_0';

-- Add a third test run with different model
INSERT INTO test_runs (model_name, quantization, backend, backend_version, hardware_profile_id, status) 
SELECT 
    'Llama 3.1 8B', 
    'Q8_0', 
    'llama_cpp', 
    'b3312', 
    id, 
    'completed'
FROM hardware_profiles 
WHERE cpu_arch = 'Zen1' 
LIMIT 1;

-- Add performance metrics for Llama 3.1 8B
INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    metric_name,
    value,
    unit
FROM test_runs tr,
(VALUES 
    ('tokens_per_second', 92.5, 'tok/s'),
    ('memory_usage_gb', 8.7, 'GB'),
    ('model_loading_time', 2.1, 's')
) AS metrics(metric_name, value, unit)
WHERE tr.model_name = 'Llama 3.1 8B' AND tr.quantization = 'Q8_0';

-- Add MMLU-Pro scores for Llama 3.1 8B
INSERT INTO quality_scores (test_run_id, benchmark_name, category, score, total_questions, correct_answers)
SELECT 
    tr.id,
    'mmlu_pro',
    category,
    score,
    100,
    (score::integer)
FROM test_runs tr,
(VALUES 
    ('Math', 68.4),
    ('Physics', 74.2),
    ('Chemistry', 71.8),
    ('Biology', 73.1),
    ('Computer Science', 79.6),
    ('Economics', 66.3),
    ('History', 69.7),
    ('Law', 61.2),
    ('Philosophy', 63.8),
    ('Psychology', 70.5)
) AS categories(category, score)
WHERE tr.model_name = 'Llama 3.1 8B' AND tr.quantization = 'Q8_0';