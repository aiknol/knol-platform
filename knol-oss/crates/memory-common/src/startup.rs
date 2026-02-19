//! Startup configuration validation.
//!
//! Call [`validate_env`] early in each service's `main()` to catch missing or
//! invalid environment variables before the service attempts to connect to
//! infrastructure. This turns silent runtime failures into clear boot-time errors.

use tracing::{error, info, warn};

/// Describes a required or recommended environment variable.
struct EnvCheck {
    name: &'static str,
    required: bool,
    description: &'static str,
}

/// Validate that critical environment variables are set.
///
/// - **Required** vars cause a hard error (returns `Err`) if missing.
/// - **Recommended** vars emit a warning but allow startup to continue.
///
/// # Example
///
/// ```rust,ignore
/// memory_common::startup::validate_env("service-gateway")?;
/// ```
pub fn validate_env(service_name: &str) -> anyhow::Result<()> {
    let checks = vec![
        // ── Infrastructure (required for all services) ──
        EnvCheck {
            name: "DATABASE_URL",
            required: true,
            description: "PostgreSQL connection string",
        },
        // ── Per-service extras ──
        EnvCheck {
            name: "NATS_URL",
            required: matches!(service_name, "service-write" | "service-graph"),
            description: "NATS JetStream connection URL",
        },
        EnvCheck {
            name: "REDIS_URL",
            required: matches!(service_name, "service-gateway"),
            description: "Redis connection URL (rate limiting & cache)",
        },
        // ── Security (recommended) ──
        EnvCheck {
            name: "ADMIN_ENCRYPTION_KEY",
            required: false,
            description: "AES-256 key for encrypting credentials in the database",
        },
    ];

    let mut missing_required = Vec::new();

    for check in &checks {
        match std::env::var(check.name) {
            Ok(val) if !val.is_empty() => {}
            _ => {
                if check.required {
                    error!(
                        "{} — missing required env var: {} ({})",
                        service_name, check.name, check.description
                    );
                    missing_required.push(check.name);
                } else {
                    warn!(
                        "{} — recommended env var not set: {} ({})",
                        service_name, check.name, check.description
                    );
                }
            }
        }
    }

    if missing_required.is_empty() {
        info!("{} — environment validation passed", service_name);
        Ok(())
    } else {
        anyhow::bail!(
            "{} cannot start: missing required environment variables: {}. \
             See .env.example for reference.",
            service_name,
            missing_required.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_env_with_all_set() {
        // Set required vars
        std::env::set_var("DATABASE_URL", "postgresql://test@localhost/test");
        std::env::set_var("NATS_URL", "nats://localhost:4222");
        std::env::set_var("REDIS_URL", "redis://localhost:6379");

        // Should pass for any service
        assert!(validate_env("service-gateway").is_ok());
        assert!(validate_env("service-write").is_ok());
        assert!(validate_env("service-retrieve").is_ok());
        assert!(validate_env("service-graph").is_ok());
    }
}
