//! Content templates ported from marketing/engine/generate.js.
//!
//! Each template category is a `&[&str]` array of variants.
//! The generator picks one at random for each publish cycle.

// ─── Tweet templates ───────────────────────────────────────────

pub const TWEET_LAUNCH: &[&str] = &[
    "Knol gives your AI app long-term memory in one line of code. Open-source, Rust-powered, sub-millisecond lookups. Try it: https://github.com/aiknol/knol",
    "Your LLM forgets everything after each session. Knol fixes that — persistent memory for AI apps, built in Rust. https://aiknol.com",
    "We built Knol so AI agents can actually remember. Open-source memory layer, works with any LLM, deploys in 5 minutes. https://aiknol.com",
    "AI without memory is like a notebook you burn after every page. Knol gives your AI apps persistent, searchable memory. https://github.com/aiknol/knol",
];

pub const TWEET_TECHNICAL: &[&str] = &[
    "Knol's memory layer: sub-ms reads via Redis L1, PostgreSQL L2, HNSW vector search. All in Rust with zero-copy serialization. Benchmarks: https://aiknol.com/benchmarks",
    "How Knol handles 100K memories per user: tiered caching (Redis → Postgres → S3), automatic importance scoring, and background consolidation. Thread 🧵",
    "Knol's architecture: Axum for HTTP, SQLx for async Postgres, Redis for hot cache, HNSW for semantic search. All open-source. https://github.com/aiknol/knol",
];

pub const TWEET_COMPARISON: &[&str] = &[
    "Vector DBs store embeddings. Knol stores *memories* — with context, importance, relationships, and time decay. Different problem, better solution. https://aiknol.com",
    "Compared Knol vs rolling your own memory with pgvector: 10x less code, 3x faster retrieval, built-in importance scoring. https://aiknol.com/benchmarks",
];

// ─── LinkedIn templates ────────────────────────────────────────

pub const LINKEDIN_LAUNCH: &[&str] = &[
    "Excited to share Knol — an open-source memory layer for AI applications.\n\nThe problem: LLMs have no persistent memory. Every conversation starts from scratch.\n\nKnol solves this with:\n• Sub-millisecond memory retrieval\n• Automatic importance scoring\n• Semantic search across memories\n• Built in Rust for reliability\n\nWhether you're building AI agents, chatbots, or copilots — Knol gives them the ability to learn and remember.\n\nOpen-source and ready to use: https://aiknol.com",
];

pub const LINKEDIN_TECHNICAL: &[&str] = &[
    "Technical deep-dive: How we built Knol's memory architecture in Rust.\n\nThe challenge: AI apps need memory that's fast enough for real-time use, smart enough to prioritize what matters, and reliable enough for production.\n\nOur approach:\n1. Tiered storage: Redis (hot) → PostgreSQL (warm) → S3 (cold)\n2. HNSW vectors for semantic similarity\n3. Importance scoring with time decay\n4. Zero-copy serialization for minimal overhead\n\nResult: Sub-millisecond lookups at scale, with automatic memory management.\n\nFull architecture writeup: https://aiknol.com/architecture",
];

// ─── Reddit templates ──────────────────────────────────────────

pub const REDDIT_RUST: &[&str] = &[
    "Show r/rust: Knol — an open-source memory layer for AI apps, built entirely in Rust\n\nWe've been building Knol to solve persistent memory for LLM applications. The entire stack is Rust: Axum for HTTP, SQLx for async Postgres, custom HNSW implementation for vector search.\n\nKey design decisions:\n- Zero-copy deserialization with serde\n- Tiered caching (Redis L1 → Postgres L2)\n- Tokio for async everywhere\n- 207 unit tests, full CI/CD\n\nWould love feedback from the community: https://github.com/aiknol/knol",
];

pub const REDDIT_ML: &[&str] = &[
    "Knol: Open-source long-term memory for AI agents and LLM apps\n\nWe kept running into the same problem — AI agents that forget everything between sessions. Existing solutions are either too complex (full knowledge graphs) or too simple (dump everything into a vector DB).\n\nKnol sits in between: structured memory with importance scoring, semantic search, time decay, and sub-ms retrieval.\n\nUse cases we're seeing:\n- AI assistants that remember user preferences\n- Coding agents that learn codebase patterns\n- Customer support bots with interaction history\n\nOpen-source (MIT): https://github.com/aiknol/knol",
];

// ─── Dev.to templates ──────────────────────────────────────────

