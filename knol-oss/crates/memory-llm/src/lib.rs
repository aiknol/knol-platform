//! LLM provider abstraction for the Knol memory platform.
//!
//! This crate defines a [`LlmProvider`] trait that allows the rest of the system
//! to call `extract_memories` without being tied to a specific LLM vendor.
//!
//! ## Supported providers
//!
//! | Provider    | Struct                | Models                          |
//! |-------------|----------------------|---------------------------------|
//! | Anthropic   | [`AnthropicProvider`] | Claude Haiku / Sonnet / Opus   |
//! | OpenAI      | [`OpenAiProvider`]    | GPT-4o / GPT-4o-mini / etc.   |
//! | Google      | [`GeminiProvider`]    | Gemini 2.0 Flash / Pro / etc. |
//!
//! ## Quick start
//!
//! ```rust,ignore
//! // Build from admin DB config (recommended):
//! let provider = memory_llm::build_provider_from_db(&db_pool).await?;
//!
//! // Or build manually:
//! let provider = memory_llm::build_provider(&LlmConfig {
//!     provider: LlmProviderKind::Anthropic,
//!     api_key: "sk-ant-...".into(),
//!     model: "claude-haiku-4-5-20251001".into(),
//!     api_url: None,
//! });
//!
//! let result = provider.extract_memories("I like Rust", "user", &[]).await?;
//! ```

pub mod error;
pub mod types;
pub mod prompt;
pub mod provider;
pub mod guardrails;
pub mod triage;
pub mod cache;
pub mod usage;
pub mod dynamic;
pub mod embedding;
pub mod decay;
pub mod conflict;
pub mod anthropic;
pub mod openai;
pub mod gemini;
pub mod factory;

// ── Public re-exports ──

pub use error::LlmError;
pub use types::{TokenUsage, BatchExtractionConfig, ExtractionMessage, LlmProviderKind};
pub use provider::{LlmProvider, ExtractionOptions};
pub use anthropic::AnthropicProvider;
pub use openai::OpenAiProvider;
pub use gemini::GeminiProvider;
pub use factory::{LlmConfig, build_provider, build_provider_from_db, build_guardrail_config_from_db, build_grounding_config_from_db, build_triage_config_from_db};
pub use guardrails::{GuardrailConfig, sanitize_extraction, validate_input, detect_prompt_injection};
pub use triage::{TriageConfig, TriageDecision, triage_content, dynamic_output_tokens, prune_entity_context};
pub use cache::{LlmCacheConfig, cache_key, get_cached, set_cached};
pub use usage::log_token_usage;
pub use dynamic::DynamicLlmProvider;
pub use embedding::{EmbeddingConfig, EmbeddingProvider};
pub use decay::{DecayConfig, DecayFunction, decayed_score, apply_access_boost, batch_decay_scores, build_decay_config_from_db};
pub use conflict::{ConflictConfig, ConflictResolution, ConflictDetection, ConflictType, ConflictAction, ExistingMemory, detect_conflicts, build_conflict_config_from_db};

// ── Backward compatibility ──
// Existing code references `memory_llm::AnthropicClient`. Keep that working.

