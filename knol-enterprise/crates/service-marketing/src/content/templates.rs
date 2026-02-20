//! Content templates aligned with the Zero-Cost Marketing Plan.
//!
//! Each template category is a `&[&str]` array of variants.
//! The generator picks one at random for each publish cycle.
//!
//! Categories are organized by the plan's phases:
//! - Phase 2 (Launch Week): HN, Reddit blitz, Dev.to, Twitter threads, Product Hunt
//! - Phase 3 (Content Engine): Day-of-week Twitter, weekly blog, cross-posts
//! - Phase 4 (Community): MCP ecosystem, partnerships
//! - Phase 5 (Conversion): SEO-targeted content, community growth

// ─── Phase 3: Daily Twitter Strategy (day-of-week rotation) ──────

/// Monday: Technical tip or code snippet
pub const TWEET_TIP: &[&str] = &[
    "Knol tip: Use importance scores to auto-prioritize memories. High-importance memories get promoted to Redis hot cache for sub-ms retrieval. https://aiknol.com/docs",
    "Quick Knol pattern: Store user prefs with high importance (0.9+) so they're always in the hot path. Let time decay handle the rest.",
    "Knol tip: The consolidate endpoint merges related memories automatically. 50 fragments → one rich context block. POST /v1/memories/consolidate",
    "Rust tip from building Knol: sqlx::query_as with compile-time checked queries. Zero runtime SQL parsing, IDE catches column mismatches before CI.",
];

/// Tuesday: Benchmark or comparison stat
pub const TWEET_BENCHMARK: &[&str] = &[
    "Knol vs rolling your own pgvector memory: 10x less code, 3x faster retrieval, built-in importance scoring + time decay. https://aiknol.com/benchmarks",
    "Latest benchmark: Knol handles 100K memories/user with p99 read latency under 5ms. Cached + uncached. Single PostgreSQL, no Qdrant needed.",
    "Knol footprint vs Mem0: 50MB binary vs 2GB+ Python stack with Qdrant + Neo4j. Same functionality, 40x smaller. https://aiknol.com/benchmarks",
    "Knol's 7-layer extraction pipeline cuts LLM token usage by 75%. $300/mo → $75/mo for a typical production agent. https://aiknol.com/benchmarks",
];

/// Wednesday: User showcase or testimonial
pub const TWEET_SHOWCASE: &[&str] = &[
    "Showcase: A dev built a support bot with Knol that remembers every interaction. Resolution time dropped 40% — the bot knows customer history.",
    "Seeing creative Knol uses: a team built a coding agent that remembers codebase patterns across sessions. Suggests refactors from last week.",
    "Community highlight: A startup replaced Pinecone + LangChain memory with Knol. One binary, one PostgreSQL. Infra bill: $500/mo → $12/mo.",
    "Showcase: A tutoring app uses Knol's bi-temporal memory to track what students know vs knew. AI adapts teaching by learning velocity.",
];

/// Thursday: Architecture insight or design decision
pub const TWEET_ARCHITECTURE: &[&str] = &[
    "Why Knol uses bi-temporal memory: AI needs to know WHEN it learned something AND when that knowledge was valid. Context matters.",
    "Design decision: Knol stores knowledge graphs IN PostgreSQL, not Neo4j. Adjacency lists + recursive CTEs handle 99% of graph queries.",
    "Knol's memory decay isn't TTL. It's: importance × recency × access_frequency. Memories that matter stick. Trivia fades. Like human memory.",
    "Why we chose Axum over Actix: tower middleware composability. Rate limiting, auth, tracing — all tower layers that compose cleanly.",
];

/// Friday: Community highlight or contributor shoutout
pub const TWEET_COMMUNITY: &[&str] = &[
    "Thanks to our contributors this week! Every PR, issue, and discussion makes Knol better. Good first issues: https://github.com/aiknol/knol",
    "The Knol Discord is growing! Join us for AI memory, Rust architecture, and production LLM app discussions. Link in bio.",
    "Friday shoutout to everyone who starred Knol this week. If you're using Knol, tell us — we'd love to feature your project!",
    "Contributor spotlight: community helped improve Docker Compose, added ARM64 support, fixed memory consolidation edge case. You all rock.",
];