pub const DEVTO_TUTORIAL: &[&str] = &[
    "# Building AI Apps with Persistent Memory Using Knol\n\nEvery AI application eventually hits the same wall: your LLM has no memory. Users repeat themselves, context is lost, and the experience suffers.\n\nIn this tutorial, I'll show you how to add persistent memory to any AI app using Knol — an open-source memory layer built in Rust.\n\n## What You'll Build\n\nA simple AI assistant that remembers:\n- User preferences\n- Previous conversations\n- Important facts mentioned by the user\n\n## Prerequisites\n\n- Docker (for Knol services)\n- Python 3.8+ or Node.js 18+\n- An OpenAI API key (or any LLM provider)\n\n## Step 1: Start Knol\n\n```bash\ndocker compose up -d\n```\n\nThis starts the memory service on port 8080.\n\n## Step 2: Store a Memory\n\n```python\nimport requests\n\nrequests.post('http://localhost:8080/v1/memories', json={\n    'user_id': 'user_123',\n    'content': 'User prefers dark mode',\n    'importance': 0.8\n})\n```\n\n## Step 3: Retrieve Relevant Memories\n\n```python\nresponse = requests.post('http://localhost:8080/v1/memories/search', json={\n    'user_id': 'user_123',\n    'query': 'What are the user preferences?',\n    'limit': 5\n})\nmemories = response.json()\n```\n\nThe search uses semantic similarity — you don't need exact keyword matches.\n\n## Next Steps\n\nCheck out the full documentation at https://aiknol.com/docs for advanced features like importance scoring, memory consolidation, and multi-user support.",
];

// ─── Hacker News templates ─────────────────────────────────────

pub const HN_LAUNCH: &[&str] = &[
    "Show HN: Knol – Open-source memory layer for AI applications (Rust)",
    "Show HN: We built persistent memory for LLM apps in Rust",
];

// ─── Blog templates ────────────────────────────────────────────

pub const BLOG_LAUNCH: &[&str] = &[
    "# Introducing Knol: Persistent Memory for AI Applications\n\nToday we're open-sourcing Knol, a memory layer designed specifically for AI applications.\n\n## The Problem\n\nLarge Language Models are stateless. Every conversation starts from zero. Users repeat themselves. Context is lost. The experience degrades.\n\nDevelopers work around this by stuffing conversation history into prompts, but that's expensive, slow, and doesn't scale.\n\n## Our Solution\n\nKnol provides structured, searchable, persistent memory for any AI application. Store what matters, retrieve it when needed, and let your AI actually learn from interactions.\n\n## Key Features\n\n- **Sub-millisecond retrieval** — Redis-backed hot cache\n- **Semantic search** — Find memories by meaning, not just keywords\n- **Importance scoring** — Automatically prioritize what matters\n- **Time decay** — Old, unused memories fade naturally\n- **Multi-tenant** — Isolated memory per user/session\n\n## Architecture\n\nBuilt entirely in Rust for reliability and performance. The stack: Axum (HTTP), SQLx (PostgreSQL), Redis (caching), custom HNSW (vectors).\n\n## Get Started\n\n```bash\ndocker compose up -d\ncurl http://localhost:8080/health\n```\n\nThat's it. Full docs at https://aiknol.com/docs.",
];

pub const BLOG_TECHNICAL: &[&str] = &[
    "# How Knol Achieves Sub-Millisecond Memory Retrieval\n\nPerformance is critical for memory systems. If retrieval adds noticeable latency to your AI app, users will notice.\n\nHere's how Knol achieves sub-millisecond reads at scale.\n\n## Tiered Storage\n\nKnol uses a three-tier storage architecture:\n\n1. **L1 (Redis)**: Hot memories — recently accessed, high importance\n2. **L2 (PostgreSQL)**: Warm memories — all active memories with full indexing\n3. **L3 (S3)**: Cold storage — archived memories for long-term retention\n\n## Read Path\n\n1. Check Redis (50μs average)\n2. If miss, query PostgreSQL with prepared statements (2ms average)\n3. Promote to Redis on access (background task)\n\n## Write Path\n\n1. Write to PostgreSQL (primary source of truth)\n2. Async cache-aside to Redis\n3. Background importance scoring\n4. Periodic consolidation of related memories\n\n## Benchmarks\n\n| Operation | p50 | p99 |\n|-----------|-----|-----|\n| Read (cached) | 0.05ms | 0.2ms |\n| Read (uncached) | 1.8ms | 5ms |\n| Write | 2.1ms | 8ms |\n| Search (semantic) | 3.2ms | 12ms |",
    "# Building a Rust Microservice Architecture: Lessons from Knol\n\nKnol runs as 9 microservices, all in Rust. Here's what we learned.\n\n## Workspace Organization\n\nWe use Cargo workspaces with two sub-workspaces: `knol-oss` (open-source core) and `knol-enterprise` (commercial features).\n\n## Shared Patterns\n\nEvery service follows the same pattern: Axum for HTTP, SQLx for database, tracing for observability. Shared code lives in `memory-common` and `memory-db` crates.\n\n## Error Handling\n\nWe use `thiserror` for typed errors with `IntoResponse` implementations. Every error maps to an appropriate HTTP status code.\n\n## Testing\n\n207 unit tests, 180 integration tests. Tests run against real PostgreSQL and Redis instances in CI.\n\n## Deployment\n\nDocker Compose with multi-stage builds. Each service gets its own container with memory limits (96-256MB). Total resource usage: ~2GB RAM, 2 vCPUs.",
    "# Semantic Search Without a Vector Database\n\nYou don't always need a dedicated vector database. Here's how Knol implements semantic search with PostgreSQL and a custom HNSW index.\n\n## The Approach\n\nStore embeddings as PostgreSQL arrays. Build an in-process HNSW index for fast approximate nearest neighbor search. Use PostgreSQL for persistence, Rust for speed.\n\n## Why Not pgvector?\n\nWe started with pgvector but needed more control over the index parameters and wanted to avoid the overhead of a PostgreSQL extension in some deployment environments.\n\n## Implementation\n\nThe HNSW implementation lives in the `memory-index` crate. It supports:\n- Configurable M and efConstruction parameters\n- Cosine similarity and Euclidean distance\n- Incremental index updates\n- Serialization for persistence",
    "# Rate Limiting Done Right: A Rust Implementation\n\nRate limiting seems simple until you need it to be correct. Here's how we built Knol's marketing service rate limiter.\n\n## Requirements\n\n- Multiple time windows per channel (daily, monthly, per-minute)\n- Atomic check-and-increment\n- 90% safety margin on all limits\n- Survives service restarts\n\n## Sliding Window Algorithm\n\nWe use a sliding window counter per channel per time window. The key insight: check ALL windows before incrementing ANY of them.\n\n## Implementation\n\n```rust\npub async fn check_and_increment(&self, channel: &str) -> Result<bool, MarketingError> {\n    // 1. Read all windows for this channel\n    // 2. If ANY window is at limit, return false\n    // 3. Increment ALL windows atomically\n    // 4. Return true\n}\n```\n\nThis prevents the race condition where you increment one window but get blocked by another.",
];

