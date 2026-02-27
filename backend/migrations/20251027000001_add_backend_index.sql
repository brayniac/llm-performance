-- Add index for backend column to improve query performance
CREATE INDEX IF NOT EXISTS idx_test_runs_backend ON test_runs(backend);

-- Add composite index for backend + status for common queries
CREATE INDEX IF NOT EXISTS idx_test_runs_backend_status ON test_runs(backend, status);
