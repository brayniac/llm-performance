-- Add lora_adapter column to model_variants
-- LoRA adapters change quality (MMLU scores) but not performance (speed, memory).
-- Use empty string '' as default instead of NULL to avoid NULL-in-unique-constraint ambiguity.

ALTER TABLE model_variants ADD COLUMN lora_adapter VARCHAR(255) NOT NULL DEFAULT '';

ALTER TABLE model_variants DROP CONSTRAINT model_variants_model_name_quantization_key;
ALTER TABLE model_variants ADD CONSTRAINT model_variants_model_name_quantization_lora_key
    UNIQUE(model_name, quantization, lora_adapter);
