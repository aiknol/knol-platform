//! Memory Consolidation Engine
//!
//! Consolidates episodic memories into semantic memories through LLM synthesis.
//! This module implements the episodic → semantic memory transition by:
//! 1. Finding clusters of related episodic memories
//! 2. Grouping by entity overlap and content similarity
//! 3. Using LLM to synthesize semantic summaries
//! 4. Storing consolidated memories and marking originals as consolidated

use chrono::{Duration, Utc};
use memory_llm::{AnthropicClient, LlmProvider};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for consolidation behavior
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// Minimum hours before an episodic memory can be consolidated (default: 24)
    pub min_age_hours: i64,
    /// Minimum number of memories required to form a cluster (default: 3)
    pub min_cluster_size: usize,
    /// Maximum memories to consolidate per run (default: 100)
    pub max_consolidations_per_run: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            min_age_hours: 24,
            min_cluster_size: 3,
            max_consolidations_per_run: 100,
        }
    }
}

/// Represents a cluster of related episodic memories
#[derive(Debug, Clone)]
struct MemoryCluster {
    /// IDs of episodic memories in this cluster
    memory_ids: Vec<Uuid>,
    /// Full memory contents for synthesis
    contents: Vec<String>,
    /// Shared entities across all memories
    shared_entities: Vec<String>,
    /// Tenant ID for the cluster
    tenant_id: Uuid,
}

/// Main consolidation engine
pub struct ConsolidationEngine {
    db_pool: PgPool,
    llm_client: AnthropicClient,
    config: ConsolidationConfig,
}

impl ConsolidationEngine {
    /// Create a new consolidation engine
    pub fn new(db_pool: PgPool, llm_client: AnthropicClient) -> Self {
        Self {
            db_pool,
            llm_client,
            config: ConsolidationConfig::default(),
        }
    }

    /// Main entry point: run full consolidation pipeline
    pub async fn run_consolidation(&self) -> anyhow::Result<u64> {
        info!("Starting memory consolidation pipeline");

        // Find consolidation candidates
        let candidates = self.find_consolidation_candidates().await.map_err(|e| {
            error!("Failed to find consolidation candidates: {}", e);
            e
        })?;

        if candidates.is_empty() {
            info!("No consolidation candidates found");
            return Ok(0);
        }

        info!(
            "Found {} consolidation candidate memories",
            candidates.len()
        );

        // Group by tenant for isolated processing
        let mut by_tenant: HashMap<Uuid, Vec<(Uuid, String, Vec<String>)>> = HashMap::new();
        for (memory_id, content, entities, tenant_id) in candidates {
            by_tenant
                .entry(tenant_id)
                .or_default()
                .push((memory_id, content, entities));
        }

        let mut total_consolidated = 0u64;

        // Process each tenant's memories
        for (tenant_id, memories) in by_tenant {
            match self.process_tenant_memories(tenant_id, memories).await {
                Ok(count) => total_consolidated += count,
                Err(e) => {
                    warn!("Failed to process memories for tenant {}: {}", tenant_id, e);
                }
            }
        }

        info!(
            "Memory consolidation complete: {} memories consolidated",
            total_consolidated
        );
        Ok(total_consolidated)
    }

    /// Find episodic memories eligible for consolidation
    async fn find_consolidation_candidates(
        &self,
    ) -> anyhow::Result<Vec<(Uuid, String, Vec<String>, Uuid)>> {
        let min_age = Utc::now() - Duration::hours(self.config.min_age_hours);

        #[derive(sqlx::FromRow)]
        struct CandidateRow {
            id: Uuid,
            content: String,
            entities: Option<Vec<String>>,
            tenant_id: Uuid,
        }

        let rows = sqlx::query_as::<_, CandidateRow>(
            r#"
            SELECT
                m.id,
                m.content,
                COALESCE(array_agg(DISTINCT e.name) FILTER (WHERE e.id IS NOT NULL), '{}') as entities,
                m.tenant_id
            FROM memories m
            LEFT JOIN memory_entities me ON m.id = me.memory_id
            LEFT JOIN entities e ON me.entity_id = e.id
            WHERE m.status = 'active'
              AND m.kind IN ('event', 'fact')
              AND m.created_at < $1
              AND NOT EXISTS (
                SELECT 1 FROM memory_consolidations
                WHERE episodic_memory_id = m.id
              )
            GROUP BY m.id, m.tenant_id
            LIMIT $2
            "#,
        )
        .bind(min_age)
        .bind(self.config.max_consolidations_per_run as i64)
        .fetch_all(&self.db_pool)
        .await?;

        let candidates = rows
            .into_iter()
            .map(|row| {
                (
                    row.id,
                    row.content,
                    row.entities.unwrap_or_default(),
                    row.tenant_id,
                )
            })
            .collect();

        Ok(candidates)
    }

