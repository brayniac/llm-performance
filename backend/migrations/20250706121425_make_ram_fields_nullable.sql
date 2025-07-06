-- Make RAM fields nullable in hardware_profiles table
-- This allows systems where RAM information is not available

-- Make ram_gb nullable
ALTER TABLE hardware_profiles 
ALTER COLUMN ram_gb DROP NOT NULL;

-- Make ram_type nullable  
ALTER TABLE hardware_profiles
ALTER COLUMN ram_type DROP NOT NULL;

-- Update any existing rows that have placeholder values (optional)
-- This identifies rows that likely don't have real RAM data
UPDATE hardware_profiles
SET ram_gb = NULL, ram_type = NULL
WHERE ram_gb = 0 AND ram_type = 'Unknown';