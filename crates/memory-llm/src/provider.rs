//! LLM provider trait — the abstraction layer for switching between providers.
//!
//! Any LLM backend (Anthropic, OpenAI, Ollama, etc.) implements [`LlmProvider`]
//! so the rest of the system can call `extract_memories` without caring which model
//! is behind it.

use crate::error::LlmError;
use crate::types::TokenUsage;
use async_trait::async_trait;
use memory_common::{ExtractedMemory, ExtractionResult, MemoryVerification};

/// Options that callers can pass to tune extraction behavior per-call.
#[derive(Debug, Clone, Default)]
pub struct ExtractionOptions {
    /// Override the default max_output_tokens for this call.
    /// When `None`, the provider uses its compiled default (4096).
    pub max_output_tokens: Option<u32>,
    /// When true, ask the LLM to include inline verification fields
    /// (`grounded`, `ground_score`) in the extraction JSON, eliminating
    /// the need for a separate verification call.
    pub inline_verification: bool,
}

/// Trait that every LLM provider must implement.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Human-readable provider name (e.g., "anthropic", "openai", "ollama").
    fn provider_name(&self) -> &str;

    /// The model identifier currently in use.
    fn model_name(&self) -> &str;

    /// Extract memories, entities, and relationships from text.
    async fn extract_memories(
        &self,
        content: &str,
        role: &str,
        existing_entities: &[String],
    ) -> Result<ExtractionResult, LlmError>;

    /// Extract memories with per-call tuning options (dynamic token budget,
    /// inline verification, etc.).
    ///
    /// Default delegates to [`extract_memories`] ignoring options.
    async fn extract_memories_with_options(
        &self,
        content: &str,
        role: &str,
        existing_entities: &[String],
        _options: &ExtractionOptions,
    ) -> Result<ExtractionResult, LlmError> {
        self.extract_memories(content, role, existing_entities)
            .await
    }

    /// Verify extracted memories against their source content (factual grounding).
    ///
    /// Makes a second LLM call asking whether the source content supports each
    /// extracted memory. Returns a verification result per memory.
    ///
    /// Default implementation returns all memories as unverified.
    async fn verify_memories(
        &self,
        _memories: &[ExtractedMemory],
        _source_content: &str,
    ) -> Result<Vec<MemoryVerification>, LlmError> {
        Ok(Vec::new())
    }

    /// Get accumulated token usage statistics.
    async fn get_token_usage(&self) -> TokenUsage;

    /// Reset token usage statistics.
    async fn reset_token_usage(&self);
}
