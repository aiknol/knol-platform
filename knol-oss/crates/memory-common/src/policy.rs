//! Policy enforcement engine for write and retrieve paths.
//!
//! This module provides a comprehensive policy enforcement system for the memory platform,
//! allowing tenants to define and apply policies for data retention, access control,
//! PII redaction, rate limiting, and content filtering.

use crate::pii::PiiDetector;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Represents a policy enforcement decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyResult {
    /// Policy check passed; allow the operation
    Allow,
    /// Policy check failed; deny with a reason
    Deny(String),
    /// Policy allows operation but with modifications
    Modify(PolicyModifications),
}

/// Modifications to apply to content during policy enforcement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyModifications {
    /// Modified content after applying policies
    pub content: Option<String>,
    /// PII redaction was applied
    pub pii_redacted: bool,
    /// Reason for modifications
    pub reason: Option<String>,
}

/// Types of policies that can be enforced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum PolicyType {
    /// Data retention policy
    Retention,
    /// Access control policy
    Access,
    /// PII redaction policy
    Redaction,
    /// Rate limiting policy
    RateLimit,
    /// Content filtering policy
    ContentFilter,
}

impl PolicyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Retention => "retention",
            Self::Access => "access",
            Self::Redaction => "redaction",
            Self::RateLimit => "rate_limit",
            Self::ContentFilter => "content_filter",
        }
    }
}

/// Rules for policy enforcement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyRules {
    /// Maximum retention period in days
    pub max_retention_days: Option<u32>,
    /// Allowed memory access scopes
    pub allowed_scopes: Option<Vec<String>>,
    /// Blocked keywords that trigger content filtering
    pub blocked_keywords: Option<Vec<String>>,
    /// Whether to require automatic PII redaction
    pub require_pii_redaction: Option<bool>,
    /// Maximum memory size in bytes
    pub max_memory_size_bytes: Option<usize>,
    /// Allowed memory types (episodic, semantic, etc.)
    pub allowed_memory_types: Option<Vec<String>>,
}

/// Represents a policy stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier for the policy
    pub id: Uuid,
    /// Tenant ID this policy belongs to
    pub tenant_id: Uuid,
    /// Descriptive name for the policy
    pub name: String,
    /// Type of policy
    pub policy_type: PolicyType,
    /// Policy enforcement rules
    pub rules: PolicyRules,
    /// Whether this policy is currently active
    pub enabled: bool,
}

/// Policy enforcement engine with database integration
#[derive(Clone)]
pub struct PolicyEngine {
    db: PgPool,
}

impl PolicyEngine {
    /// Creates a new PolicyEngine with database connection pool
    pub fn new(db: PgPool) -> Self {
        PolicyEngine { db }
    }

    /// Loads all active policies for a tenant from the database
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant whose policies should be loaded
    ///
    /// # Returns
    /// A vector of policies, or an error if the database query fails
    pub async fn load_policies(&self, tenant_id: Uuid) -> Result<Vec<Policy>, sqlx::Error> {
        sqlx::query_as::<_, (String, String, String, String, bool)>(
            r#"
            SELECT id, tenant_id, name, policy_type, enabled
            FROM policies
            WHERE tenant_id = $1 AND enabled = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.db)
        .await
        .map(|rows| {
            rows.into_iter()
                .filter_map(|(id, tenant_id, name, policy_type, enabled)| {
                    let parsed_id = Uuid::parse_str(&id).ok()?;
                    let parsed_tenant = Uuid::parse_str(&tenant_id).ok()?;
                    let policy_type_parsed = match policy_type.as_str() {
                        "retention" => PolicyType::Retention,
                        "access" => PolicyType::Access,
                        "redaction" => PolicyType::Redaction,
                        "rate_limit" => PolicyType::RateLimit,
                        "content_filter" => PolicyType::ContentFilter,
                        _ => return None,
                    };

                    Some(Policy {
                        id: parsed_id,
                        tenant_id: parsed_tenant,
                        name,
                        policy_type: policy_type_parsed,
                        rules: PolicyRules::default(),
                        enabled,
                    })
                })
                .collect()
        })
    }

