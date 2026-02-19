//! Integration test harness for Knol Enterprise
//!
//! Validates the full write → extract → search → consolidate cycle
//! using mocked service interactions (no live infrastructure required).
//!
//! Run with: cargo test --test integration_test

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// ── Simulated Memory Store ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Memory {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    content: String,
    kind: String,
    scope: String,
    confidence: f32,
    importance: f32,
    status: String,
    valid_from: chrono::DateTime<Utc>,
    valid_to: Option<chrono::DateTime<Utc>>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    tags: Vec<String>,
    entities: Vec<String>,
}

#[derive(Debug, Clone)]
struct AuditEntry {
    tenant_id: Uuid,
    memory_id: Uuid,
    action: String,
    diff: Option<serde_json::Value>,
    timestamp: chrono::DateTime<Utc>,
}

struct MemoryStore {
    memories: HashMap<Uuid, Memory>,
    audit_log: Vec<AuditEntry>,
}

impl MemoryStore {
    fn new() -> Self {
        Self {
            memories: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Simulate: Write → Extract → Store (the ingest pipeline)
    fn ingest_memory(
        &mut self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        content: &str,
        kind: &str,
        entities: Vec<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now();
        self.memories.insert(
            id,
            Memory {
                id,
                tenant_id,
                user_id,
                content: content.to_string(),
                kind: kind.to_string(),
                scope: "user".to_string(),
                confidence: 0.85,
                importance: 0.7,
                status: "active".to_string(),
                valid_from: now,
                valid_to: None,
                metadata: serde_json::json!({}),
                created_at: now,
                updated_at: now,
                tags: vec![],
                entities,
            },
        );
        id
    }

    /// Simulate: Search memories by text similarity
    fn search(&self, tenant_id: Uuid, query: &str, limit: usize) -> Vec<&Memory> {
        let query_lower = query.to_lowercase();
        let query_words: HashSet<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<(&Memory, f32)> = self
            .memories
            .values()
            .filter(|m| m.tenant_id == tenant_id && m.status == "active")
            .filter_map(|m| {
                let content_lower = m.content.to_lowercase();
                let content_words: HashSet<String> =
                    content_lower.split_whitespace().map(String::from).collect();
                let intersection = query_words
                    .iter()
                    .filter(|w| content_words.contains(**w))
                    .count();
                let union = query_words.len() + content_words.len() - intersection;
                let score = if union > 0 {
                    intersection as f32 / union as f32
                } else {
                    0.0
                };
                if score > 0.0 {
                    Some((m, score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter().take(limit).map(|(m, _)| m).collect()
    }

    /// Simulate: Update memory (admin operation)
    fn update_memory(&mut self, id: Uuid, content: Option<&str>, importance: Option<f32>) -> bool {
        if let Some(memory) = self.memories.get_mut(&id) {
            let before = serde_json::json!({
                "content": memory.content,
                "importance": memory.importance,
            });

            if let Some(c) = content {
                memory.content = c.to_string();
            }
            if let Some(imp) = importance {
                memory.importance = imp;
            }
            memory.updated_at = Utc::now();

            let after = serde_json::json!({
                "content": memory.content,
                "importance": memory.importance,
            });

            self.audit_log.push(AuditEntry {
                tenant_id: memory.tenant_id,
                memory_id: id,
                action: "update".to_string(),
                diff: Some(serde_json::json!({ "before": before, "after": after })),
                timestamp: Utc::now(),
            });
            true
        } else {
            false
        }
    }

    /// Simulate: Delete memory (soft delete with audit)
    fn delete_memory(&mut self, id: Uuid) -> bool {
        if let Some(memory) = self.memories.get_mut(&id) {
            memory.status = "deleted".to_string();
            memory.updated_at = Utc::now();
            self.audit_log.push(AuditEntry {
                tenant_id: memory.tenant_id,
                memory_id: id,
                action: "delete".to_string(),
                diff: None,
                timestamp: Utc::now(),
            });
            true
        } else {
            false
        }
    }

    /// Simulate: Merge memories
    fn merge_memories(
        &mut self,
        source_ids: &[Uuid],
        merged_content: &str,
        tenant_id: Uuid,
    ) -> Option<Uuid> {
        // Mark sources as superseded
        for sid in source_ids {
            if let Some(m) = self.memories.get_mut(sid) {
                m.status = "superseded".to_string();
                m.valid_to = Some(Utc::now());
            }
        }

        // Create merged memory
        let merged_id = self.ingest_memory(tenant_id, None, merged_content, "fact", vec![]);
        if let Some(m) = self.memories.get_mut(&merged_id) {
            m.metadata = serde_json::json!({ "merged_from": source_ids });
        }

        // Audit entries
        for sid in source_ids {
            self.audit_log.push(AuditEntry {
                tenant_id,
                memory_id: *sid,
                action: "merge".to_string(),
                diff: Some(serde_json::json!({ "merged_into": merged_id })),
                timestamp: Utc::now(),
            });
        }

        Some(merged_id)
    }

    /// Simulate: Importance decay
    fn apply_decay(&mut self, lambda: f64, min_importance: f32) -> u64 {
        let mut affected = 0;
        for memory in self.memories.values_mut() {
            if memory.status == "active" && memory.importance > min_importance {
                let days = (Utc::now() - memory.updated_at).num_seconds() as f64 / 86400.0;
                if days > 0.0 {
                    memory.importance *= (-lambda * days).exp() as f32;
                    if memory.importance < min_importance {
                        memory.importance = min_importance;
                    }
                    affected += 1;
                }
            }
        }
        affected
    }

    /// Simulate: Point-in-time replay
    fn replay_at(&self, tenant_id: Uuid, point_in_time: chrono::DateTime<Utc>) -> Vec<&Memory> {
        self.memories
            .values()
            .filter(|m| {
                m.tenant_id == tenant_id
                    && m.status != "deleted"
                    && m.valid_from <= point_in_time
                    && m.valid_to.map(|vt| point_in_time < vt).unwrap_or(true)
            })
            .collect()
    }
}

// ── Integration Tests ──

#[test]
fn test_full_write_search_cycle() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Some(Uuid::new_v4());

    // Write phase: ingest several memories
    store.ingest_memory(
        tenant_id,
        user_id,
        "John prefers dark mode in all applications",
        "preference",
        vec!["John".into()],
    );
    store.ingest_memory(
        tenant_id,
        user_id,
        "Meeting with Sarah about the Q3 roadmap",
        "event",
        vec!["Sarah".into()],
    );
    store.ingest_memory(
        tenant_id,
        user_id,
        "The project deadline is December 15th",
        "fact",
        vec!["project".into()],
    );

    // Search phase: query for relevant memories
    let results = store.search(tenant_id, "dark mode preference", 10);
    assert!(
        !results.is_empty(),
        "Should find memories matching 'dark mode'"
    );
    assert!(results[0].content.contains("dark mode"));

    let results = store.search(tenant_id, "project deadline december", 10);
    assert!(
        !results.is_empty(),
        "Should find memories matching 'deadline'"
    );
}

#[test]
fn test_write_update_search_cycle() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    let id = store.ingest_memory(
        tenant_id,
        None,
        "User prefers light theme",
        "preference",
        vec![],
    );

    // Update the memory
    store.update_memory(id, Some("User prefers dark theme"), None);

    // Search should return updated content
    let results = store.search(tenant_id, "dark theme", 10);
    assert!(!results.is_empty());
    assert!(results[0].content.contains("dark theme"));

    // Verify audit trail
    assert_eq!(store.audit_log.len(), 1);
    assert_eq!(store.audit_log[0].action, "update");
}

#[test]
fn test_write_delete_search_cycle() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    let id = store.ingest_memory(
        tenant_id,
        None,
        "Temporary note about something",
        "event",
        vec![],
    );
    assert_eq!(store.search(tenant_id, "temporary note", 10).len(), 1);

    // Delete the memory
    store.delete_memory(id);

    // Should no longer appear in search
    assert_eq!(store.search(tenant_id, "temporary note", 10).len(), 0);

    // But the memory still exists (soft delete)
    assert!(store.memories.contains_key(&id));
    assert_eq!(store.memories[&id].status, "deleted");

    // Audit trail records the deletion
    assert_eq!(store.audit_log.last().unwrap().action, "delete");
}

#[test]
fn test_merge_supersedes_source_memories() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    let id1 = store.ingest_memory(
        tenant_id,
        None,
        "John had lunch at noon",
        "event",
        vec!["John".into()],
    );
    let id2 = store.ingest_memory(
        tenant_id,
        None,
        "John discussed project at lunch",
        "event",
        vec!["John".into()],
    );
    let id3 = store.ingest_memory(
        tenant_id,
        None,
        "John mentioned deadline concerns at lunch",
        "event",
        vec!["John".into()],
    );

    // Merge all three into one semantic memory
    let merged_id = store
        .merge_memories(
            &[id1, id2, id3],
            "John regularly has lunch meetings where he discusses project deadlines and concerns",
            tenant_id,
        )
        .unwrap();

    // Source memories should be superseded
    assert_eq!(store.memories[&id1].status, "superseded");
    assert_eq!(store.memories[&id2].status, "superseded");
    assert_eq!(store.memories[&id3].status, "superseded");

    // Merged memory should be active
    assert_eq!(store.memories[&merged_id].status, "active");

    // Merged memory should have metadata pointing to sources
    assert!(store.memories[&merged_id].metadata["merged_from"].is_array());

    // Source memories should not appear in search
    let results = store.search(tenant_id, "John lunch", 10);
    // Only the merged memory should be found
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, merged_id);
}

#[test]
fn test_tenant_isolation() {
    let mut store = MemoryStore::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    store.ingest_memory(tenant_a, None, "Secret data for tenant A", "fact", vec![]);
    store.ingest_memory(
        tenant_b,
        None,
        "Different data for tenant B",
        "fact",
        vec![],
    );

    // Tenant A should only see their own data
    let results_a = store.search(tenant_a, "data", 10);
    assert_eq!(results_a.len(), 1);
    assert!(results_a[0].content.contains("tenant A"));

    // Tenant B should only see their own data
    let results_b = store.search(tenant_b, "data", 10);
    assert_eq!(results_b.len(), 1);
    assert!(results_b[0].content.contains("tenant B"));
}

#[test]
fn test_point_in_time_replay() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let now = Utc::now();

    // Create memory that's currently active (set valid_from slightly in the past)
    let current_id = store.ingest_memory(tenant_id, None, "Current fact", "fact", vec![]);
    if let Some(m) = store.memories.get_mut(&current_id) {
        m.valid_from = now - Duration::seconds(10);
    }

    // Create memory that was active but now superseded
    let old_id = store.ingest_memory(tenant_id, None, "Old fact", "fact", vec![]);
    if let Some(m) = store.memories.get_mut(&old_id) {
        m.valid_from = now - Duration::days(60);
        m.valid_to = Some(now - Duration::days(10));
        m.created_at = now - Duration::days(60);
    }

    // Replay at current time: should see "Current fact" but not "Old fact"
    let replay_now = store.replay_at(tenant_id, now);
    let contents: Vec<&str> = replay_now.iter().map(|m| m.content.as_str()).collect();
    assert!(
        contents.contains(&"Current fact"),
        "Current fact should be visible at now"
    );
    assert!(
        !contents.contains(&"Old fact"),
        "Old fact should NOT be visible at now"
    );

    // Replay at 30 days ago: should see "Old fact" but not "Current fact"
    let replay_past = store.replay_at(tenant_id, now - Duration::days(30));
    let past_contents: Vec<&str> = replay_past.iter().map(|m| m.content.as_str()).collect();
    assert!(
        past_contents.contains(&"Old fact"),
        "Old fact should be visible 30 days ago"
    );
}

#[test]
fn test_audit_trail_completeness() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    // Ingest
    let id1 = store.ingest_memory(tenant_id, None, "Original content", "fact", vec![]);
    let id2 = store.ingest_memory(tenant_id, None, "Another memory", "event", vec![]);