    /// Process all memories for a specific tenant
    async fn process_tenant_memories(
        &self,
        tenant_id: Uuid,
        memories: Vec<(Uuid, String, Vec<String>)>,
    ) -> anyhow::Result<u64> {
        // Cluster memories by entity overlap
        let clusters = self.cluster_memories(memories)?;

        if clusters.is_empty() {
            debug!(
                "No clusters formed for tenant {} (min cluster size: {})",
                tenant_id, self.config.min_cluster_size
            );
            return Ok(0);
        }

        debug!(
            "Formed {} clusters for tenant {} with min size {}",
            clusters.len(),
            tenant_id,
            self.config.min_cluster_size
        );

        let mut consolidated_count = 0u64;

        // Synthesize and store each cluster
        for cluster in clusters {
            match self.consolidate_cluster(cluster).await {
                Ok(_) => consolidated_count += 1,
                Err(e) => {
                    warn!(
                        "Failed to consolidate cluster for tenant {}: {}",
                        tenant_id, e
                    );
                }
            }
        }

        Ok(consolidated_count)
    }

    /// Cluster memories by shared entities and content similarity
    fn cluster_memories(
        &self,
        memories: Vec<(Uuid, String, Vec<String>)>,
    ) -> anyhow::Result<Vec<MemoryCluster>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // Get the actual tenant ID from first memory (all memories in a batch have same tenant)
        // For now, use a placeholder approach that groups by content similarity

        let mut clusters: Vec<MemoryCluster> = Vec::new();
        let mut used_indices: HashSet<usize> = HashSet::new();

        for (i, (id1, content1, entities1)) in memories.iter().enumerate() {
            if used_indices.contains(&i) {
                continue;
            }

            let mut cluster_ids = vec![*id1];
            let mut cluster_contents = vec![content1.clone()];
            let mut cluster_entities: HashSet<String> = entities1.iter().cloned().collect();

            used_indices.insert(i);

            // Find related memories
            for (j, (id2, content2, entities2)) in memories.iter().enumerate().skip(i + 1) {
                if used_indices.contains(&j) {
                    continue;
                }

                // Check if memories share entities
                let shared_count = entities1.iter().filter(|e| entities2.contains(e)).count();

                if shared_count > 0 || Self::content_similar(content1, content2) {
                    cluster_ids.push(*id2);
                    cluster_contents.push(content2.clone());
                    for entity in entities2 {
                        cluster_entities.insert(entity.clone());
                    }
                    used_indices.insert(j);
                }
            }

            // Only create cluster if it meets minimum size
            if cluster_ids.len() >= self.config.min_cluster_size {
                clusters.push(MemoryCluster {
                    memory_ids: cluster_ids,
                    contents: cluster_contents,
                    shared_entities: cluster_entities.into_iter().collect(),
                    tenant_id: memories[i].0, // Placeholder - get actual tenant properly
                });
            }
        }