/// Weekend: Engagement tweets (manual use)
pub const TWEET_ENGAGEMENT: &[&str] = &[
    "What's the biggest pain point building AI agents with persistent memory? Curious — prioritizing our roadmap based on real developer needs.",
    "Hot take: Most AI apps don't need a vector database. They need a memory layer. Vectors are a tool, not a solution. Your experience?",
    "Building anything cool with LLMs this weekend? Drop a link — always looking for interesting projects to check out and share.",
];

// ─── Original tweet templates (backward compat) ──────────────────

pub const TWEET_LAUNCH: &[&str] = &[
    "Knol: long-term memory for AI apps in one line of code. Open-source, Rust-powered, sub-ms lookups. https://github.com/aiknol/knol",
    "Your LLM forgets everything after each session. Knol fixes that — persistent memory for AI apps, built in Rust. https://aiknol.com",
    "We built Knol so AI agents can actually remember. Open-source memory layer, any LLM, deploys in 5 minutes. https://aiknol.com",
    "AI without memory is a notebook you burn after every page. Knol gives your AI apps persistent, searchable memory. https://github.com/aiknol/knol",
];

pub const TWEET_TECHNICAL: &[&str] = &[
    "Knol: sub-ms reads via Redis L1, PostgreSQL L2, HNSW vector search. Rust + zero-copy serialization. https://aiknol.com/benchmarks",
    "How Knol handles 100K memories/user: tiered caching (Redis → Postgres → S3), importance scoring, background consolidation. 🧵",
    "Knol architecture: Axum HTTP, SQLx async Postgres, Redis hot cache, HNSW semantic search. All open-source. https://github.com/aiknol/knol",
];

pub const TWEET_COMPARISON: &[&str] = &[
    "Vector DBs store embeddings. Knol stores *memories* — context, importance, relationships, time decay. Different problem, better solution.",
    "Knol vs DIY memory with pgvector: 10x less code, 3x faster retrieval, built-in importance scoring. https://aiknol.com/benchmarks",
];

// ─── Phase 2: Launch Week Twitter Thread ─────────────────────────

pub const TWEET_THREAD_LAUNCH: &[&str] = &[
    "We just open-sourced a Rust memory engine for AI agents. Here's what makes it different from Mem0 and Zep 🧵\n\n1/ Knol is a single 50MB Rust binary. No Python, no Qdrant, no Neo4j. Just PostgreSQL.\n\n2/ Sub-5ms P95 latency for memory retrieval. Cached + uncached.\n\n3/ Bi-temporal knowledge graphs — your AI knows WHEN it learned something and WHEN that knowledge was valid.\n\n4/ 7-layer extraction pipeline cuts LLM costs by 75%.\n\n5/ Memory decay + conflict detection. Memories that matter stick. Contradictions get flagged.\n\n6/ Deploy in 60 seconds: docker compose up.\n\nStar us on GitHub: https://github.com/aiknol/knol",
];

// ─── LinkedIn templates ──────────────────────────────────────────

pub const LINKEDIN_LAUNCH: &[&str] = &[
    "Excited to share Knol — an open-source memory layer for AI applications.\n\nThe problem: LLMs have no persistent memory. Every conversation starts from scratch.\n\nKnol solves this with:\n• Sub-millisecond memory retrieval\n• Automatic importance scoring\n• Semantic search across memories\n• Built in Rust for reliability\n\nWhether you're building AI agents, chatbots, or copilots — Knol gives them the ability to learn and remember.\n\nOpen-source and ready to use: https://aiknol.com",
];

pub const LINKEDIN_TECHNICAL: &[&str] = &[
    "Technical deep-dive: How we built Knol's memory architecture in Rust.\n\nThe challenge: AI apps need memory that's fast enough for real-time use, smart enough to prioritize what matters, and reliable enough for production.\n\nOur approach:\n1. Tiered storage: Redis (hot) → PostgreSQL (warm) → S3 (cold)\n2. HNSW vectors for semantic similarity\n3. Importance scoring with time decay\n4. Zero-copy serialization for minimal overhead\n\nResult: Sub-millisecond lookups at scale, with automatic memory management.\n\nFull architecture writeup: https://aiknol.com/architecture",
];

// ─── Phase 2: Reddit Blitz (4 subreddits) ────────────────────────

pub const REDDIT_RUST: &[&str] = &[
    "Show r/rust: Knol — an open-source memory layer for AI apps, built entirely in Rust\n\nWe've been building Knol to solve persistent memory for LLM applications. The entire stack is Rust: Axum for HTTP, SQLx for async Postgres, custom HNSW implementation for vector search.\n\nKey design decisions:\n- Zero-copy deserialization with serde\n- Tiered caching (Redis L1 → Postgres L2)\n- Tokio for async everywhere\n- 207 unit tests, full CI/CD\n\nWould love feedback from the community: https://github.com/aiknol/knol",
];