// ─── Email templates ───────────────────────────────────────────

pub const EMAIL_WELCOME: &[&str] = &[
    "<h2>Welcome to Knol</h2>\n<p>Thanks for your interest in Knol — the open-source memory layer for AI applications.</p>\n<p>Here's what you can do next:</p>\n<ul>\n<li><a href=\"https://aiknol.com/docs\">Read the documentation</a></li>\n<li><a href=\"https://github.com/aiknol/knol\">Star us on GitHub</a></li>\n<li><a href=\"https://aiknol.com/quickstart\">5-minute quickstart guide</a></li>\n</ul>\n<p>Questions? Reply to this email — a human reads every one.</p>",
];

pub const EMAIL_WEEKLY: &[&str] = &[
    "<h2>This Week at Knol</h2>\n<p>Here's what's new in the Knol ecosystem this week:</p>\n<ul>\n<li><strong>New Release</strong>: Check the latest version for performance improvements and bug fixes</li>\n<li><strong>Community</strong>: Growing discussions on Reddit and GitHub</li>\n<li><strong>Tutorial</strong>: New guide on integrating Knol with popular AI frameworks</li>\n</ul>\n<p>As always, we'd love your feedback. Star us on <a href=\"https://github.com/aiknol/knol\">GitHub</a> if you haven't already!</p>",
];

/// All template categories mapped by name for easy lookup.
pub fn get_templates(category: &str) -> Option<&'static [&'static str]> {
    match category {
        "tweet_launch" => Some(TWEET_LAUNCH),
        "tweet_technical" => Some(TWEET_TECHNICAL),
        "tweet_comparison" => Some(TWEET_COMPARISON),
        "linkedin_launch" => Some(LINKEDIN_LAUNCH),
        "linkedin_technical" => Some(LINKEDIN_TECHNICAL),
        "reddit_rust" => Some(REDDIT_RUST),
        "reddit_ml" => Some(REDDIT_ML),
        "devto_tutorial" => Some(DEVTO_TUTORIAL),
        "hn_launch" => Some(HN_LAUNCH),
        "blog_launch" => Some(BLOG_LAUNCH),
        "blog_technical" => Some(BLOG_TECHNICAL),
        "email_welcome" => Some(EMAIL_WELCOME),
        "email_weekly" => Some(EMAIL_WEEKLY),
        _ => None,
    }
}

/// List all available template categories.
#[cfg(test)]
pub fn all_categories() -> &'static [&'static str] {
    &[
        "tweet_launch",
        "tweet_technical",
        "tweet_comparison",
        "linkedin_launch",
        "linkedin_technical",
        "reddit_rust",
        "reddit_ml",
        "devto_tutorial",
        "hn_launch",
        "blog_launch",
        "blog_technical",
        "email_welcome",
        "email_weekly",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_categories_resolve() {
        for cat in all_categories() {
            assert!(get_templates(cat).is_some(), "Category '{}' not found", cat);
            assert!(
                !get_templates(cat).unwrap().is_empty(),
                "Category '{}' is empty",
                cat
            );
        }
    }

    #[test]
    fn tweet_templates_under_280_chars() {
        for templates in &[TWEET_LAUNCH, TWEET_TECHNICAL, TWEET_COMPARISON] {
            for t in *templates {
                assert!(
                    t.len() <= 280,
                    "Tweet template exceeds 280 chars ({} chars): {}...",
                    t.len(),
                    &t[..50]
                );
            }
        }
    }
}
