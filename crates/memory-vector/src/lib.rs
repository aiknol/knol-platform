//! Vector search operations using pgvector.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::debug;

/// Store an embedding vector for a memory.
pub async fn store_vector(
    pool: &PgPool,
    memory_id: Uuid,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    scope: &str,
    kind: &str,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
    embedding: &[f32],
    content_hash: &str,
) -> Result<Uuid, VectorError> {
    // Convert f32 slice to pgvector format string
    let vec_str = format!(
        "[{}]",
        embedding
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO memory_vectors (memory_id, tenant_id, user_id, scope, kind, valid_from, valid_to, embedding, content_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::vector, $9)
        RETURNING id
        "#,
    )
    .bind(memory_id)
    .bind(tenant_id)
    .bind(user_id)
    .bind(scope)
    .bind(kind)
    .bind(valid_from)
    .bind(valid_to)
    .bind(&vec_str)
    .bind(content_hash)
    .fetch_one(pool)
    .await
    .map_err(VectorError::Database)?;

    debug!("Stored vector for memory {} (vector_id={})", memory_id, row);
    Ok(row)
}

/// Search for similar memories using cosine similarity.
pub async fn search_similar(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    query_embedding: &[f32],
    limit: i64,
    min_confidence: Option<f32>,
    scope: Option<&str>,
    kind: Option<&str>,
) -> Result<Vec<VectorSearchHit>, VectorError> {
    let vec_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let min_conf = min_confidence.unwrap_or(0.0);

    let rows = sqlx::query_as::<_, VectorSearchHit>(
        r#"
        SELECT
            m.id,
            m.tenant_id,
            m.user_id,
            m.scope,
            m.kind,
            m.content,
            m.content_json,
            m.confidence,
            m.importance,
            m.status,
            m.valid_from,
            m.valid_to,
            m.event_time,
            m.ingested_at,
            m.source_episode_id,
            m.created_by,
            m.tags,
            m.metadata,
            m.created_at,
            m.updated_at,
            1.0 - (mv.embedding <=> $1::vector) as similarity
        FROM memory_vectors mv
        JOIN memories m ON m.id = mv.memory_id
        WHERE mv.tenant_id = $2
          AND mv.status = 'active'
          AND m.status = 'active'
          AND m.confidence >= $3
          AND ($4::uuid IS NULL OR mv.user_id = $4)
          AND ($5::text IS NULL OR mv.scope = $5)
          AND ($6::text IS NULL OR mv.kind = $6)
          AND (m.valid_to IS NULL OR m.valid_to > now())
        ORDER BY mv.embedding <=> $1::vector
        LIMIT $7
        "#,
    )
    .bind(&vec_str)
    .bind(tenant_id)
    .bind(min_conf)
    .bind(user_id)
    .bind(scope)
    .bind(kind)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(VectorError::Database)?;

    debug!("Vector search returned {} results", rows.len());
    Ok(rows)
}

/// Check for duplicate content by hash.
pub async fn find_duplicate(
    pool: &PgPool,
    tenant_id: Uuid,
    content_hash: &str,
) -> Result<Option<Uuid>, VectorError> {
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT memory_id FROM memory_vectors
        WHERE tenant_id = $1 AND content_hash = $2 AND status = 'active'
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(content_hash)
    .fetch_optional(pool)
    .await
    .map_err(VectorError::Database)?;

    Ok(row)
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct VectorSearchHit {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub scope: String,
    pub kind: String,
    pub content: String,
    pub content_json: Option<serde_json::Value>,
    pub confidence: f32,
    pub importance: f32,
    pub status: String,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub event_time: Option<DateTime<Utc>>,
    pub ingested_at: DateTime<Utc>,
    pub source_episode_id: Option<Uuid>,
    pub created_by: String,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub similarity: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
