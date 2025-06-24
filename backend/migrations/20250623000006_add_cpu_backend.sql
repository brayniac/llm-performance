-- Add CPU-only hardware profiles
INSERT INTO hardware_profiles (gpu_model, gpu_memory_gb, cpu_model, cpu_arch, ram_gb, ram_type, virtualization_type, optimizations) VALUES
('CPU Only', 0, 'AMD Threadripper 1950X', 'Zen1', 256, 'DDR4', 'KVM', ARRAY['cpu_isolation', 'hugepages_1gb']),
('CPU Only', 0, 'Intel i9-13900K', 'x86_64', 64, 'DDR5', NULL, ARRAY['bare_metal']);

-- Add a CPU-only test run for Llama 3.1 8B (should be much slower but same quality)
INSERT INTO test_runs (model_name, quantization, backend, backend_version, hardware_profile_id, status) 
SELECT 
    'Llama 3.1 8B', 
    'Q4_0',  -- Q4_0 more practical for CPU inference
    'llama_cpp', 
    'b3312', 
    id, 
    'completed'
FROM hardware_profiles 
WHERE gpu_model = 'CPU Only' AND cpu_arch = 'Zen1'
LIMIT 1;

-- Add performance metrics for CPU inference (much slower generation, similar prompt processing per token)
INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    metric_name,
    value,
    unit
FROM test_runs tr,
(VALUES 
    ('tokens_per_second', 3.8, 'tok/s'),           -- Much slower generation
    ('memory_usage_gb', 6.2, 'GB'),                -- Lower memory without GPU overhead
    ('model_loading_time', 8.5, 's'),              -- Longer loading time
    ('prompt_processing_speed', 125.4, 'tok/s')    -- Slower but still reasonable for prompts
) AS metrics(metric_name, value, unit)
WHERE tr.model_name = 'Llama 3.1 8B' AND tr.quantization = 'Q4_0' 
AND tr.id IN (
    SELECT tr2.id FROM test_runs tr2 
    JOIN hardware_profiles hp ON tr2.hardware_profile_id = hp.id 
    WHERE hp.gpu_model = 'CPU Only' 
    AND tr2.model_name = 'Llama 3.1 8B' AND tr2.quantization = 'Q4_0'
);

-- Add IDENTICAL quality scores (since it's the same model)
INSERT INTO quality_scores (test_run_id, benchmark_name, category, score, total_questions, correct_answers)
SELECT 
    tr_cpu.id,
    'mmlu_pro',
    category,
    score,
    total_questions,
    correct_answers
FROM test_runs tr_cpu
JOIN hardware_profiles hp ON tr_cpu.hardware_profile_id = hp.id,
(VALUES 
    ('Biology', 63.38, 71, 45),
    ('Business', 52.56, 78, 41),
    ('Chemistry', 42.48, 113, 48),
    ('Computer Science', 53.66, 41, 22),
    ('Economics', 52.38, 84, 44),
    ('Engineering', 41.67, 96, 40),
    ('Health', 46.91, 81, 38),
    ('History', 26.32, 38, 10),
    ('Law', 33.64, 110, 37),
    ('Math', 54.81, 135, 74),
    ('Philosophy', 46.94, 49, 23),
    ('Physics', 42.64, 129, 55),
    ('Psychology', 58.23, 79, 46),
    ('Other', 35.87, 92, 33)
) AS categories(category, score, total_questions, correct_answers)
WHERE hp.gpu_model = 'CPU Only' 
AND tr_cpu.model_name = 'Llama 3.1 8B' AND tr_cpu.quantization = 'Q4_0';