    /// Enforces write policies on memory content before storage
    ///
    /// Checks:
    /// - Content size limits
    /// - Blocked keywords
    /// - Memory type allowance
    /// - Auto-applies PII redaction if required
    ///
    /// # Arguments
    /// * `policies` - Policies to enforce
    /// * `content` - Memory content to validate
    /// * `memory_type` - Type of memory being stored
    ///
    /// # Returns
    /// A PolicyResult indicating whether to Allow, Deny, or Modify the content
    pub fn enforce_write_policies(
        &self,
        policies: &[Policy],
        content: &str,
        memory_type: &str,
    ) -> PolicyResult {
        // Check content size limits
        for policy in policies {
            if policy.policy_type == PolicyType::Retention {
                if let Some(max_size) = policy.rules.max_memory_size_bytes {
                    if content.len() > max_size {
                        return PolicyResult::Deny(format!(
                            "Content size {} exceeds limit of {} bytes",
                            content.len(),
                            max_size
                        ));
                    }
                }
            }
        }

        // Check blocked keywords
        for policy in policies {
            if policy.policy_type == PolicyType::ContentFilter {
                if let Some(blocked_keywords) = &policy.rules.blocked_keywords {
                    let content_lower = content.to_lowercase();
                    for keyword in blocked_keywords {
                        if content_lower.contains(&keyword.to_lowercase()) {
                            return PolicyResult::Deny(format!(
                                "Content contains blocked keyword: '{}'",
                                keyword
                            ));
                        }
                    }
                }
            }
        }

        // Check allowed memory types
        for policy in policies {
            if policy.policy_type == PolicyType::Access {
                if let Some(allowed_types) = &policy.rules.allowed_memory_types {
                    if !allowed_types
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(memory_type))
                    {
                        return PolicyResult::Deny(format!(
                            "Memory type '{}' is not allowed by policy",
                            memory_type
                        ));
                    }
                }
            }
        }

        // Check if PII redaction is required
        for policy in policies {
            if policy.policy_type == PolicyType::Redaction
                && policy.rules.require_pii_redaction == Some(true)
            {
                let detector = PiiDetector::new();
                let redacted = detector.redact(content);

                if !redacted.redactions.is_empty() {
                    return PolicyResult::Modify(PolicyModifications {
                        content: Some(redacted.text),
                        pii_redacted: true,
                        reason: Some(format!(
                            "Automatic PII redaction applied: {} items redacted",
                            redacted.redactions.len()
                        )),
                    });
                }
            }
        }