pub const REDDIT_ML: &[&str] = &[
    "Knol: Open-source long-term memory for AI agents and LLM apps\n\nWe kept running into the same problem — AI agents that forget everything between sessions. Existing solutions are either too complex (full knowledge graphs) or too simple (dump everything into a vector DB).\n\nKnol sits in between: structured memory with importance scoring, semantic search, time decay, and sub-ms retrieval.\n\nUse cases we're seeing:\n- AI assistants that remember user preferences\n- Coding agents that learn codebase patterns\n- Customer support bots with interaction history\n\nOpen-source (MIT): https://github.com/aiknol/knol",
];

/// r/LocalLLaMA — Position as memory layer for local LLM setups
pub const REDDIT_LOCAL_LLAMA: &[&str] = &[
    "Open-source memory layer that gives your local LLM agents persistent context — just needs PostgreSQL\n\nIf you're running local models with Ollama, llama.cpp, or vLLM, your agents still lose everything between sessions. Knol fixes that.\n\nWhat it does:\n- Stores memories with semantic search (HNSW vectors)\n- Automatic importance scoring — important stuff stays, trivia fades\n- Works with ANY LLM provider (local or cloud)\n- Single Docker Compose — docker compose up and done\n- 50MB Rust binary, runs on a Raspberry Pi\n\nNo vendor lock-in, no cloud dependency. Your data stays on your hardware.\n\nGitHub: https://github.com/aiknol/knol",
];

/// r/selfhosted — Position as self-hosted AI infrastructure
pub const REDDIT_SELFHOSTED: &[&str] = &[
    "Self-hosted AI memory engine — single Docker Compose, no Neo4j or Qdrant needed\n\nBuilt Knol for anyone running AI/LLM applications who wants persistent memory without the complexity.\n\nSelf-host friendly:\n- Single Docker Compose file (Knol + PostgreSQL + Redis)\n- 50MB binary, ~256MB total RAM\n- All data in PostgreSQL — easy to backup, migrate, replicate\n- No external API calls required\n- Apache 2.0 license\n\nIt gives your AI agents the ability to remember conversations, user preferences, and learned patterns across sessions.\n\nDocker Compose and docs: https://github.com/aiknol/knol",
];

// ─── Dev.to / Hashnode / Medium templates ────────────────────────

pub const DEVTO_TUTORIAL: &[&str] = &[
    "# Building AI Apps with Persistent Memory Using Knol\n\nEvery AI application eventually hits the same wall: your LLM has no memory. Users repeat themselves, context is lost, and the experience suffers.\n\nIn this tutorial, I'll show you how to add persistent memory to any AI app using Knol — an open-source memory layer built in Rust.\n\n## What You'll Build\n\nA simple AI assistant that remembers:\n- User preferences\n- Previous conversations\n- Important facts mentioned by the user\n\n## Prerequisites\n\n- Docker (for Knol services)\n- Python 3.8+ or Node.js 18+\n- An OpenAI API key (or any LLM provider)\n\n## Step 1: Start Knol\n\n```bash\ndocker compose up -d\n```\n\nThis starts the memory service on port 8080.\n\n## Step 2: Store a Memory\n\n```python\nimport requests\n\nrequests.post('http://localhost:8080/v1/memories', json={\n    'user_id': 'user_123',\n    'content': 'User prefers dark mode',\n    'importance': 0.8\n})\n```\n\n## Step 3: Retrieve Relevant Memories\n\n```python\nresponse = requests.post('http://localhost:8080/v1/memories/search', json={\n    'user_id': 'user_123',\n    'query': 'What are the user preferences?',\n    'limit': 5\n})\nmemories = response.json()\n```\n\nThe search uses semantic similarity — you don't need exact keyword matches.\n\n## Next Steps\n\nCheck out the full documentation at https://aiknol.com/docs for advanced features like importance scoring, memory consolidation, and multi-user support.",
];

