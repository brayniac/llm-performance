-- Check what "Generic (Benchmark Only)" entries look like
SELECT 
    tr.model_name,
    tr.quantization,
    hp.gpu_model,
    hp.cpu_arch,
    pm_speed.value as tokens_per_second,
    pm_memory.value as memory_gb
FROM test_runs tr
JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
LEFT JOIN performance_metrics pm_speed ON pm_speed.test_run_id = tr.id 
    AND pm_speed.metric_name = 'tokens_per_second'
LEFT JOIN performance_metrics pm_memory ON pm_memory.test_run_id = tr.id 
    AND pm_memory.metric_name = 'memory_usage_gb'
WHERE hp.gpu_model LIKE '%Generic%' 
   OR hp.cpu_arch LIKE '%Generic%'
   OR tr.backend LIKE '%Benchmark Only%'
LIMIT 10;