        PolicyResult::Allow
    }

    /// Enforces read policies on memory content before retrieval
    ///
    /// Filters by:
    /// - Allowed scopes
    /// - Applies PII redaction on output if required
    ///
    /// # Arguments
    /// * `policies` - Policies to enforce
    /// * `content` - Memory content to filter/redact
    /// * `memory_scope` - Scope of the memory
    ///
    /// # Returns
    /// A PolicyResult with filtered/redacted content or denial
    pub fn enforce_read_policies(
        &self,
        policies: &[Policy],
        content: &str,
        memory_scope: &str,
    ) -> PolicyResult {
        // Filter by allowed scopes
        for policy in policies {
            if policy.policy_type == PolicyType::Access {
                if let Some(allowed_scopes) = &policy.rules.allowed_scopes {
                    if !allowed_scopes
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(memory_scope))
                    {
                        return PolicyResult::Deny(format!(
                            "Memory scope '{}' is not allowed by policy",
                            memory_scope
                        ));
                    }
                }
            }
        }

        // Apply PII redaction on output if policy requires it
        for policy in policies {
            if policy.policy_type == PolicyType::Redaction
                && policy.rules.require_pii_redaction == Some(true)
            {
                let detector = PiiDetector::new();
                let redacted = detector.redact(content);

                if !redacted.redactions.is_empty() {
                    return PolicyResult::Modify(PolicyModifications {
                        content: Some(redacted.text),
                        pii_redacted: true,
                        reason: Some(format!(
                            "PII redaction applied on retrieval: {} items redacted",
                            redacted.redactions.len()
                        )),
                    });
                }
            }
        }

        PolicyResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy(policy_type: PolicyType, rules: PolicyRules) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "Test Policy".to_string(),
            policy_type,
            rules,
            enabled: true,
        }
    }

    // Test helper: enforcement logic is pure/synchronous and doesn't touch DB,
    // but PolicyEngine wraps a PgPool. Use tokio::test to provide the runtime context.

    fn test_engine() -> PolicyEngine {
        let opts = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .database("test");
        let pool = sqlx::Pool::connect_lazy_with(opts);
        PolicyEngine::new(pool)
    }

    #[test]
    fn test_policy_type_as_str() {
        assert_eq!(PolicyType::Retention.as_str(), "retention");
        assert_eq!(PolicyType::Access.as_str(), "access");
        assert_eq!(PolicyType::Redaction.as_str(), "redaction");
        assert_eq!(PolicyType::RateLimit.as_str(), "rate_limit");
        assert_eq!(PolicyType::ContentFilter.as_str(), "content_filter");
    }

    #[tokio::test]
    async fn test_enforce_write_policies_content_size_limit() {
        let engine = test_engine();
        let rules = PolicyRules {
            max_memory_size_bytes: Some(50),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Retention, rules);
        let content = "This is a very long piece of content that exceeds the maximum size limit";
        let result = engine.enforce_write_policies(&[policy], content, "semantic");
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("exceeds limit")),
            _ => panic!("Expected Deny result"),
        }
    }

    #[tokio::test]
    async fn test_enforce_write_policies_content_size_pass() {
        let engine = test_engine();
        let rules = PolicyRules {
            max_memory_size_bytes: Some(100),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Retention, rules);
        let content = "Short content";
        let result = engine.enforce_write_policies(&[policy], content, "semantic");
        assert!(matches!(result, PolicyResult::Allow));
    }

    #[tokio::test]
    async fn test_enforce_write_policies_blocked_keywords() {
        let engine = test_engine();
        let rules = PolicyRules {
            blocked_keywords: Some(vec!["forbidden".to_string(), "banned".to_string()]),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::ContentFilter, rules);

        let result = engine.enforce_write_policies(
            &[policy.clone()],
            "This contains forbidden content",
            "semantic",
        );
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("forbidden")),
            _ => panic!("Expected Deny result for blocked keyword"),
        }

        let result2 = engine.enforce_write_policies(&[policy], "This is clean content", "semantic");
        assert!(matches!(result2, PolicyResult::Allow));
    }

    #[tokio::test]
    async fn test_enforce_write_policies_allowed_memory_types() {
        let engine = test_engine();
        let rules = PolicyRules {
            allowed_memory_types: Some(vec!["episodic".to_string(), "semantic".to_string()]),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Access, rules);

        let result = engine.enforce_write_policies(&[policy.clone()], "content", "semantic");
        assert!(matches!(result, PolicyResult::Allow));

        let result = engine.enforce_write_policies(&[policy], "content", "unknown_type");
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("not allowed")),
            _ => panic!("Expected Deny result for disallowed memory type"),
        }
    }

    #[tokio::test]
    async fn test_enforce_write_policies_pii_redaction() {
        let engine = test_engine();
        let rules = PolicyRules {
            require_pii_redaction: Some(true),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Redaction, rules);
        let result = engine.enforce_write_policies(
            &[policy],
            "Contact me at john@example.com for details",
            "semantic",
        );
        match result {
            PolicyResult::Modify(mods) => {
                assert!(mods.pii_redacted);
                let redacted = mods.content.unwrap();
                assert!(!redacted.contains("john@example.com"));
                assert!(redacted.contains("[REDACTED:Email]"));
            }
            _ => panic!("Expected Modify result with PII redaction"),
        }
    }

    #[tokio::test]
    async fn test_enforce_write_policies_no_pii_to_redact() {
        let engine = test_engine();
        let rules = PolicyRules {
            require_pii_redaction: Some(true),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Redaction, rules);
        let result = engine.enforce_write_policies(
            &[policy],
            "This is clean content without PII",
            "semantic",
        );
        assert!(matches!(result, PolicyResult::Allow));
    }

    #[tokio::test]
    async fn test_enforce_read_policies_scope_filtering() {
        let engine = test_engine();
        let rules = PolicyRules {
            allowed_scopes: Some(vec!["user".to_string(), "team".to_string()]),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Access, rules);

        let result = engine.enforce_read_policies(&[policy.clone()], "Memory content", "user");
        assert!(matches!(result, PolicyResult::Allow));

        let result = engine.enforce_read_policies(&[policy], "Memory content", "secret");
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("not allowed")),
            _ => panic!("Expected Deny result for disallowed scope"),
        }
    }

    #[tokio::test]
    async fn test_enforce_read_policies_pii_redaction() {
        let engine = test_engine();
        let rules = PolicyRules {
            require_pii_redaction: Some(true),
            ..Default::default()
        };
        let policy = create_test_policy(PolicyType::Redaction, rules);
        let result = engine.enforce_read_policies(&[policy], "User phone: 555-123-4567", "user");
        match result {
            PolicyResult::Modify(mods) => {
                assert!(mods.pii_redacted);
                let redacted = mods.content.unwrap();
                assert!(!redacted.contains("555-123-4567"));
                assert!(redacted.contains("[REDACTED:Phone]"));
            }
            _ => panic!("Expected Modify result with PII redaction"),
        }
    }

    #[test]
    fn test_policy_modifications_default() {
        let mods = PolicyModifications::default();
        assert!(mods.content.is_none());
        assert!(!mods.pii_redacted);
        assert!(mods.reason.is_none());
    }

    #[test]
    fn test_policy_rules_default() {
        let rules = PolicyRules::default();
        assert!(rules.max_retention_days.is_none());
        assert!(rules.allowed_scopes.is_none());
        assert!(rules.blocked_keywords.is_none());
        assert!(rules.require_pii_redaction.is_none());
        assert!(rules.max_memory_size_bytes.is_none());
        assert!(rules.allowed_memory_types.is_none());
    }

    #[test]
    fn test_policy_result_serialization() {
        let json = serde_json::to_string(&PolicyResult::Allow).unwrap();
        assert_eq!(json, "\"Allow\"");

        let json = serde_json::to_string(&PolicyResult::Deny("Test reason".to_string())).unwrap();
        assert!(json.contains("Test reason"));
    }

    #[tokio::test]
    async fn test_multiple_policies_enforcement() {
        let engine = test_engine();
        let size_policy = create_test_policy(
            PolicyType::Retention,
            PolicyRules {
                max_memory_size_bytes: Some(1000),
                ..Default::default()
            },
        );
        let keyword_policy = create_test_policy(
            PolicyType::ContentFilter,
            PolicyRules {
                blocked_keywords: Some(vec!["restricted".to_string()]),
                ..Default::default()
            },
        );

        let result = engine.enforce_write_policies(
            &[size_policy.clone(), keyword_policy.clone()],
            "This contains restricted information",
            "semantic",
        );
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("restricted")),
            _ => panic!("Expected Deny result"),
        }

        let result = engine.enforce_write_policies(
            &[size_policy, keyword_policy],
            &"x".repeat(2000),
            "semantic",
        );
        match result {
            PolicyResult::Deny(reason) => assert!(reason.contains("exceeds limit")),
            _ => panic!("Expected Deny result"),
        }
    }

    #[test]
    fn test_policy_type_equality() {
        assert_eq!(PolicyType::Retention, PolicyType::Retention);
        assert_ne!(PolicyType::Retention, PolicyType::Access);
    }
}