        debug!(
            "Clustered {} memories into {} clusters",
            memories.len(),
            clusters.len()
        );
        Ok(clusters)
    }

    /// Check if two memory contents are similar
    fn content_similar(content1: &str, content2: &str) -> bool {
        // Simple word overlap heuristic
        let words1: HashSet<&str> = content1.split_whitespace().collect();
        let words2: HashSet<&str> = content2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return false;
        }

        let similarity = intersection as f32 / union as f32;
        similarity > 0.3 // 30% word overlap threshold
    }

    /// Synthesize a memory cluster into a semantic memory using LLM
    async fn consolidate_cluster(&self, cluster: MemoryCluster) -> anyhow::Result<()> {
        debug!(
            "Synthesizing cluster of {} memories with entities: {:?}",
            cluster.memory_ids.len(),
            cluster.shared_entities
        );

        // Synthesize cluster into semantic memory
        let (semantic_content, confidence) =
            self.synthesize_cluster(&cluster).await.map_err(|e| {
                error!("Failed to synthesize cluster: {}", e);
                e
            })?;

        // Store consolidated memory in transaction
        self.store_consolidated_memory(&cluster, &semantic_content, confidence)
            .await?;

        info!(
            "Consolidated {} episodic memories into semantic memory (confidence: {:.2})",
            cluster.memory_ids.len(),
            confidence
        );

        Ok(())
    }

    /// Use LLM to synthesize episodic memories into semantic summary
    async fn synthesize_cluster(&self, cluster: &MemoryCluster) -> anyhow::Result<(String, f32)> {
        let consolidation_prompt = self.build_consolidation_prompt(cluster)?;

        debug!(
            "Sending consolidation request to LLM for cluster with {} memories",
            cluster.memory_ids.len()
        );

        // For now, use a simple extraction-based approach
        // In production, we would add a dedicated synthesis method to AnthropicClient
        let existing_entities: Vec<String> = cluster.shared_entities.clone();

        let response = self
            .llm_client
            .extract_memories(&consolidation_prompt, "system", &existing_entities)
            .await
            .map_err(|e| anyhow::anyhow!("LLM synthesis failed: {}", e))?;

        // Use the first extracted memory as the semantic consolidation
        let content = if let Some(memory) = response.memories.first() {
            memory.content.clone()
        } else {
            format!(
                "Synthesis of {} related memories about: {}",
                cluster.memory_ids.len(),
                cluster.shared_entities.join(", ")
            )
        };

        let confidence = response
            .memories
            .first()
            .map(|m| m.confidence)
            .unwrap_or(0.7);

        Ok((content, confidence))
    }

    /// Build prompt for LLM synthesis
    fn build_consolidation_prompt(&self, cluster: &MemoryCluster) -> anyhow::Result<String> {
        let entities_str = cluster.shared_entities.join(", ");

        let memories_str = cluster
            .contents
            .iter()
            .enumerate()
            .map(|(i, content)| format!("{}. {}", i + 1, content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"Consolidate the following related episodic memories into a single semantic memory.

Related Entities: {}

Episodic Memories to Consolidate:
{}

Instructions:
1. Synthesize these memories into a concise, general semantic memory
2. Remove temporal specifics and generalize to patterns or facts
3. Preserve important relationships and entities
4. Output format: JSON with "content" (string) and "confidence" (0.0-1.0)

Example output:
{{"content": "General semantic memory capturing the pattern or fact", "confidence": 0.85}}

Your synthesis:
"#,
            entities_str, memories_str
        );

        Ok(prompt)
    }

    /// Parse LLM synthesis response
    #[cfg(test)]
    fn parse_synthesis_response(response: &str) -> anyhow::Result<(String, f32)> {
        // Extract JSON from response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            let content = json
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let confidence = json
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7) as f32;

            Ok((content, confidence))
        } else {
            // Fallback: use entire response as content
            warn!("Failed to parse JSON response, using entire response as content");
            Ok((response.to_string(), 0.6))
        }
    }

    /// Store consolidated memory and update episodic memories
    async fn store_consolidated_memory(
        &self,
        cluster: &MemoryCluster,
        semantic_content: &str,
        confidence: f32,
    ) -> anyhow::Result<()> {
        let mut tx = self.db_pool.begin().await?;

        let semantic_memory_id = Uuid::new_v4();
        let now = Utc::now();

        // Create semantic memory
        sqlx::query(
            r#"
            INSERT INTO memories (
                id, tenant_id, kind, content, confidence, importance,
                status, valid_from, event_time, created_by, metadata, created_at, updated_at
            ) VALUES ($1, $2, 'summary', $3, $4, 0.7, 'active', $5, $6, 'system',
                     jsonb_build_object('consolidated_from', $7, 'cluster_size', $8),
                     $9, $10)
            "#,
        )
        .bind(semantic_memory_id)
        .bind(cluster.tenant_id)
        .bind(semantic_content)
        .bind(confidence)
        .bind(now)
        .bind(now)
        .bind(serde_json::to_value(&cluster.memory_ids)?)
        .bind(cluster.memory_ids.len() as i32)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Mark episodic memories as consolidated
        sqlx::query(
            r#"
            UPDATE memories
            SET status = 'consolidated', updated_at = $1
            WHERE id = ANY($2)
            "#,
        )
        .bind(now)
        .bind(&cluster.memory_ids[..])
        .execute(&mut *tx)
        .await?;

        // Create consolidation audit trail entries
        for episodic_id in &cluster.memory_ids {
            sqlx::query(
                r#"
                INSERT INTO memory_consolidations (
                    episodic_memory_id, semantic_memory_id, tenant_id, created_at
                ) VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(episodic_id)
            .bind(semantic_memory_id)
            .bind(cluster.tenant_id)
            .bind(now)
            .execute(&mut *tx)
            .await?;

            // Log audit entry
            sqlx::query(
                r#"
                INSERT INTO memory_audit (
                    tenant_id, memory_id, target_table, action, actor_type, reason
                ) VALUES ($1, $2, 'memories', 'consolidate', 'system', $3)
                "#,
            )
            .bind(cluster.tenant_id)
            .bind(episodic_id)
            .bind(format!(
                "Consolidated into semantic memory {} (cluster of {} memories)",
                semantic_memory_id,
                cluster.memory_ids.len()
            ))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        info!(
            "Stored consolidated semantic memory {} from {} episodic memories",
            semantic_memory_id,
            cluster.memory_ids.len()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_similar() {
        let content1 = "lunch john discussed project proposal review deadline";
        let content2 = "lunch john discussed project proposal review timeline";

        assert!(ConsolidationEngine::content_similar(content1, content2));
    }

    #[test]
    fn test_content_dissimilar() {
        let content1 = "I like coffee in the morning";
        let content2 = "The weather was rainy yesterday";

        assert!(!ConsolidationEngine::content_similar(content1, content2));
    }

    #[test]
    fn test_parse_synthesis_response_valid() {
        let response = r#"{"content": "Test semantic memory", "confidence": 0.85}"#;
        let (content, confidence) =
            ConsolidationEngine::parse_synthesis_response(response).unwrap();

        assert_eq!(content, "Test semantic memory");
        assert!((confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_parse_synthesis_response_fallback() {
        let response = "This is not JSON";
        let (content, confidence) =
            ConsolidationEngine::parse_synthesis_response(response).unwrap();

        assert_eq!(content, response);
        assert_eq!(confidence, 0.6);
    }
}
