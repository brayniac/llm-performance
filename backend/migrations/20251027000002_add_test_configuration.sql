-- Add test configuration fields to test_runs table
-- These are critical parameters that affect performance results

ALTER TABLE test_runs
  ADD COLUMN concurrent_requests INT,
  ADD COLUMN max_context_length INT,
  ADD COLUMN load_pattern VARCHAR(50),
  ADD COLUMN dataset_name VARCHAR(255),
  ADD COLUMN gpu_power_limit_watts INT;

-- Add comments explaining these fields
COMMENT ON COLUMN test_runs.concurrent_requests IS 'Number of concurrent requests during the test (affects throughput/latency)';
COMMENT ON COLUMN test_runs.max_context_length IS 'Maximum context length in tokens (affects memory usage and which models can run)';
COMMENT ON COLUMN test_runs.load_pattern IS 'Load pattern used: Concurrent, QPS, Burst, etc.';
COMMENT ON COLUMN test_runs.dataset_name IS 'Dataset used for testing: OpenOrca, ShareGPT, etc.';
COMMENT ON COLUMN test_runs.gpu_power_limit_watts IS 'GPU power limit in watts (e.g., 300W for RTX 4090 with power limit)';

-- Create index for filtering by configuration
CREATE INDEX idx_test_runs_config ON test_runs(concurrent_requests, max_context_length);
CREATE INDEX idx_test_runs_power_limit ON test_runs(gpu_power_limit_watts) WHERE gpu_power_limit_watts IS NOT NULL;
