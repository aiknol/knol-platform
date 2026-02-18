//! Configuration types loaded from environment variables.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub database_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_db_url() -> String {
    "postgresql://memory:memory_dev@localhost:5432/memory".to_string()
}

fn default_max_connections() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
}

fn default_nats_url() -> String {
    "nats://localhost:4222".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub anthropic_api_key: String,
    #[serde(default = "default_extraction_model")]
    pub extraction_model: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: usize,
}

fn default_extraction_model() -> String {
    "claude-haiku-4-5-20251001".to_string()
}

fn default_embedding_model() -> String {
    "voyage-3-lite".to_string()
}

fn default_embedding_dim() -> usize {
    1024
}

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    pub jwt_secret: Option<String>,
}

fn default_gateway_port() -> u16 {
    8080
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub nats: NatsConfig,
    pub llm: LlmConfig,
    pub gateway: GatewayConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig {
            database_url: default_db_url(),
            max_connections: default_max_connections(),
        };
        assert!(config.database_url.contains("memory"));
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_redis_config_defaults() {
        let config = RedisConfig {
            redis_url: default_redis_url(),
        };
        assert!(config.redis_url.contains("6379"));
    }

    #[test]
    fn test_nats_config_defaults() {
        let config = NatsConfig {
            nats_url: default_nats_url(),
        };
        assert!(config.nats_url.contains("4222"));
    }

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig {
            anthropic_api_key: "test-key".to_string(),
            extraction_model: default_extraction_model(),
            embedding_model: default_embedding_model(),
            embedding_dim: default_embedding_dim(),
        };
        assert!(config.extraction_model.contains("claude"));
        assert_eq!(config.embedding_dim, 1024);
    }

    #[test]
    fn test_gateway_config_defaults() {
        let config = GatewayConfig {
            port: default_gateway_port(),
            jwt_secret: None,
        };
        assert_eq!(config.port, 8080);
    }
}
