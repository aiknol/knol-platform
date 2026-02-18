# Knol LLM Optimization Strategy

## Current State Analysis

Every message written to Knol triggers this flow:

```
Client → service-write → NATS → service-graph → LLM extraction (Call #1)
                                                → LLM verification (Call #2, optional)
```

**Cost per message today:**

| Step | Input Tokens | Output Tokens | Notes |
|------|-------------|---------------|-------|
| Extraction | ~800 (system prompt) + content | ~200-1500 | Always runs |
| Verification | ~400 (prompt) + content + memories | ~100-500 | Optional 2nd call |
| Entity context | +50 per known entity | — | Up to 100 entities loaded |

**Key problems identified:**

1. **No LLM response caching** — identical content triggers fresh extraction every time
2. **System prompt sent every call** — ~800 tokens repeated on every single request
3. **No content triage** — "Hi" and "I work at Google as a senior ML engineer leading the TPU compiler team since 2019" both trigger full extraction
4. **Entity context is unbounded** — loads up to 100 entity names into every prompt
5. **Verification doubles cost** — sends content + extracted memories as a second full call
6. **No batching** — `BatchExtractionConfig` exists but is unused; each NATS message = 1 LLM call
7. **Fixed max_output_tokens = 4096** — even for short content that will produce ~100 tokens of output

---

## Strategy: 7 Layers of Optimization

### Layer 1: Content Triage (Skip LLM entirely)

**Impact: Eliminate 30-60% of LLM calls**

Before calling the LLM, classify content locally using simple heuristics:

```
Content → word_count < 3?           → SKIP (greetings, "ok", "thanks")
        → is_duplicate_hash?        → SKIP (already extracted)
        → no_extractable_signals?   → SKIP (no nouns/verbs/entities detected)
        → word_count < 15?          → LIGHT extraction (smaller output budget)
        → else                      → FULL extraction
```

**Implementation:** Add a `triage_content()` function in `service-graph` that runs *before* the LLM call. Uses the existing `content_hash` index (`idx_episodes_hash`) to check if we've already extracted from identical content.

**Rules:**
- Content < 3 words → skip (e.g., "hi", "ok thanks", "yes")
- Content matches existing content_hash in memories table → skip
- Content is purely questions with no assertions → skip (e.g., "what time is it?")
- Content < 15 words → use reduced `max_output_tokens` (1024 instead of 4096)

### Layer 2: LLM Response Cache (Redis)

**Impact: Eliminate 20-40% of LLM calls for recurring patterns**

Cache extraction results in Redis keyed by `SHA256(content + role + entity_context_hash)`.

```
cache_key = SHA256(content + "|" + role + "|" + sorted_entities.join(","))
TTL = 1 hour (configurable via admin: llm.cache_ttl_secs)
```

**Why this works:** In production, many users send similar messages. A company with 50 users will have many overlapping extractions ("I work at CompanyX", onboarding messages, etc.). The entity context hash ensures cache invalidation when the knowledge graph changes significantly.

**Implementation:** Add to `memory-cache` crate. Check cache before LLM call in each provider's `extract_memories`.

### Layer 3: Prompt Compression

**Impact: Reduce input tokens by 30-40% per call**

The system prompt is ~800 tokens and sent identically every time. Optimizations:

**a) Compact system prompt:** Rewrite the extraction prompt to be more concise. Remove the verbose JSON schema example (the LLM already knows JSON) and replace with a terse spec:

```
Current: ~2500 characters, ~800 tokens
Target:  ~1200 characters, ~400 tokens
Savings: ~400 tokens per call
```

**b) Entity context pruning:** Currently loads up to 100 entity names. Instead:
- Only send entities relevant to the content (substring match against content)
- Cap at 20 most relevant entities instead of 100
- Use compact format: `"Alice,Bob,TechCorp"` instead of one per line

**c) Dynamic max_output_tokens:** Instead of always 4096:
- Short content (< 50 words): `max_output_tokens = 1024`
- Medium content (50-200 words): `max_output_tokens = 2048`
- Long content (200+ words): `max_output_tokens = 4096`

This reduces output token billing on providers that charge per-output-token.

### Layer 4: Combined Extraction + Verification (Single Call)

**Impact: Eliminate the verification call entirely (50% fewer calls when verification is enabled)**

Instead of two separate calls, merge verification into the extraction prompt:

```
Current flow:  Extract → Verify (2 LLM calls)
Proposed flow: Extract + self-verify (1 LLM call)
```

Add to the extraction prompt:
```
For each memory, also include:
  "grounded": true/false  (is this directly stated in the input?)
  "ground_score": 0.0-1.0 (how confident are you this is factually in the source?)
```

The LLM already has the source content when extracting — asking it to self-assess grounding in the same call is nearly free. The separate verification call exists because it was designed as a "second opinion" but in practice the same model reviewing its own output in a separate call provides minimal additional signal vs. inline assessment.

