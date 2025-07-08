-- Verification queries to check model variants and benchmark scores

-- 1. Check model variants
SELECT 
    mv.id,
    mv.model_name,
    mv.quantization,
    mv.created_at,
    mv.updated_at
FROM model_variants mv
ORDER BY mv.updated_at DESC
LIMIT 10;

-- 2. Check MMLU scores for model variants
SELECT 
    mv.model_name,
    mv.quantization,
    COUNT(DISTINCT ms.category) as category_count,
    AVG(ms.score) as avg_score,
    MIN(ms.timestamp) as first_upload,
    MAX(ms.timestamp) as last_upload
FROM model_variants mv
JOIN mmlu_scores_v2 ms ON ms.model_variant_id = mv.id
GROUP BY mv.id, mv.model_name, mv.quantization
ORDER BY mv.model_name, mv.quantization;

-- 3. Check test runs for hardware-specific data
SELECT 
    tr.model_name,
    tr.quantization,
    tr.backend,
    hp.gpu_model,
    hp.cpu_arch,
    pm_speed.value as tokens_per_second,
    pm_memory.value as memory_gb,
    tr.created_at
FROM test_runs tr
JOIN hardware_profiles hp ON tr.hardware_profile_id = hp.id
LEFT JOIN performance_metrics pm_speed ON pm_speed.test_run_id = tr.id 
    AND pm_speed.metric_name = 'tokens_per_second'
LEFT JOIN performance_metrics pm_memory ON pm_memory.test_run_id = tr.id 
    AND pm_memory.metric_name = 'memory_usage_gb'
ORDER BY tr.created_at DESC
LIMIT 10;

-- 4. Verify that MMLU scores are shared across hardware configs
WITH model_variant_scores AS (
    SELECT 
        mv.model_name,
        mv.quantization,
        AVG(ms.score) as mmlu_avg
    FROM model_variants mv
    JOIN mmlu_scores_v2 ms ON ms.model_variant_id = mv.id
    GROUP BY mv.id, mv.model_name, mv.quantization
)
SELECT 
    tr.model_name,
    tr.quantization,
    mvs.mmlu_avg,
    COUNT(DISTINCT tr.hardware_profile_id) as hardware_configs,
    COUNT(DISTINCT tr.id) as test_runs
FROM test_runs tr
LEFT JOIN model_variant_scores mvs ON mvs.model_name = tr.model_name 
    AND mvs.quantization = tr.quantization
GROUP BY tr.model_name, tr.quantization, mvs.mmlu_avg
HAVING COUNT(DISTINCT tr.hardware_profile_id) > 1
ORDER BY tr.model_name, tr.quantization;