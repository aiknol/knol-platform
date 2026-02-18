//! Shared extraction prompt and validation logic.
//!
//! The system prompt is the same regardless of provider — it instructs the model
//! to return structured JSON with memories, entities, and relationships.

use memory_common::{ExtractionResult, ExtractedMemory, MemoryVerification, VerificationStatus};
use serde::Deserialize;
use tracing::warn;
use crate::error::LlmError;

/// Build the extraction system prompt, optionally including known entity context.
pub fn build_system_prompt(existing_entities: &[String]) -> String {
    build_system_prompt_ext(existing_entities, false)
}

/// Build the extraction system prompt with optional inline verification.
///
/// When `inline_verification` is true, each memory object also includes
/// `"grounded": bool` and `"ground_score": 0.0-1.0` — eliminating the
/// need for a second LLM verification call.
pub fn build_system_prompt_ext(existing_entities: &[String], inline_verification: bool) -> String {
    let entity_context = if existing_entities.is_empty() {
        String::new()
    } else {
        format!(
            "\nKnown entities (reuse if matching): {}",
            existing_entities.join(", ")
        )
    };

    let verification_fields = if inline_verification {
        r#","grounded":true,"ground_score":0.95"#
    } else {
        ""
    };

    let verification_rule = if inline_verification {
        "\n- For each memory, set grounded=true/false and ground_score (0.0-1.0) indicating how well the source supports it"
    } else {
        ""
    };

    format!(
        r#"Extract memories from conversation input. Return ONLY valid JSON (no markdown/code blocks).

CATEGORIES: Preference, Fact, Task, Event, Relationship, TemporalChange
ENTITY TYPES: person, organization, product, concept, location, event, time

RULES:
- Only extract explicit statements, not inferences
- Confidence: explicit=0.9-1.0, stated=0.7-0.9, implied=0.4-0.7
- Include source_quote (verbatim, <150 chars) with character offsets for each memory
- Assign tags for filtering (#work, #personal, etc.){verification_rule}
{entity_context}

JSON SCHEMA:
{{"memories":[{{"content":"str","memory_type":"Preference|Fact|Task|Event|Relationship|TemporalChange","classification":"optional","confidence":0.0-1.0,"tags":["str"],"source_quote":"verbatim","source_offset_start":0,"source_offset_end":0{verification_fields}}}],"entities":[{{"name":"str","entity_type":"person|organization|product|concept|location|event|time","description":"str"}}],"relationships":[{{"source":"str","target":"str","relation_type":"str","description":"str","weight":0.0-1.0}}]}}

Return empty arrays if no meaningful content."#
    )
}

/// Build the verification prompt for factual grounding.
///
/// Given a list of extracted memories and the original source content,
/// produces a prompt asking the LLM to verify each memory against the source.
pub fn build_verification_prompt(memories: &[ExtractedMemory], source_content: &str) -> String {
    let mut memory_list = String::new();
    for (i, mem) in memories.iter().enumerate() {
        memory_list.push_str(&format!(
            "  {}: \"{}\"\n",
            i, mem.content
        ));
    }

    format!(
        r#"You are a factual verification engine. Your job is to check whether each extracted memory
is actually supported by the source content.

SOURCE CONTENT:
---
{source_content}
---

EXTRACTED MEMORIES:
{memory_list}
For each memory (by index), determine:
1. Whether the source content supports or contradicts it
2. A confidence score (0.0-1.0) that the memory is factually grounded
3. Brief reasoning (1 sentence)

Return ONLY valid JSON (no markdown, no code blocks):

{{
  "verifications": [
    {{
      "memory_index": 0,
      "status": "verified|contested|failed",
      "score": 0.0-1.0,
      "reasoning": "string"
    }}
  ]
}}"#
    )
}

/// Validate the extraction result structure and content.
pub fn validate_extraction_result(result: &ExtractionResult) -> Result<(), LlmError> {
    for memory in &result.memories {
        if memory.content.is_empty() {
            return Err(LlmError::Parse(
                "Memory content cannot be empty".to_string(),
            ));
        }
        if memory.kind.is_empty() {
            return Err(LlmError::Parse(
                "Memory kind cannot be empty".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&memory.confidence) {
            return Err(LlmError::Parse(format!(
                "Invalid confidence score: {}",
                memory.confidence
            )));
        }
    }

    for entity in &result.entities {
        if entity.name.is_empty() {
            return Err(LlmError::Parse(
                "Entity name cannot be empty".to_string(),
            ));
        }
    }

    for rel in &result.relationships {
        if rel.source_entity.is_empty() || rel.target_entity.is_empty() {
            return Err(LlmError::Parse(
                "Relationship source and target cannot be empty".to_string(),
            ));
        }
        if let Some(w) = rel.weight {
            if !(0.0..=1.0).contains(&w) {
                return Err(LlmError::Parse(format!(
                    "Invalid relationship weight: {}",
                    w
                )));
            }
        }
    }

    Ok(())
}

/// Parse the LLM verification JSON response into `MemoryVerification` structs.
///
/// Shared by all providers — each sends a verification prompt and gets back JSON
/// matching the schema from [`build_verification_prompt`].
pub fn parse_verification_response(text: &str) -> Result<Vec<MemoryVerification>, LlmError> {
    #[derive(Deserialize)]
    struct VerificationResponse {
        verifications: Vec<VerificationItem>,
    }

    #[derive(Deserialize)]
    struct VerificationItem {
        memory_index: usize,
        status: String,
        score: f32,
        #[serde(default)]
        reasoning: String,
    }

    let parsed: VerificationResponse = serde_json::from_str(text).map_err(|e| {
        warn!("Verification parse error: {}. Raw: {}", e, &text[..text.len().min(500)]);
        LlmError::Parse(format!("Verification response parse error: {}", e))
    })?;

    Ok(parsed
        .verifications
        .into_iter()
        .map(|v| MemoryVerification {
            memory_index: v.memory_index,
            status: match v.status.as_str() {
                "verified" => VerificationStatus::Verified,
                "contested" => VerificationStatus::Contested,
                "failed" => VerificationStatus::Failed,
                _ => VerificationStatus::Unverified,
            },
            score: v.score.clamp(0.0, 1.0),
            reasoning: v.reasoning,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_common::{ExtractedEntity, ExtractedRelationship};

    fn make_memory(content: &str, kind: &str, confidence: f32) -> ExtractedMemory {
        ExtractedMemory {
            content: content.to_string(),
            kind: kind.to_string(),
            confidence,
            importance: 0.5,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }
    }

    fn make_entity(name: &str) -> ExtractedEntity {
        ExtractedEntity {
            name: name.to_string(),
            entity_type: "person".to_string(),
            summary: None,
            attributes: None,
        }
    }

    fn make_rel(source: &str, target: &str, weight: Option<f32>) -> ExtractedRelationship {
        ExtractedRelationship {
            source_entity: source.to_string(),
            target_entity: target.to_string(),
            rel_type: "knows".to_string(),
            properties: None,
            weight,
        }
    }

    // ── validate_extraction_result ──

    #[test]
    fn test_validate_valid_result() {
        let result = ExtractionResult {
            memories: vec![make_memory("User likes Rust", "Preference", 0.9)],
            entities: vec![make_entity("Rust")],
            relationships: vec![],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    #[test]
    fn test_validate_empty_result() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    #[test]
    fn test_validate_empty_memory_content() {
        let result = ExtractionResult {
            memories: vec![make_memory("", "Fact", 0.8)],
            entities: vec![],
            relationships: vec![],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("content cannot be empty"));
    }

    #[test]
    fn test_validate_empty_memory_kind() {
        let result = ExtractionResult {
            memories: vec![make_memory("content", "", 0.8)],
            entities: vec![],
            relationships: vec![],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("kind cannot be empty"));
    }

    #[test]
    fn test_validate_confidence_out_of_range_high() {
        let result = ExtractionResult {
            memories: vec![make_memory("content", "Fact", 1.5)],
            entities: vec![],
            relationships: vec![],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("Invalid confidence"));
    }

    #[test]
    fn test_validate_confidence_out_of_range_negative() {
        let result = ExtractionResult {
            memories: vec![make_memory("content", "Fact", -0.1)],
            entities: vec![],
            relationships: vec![],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("Invalid confidence"));
    }

    #[test]
    fn test_validate_confidence_boundary_zero() {
        let result = ExtractionResult {
            memories: vec![make_memory("content", "Fact", 0.0)],
            entities: vec![],
            relationships: vec![],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    #[test]
    fn test_validate_confidence_boundary_one() {
        let result = ExtractionResult {
            memories: vec![make_memory("content", "Fact", 1.0)],
            entities: vec![],
            relationships: vec![],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    #[test]
    fn test_validate_empty_entity_name() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![make_entity("")],
            relationships: vec![],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("Entity name cannot be empty"));
    }

    #[test]
    fn test_validate_empty_relationship_source() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("", "target", None)],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("source and target cannot be empty"));
    }

    #[test]
    fn test_validate_empty_relationship_target() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("source", "", None)],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("source and target cannot be empty"));
    }

    #[test]
    fn test_validate_relationship_weight_out_of_range() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("a", "b", Some(1.5))],
        };
        let err = validate_extraction_result(&result).unwrap_err();
        assert!(err.to_string().contains("Invalid relationship weight"));
    }

    #[test]
    fn test_validate_relationship_weight_negative() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("a", "b", Some(-0.1))],
        };
        assert!(validate_extraction_result(&result).is_err());
    }

    #[test]
    fn test_validate_relationship_weight_valid() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("a", "b", Some(0.8))],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    #[test]
    fn test_validate_relationship_no_weight() {
        let result = ExtractionResult {
            memories: vec![],
            entities: vec![],
            relationships: vec![make_rel("a", "b", None)],
        };
        assert!(validate_extraction_result(&result).is_ok());
    }

    // ── parse_verification_response ──

    #[test]
    fn test_parse_verification_valid() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "verified", "score": 0.95, "reasoning": "Exact match"},
            {"memory_index": 1, "status": "contested", "score": 0.4, "reasoning": "Partial match"}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].status, VerificationStatus::Verified);
        assert!((result[0].score - 0.95).abs() < f32::EPSILON);
        assert_eq!(result[1].status, VerificationStatus::Contested);
    }

    #[test]
    fn test_parse_verification_failed_status() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "failed", "score": 0.1, "reasoning": "Not found"}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert_eq!(result[0].status, VerificationStatus::Failed);
    }

    #[test]
    fn test_parse_verification_unknown_status_defaults_unverified() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "maybe", "score": 0.5, "reasoning": ""}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert_eq!(result[0].status, VerificationStatus::Unverified);
    }

    #[test]
    fn test_parse_verification_score_clamped_high() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "verified", "score": 1.5, "reasoning": ""}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert!((result[0].score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_verification_score_clamped_low() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "failed", "score": -0.5, "reasoning": ""}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert!((result[0].score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_verification_empty_list() {
        let json = r#"{"verifications": []}"#;
        let result = parse_verification_response(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_verification_invalid_json() {
        let result = parse_verification_response("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_verification_missing_reasoning_defaults() {
        let json = r#"{"verifications": [
            {"memory_index": 0, "status": "verified", "score": 0.9}
        ]}"#;
        let result = parse_verification_response(json).unwrap();
        assert_eq!(result[0].reasoning, "");
    }

    // ── build_system_prompt ──

    #[test]
    fn test_system_prompt_contains_categories() {
        let prompt = build_system_prompt(&[]);
        assert!(prompt.contains("Preference"));
        assert!(prompt.contains("Fact"));
        assert!(prompt.contains("Task"));
        assert!(prompt.contains("Event"));
        assert!(prompt.contains("Relationship"));
    }

    #[test]
    fn test_system_prompt_contains_entity_types() {
        let prompt = build_system_prompt(&[]);
        assert!(prompt.contains("person"));
        assert!(prompt.contains("organization"));
        assert!(prompt.contains("location"));
    }

    #[test]
    fn test_system_prompt_includes_existing_entities() {
        let entities = vec!["Alice".to_string(), "Acme Corp".to_string()];
        let prompt = build_system_prompt(&entities);
        assert!(prompt.contains("Alice"));
        assert!(prompt.contains("Acme Corp"));
    }

    #[test]
    fn test_system_prompt_no_existing_entities() {
        let prompt = build_system_prompt(&[]);
        // When no entities are provided, the "Known entities" section should be absent
        assert!(!prompt.contains("Known entities"));
    }

    #[test]
    fn test_system_prompt_contains_citation_grounding() {
        let prompt = build_system_prompt(&[]);
        assert!(prompt.contains("source_quote"));
        assert!(prompt.contains("source_offset_start"));
    }

    // ── build_verification_prompt ──

    #[test]
    fn test_verification_prompt_contains_memories() {
        let memories = vec![
            make_memory("User likes Rust", "Preference", 0.9),
            make_memory("Meeting at 3pm", "Event", 0.8),
        ];
        let prompt = build_verification_prompt(&memories, "Some conversation content");
        assert!(prompt.contains("User likes Rust"));
        assert!(prompt.contains("Meeting at 3pm"));
        assert!(prompt.contains("Some conversation content"));
    }

    #[test]
    fn test_verification_prompt_contains_source() {
        let memories = vec![make_memory("fact", "Fact", 0.9)];
        let source = "The original conversation text here.";
        let prompt = build_verification_prompt(&memories, source);
        assert!(prompt.contains(source));
    }
}