**Trade-off:** Slightly less rigorous than a separate verification pass. Offer as configurable: `grounding.inline_verification = true` (default) vs. `false` (legacy 2-call mode).

### Layer 5: Smart Batching

**Impact: Reduce per-message overhead by batching N messages into 1 LLM call**

The NATS consumer already fetches up to 10 messages at a time (`max_messages(10)`). Instead of processing each independently:

```
Current:  10 messages → 10 LLM calls
Proposed: 10 messages → 1-3 LLM calls (batched by tenant)
```

**Implementation:**
- Group messages by tenant_id (they share entity context)
- Concatenate content with separators: `[MSG 1]: content\n[MSG 2]: content\n...`
- Ask LLM to return extraction per message index
- Split results back to individual messages

**Constraints:**
- Max batch size: 5 messages or 4000 input tokens (whichever comes first)
- Only batch messages from same tenant (shared entity context)
- Fall back to single-message mode if batch parsing fails

### Layer 6: Tiered Model Selection

**Impact: 40-70% cost reduction for simple content**

Not all content needs the same model. Use a lightweight classifier to route:

```
Simple content  → gemini-2.0-flash-lite / gpt-4o-mini  (cheapest)
Medium content  → gemini-2.0-flash / claude-haiku       (default)
Complex content → gemini-2.0-pro / claude-sonnet         (premium)
```

**Classification heuristic (no LLM needed):**
- **Simple:** < 30 words, no entities mentioned, single topic
- **Complex:** > 200 words, multiple entities, technical content, multi-topic
- **Medium:** everything else

**Admin config:** `llm.enable_tiered_models = true`, `llm.simple_model`, `llm.complex_model`

### Layer 7: Incremental Entity Context

**Impact: Reduce entity context tokens from ~500 to ~50 per call**

Currently loads 100 entity names blindly. Instead:

```
Current:  SELECT name FROM entities WHERE tenant_id = $1 LIMIT 100
Proposed: SELECT name FROM entities WHERE tenant_id = $1
          AND name ILIKE ANY($2)  -- only entities mentioned in content
          LIMIT 20
```

Extract candidate entity mentions from content using simple NLP (capitalized words, known entity patterns), then only fetch matching entities from the DB. This turns a 100-entity context into a 5-10 entity context.

---

## Implementation Priority

| Priority | Layer | Effort | Impact | Dependency |
|----------|-------|--------|--------|------------|
| **P0** | 1. Content Triage | Small | High (30-60% fewer calls) | None |
| **P0** | 3. Prompt Compression | Small | Medium (400 tokens/call) | None |
| **P1** | 4. Inline Verification | Medium | High (50% fewer calls with verification) | None |
| **P1** | 7. Entity Context Pruning | Small | Medium (reduce prompt size) | None |
| **P2** | 2. Redis Cache | Medium | Medium (20-40% cache hits) | memory-cache |
| **P2** | 6. Tiered Models | Medium | High (40-70% cost on simple) | Admin config |
| **P3** | 5. Smart Batching | Large | Medium (amortize overhead) | NATS consumer refactor |

---

## Projected Savings

Assuming a workload of 1000 messages/day with verification enabled:

| Metric | Current | After Optimization | Reduction |
|--------|---------|-------------------|-----------|
| LLM calls/day | 2000 (1000 extract + 1000 verify) | ~500 | **75%** |
| Input tokens/day | ~1.6M | ~400K | **75%** |
| Output tokens/day | ~800K | ~250K | **69%** |
| Avg latency/message | ~2s (2 calls) | ~0.8s (1 call, cached) | **60%** |

**Breakdown:**
- Content triage skips ~40% of messages → 1200 calls → 600 after inline verification
- Cache hits on ~15% of remaining → ~510 calls
- Prompt compression saves ~400 tokens × 510 calls = 204K tokens
- Entity pruning saves ~200 tokens × 510 calls = 102K tokens

---

## New Admin Config Keys

| Key | Default | Description |
|-----|---------|-------------|
| `llm.enable_triage` | `true` | Skip LLM for trivial content |
| `llm.triage_min_words` | `3` | Minimum words to trigger extraction |
| `llm.enable_cache` | `true` | Cache LLM responses in Redis |
| `llm.cache_ttl_secs` | `3600` | Cache TTL in seconds |
| `llm.enable_tiered_models` | `false` | Route by content complexity |
| `llm.simple_model` | `""` | Model for simple content |
| `llm.complex_model` | `""` | Model for complex content |
| `llm.max_entity_context` | `20` | Max entities in prompt |
| `llm.dynamic_output_tokens` | `true` | Scale max_output_tokens by content |
| `grounding.inline_verification` | `true` | Merge verification into extraction |
