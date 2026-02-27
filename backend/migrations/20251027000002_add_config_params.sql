-- Add max_context_length and concurrency columns to test_runs table
ALTER TABLE test_runs
ADD COLUMN max_context_length INTEGER,
ADD COLUMN concurrency INTEGER;

-- Add index for filtering by max_context_length
CREATE INDEX IF NOT EXISTS idx_test_runs_max_context_length ON test_runs(max_context_length) WHERE max_context_length IS NOT NULL;

-- Add index for filtering by concurrency
CREATE INDEX IF NOT EXISTS idx_test_runs_concurrency ON test_runs(concurrency) WHERE concurrency IS NOT NULL;

-- Add comment explaining the columns
COMMENT ON COLUMN test_runs.max_context_length IS 'Maximum context/sequence length supported in this test run';
COMMENT ON COLUMN test_runs.concurrency IS 'Number of concurrent requests/sequences during the test';
