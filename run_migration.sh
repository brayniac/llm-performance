#!/bin/bash

# Script to run the model variants migration

echo "Running model variants migration..."
echo "This will:"
echo "1. Create new model_variants table"
echo "2. Create new benchmark score tables (_v2)"
echo "3. Migrate existing benchmark data"
echo "4. Keep old tables for rollback"
echo ""
echo "Make sure to backup your database first!"
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    echo "Migration cancelled."
    exit 1
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "DATABASE_URL is not set. Using default development database."
    export DATABASE_URL="postgres://benchmark_user:your_password@localhost/llm_benchmarks"
fi

echo "Using database: $DATABASE_URL"

# Run the migration
psql $DATABASE_URL -f backend/migrations/20250708000001_separate_benchmarks_from_hardware.sql

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Migration completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Test the new endpoints"
    echo "2. Once verified, you can drop the old tables:"
    echo "   - mmlu_scores"
    echo "   - gsm8k_scores"
    echo "   - humaneval_scores"
    echo "   - hellaswag_scores"
    echo "   - truthfulqa_scores"
    echo "   - generic_benchmark_scores"
else
    echo ""
    echo "❌ Migration failed!"
    echo "Check the error messages above."
fi