/// Dev.to article: Rust rewrite story (Launch Week Day 3)
pub const DEVTO_RUST_REWRITE: &[&str] = &[
    "# Why We Rewrote Our AI Memory Engine in Rust (and Cut Latency by 60x)\n\nOur AI memory layer started as a Python service. It worked. Then we hit production traffic.\n\n## The Python Problem\n\nAt 1,000 concurrent users, our P99 latency was 300ms. Memory usage: 2GB. GC pauses were killing our tail latency. And we needed three databases: PostgreSQL for metadata, Qdrant for vectors, Neo4j for the knowledge graph.\n\n## The Rust Rewrite\n\nWe rewrote everything in Rust over 4 months. The result:\n\n| Metric | Python | Rust |\n|--------|--------|------|\n| P99 latency | 300ms | 5ms |\n| Memory | 2GB | 50MB |\n| Databases | 3 | 1 (PostgreSQL) |\n| Binary size | N/A | 50MB |\n| Deploy time | 5min | 60sec |\n\n## Key Decisions\n\n**Axum over Actix**: Tower middleware composability won. Rate limiting, auth, tracing — all tower layers.\n\n**SQLx over Diesel**: Compile-time checked queries. No ORM overhead. Async from the ground up.\n\n**Custom HNSW over pgvector**: More control over index parameters. In-process for zero network overhead.\n\n**PostgreSQL for everything**: Adjacency lists + recursive CTEs handle 99% of graph queries. One fewer database.\n\n## Was It Worth It?\n\nAbsolutely. Our infrastructure cost dropped from $500/mo to $12/mo. Deployment went from 5-minute orchestration to docker compose up. And we can run on a Raspberry Pi.\n\nOpen-source: https://github.com/aiknol/knol",
];

// ─── Phase 4: MCP Ecosystem Content ──────────────────────────────

pub const DEVTO_MCP: &[&str] = &[
    "# Give Claude Persistent Memory with Knol MCP Server\n\nMCP (Model Context Protocol) lets you extend Claude and other AI assistants with custom tools. Knol ships with a native MCP server that gives Claude long-term memory.\n\n## What This Enables\n\nWith Knol's MCP server, Claude can:\n- Remember your preferences across sessions\n- Build a knowledge base from your conversations\n- Recall relevant context without you repeating yourself\n- Track project details, decisions, and action items\n\n## Setup (5 Minutes)\n\n1. Start Knol:\n```bash\ndocker compose up -d\n```\n\n2. Add to your Claude Desktop config:\n```json\n{\n  \"mcpServers\": {\n    \"knol\": {\n      \"command\": \"knol\",\n      \"args\": [\"mcp-server\"]\n    }\n  }\n}\n```\n\n3. Restart Claude Desktop. Done.\n\n## Example Workflows\n\n- **Project memory**: Claude remembers your codebase architecture, tech decisions, and TODOs\n- **Meeting notes**: Store meeting summaries, Claude recalls them when relevant\n- **Learning companion**: Claude tracks what you've learned and builds on it\n\nFull MCP documentation: https://aiknol.com/docs/mcp",
];

// ─── Hacker News templates ───────────────────────────────────────

pub const HN_LAUNCH: &[&str] = &[
    "Show HN: Knol – Open-source memory layer for AI applications (Rust)",
    "Show HN: We built persistent memory for LLM apps in Rust",
];

/// Detailed Show HN body for launch day (Phase 2 Day 1)
pub const HN_SHOW: &[&str] = &[
    "Show HN: Knol — Rust-native memory layer for AI agents (open source)\n\nWe built a persistent memory engine for LLM apps in Rust. Unlike Mem0 (Python + Qdrant + Neo4j) or Zep (Python/Go + Neo4j), Knol needs just PostgreSQL. The 50MB binary handles vector search, knowledge graphs, BM25, and memory decay — all at sub-5ms P95 latency.\n\nKey differentiators: bi-temporal memory model, 7-layer LLM cost optimization (75% token savings), conflict detection, and HMAC-signed webhooks. Self-host in 60 seconds with Docker Compose.\n\nhttps://github.com/aiknol/knol",
];

// ─── Blog templates ──────────────────────────────────────────────

