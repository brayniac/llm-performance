# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a full-stack LLM performance benchmarking application with three main components:

### Backend (`/backend`)
- **Rust/Axum API server** serving PostgreSQL-backed REST endpoints
- **Database**: PostgreSQL with SQLx migrations in `/migrations`
- **Port**: 3000 (serves both API and static frontend files)
- **Key modules**: `handlers.rs` (API endpoints), `models.rs` (database models)
- **API endpoints**: performance grid, comparisons, configurations, detailed results, experiment uploads

### Frontend (`/frontend`) 
- **SvelteKit application** with TypeScript
- **UI Framework**: Svelte 5 with custom CSS
- **Charts**: ECharts for radar charts and performance visualizations
- **Key components**: `PerformanceGrid`, `ComparisonView`, `DetailView`, `RadarChart`
- **Routes**: home grid, comparison (`/compare/[configA]/[configB]`), detail views (`/detail/[configId]`)

### Types (`/types`)
- **Shared Rust crate** with common data structures
- **Modules**: `api.rs`, `experiment.rs`, `hardware.rs`, `metrics.rs`, `validation.rs`
- **Used by**: Both backend and frontend build processes

## Development Commands

### Frontend (from `/frontend`)
```bash
npm run dev          # Development server with hot reload
npm run build        # Production build
npm run preview      # Preview production build
npm run check        # TypeScript and Svelte checks
npm run check:watch  # Watch mode for checks
npm run lint         # ESLint + Prettier checks
npm run format       # Format code with Prettier
```

### Backend (from `/backend`)
```bash
cargo run           # Start development server
cargo build         # Build the project
cargo test          # Run tests
cargo check         # Check code without building
```

### Types (from `/types`)
```bash
cargo build         # Build shared types crate
cargo test          # Run type validation tests
cargo check         # Check types compilation
```

## Database Setup

The backend expects a PostgreSQL database. Set the `DATABASE_URL` environment variable or it defaults to:
```
postgres://benchmark_user:your_password@localhost/llm_benchmarks
```

Migrations are located in `/backend/migrations` and should be run with `sqlx migrate run`.

## Key Data Flow

1. **Experiment Data**: Uploaded via `/api/upload-experiment` endpoint
2. **Benchmark Scores**: Uploaded via `/api/upload-benchmarks` endpoint (model/quantization specific)
3. **Performance Grid**: Aggregated view from `/api/grouped-performance` showing model comparisons
4. **Detailed Views**: Individual configuration results via `/api/detail/:test_run_id`
5. **Comparisons**: Side-by-side analysis via `/api/comparison` with configA/configB parameters

## Data Architecture

The system separates model-specific quality benchmarks from hardware-specific performance metrics:

- **Model Variants**: Unique combinations of model_name + quantization
- **Benchmark Scores** (MMLU, GSM8K, etc.): Attached to model variants, shared across all hardware
- **Performance Metrics** (tokens/s, memory): Attached to specific test runs with hardware profiles
- This allows uploading MMLU-Pro scores separately without losing performance data

## Development Workflow

1. Start backend: `cd backend && cargo run`
2. Start frontend dev server: `cd frontend && npm run dev`
3. Frontend connects to backend API on port 3000
4. Types changes require rebuilding both backend and frontend

## Important Architecture Notes

- The types crate ensures type safety between frontend and backend
- Backend serves both API endpoints and built frontend static files in production
- Database models use SQLx with compile-time query verification
- Frontend uses server-side rendering with SvelteKit
- All metric definitions and validation logic centralized in the types crate