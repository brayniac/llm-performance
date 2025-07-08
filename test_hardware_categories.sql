-- Test query to check hardware categories in database
SELECT DISTINCT 
    hp.gpu_model,
    hp.cpu_arch,
    CASE 
        WHEN hp.gpu_model LIKE '%RTX%' OR hp.gpu_model LIKE '%GTX%' THEN 'consumer_gpu'
        WHEN hp.gpu_model LIKE '%A100%' OR hp.gpu_model LIKE '%H100%' 
            OR hp.gpu_model LIKE '%L4%' OR hp.gpu_model LIKE '%L40%'
            OR hp.gpu_model LIKE '%V100%' OR hp.gpu_model LIKE '%T4%' THEN 'datacenter_gpu'
        WHEN hp.gpu_model = 'CPU Only' OR hp.gpu_model = 'N/A' OR hp.gpu_model LIKE 'CPU%' THEN
            CASE 
                WHEN hp.cpu_arch LIKE '%Xeon%' OR hp.cpu_arch LIKE '%EPYC%' THEN 'datacenter_cpu'
                ELSE 'consumer_cpu'
            END
        ELSE 'consumer_gpu'
    END as hardware_category,
    COUNT(DISTINCT tr.id) as test_runs
FROM hardware_profiles hp
JOIN test_runs tr ON tr.hardware_profile_id = hp.id
WHERE tr.status = 'completed'
GROUP BY hp.gpu_model, hp.cpu_arch
ORDER BY hardware_category, test_runs DESC;