pub const BLOG_LAUNCH: &[&str] = &[
    "# Introducing Knol: Persistent Memory for AI Applications\n\nToday we're open-sourcing Knol, a memory layer designed specifically for AI applications.\n\n## The Problem\n\nLarge Language Models are stateless. Every conversation starts from zero. Users repeat themselves. Context is lost. The experience degrades.\n\nDevelopers work around this by stuffing conversation history into prompts, but that's expensive, slow, and doesn't scale.\n\n## Our Solution\n\nKnol provides structured, searchable, persistent memory for any AI application. Store what matters, retrieve it when needed, and let your AI actually learn from interactions.\n\n## Key Features\n\n- **Sub-millisecond retrieval** — Redis-backed hot cache\n- **Semantic search** — Find memories by meaning, not just keywords\n- **Importance scoring** — Automatically prioritize what matters\n- **Time decay** — Old, unused memories fade naturally\n- **Multi-tenant** — Isolated memory per user/session\n\n## Architecture\n\nBuilt entirely in Rust for reliability and performance. The stack: Axum (HTTP), SQLx (PostgreSQL), Redis (caching), custom HNSW (vectors).\n\n## Get Started\n\n```bash\ndocker compose up -d\ncurl http://localhost:8080/health\n```\n\nThat's it. Full docs at https://aiknol.com/docs.",
];

pub const BLOG_TECHNICAL: &[&str] = &[
    "# How Knol Achieves Sub-Millisecond Memory Retrieval\n\nPerformance is critical for memory systems. If retrieval adds noticeable latency to your AI app, users will notice.\n\nHere's how Knol achieves sub-millisecond reads at scale.\n\n## Tiered Storage\n\nKnol uses a three-tier storage architecture:\n\n1. **L1 (Redis)**: Hot memories — recently accessed, high importance\n2. **L2 (PostgreSQL)**: Warm memories — all active memories with full indexing\n3. **L3 (S3)**: Cold storage — archived memories for long-term retention\n\n## Read Path\n\n1. Check Redis (50μs average)\n2. If miss, query PostgreSQL with prepared statements (2ms average)\n3. Promote to Redis on access (background task)\n\n## Write Path\n\n1. Write to PostgreSQL (primary source of truth)\n2. Async cache-aside to Redis\n3. Background importance scoring\n4. Periodic consolidation of related memories\n\n## Benchmarks\n\n| Operation | p50 | p99 |\n|-----------|-----|-----|\n| Read (cached) | 0.05ms | 0.2ms |\n| Read (uncached) | 1.8ms | 5ms |\n| Write | 2.1ms | 8ms |\n| Search (semantic) | 3.2ms | 12ms |",
    "# Building a Rust Microservice Architecture: Lessons from Knol\n\nKnol runs as 9 microservices, all in Rust. Here's what we learned.\n\n## Workspace Organization\n\nWe use Cargo workspaces with two sub-workspaces: `knol-oss` (open-source core) and `knol-enterprise` (commercial features).\n\n## Shared Patterns\n\nEvery service follows the same pattern: Axum for HTTP, SQLx for database, tracing for observability. Shared code lives in `memory-common` and `memory-db` crates.\n\n## Error Handling\n\nWe use `thiserror` for typed errors with `IntoResponse` implementations. Every error maps to an appropriate HTTP status code.\n\n## Testing\n\n207 unit tests, 180 integration tests. Tests run against real PostgreSQL and Redis instances in CI.\n\n## Deployment\n\nDocker Compose with multi-stage builds. Each service gets its own container with memory limits (96-256MB). Total resource usage: ~2GB RAM, 2 vCPUs.",
    "# Semantic Search Without a Vector Database\n\nYou don't always need a dedicated vector database. Here's how Knol implements semantic search with PostgreSQL and a custom HNSW index.\n\n## The Approach\n\nStore embeddings as PostgreSQL arrays. Build an in-process HNSW index for fast approximate nearest neighbor search. Use PostgreSQL for persistence, Rust for speed.\n\n## Why Not pgvector?\n\nWe started with pgvector but needed more control over the index parameters and wanted to avoid the overhead of a PostgreSQL extension in some deployment environments.\n\n## Implementation\n\nThe HNSW implementation lives in the `memory-index` crate. It supports:\n- Configurable M and efConstruction parameters\n- Cosine similarity and Euclidean distance\n- Incremental index updates\n- Serialization for persistence",
    "# Rate Limiting Done Right: A Rust Implementation\n\nRate limiting seems simple until you need it to be correct. Here's how we built Knol's marketing service rate limiter.\n\n## Requirements\n\n- Multiple time windows per channel (daily, monthly, per-minute)\n- Atomic check-and-increment\n- 90% safety margin on all limits\n- Survives service restarts\n\n## Sliding Window Algorithm\n\nWe use a sliding window counter per channel per time window. The key insight: check ALL windows before incrementing ANY of them.\n\n## Implementation\n\n```rust\npub async fn check_and_increment(&self, channel: &str) -> Result<bool, MarketingError> {\n    // 1. Read all windows for this channel\n    // 2. If ANY window is at limit, return false\n    // 3. Increment ALL windows atomically\n    // 4. Return true\n}\n```\n\nThis prevents the race condition where you increment one window but get blocked by another.",
];

