//! Application state for the gateway service.

use fred::prelude::*;
use memory_common::db_config;
use sqlx::PgPool;

pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
    pub http_client: reqwest::Client,
    pub port: u16,
    pub write_service_url: String,
    pub retrieve_service_url: String,
    pub admin_service_url: String,
    pub cors_origins: String,
}

impl AppState {
    pub async fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());

        let db_pool = memory_db::create_pool(&database_url, 8).await?;
        let redis_client = memory_cache::create_client(&redis_url).await?;
        let http_client = reqwest::Client::new();

        // Port: DB → env → default
        let port = db_config::load_u64(
            &db_pool, "services.gateway_port", "GATEWAY_PORT", 8080,
        ).await as u16;

        // Service URLs: DB → env → default
        let write_service_url = db_config::load_str(
            &db_pool, "gateway.write_service_url", "WRITE_SERVICE_URL", "http://localhost:8081"
        ).await;
        let retrieve_service_url = db_config::load_str(
            &db_pool, "gateway.retrieve_service_url", "RETRIEVE_SERVICE_URL", "http://localhost:8082"
        ).await;
        let admin_service_url = db_config::load_str(
            &db_pool, "gateway.admin_service_url", "ADMIN_SERVICE_URL", "http://localhost:8084"
        ).await;
        let cors_origins = db_config::load_str(
            &db_pool,
            "gateway.cors_origins",
            "GATEWAY_CORS_ORIGINS",
            "http://localhost:3005,http://localhost:3006,http://localhost:8080",
        )
        .await;

        Ok(Self {
            db_pool,
            redis_client,
            http_client,
            port,
            write_service_url,
            retrieve_service_url,
            admin_service_url,
            cors_origins,
        })
    }
}
