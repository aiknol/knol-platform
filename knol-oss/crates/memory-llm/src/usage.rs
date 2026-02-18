//! Token usage persistence — log LLM token consumption to the database.
//!
//! Provides an async function that inserts a row into `llm_usage_log` after
//! each extraction or verification call. This enables cost monitoring and
//! per-tenant billing.

use sqlx::PgPool;
use tracing::{debug, warn};
use uuid::Uuid;

/// Log a single LLM API call's token usage to the database.
///
/// Fails silently (with a warning log) if the table doesn't exist or the
/// insert fails — token logging must never block the extraction pipeline.
pub async fn log_token_usage(
    pool: &PgPool,
    tenant_id: Uuid,
    provider: &str,
    model: &str,
    call_type: &str, // "extraction" or "verification"
    input_tokens: u32,
    output_tokens: u32,
    cache_hit: bool,
) {
    let total = input_tokens + output_tokens;
    let result = sqlx::query(
        r#"
        INSERT INTO llm_usage_log (tenant_id, provider, model, call_type,
                                    input_tokens, output_tokens, total_tokens, cache_hit)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(tenant_id)
    .bind(provider)
    .bind(model)
    .bind(call_type)
    .bind(input_tokens as i32)
    .bind(output_tokens as i32)
    .bind(total as i32)
    .bind(cache_hit)
    .execute(pool)
    .await;

    match result {
        Ok(_) => debug!(
            "Token usage logged: tenant={} provider={} type={} tokens={}",
            tenant_id, provider, call_type, total
        ),
        Err(e) => warn!(
            "Failed to log token usage (non-fatal): {}",
            e
        ),
    }
}

#[cfg(test)]
mod tests {
    // Token usage logging requires a real DB connection, so we only test
    // that the module compiles and the function signature is correct.
    // Integration tests would use a test database.

    #[test]
    fn test_module_compiles() {
        // If this test runs, the module compiles successfully.
        assert!(true);
    }
}