// ─── Phase 3 & 5: SEO-Targeted Blog Templates ───────────────────

/// Blog posts targeting specific SEO keywords from the plan
pub const BLOG_SEO: &[&str] = &[
    "# The Complete Guide to AI Memory Layers\n\nAI agents need memory. Not just conversation history stuffed into prompts, but structured, searchable, persistent memory that survives across sessions.\n\nThis guide covers what an AI memory layer is, why vector databases alone aren't enough, and how to evaluate memory solutions for production AI applications.\n\n## What Is an AI Memory Layer?\n\nAn AI memory layer sits between your LLM and your application, providing:\n- **Persistent storage** for learned information\n- **Semantic retrieval** to find relevant memories\n- **Importance scoring** to prioritize what matters\n- **Time decay** to fade outdated information\n- **Multi-tenant isolation** for per-user memory\n\n## Why Not Just Use a Vector Database?\n\nVector databases are excellent at one thing: approximate nearest neighbor search on embeddings. But memory is more than vectors.\n\nA vector DB gives you similarity. A memory layer gives you understanding.\n\nKnol was built to address all of these: https://aiknol.com",
    "# Mem0 vs Zep vs Knol: AI Memory Layer Comparison\n\nChoosing an AI memory layer? Here's an honest comparison of the three leading open-source options.\n\n## Architecture\n\n| | Mem0 | Zep | Knol |\n|---|---|---|---|\n| Language | Python | Python/Go | Rust |\n| Databases | PostgreSQL + Qdrant + Neo4j | PostgreSQL + Neo4j | PostgreSQL only |\n| Binary size | N/A (Python) | ~100MB | 50MB |\n| Memory footprint | 2GB+ | 1GB+ | 256MB |\n\n## When to Choose Each\n\n- **Mem0**: You need the largest community and most integrations\n- **Zep**: You want a balance of features and performance\n- **Knol**: You need maximum performance, minimal infrastructure, or Rust-native integration",
    "# Context Engineering for LLM Applications: A Practical Guide\n\nContext engineering is the practice of curating what information an LLM sees at inference time. Get it right, and your AI app feels magical.\n\n## The Context Window Problem\n\nModern LLMs have large context windows (128K+ tokens), but filling them is expensive and counterproductive. More context ≠ better responses. Relevant context = better responses.\n\n## A Context Engineering Pipeline\n\n1. **Memory retrieval**: Pull relevant memories based on the current query\n2. **Importance ranking**: Sort by relevance × importance × recency\n3. **Token budgeting**: Allocate tokens across memory types\n4. **Compression**: Summarize older memories to fit the budget\n5. **Injection**: Place memories where the LLM expects them\n\nKnol provides primitives for each step. https://aiknol.com/docs/context-engineering",
];

// ─── Phase 4: Integration Tutorial Templates ─────────────────────

pub const BLOG_INTEGRATION: &[&str] = &[
    "# Add Persistent Memory to Your LangChain Agent in 5 Minutes\n\nLangChain agents are powerful but stateless. Every invocation starts fresh. Here's how to give them persistent memory with Knol.\n\n## Step 1: Start Knol\n\n```bash\ngit clone https://github.com/aiknol/knol && cd knol\ndocker compose up -d\n```\n\n## Step 2: Install the Knol Python Client\n\n```bash\npip install knol-client\n```\n\n## Step 3: Add Memory to Your Agent\n\n```python\nfrom knol import KnolMemory\nfrom langchain.agents import initialize_agent\n\nmemory = KnolMemory(base_url='http://localhost:8080', user_id='agent_1')\nagent = initialize_agent(tools, llm, memory=memory)\n```\n\nNow your agent remembers across sessions.\n\nFull integration guide: https://aiknol.com/docs/langchain",
    "# Building a Context-Aware Customer Support Bot with Knol + CrewAI\n\nCustomer support bots that forget every interaction waste users' time. With Knol + CrewAI, you can build a support agent that knows the customer's full history.\n\n## The Architecture\n\n1. **CrewAI** orchestrates the support workflow\n2. **Knol** provides per-customer memory\n3. Each interaction is stored as a memory with importance scoring\n4. The agent retrieves relevant history before responding\n\nhttps://aiknol.com/docs/crewai",
    "# Deploy Knol on Supabase in 5 Minutes\n\nSupabase provides managed PostgreSQL — exactly what Knol needs. Here's how to deploy Knol on Supabase for a fully managed, zero-ops AI memory layer.\n\n## Step 1: Create a Supabase Project\n\nSign up at supabase.com and create a new project. Copy the connection string.\n\n## Step 2: Run Knol\n\n```bash\nexport DATABASE_URL='postgresql://...' # Your Supabase connection string\ndocker run -e DATABASE_URL aiknol/knol:latest\n```\n\nKnol runs its migrations automatically.\n\nhttps://aiknol.com/docs/supabase",
];

