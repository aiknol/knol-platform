//! Shared types used across all LLM providers.

use std::time::Duration;

/// Tracks API usage statistics (tokens consumed).
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
    }
}

/// Configuration for batch extraction.
pub struct BatchExtractionConfig {
    pub max_parallel_requests: usize,
    pub timeout_per_request: Duration,
}

impl Default for BatchExtractionConfig {
    fn default() -> Self {
        Self {
            max_parallel_requests: 5,
            timeout_per_request: Duration::from_secs(30),
        }
    }
}

/// A single message to extract memories from (batch API).
pub struct ExtractionMessage {
    pub content: String,
    pub role: String,
}

/// Which LLM provider to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmProviderKind {
    Anthropic,
    OpenAi,
    Gemini,
}

impl LlmProviderKind {
    /// Parse from a string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" | "open_ai" | "open-ai" => Self::OpenAi,
            "gemini" | "google" | "google-ai" | "google_ai" => Self::Gemini,
            _ => Self::Anthropic, // default
        }
    }
}
