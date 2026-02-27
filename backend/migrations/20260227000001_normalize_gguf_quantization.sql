-- Normalize quantization values by stripping redundant -GGUF suffix.
-- The GGUF format is implied by the backend (llama.cpp), so Q8_0-GGUF
-- should just be Q8_0 to avoid duplicate groupings.

-- Fix test_runs
UPDATE test_runs
SET quantization = REPLACE(quantization, '-GGUF', '')
WHERE quantization LIKE '%-GGUF';

-- Fix model_variants: merge duplicates by reassigning scores before dedup.
-- First, update mmlu_scores_v2 to point to the non-GGUF variant where one exists.
UPDATE mmlu_scores_v2 ms
SET model_variant_id = target.id
FROM model_variants source
JOIN model_variants target
    ON target.model_name = source.model_name
    AND target.quantization = REPLACE(source.quantization, '-GGUF', '')
    AND target.id != source.id
WHERE ms.model_variant_id = source.id
    AND source.quantization LIKE '%-GGUF';

-- Delete the now-orphaned -GGUF model variants that have a non-GGUF counterpart
DELETE FROM model_variants
WHERE quantization LIKE '%-GGUF'
    AND EXISTS (
        SELECT 1 FROM model_variants mv2
        WHERE mv2.model_name = model_variants.model_name
            AND mv2.quantization = REPLACE(model_variants.quantization, '-GGUF', '')
            AND mv2.id != model_variants.id
    );

-- For remaining -GGUF variants with no non-GGUF counterpart, just rename them
UPDATE model_variants
SET quantization = REPLACE(quantization, '-GGUF', '')
WHERE quantization LIKE '%-GGUF';