pub type AnthropicClient = AnthropicProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_token_usage_add() {
        let mut u1 = TokenUsage { input_tokens: 100, output_tokens: 50, total_tokens: 150 };
        let u2 = TokenUsage { input_tokens: 200, output_tokens: 100, total_tokens: 300 };
        u1.add(&u2);
        assert_eq!(u1.input_tokens, 300);
        assert_eq!(u1.output_tokens, 150);
        assert_eq!(u1.total_tokens, 450);
    }

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchExtractionConfig::default();
        assert_eq!(config.max_parallel_requests, 5);
        assert_eq!(config.timeout_per_request, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test-key".into(), "claude-3-haiku-20240307".into());
        assert_eq!(provider.provider_name(), "anthropic");
        assert_eq!(provider.model_name(), "claude-3-haiku-20240307");
        let usage = provider.get_token_usage().await;
        assert_eq!(usage.total_tokens, 0);
    }

    #[tokio::test]
    async fn test_openai_provider_creation() {
        let provider = OpenAiProvider::new("test-key".into(), "gpt-4o-mini".into());
        assert_eq!(provider.provider_name(), "openai");
        assert_eq!(provider.model_name(), "gpt-4o-mini");
        let usage = provider.get_token_usage().await;
        assert_eq!(usage.total_tokens, 0);
    }

    #[tokio::test]
    async fn test_gemini_provider_creation() {
        let provider = GeminiProvider::new("test-key".into(), "gemini-2.0-flash".into());
        assert_eq!(provider.provider_name(), "gemini");
        assert_eq!(provider.model_name(), "gemini-2.0-flash");
        let usage = provider.get_token_usage().await;
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_provider_kind_parsing() {
        assert_eq!(LlmProviderKind::from_str_loose("anthropic"), LlmProviderKind::Anthropic);
        assert_eq!(LlmProviderKind::from_str_loose("openai"), LlmProviderKind::OpenAi);
        assert_eq!(LlmProviderKind::from_str_loose("OpenAI"), LlmProviderKind::OpenAi);
        assert_eq!(LlmProviderKind::from_str_loose("open-ai"), LlmProviderKind::OpenAi);
        assert_eq!(LlmProviderKind::from_str_loose("gemini"), LlmProviderKind::Gemini);
        assert_eq!(LlmProviderKind::from_str_loose("google"), LlmProviderKind::Gemini);
        assert_eq!(LlmProviderKind::from_str_loose("Google-AI"), LlmProviderKind::Gemini);
        assert_eq!(LlmProviderKind::from_str_loose("unknown"), LlmProviderKind::Anthropic); // default
    }

    #[test]
    fn test_backward_compat_type_alias() {
        // AnthropicClient should still work as a type
        let _client: AnthropicClient = AnthropicClient::new("key".into(), "model".into());
    }

    // ── Grounding Tests ──

    #[test]
    fn test_grounding_config_defaults() {
        let config = memory_common::GroundingConfig::default();
        assert!(config.enable_citations);
        assert!(!config.enable_verification);
        assert_eq!(config.verification_model, "same");
        assert_eq!(config.min_verification_score, 0.5);
    }

    #[test]
    fn test_grounding_config_serde_roundtrip() {
        let config = memory_common::GroundingConfig {
            enable_citations: true,
            enable_verification: true,
            verification_model: "claude-haiku-4-5-20251001".to_string(),
            min_verification_score: 0.7,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: memory_common::GroundingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enable_verification, true);
        assert_eq!(parsed.min_verification_score, 0.7);
    }

    #[test]
    fn test_verification_status_as_str() {
        use memory_common::VerificationStatus;
        assert_eq!(VerificationStatus::Unverified.as_str(), "unverified");
        assert_eq!(VerificationStatus::Verified.as_str(), "verified");
        assert_eq!(VerificationStatus::Contested.as_str(), "contested");
        assert_eq!(VerificationStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_extracted_memory_with_source_quote() {
        let mem = memory_common::ExtractedMemory {
            content: "User prefers Rust".to_string(),
            kind: "preference".to_string(),
            confidence: 0.95,
            importance: 0.8,
            tags: vec!["tech".into()],
            source_quote: Some("I really prefer Rust".to_string()),
            source_offset_start: Some(10),
            source_offset_end: Some(30),
        };
        let json = serde_json::to_string(&mem).unwrap();
        assert!(json.contains("source_quote"));
        assert!(json.contains("I really prefer Rust"));
        let back: memory_common::ExtractedMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_quote.as_deref(), Some("I really prefer Rust"));
        assert_eq!(back.source_offset_start, Some(10));
    }

    #[test]
    fn test_extracted_memory_without_source_quote_compat() {
        // Old JSON without source_quote fields should still deserialize
        let json = r#"{
            "content": "User likes Go",
            "kind": "preference",
            "confidence": 0.8,
            "importance": 0.5,
            "tags": ["lang"]
        }"#;
        let mem: memory_common::ExtractedMemory = serde_json::from_str(json).unwrap();
        assert_eq!(mem.content, "User likes Go");
        assert!(mem.source_quote.is_none());
        assert!(mem.source_offset_start.is_none());
    }

    #[test]
    fn test_verification_prompt_generation() {
        use crate::prompt::build_verification_prompt;
        let memories = vec![memory_common::ExtractedMemory {
            content: "Alex works at TechCorp".to_string(),
            kind: "Fact".to_string(),
            confidence: 0.9,
            importance: 0.7,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }];
        let prompt = build_verification_prompt(&memories, "Alex told me he works at TechCorp as a senior engineer.");
        assert!(prompt.contains("Alex works at TechCorp"));
        assert!(prompt.contains("Alex told me he works at TechCorp"));
        assert!(prompt.contains("verifications"));
    }

    #[test]
    fn test_parse_verification_response() {
        use crate::prompt::parse_verification_response;
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "verified", "score": 0.95, "reasoning": "Directly stated"},
            {"memory_index": 1, "status": "contested", "score": 0.3, "reasoning": "Not fully supported"}
        ]}"#;
        let results = parse_verification_response(json).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, memory_common::VerificationStatus::Verified);
        assert_eq!(results[0].score, 0.95);
        assert_eq!(results[1].status, memory_common::VerificationStatus::Contested);
    }
}
