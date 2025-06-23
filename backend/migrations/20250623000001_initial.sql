-- Create hardware_profiles table
CREATE TABLE hardware_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gpu_model VARCHAR NOT NULL,
    gpu_memory_gb INTEGER NOT NULL,
    cpu_model VARCHAR NOT NULL,
    cpu_arch VARCHAR NOT NULL,
    ram_gb INTEGER NOT NULL,
    ram_type VARCHAR NOT NULL,
    virtualization_type VARCHAR,
    optimizations TEXT[], -- Array of optimization strings
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create test_runs table
CREATE TABLE test_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR NOT NULL,
    quantization VARCHAR NOT NULL,
    backend VARCHAR NOT NULL,
    backend_version VARCHAR NOT NULL,
    hardware_profile_id UUID REFERENCES hardware_profiles(id),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR NOT NULL DEFAULT 'pending',
    notes TEXT
);

-- Create performance_metrics table
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID REFERENCES test_runs(id) ON DELETE CASCADE,
    metric_name VARCHAR NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create quality_scores table
CREATE TABLE quality_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_run_id UUID REFERENCES test_runs(id) ON DELETE CASCADE,
    benchmark_name VARCHAR NOT NULL,
    category VARCHAR NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    total_questions INTEGER,
    correct_answers INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_test_runs_model_quant ON test_runs(model_name, quantization);
CREATE INDEX idx_test_runs_status ON test_runs(status);
CREATE INDEX idx_test_runs_timestamp ON test_runs(timestamp);
CREATE INDEX idx_performance_metrics_test_run ON performance_metrics(test_run_id);
CREATE INDEX idx_performance_metrics_name ON performance_metrics(metric_name);
CREATE INDEX idx_quality_scores_test_run ON quality_scores(test_run_id);
CREATE INDEX idx_quality_scores_benchmark ON quality_scores(benchmark_name, category);

-- Insert some sample hardware profiles
INSERT INTO hardware_profiles (gpu_model, gpu_memory_gb, cpu_model, cpu_arch, ram_gb, ram_type, virtualization_type, optimizations) VALUES
('RTX 4090', 24, 'AMD Threadripper 1950X', 'Zen1', 256, 'DDR4', 'KVM', ARRAY['pci_passthrough', 'hugepages_1gb', 'cpu_isolation']),
('RTX 4090', 24, 'AMD Threadripper 2950X', 'Zen2', 256, 'DDR4', 'KVM', ARRAY['pci_passthrough', 'hugepages_1gb', 'cpu_isolation']),
('RTX 4080', 16, 'Intel i9-13900K', 'x86_64', 64, 'DDR5', NULL, ARRAY['bare_metal']);

-- Insert some sample test runs and data
INSERT INTO test_runs (model_name, quantization, backend, backend_version, hardware_profile_id, status) 
SELECT 
    'Mistral Small 3.2 24B', 
    'Q8_0', 
    'llama_cpp', 
    'b3312', 
    id, 
    'completed'
FROM hardware_profiles 
WHERE cpu_arch = 'Zen2' 
LIMIT 1;

-- Get the test run ID for sample data
INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    'tokens_per_second',
    45.2,
    'tok/s'
FROM test_runs tr 
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q8_0'
LIMIT 1;

INSERT INTO performance_metrics (test_run_id, metric_name, value, unit)
SELECT 
    tr.id,
    'memory_usage_gb',
    18.5,
    'GB'
FROM test_runs tr 
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q8_0'
LIMIT 1;

-- Sample MMLU-Pro scores
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
    ('Math', 75.2),
    ('Physics', 82.1),
    ('Chemistry', 70.3),
    ('Biology', 68.7),
    ('Computer Science', 85.4),
    ('Economics', 72.8),
    ('History', 65.2),
    ('Law', 60.1),
    ('Philosophy', 58.9),
    ('Psychology', 71.5)
) AS categories(category, score)
WHERE tr.model_name = 'Mistral Small 3.2 24B' AND tr.quantization = 'Q8_0'
LIMIT 10;