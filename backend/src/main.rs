// backend/src/main.rs
use axum::{
    routing::{get, post, delete},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

// Import the types crate
use llm_benchmark_types::HealthResponse;

mod models;
mod handlers;

use handlers::{get_performance_grid, get_comparison, get_configurations, get_detail, upload_experiment, get_grouped_performance, delete_test_run, delete_by_model_quant};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://benchmark_user:your_password@localhost/llm_benchmarks".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations (you'll need to install sqlx-cli: cargo install sqlx-cli)
    // sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { db: pool };

    // Build our application with routes
    let app = Router::new()
        .route("/api/performance-grid", get(get_performance_grid))
        .route("/api/grouped-performance", get(get_grouped_performance))
        .route("/api/comparison", get(get_comparison))
        .route("/api/configurations", get(get_configurations))
        .route("/api/detail/:test_run_id", get(get_detail))
        .route("/api/upload-experiment", post(upload_experiment))
        .route("/api/delete/:test_run_id", delete(delete_test_run))
        .route("/api/delete-by-model", post(delete_by_model_quant))
        .route("/health", get(health_check))
        // Serve static files (your built frontend)
        .nest_service("/", ServeDir::new("../frontend/build"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running on http://localhost:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse::healthy())
}