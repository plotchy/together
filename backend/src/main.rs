use axum::{
    routing::{get, post},
    Router,
};
use together::{handlers, utils, Config, get_db_pool};
use sqlx::PgPool;
use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method, HeaderValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::init_logging();
    
    let config = Config::from_env()?;
    let db_config = together::db::DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    // Run migrations
    together::db::migrations::run_migrations(&pool).await?;
    
    let port = config.port;
    let app = create_router(pool, config);
    
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server running on port {}", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_router(pool: PgPool, config: Config) -> Router {
    let cors_layer = create_cors_layer(&config);
    let app_state = (pool, config);
    
    Router::new()
        .route("/health", get(health_check))
        
        // Profile endpoints
        .route("/api/profile/{address}", get(handlers::get_profile))
        .route("/api/check-together/{address}", get(handlers::check_together))
        
        // User endpoints
        .route("/api/user/{address}", get(handlers::get_or_create_user))
        
        // Pending connection endpoints
        .route("/api/user/{user_id}/pending-connection", post(handlers::create_pending_connection))
        .route("/api/user/{user_id}/pending-connections", get(handlers::get_user_pending_connections))
        
        // Attestation endpoints
        .route("/api/attest", post(handlers::attest_together))
        
        // RPC proxy endpoint
        .route("/api/rpc", post(handlers::proxy_rpc))
        .layer(cors_layer)
        .with_state(app_state)
}

fn create_cors_layer(_config: &Config) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(false);

    // Check if ALLOWED_ORIGINS environment variable is set for multiple domains
    if let Ok(cors_origins) = std::env::var("ALLOWED_ORIGINS") {
        let origins: Vec<HeaderValue> = cors_origins
            .split(',')
            .filter_map(|origin| {
                let trimmed = origin.trim();
                if !trimmed.is_empty() {
                    trimmed.parse().ok()
                } else {
                    None
                }
            })
            .collect();
        
        if !origins.is_empty() {
            cors = cors.allow_origin(origins);
        } else {
            // Fallback to permissive if parsing fails
            cors = cors.allow_origin(Any);
        }
    } else {
        // Default to permissive for development
        cors = cors.allow_origin(Any);
    }

    cors
}

async fn health_check() -> &'static str {
    "OK"
}