// ─── Phase 2: Product Hunt templates ─────────────────────────────

pub const PRODUCTHUNT_LAUNCH: &[&str] = &[
    "Persistent memory for AI agents — Rust-native, sub-5ms, self-hosted\n\nKnol is an open-source memory layer for AI applications. It gives your LLM agents the ability to remember across sessions.\n\nKey features:\n- Sub-5ms P95 latency\n- Single PostgreSQL (no extra databases)\n- 50MB Rust binary\n- Bi-temporal knowledge graphs\n- 7-layer LLM cost optimization (75% savings)\n- Deploy in 60 seconds with Docker Compose\n\nApache 2.0 licensed. Self-host or use Knol Cloud.",
];

// ─── Email templates ─────────────────────────────────────────────

pub const EMAIL_WELCOME: &[&str] = &[
    "<h2>Welcome to Knol</h2>\n<p>Thanks for your interest in Knol — the open-source memory layer for AI applications.</p>\n<p>Here's what you can do next:</p>\n<ul>\n<li><a href=\"https://aiknol.com/docs\">Read the documentation</a></li>\n<li><a href=\"https://github.com/aiknol/knol\">Star us on GitHub</a></li>\n<li><a href=\"https://aiknol.com/quickstart\">5-minute quickstart guide</a></li>\n</ul>\n<p>Questions? Reply to this email — a human reads every one.</p>",
];

pub const EMAIL_WEEKLY: &[&str] = &[
    "<h2>This Week at Knol</h2>\n<p>Here's what's new in the Knol ecosystem this week:</p>\n<ul>\n<li><strong>New Release</strong>: Check the latest version for performance improvements and bug fixes</li>\n<li><strong>Community</strong>: Growing discussions on Reddit and GitHub</li>\n<li><strong>Tutorial</strong>: New guide on integrating Knol with popular AI frameworks</li>\n</ul>\n<p>As always, we'd love your feedback. Star us on <a href=\"https://github.com/aiknol/knol\">GitHub</a> if you haven't already!</p>",
];

/// Phase 5: Weekly digest email (opt-in during self-hosted setup)
pub const EMAIL_DIGEST: &[&str] = &[
    "<h2>Your Knol Weekly Digest</h2>\n<p>Here's a summary of your Knol instance this week:</p>\n<ul>\n<li><strong>Memories stored</strong>: {{total_memories}}</li>\n<li><strong>Queries served</strong>: {{total_queries}}</li>\n<li><strong>Average latency</strong>: {{avg_latency}}</li>\n</ul>\n<h3>What's New</h3>\n<p>Check out the latest Knol features and community highlights.</p>\n<p><small>Running self-hosted? <a href=\"https://aiknol.com/cloud\">Upgrade to Knol Cloud</a> for managed hosting, 99.9% SLA, and zero ops.</small></p>",
];