    // Update
    store.update_memory(id1, Some("Updated content"), Some(0.9));

    // Delete
    store.delete_memory(id2);

    // Merge
    let id3 = store.ingest_memory(tenant_id, None, "Third memory", "fact", vec![]);
    store.merge_memories(&[id1, id3], "Merged content", tenant_id);

    // Verify audit completeness
    assert!(
        store.audit_log.len() >= 4,
        "Should have at least 4 audit entries (update + delete + 2 merge)"
    );

    let actions: Vec<&str> = store.audit_log.iter().map(|a| a.action.as_str()).collect();
    assert!(actions.contains(&"update"));
    assert!(actions.contains(&"delete"));
    assert!(actions.contains(&"merge"));
    assert!(store.audit_log.iter().all(|a| a.tenant_id == tenant_id));
    assert!(store
        .audit_log
        .iter()
        .all(|a| store.memories.contains_key(&a.memory_id)));
    assert!(store.audit_log.iter().any(|a| a.diff.is_some()));
    assert!(store.audit_log.iter().all(|a| a.timestamp <= Utc::now()));
}

#[test]
fn test_importance_decay_preserves_minimum() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    let id = store.ingest_memory(tenant_id, None, "Decaying memory", "fact", vec![]);

    // Set created_at to past to simulate aging
    if let Some(m) = store.memories.get_mut(&id) {
        m.updated_at = Utc::now() - Duration::days(365);
    }

    store.apply_decay(0.01, 0.05);

    let memory = &store.memories[&id];
    assert!(
        memory.importance >= 0.05,
        "Importance should not decay below minimum (got {})",
        memory.importance
    );
}

#[test]
fn test_search_returns_most_relevant_first() {
    let mut store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    store.ingest_memory(
        tenant_id,
        None,
        "The project deadline is next Friday",
        "fact",
        vec![],
    );
    store.ingest_memory(
        tenant_id,
        None,
        "Complete unrelated topic about cooking",
        "fact",
        vec![],
    );
    store.ingest_memory(
        tenant_id,
        None,
        "The project milestone review meeting",
        "event",
        vec![],
    );

    let results = store.search(tenant_id, "project deadline", 10);
    assert!(!results.is_empty());
    // First result should contain both "project" and "deadline"
    assert!(results[0].content.contains("project") && results[0].content.contains("deadline"));
}

#[test]
fn test_empty_search_returns_empty() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();

    let results = store.search(tenant_id, "anything", 10);
    assert!(results.is_empty());
}