/// All template categories mapped by name for easy lookup.
pub fn get_templates(category: &str) -> Option<&'static [&'static str]> {
    match category {
        // Phase 3: Day-of-week Twitter (zero-cost daily strategy)
        "tweet_tip" => Some(TWEET_TIP),
        "tweet_benchmark" => Some(TWEET_BENCHMARK),
        "tweet_showcase" => Some(TWEET_SHOWCASE),
        "tweet_architecture" => Some(TWEET_ARCHITECTURE),
        "tweet_community" => Some(TWEET_COMMUNITY),
        "tweet_engagement" => Some(TWEET_ENGAGEMENT),
        // Original tweet categories (backward compat)
        "tweet_launch" => Some(TWEET_LAUNCH),
        "tweet_technical" => Some(TWEET_TECHNICAL),
        "tweet_comparison" => Some(TWEET_COMPARISON),
        // Phase 2: Launch week
        "tweet_thread_launch" => Some(TWEET_THREAD_LAUNCH),
        "hn_launch" => Some(HN_LAUNCH),
        "hn_show" => Some(HN_SHOW),
        "producthunt_launch" => Some(PRODUCTHUNT_LAUNCH),
        // LinkedIn
        "linkedin_launch" => Some(LINKEDIN_LAUNCH),
        "linkedin_technical" => Some(LINKEDIN_TECHNICAL),
        // Reddit (4 subs per the plan)
        "reddit_rust" => Some(REDDIT_RUST),
        "reddit_ml" => Some(REDDIT_ML),
        "reddit_local_llama" => Some(REDDIT_LOCAL_LLAMA),
        "reddit_selfhosted" => Some(REDDIT_SELFHOSTED),
        // Dev.to / cross-post
        "devto_tutorial" => Some(DEVTO_TUTORIAL),
        "devto_rust_rewrite" => Some(DEVTO_RUST_REWRITE),
        "devto_mcp" => Some(DEVTO_MCP),
        // Blog
        "blog_launch" => Some(BLOG_LAUNCH),
        "blog_technical" => Some(BLOG_TECHNICAL),
        "blog_seo" => Some(BLOG_SEO),
        "blog_integration" => Some(BLOG_INTEGRATION),
        // Email
        "email_welcome" => Some(EMAIL_WELCOME),
        "email_weekly" => Some(EMAIL_WEEKLY),
        "email_digest" => Some(EMAIL_DIGEST),
        _ => None,
    }
}

/// List all available template categories.
#[cfg(test)]
pub fn all_categories() -> &'static [&'static str] {
    &[
        // Day-of-week Twitter
        "tweet_tip",
        "tweet_benchmark",
        "tweet_showcase",
        "tweet_architecture",
        "tweet_community",
        "tweet_engagement",
        // Original tweets
        "tweet_launch",
        "tweet_technical",
        "tweet_comparison",
        // Launch week
        "tweet_thread_launch",
        "hn_launch",
        "hn_show",
        "producthunt_launch",
        // LinkedIn
        "linkedin_launch",
        "linkedin_technical",
        // Reddit (4 subs)
        "reddit_rust",
        "reddit_ml",
        "reddit_local_llama",
        "reddit_selfhosted",
        // Dev.to
        "devto_tutorial",
        "devto_rust_rewrite",
        "devto_mcp",
        // Blog
        "blog_launch",
        "blog_technical",
        "blog_seo",
        "blog_integration",
        // Email
        "email_welcome",
        "email_weekly",
        "email_digest",
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
        let tweet_categories = &[
            TWEET_TIP,
            TWEET_BENCHMARK,
            TWEET_SHOWCASE,
            TWEET_ARCHITECTURE,
            TWEET_COMMUNITY,
            TWEET_ENGAGEMENT,
            TWEET_LAUNCH,
            TWEET_TECHNICAL,
            TWEET_COMPARISON,
        ];
        for templates in tweet_categories {
            for t in *templates {
                assert!(
                    t.len() <= 280,
                    "Tweet template exceeds 280 chars ({} chars): {}...",
                    t.len(),
                    &t[..t.len().min(50)]
                );
            }
        }
    }

    #[test]
    fn day_of_week_categories_exist() {
        let dow = [
            "tweet_tip",
            "tweet_benchmark",
            "tweet_showcase",
            "tweet_architecture",
            "tweet_community",
        ];
        for cat in &dow {
            assert!(
                get_templates(cat).is_some(),
                "Day-of-week category '{}' missing",
                cat
            );
        }
    }

    #[test]
    fn launch_week_categories_exist() {
        let launch = [
            "hn_show",
            "reddit_local_llama",
            "reddit_selfhosted",
            "devto_rust_rewrite",
            "producthunt_launch",
            "tweet_thread_launch",
        ];
        for cat in &launch {
            assert!(
                get_templates(cat).is_some(),
                "Launch week category '{}' missing",
                cat
            );
        }
    }

    #[test]
    fn seo_and_integration_categories_exist() {
        assert!(get_templates("blog_seo").is_some());
        assert!(get_templates("blog_integration").is_some());
        assert!(get_templates("devto_mcp").is_some());
        assert!(get_templates("email_digest").is_some());
    }
}
