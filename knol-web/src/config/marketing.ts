// ── Homepage and marketing page data ─────────────────────────────

export interface BlogPost {
  title: string;
  slug: string;
  date: string;
  tag: string;
  description: string;
  href: string;
  body: string;
}

export const TECH_STACK = [
  'Rust',
  'PostgreSQL + pgvector',
  'NATS JetStream',
  'Redis',
  'Docker / Kubernetes',
  'Any LLM Provider',
] as const;

export const KEY_METRICS = [
  { value: '<5ms', label: 'Gateway Latency', sub: 'P95 measured' },
  { value: '75%', label: 'LLM Cost Savings', sub: '7-layer optimization' },
  { value: '10x', label: 'Smaller Footprint', sub: 'vs Python alternatives' },
  { value: '1', label: 'PostgreSQL', sub: 'Only database needed' },
] as const;

export const MEMORY_TYPES = [
  {
    name: 'Episodic',
    description: 'Raw conversation events with full context, timestamps, and participant metadata. The foundation of memory.',
    color: 'brand-400',
  },
  {
    name: 'Semantic',
    description: 'Distilled facts, preferences, and knowledge extracted from conversations via LLM-powered analysis.',
    color: 'brand-300',
  },
  {
    name: 'Working',
    description: 'Short-lived session context that provides immediate conversational continuity within a session.',
    color: 'brand-500',
  },
  {
    name: 'Procedural',
    description: 'Learned patterns, workflows, and behavioral preferences for deep personalization over time.',
    color: 'brand-600',
  },
] as const;

export const USE_CASES = [
  {
    title: 'AI Agents & Copilots',
    description: 'Build agents that remember user preferences, past interactions, and evolving context across sessions. Full context engineering for autonomous workflows.',
  },
  {
    title: 'Customer Support',
    description: 'Give support agents full customer history, previous resolutions, and preference data. Reduce resolution time with contextual memory.',
  },
  {
    title: 'Healthcare AI',
    description: 'Maintain patient interaction history, treatment context, and care plan continuity. HIPAA-ready encryption and compliance controls.',
  },
  {
    title: 'Education Platforms',
    description: 'Personalize learning paths by tracking student progress, knowledge gaps, and learning style. Adaptive tutoring at scale.',
  },
  {
    title: 'Enterprise RAG',
    description: 'Combine document retrieval with conversational memory for context-aware enterprise search. Graph-enhanced results.',
  },
  {
    title: 'Multi-Agent Systems',
    description: 'Shared memory across LangChain, CrewAI, and custom agents. MCP server support for tool-use architectures.',
  },
] as const;

export const COMPARISON_FEATURES = [
  { feature: 'Vector semantic retrieval', mem0: true, zep: true, knol: true },
  { feature: 'Knowledge graph storage', mem0: true, zep: true, knol: true },
  { feature: 'Temporal fact validity model', mem0: false, zep: true, knol: true },
  { feature: 'Hybrid retrieval (vector + BM25 + graph)', mem0: false, zep: false, knol: true },
  { feature: 'Memory decay & consolidation', mem0: false, zep: false, knol: true },
  { feature: 'Conflict detection & resolution', mem0: false, zep: false, knol: true },
  { feature: 'N-hop graph traversal', mem0: false, zep: true, knol: true },
  { feature: 'Working/session memory layer', mem0: false, zep: false, knol: true },
  { feature: 'Procedural memory support', mem0: false, zep: false, knol: true },
  { feature: 'PostgreSQL-only (no Neo4j/Qdrant)', mem0: false, zep: false, knol: true },
  { feature: 'Built-in PII detection/redaction', mem0: false, zep: false, knol: true },
  { feature: 'Webhook event system', mem0: false, zep: false, knol: true },
  { feature: 'Multi-tenant RLS isolation', mem0: true, zep: true, knol: true },
  { feature: 'Open-source self-host', mem0: false, zep: false, knol: true },
] as const;

export const BLOG_POSTS: BlogPost[] = [
  {
    title: 'Introducing Knol: Context Engineering for AI Applications',
    slug: 'introducing-knol',
    date: 'February 15, 2026',
    tag: 'Launch',
    description:
      'Today we\'re open-sourcing Knol — a Rust-native context engineering platform that gives LLM applications persistent memory with sub-5ms latency, powered by a single PostgreSQL database.',
    href: '/blog/introducing-knol',
    body: `## The Context Engineering Revolution

For years, the AI community has used the term "memory" to describe how applications retain information about users and past interactions. But memory is the wrong mental model. What AI applications actually need is **context engineering** — the ability to assemble precisely the right information at the right moment to ground LLM responses.

Knol is built from the ground up for context engineering. Instead of a simplistic "memory store," it provides a multi-layered system of episodic, semantic, working, and procedural memories that work together to create rich, contextual understanding. Each layer serves a specific purpose, and they integrate seamlessly.

## Why PostgreSQL + Rust Changes Everything

We made two critical architectural choices: Rust for performance and PostgreSQL for simplicity. While other memory systems spread data across multiple databases (Neo4j for graphs, Qdrant for vectors, Redis for cache), Knol runs on a single PostgreSQL instance with pgvector extension.

This means no operational complexity, no vendor lock-in, and no cross-database consistency headaches. Your data lives in one place. Your backups, permissions, and disaster recovery procedures already exist.

\`\`\`rust
// Deploy Knol with a single Helm chart
// All data in PostgreSQL with native vector support
// Sub-5ms P95 latency on retrieval operations
\`\`\`

## The Four Memory Layers

Knol's architecture mirrors human cognition, giving applications the same contextual depth that makes human conversations coherent:

- **Episodic Memory**: Raw conversation events with full context and metadata. The foundation of everything.
- **Semantic Memory**: Distilled facts, preferences, and knowledge extracted from conversations via LLM analysis.
- **Working Memory**: Short-lived session context that provides immediate conversational continuity.
- **Procedural Memory**: Learned patterns and behavioral preferences that enable deep personalization.

Together, these layers create applications that truly understand their users.

## What's Next

Knol is open-source and available today on GitHub. We've included Python and TypeScript SDKs, integrations with LangChain and CrewAI, and comprehensive documentation. Self-hosting is fully supported.

The future of AI isn't better models — it's smarter context. And context engineering is how you build it.`,
  },
  {
    title: 'Why Context Engineering Replaces "Memory" in AI',
    slug: 'context-engineering',
    date: 'February 18, 2026',
    tag: 'Strategy',
    description:
      'The industry is shifting from simple memory to context engineering — assembling the right information at the right time. Here\'s why this matters and how Knol is built for it.',
    href: '/blog/context-engineering',
    body: `## The Memory Metaphor is Broken

When we talk about "AI memory," we're borrowing a metaphor from human cognition. But the metaphor is incomplete and often misleading. Humans don't retrieve memories by exhaustively searching our entire past — we rapidly assemble contextual information based on what's relevant in the moment.

That's context engineering. And it's fundamentally different from building a memory bank.

## Three Paradigm Shifts

**From Storage to Retrieval**: The bottleneck isn't storing information; it's retrieving the *right* information when you need it. A chatbot with 10,000 conversation turns can't afford to include all of them in the context window. Knol's hybrid retrieval engine uses vector similarity, full-text search, and knowledge graph traversal to surface the 5-10 most relevant facts in under 5ms.

**From Flat to Structured**: Raw conversation logs are low-signal. Context engineering extracts structured facts, relationships, preferences, and patterns from conversations. This makes retrieval faster, cheaper, and more meaningful.

**From Static to Temporal**: Facts change. People move, get promoted, change their minds. Knol models validity periods and conflict detection at the memory layer, so your applications stay accurate as context evolves.

## The Economics of Context

Better context directly reduces LLM costs. When your prompts contain the specific information the model needs, you spend fewer tokens on irrelevant context. Fewer tokens means cheaper API calls and faster response times.

Knol's 7-layer optimization pipeline — prompt caching, intent classification, batch processing, model routing, and deduplication — combines better context engineering with smarter LLM invocation to achieve 75% cost reduction.

\`\`\`
Average cost per interaction:
- Baseline LLM calls:          $0.10
- With context engineering:    $0.025
- Savings:                     75%
\`\`\`

## Building for Context

Knol gives you the tools to practice context engineering at scale. The SDKs are designed around context assembly, not information storage. The database schema models temporal relationships and conflict resolution. The retrieval engine fuses multiple signals. The webhook system lets you react to contextual changes in real-time.

This is the future of AI applications: smarter context, not just bigger models.`,
  },
  {
    title: 'How Hybrid Retrieval Works: Vector + BM25 + Graph Fusion',
    slug: 'hybrid-retrieval',
    date: 'February 22, 2026',
    tag: 'Technical',
    description:
      'A deep dive into Knol\'s adaptive retrieval engine: intent classification, Reciprocal Rank Fusion, and how we combine three search signals for sub-5ms results.',
    href: '/blog/hybrid-retrieval',
    body: `## Three Retrieval Signals

Knol's retrieval engine doesn't rely on a single search signal. Instead, it fuses three complementary approaches:

**Vector Similarity**: Captures semantic meaning. If a user asks "What was John's favorite food?" and a past message mentioned "John loves sushi," vector similarity finds it even without exact keyword matching.

**BM25 Full-Text Search**: Captures keyword relevance. Essential for factual lookups ("What year did we switch to Postgres?") where exact terms matter more than semantic closeness.

**Knowledge Graph Traversal**: Captures relationship context. If you're asking about a user's preferences but you only know their ID, graph traversal can find connected entities (projects, team members, organizations) that provide crucial context.

## Intent Classification

Before retrieval, Knol classifies the incoming query into one of several intents:

\`\`\`
- Fact lookup (high precision BM25 weight)
- Relationship query (high precision graph weight)
- Preference inference (high precision vector weight)
- Open-ended context (balanced all three)
\`\`\`

This classification happens in real-time using a lightweight classifier. It's fast and dramatically improves retrieval quality by adjusting the retrieval strategy to the query type.

## Reciprocal Rank Fusion (RRF)

Once you have results from three different retrieval methods, how do you combine them? Simple averaging of scores doesn't work — the scales are different. Vector similarity is 0-1, BM25 scores can be arbitrarily large.

Knol uses Reciprocal Rank Fusion to combine results:

\`\`\`
score = 1/(k + rank_in_vector_results) +
        1/(k + rank_in_bm25_results) +
        1/(k + rank_in_graph_results)
\`\`\`

This method is robust to score scale differences and lets each signal contribute equally regardless of its native scale.

## Sub-5ms Performance

The entire retrieval pipeline — intent classification, three parallel searches, RRF scoring, and deduplication — completes in under 5ms on typical hardware. This is possible because:

1. PostgreSQL with pgvector does approximate nearest-neighbor search (HNSW), not exhaustive search
2. BM25 uses inverted indexes
3. Graph traversal is bounded to N hops
4. Rust implementations of the fusion logic eliminate Python overhead

The result: context assembly that feels instantaneous to the application.`,
  },
  {
    title: '75% LLM Cost Reduction: The 7-Layer Optimization Pipeline',
    slug: 'llm-cost-optimization',
    date: 'March 1, 2026',
    tag: 'Technical',
    description:
      'How Knol\'s extraction pipeline uses prompt caching, batching, model routing, and deduplication to cut LLM costs by 75% without sacrificing quality.',
    href: '/blog/llm-cost-optimization',
    body: `## The Seven Layers

Knol's cost optimization isn't a single technique — it's a orchestrated pipeline of seven complementary strategies:

## 1. Semantic Deduplication

When multiple sources convey the same fact, why extract it multiple times? Knol hashes semantic content to identify duplicates before sending to the LLM.

\`\`\`
Conversation 1: "I live in San Francisco"
Conversation 2: "My city is SF"
→ Deduplicated to one extraction request
\`\`\`

**Savings**: 15-20% on extraction volume

## 2. Prompt Caching with API Providers

OpenAI, Anthropic, and others offer prompt caching. System prompts and extraction instructions don't change between calls — they should be cached. Knol automatically batches extractions to maximize cache hits.

**Savings**: 25% on token costs (50% cheaper for cached tokens)

## 3. Intent-Based Model Routing

Not all extraction tasks need GPT-4. Simple fact extraction from recent conversations routes to Claude 3.5 Haiku. Complex disambiguation routes to Claude Opus. Routing decisions happen in real-time based on query complexity.

**Savings**: 40% overall by using the right model for each task

## 4. Batch Processing

Instead of extracting facts one-at-a-time, Knol batches 50-100 conversation turns per API call. This amortizes overhead and enables dynamic model routing based on batch characteristics.

**Savings**: 10-15% through batching efficiency

## 5. Working Memory Bypass

For queries in the current session, bypass extraction entirely. The working memory layer contains fresh data that doesn't need semantic analysis.

**Savings**: 30% of retrieval queries never hit the LLM

## 6. Conflict Resolution Caching

When Knol detects conflicting facts, it caches resolution decisions. "User prefers Postgres over MySQL" doesn't get re-extracted every time a new database preference is mentioned.

**Savings**: 5-10% for repeat patterns

## 7. Cross-Tenant Extraction Pooling

In multi-tenant deployments, Knol pools similar extraction tasks across customers and deduplicates at the semantic level. This requires privacy-preserving anonymization but can save 10-20% in shared deployments.

**Combined Savings**: 75% on total LLM invocation costs

## Real-World Impact

A customer with 100,000 monthly conversations:
- Baseline extraction cost: $2,500/month
- With 7-layer optimization: $625/month
- Annual savings: $22,500 per customer

And the extracted memories are actually *better*, because deduplication, conflict detection, and temporal modeling create higher-quality semantic data.`,
  },
  {
    title: 'Memory Decay, Conflict Detection, and Temporal Knowledge Graphs',
    slug: 'memory-intelligence',
    date: 'March 8, 2026',
    tag: 'Research',
    description:
      'Why we modeled Knol\'s memory system after human cognition — with decay scoring, conflict resolution, and bi-temporal knowledge graphs.',
    href: '/blog/memory-intelligence',
    body: `## The Forgetting Curve is a Feature

Human memory isn't permanent. Old facts fade. This isn't a bug — it's how we stay grounded in the present while maintaining historical context. Knol implements memory decay.

Each fact stored in semantic memory has an age-based decay score. Recent information is weighted heavily. Information older than 90 days gets exponentially lower weight. This prevents ancient preferences from drowning out current reality.

\`\`\`
decay_score(fact) = 1 / (1 + 0.05 * days_old)

Day 1:  score = 1.0 (full weight)
Day 30: score = 0.60 (60% weight)
Day 90: score = 0.17 (17% weight)
\`\`\`

Applications can customize decay rates. Some facts (like "customer is in Japan") decay slowly. Others (like "preferred payment method") decay quickly.

## Conflict Detection and Resolution

People change their minds. A user says "I prefer red" then six months later says "I prefer blue." Which is true? Both are, at different times.

Knol detects conflicts when new facts contradict stored facts with high confidence scores. Instead of overwriting, it marks both as valid with temporal validity windows.

\`\`\`
Fact A: "Prefers red" (confidence: 0.92, valid: 2025-01-01 to 2025-07-01)
Fact B: "Prefers blue" (confidence: 0.88, valid: 2025-07-01 to present)
\`\`\`

The retrieval engine understands temporal context. If the current date is in the blue preference window, blue gets weight. If you're querying historical context, red is used instead.

## Bi-Temporal Knowledge Graphs

Facts in Knol's knowledge graph have two timestamps:

**Valid Time**: When the fact was true in the real world (user moved to NYC in 2023)
**Transaction Time**: When we learned or updated the fact (we extracted this yesterday)

This distinction matters. A user might tell you today about a move that happened three months ago. Transaction time is when you learned it. Valid time is when it actually happened.

\`\`\`sql
-- A graph fact with bi-temporal semantics
INSERT INTO graph_facts (subject, predicate, object, valid_from, valid_to, created_at, updated_at)
VALUES ('user_123', 'worksAt', 'company_456', '2024-01-15', NULL, '2026-02-18', '2026-02-18');
\`\`\`

## Decay in Graphs

Decay isn't just for isolated facts. In the knowledge graph, edge weight decays over time. A relationship that was strong five years ago matters less now, unless it's been reinforced recently.

This prevents the graph from becoming cluttered with stale relationships while preserving the full history if needed.

## Practical Applications

These techniques solve real problems:

- **Customer churn prevention**: Detect when preferences change significantly (decay scores show shifting patterns)
- **Multi-agent systems**: Agents can query "what was user's position 6 months ago?" vs "current preference"
- **Audit compliance**: Full temporal history is preserved; decay is for inference only
- **Accuracy improvement**: Conflict detection flags contradictions for human review

The result is a memory system that's both grounded in present reality and respectful of history. It works the way human memory actually works.`,
  },
  {
    title: 'From Mem0 to Knol: A Migration Guide',
    slug: 'migration-guide',
    date: 'March 15, 2026',
    tag: 'Guide',
    description:
      'Step-by-step guide for teams migrating from Mem0 or Zep to Knol. Same API patterns, better performance, no vendor lock-in.',
    href: '/blog/migration-guide',
    body: `## Why Teams Migrate to Knol

Knol was designed with migration in mind. The API patterns are familiar to Mem0 and Zep users, but the performance, flexibility, and costs are dramatically better.

**Performance**: Sub-5ms retrieval latency vs 50-200ms
**Cost**: 75% reduction in LLM invocation costs
**Flexibility**: Single PostgreSQL database vs multiple vendor systems
**Ownership**: Open-source self-hosting vs vendor lock-in

## Step 1: Export Existing Data

Both Mem0 and Zep provide export functionality. Data formats differ slightly, but both can be converted to Knol's import schema.

\`\`\`bash
# Export from Mem0
mem0 export --format=jsonl > mem0_export.jsonl

# Export from Zep
zep export --format=jsonl > zep_export.jsonl

# Convert to Knol schema
knol convert --source=mem0 mem0_export.jsonl > knol_import.jsonl
\`\`\`

## Step 2: Set Up Knol Infrastructure

Knol's infrastructure is minimal: one PostgreSQL instance with pgvector.

\`\`\`bash
# Using Knol's Helm chart
helm repo add knol https://charts.aiknol.com
helm install knol knol/knol --namespace knol --create-namespace

# Or Docker Compose for development
docker-compose up -d
\`\`\`

## Step 3: Migrate API Calls

The migration is straightforward because Knol maintains API compatibility:

\`\`\`python
# Before (Mem0)
from mem0 import Memory
memory = Memory.from_config(config={"llm": {...}})
memory.add("User details", user_id="user_1")

# After (Knol) - minimal changes
from knol import KnolClient
client = KnolClient(api_key="your_key")
client.episodic.add("User details", user_id="user_1")

# If using LangChain, just swap the import
from knol.langchain import KnolMemory
memory = KnolMemory(client=client)
\`\`\`

## Step 4: Import Historical Data

Knol provides bulk import tools optimized for large data sets:

\`\`\`bash
knol import --source=knol_import.jsonl --batch-size=1000
\`\`\`

Import happens asynchronously. You can monitor progress:

\`\`\`bash
knol import status --job-id=job_123
\`\`\`

## Step 5: Run Dual Writes (Optional)

For zero-downtime migration, run dual writes for a period:

\`\`\`python
# Write to both systems during migration window
client.episodic.add(text, user_id=user_id)
mem0_client.add(text, user_id=user_id)  # temporary

# Query from Knol, fallback to Mem0 if needed
try:
    result = knol_client.retrieve(query, user_id=user_id)
except Exception:
    result = mem0_client.search(query, user_id=user_id)
\`\`\`

## Step 6: Update LLM Extraction Pipelines

If you have custom extraction prompts, they should work unchanged in Knol. But you might want to take advantage of Knol's structured extraction:

\`\`\`python
# Knol provides extraction types for common patterns
result = client.semantic.extract(
    conversation_turn=turn,
    extraction_type="preferences",  # Knol knows what facts to extract
    user_id=user_id
)
\`\`\`

## Rollback Plan

If issues arise, you have a complete snapshot of the old system. Knol's import is non-destructive — your original data still exists. You can:

1. Keep both systems running during a transition period
2. Query Knol as primary, fall back to Mem0 if needed
3. Compare retrieval results between systems
4. Gradually route 100% of traffic to Knol

## Common Issues and Solutions

**Query results differ slightly**: Knol's hybrid retrieval returns different results than Mem0's pure vector approach. This is usually better, but you can adjust weights in configuration.

**LLM costs increased during import**: Bulk extraction to populate semantic memory is expensive. But day-to-day operations will be 75% cheaper.

**Authentication changes**: If you're self-hosting, you control authentication entirely. Configure your preferred auth system (OAuth, SAML, API keys) directly.

## Timeline

Typical migration timeline:

- **Week 1**: Export and evaluation
- **Week 2**: Infrastructure setup and small-scale testing
- **Week 3**: Data import and validation
- **Week 4**: Dual write and monitoring
- **Week 5**: Full cutover

The process is straightforward, and our team can assist at any stage.`,
  },
  {
    title: 'Give Claude Persistent Memory with Knol MCP Server',
    slug: 'claude-persistent-memory-mcp',
    date: 'March 22, 2026',
    tag: 'Tutorial',
    description:
      'Step-by-step guide to setting up Knol as an MCP server for Claude Desktop. Your AI assistant will remember users, preferences, and context across every session.',
    href: '/blog/claude-persistent-memory-mcp',
    body: `## Why Claude Needs Persistent Memory

Every time you start a new Claude conversation, you start from scratch. Claude doesn't remember your name, your projects, your coding preferences, or the debugging session you had yesterday. You have to re-explain context every single time.

Knol's MCP server fixes this. Once connected, Claude can store and retrieve memories across sessions — facts, preferences, project context, and relationships. It's like giving Claude a brain that persists.

## Setting Up Knol (60 Seconds)

First, get Knol running locally:

\`\`\`bash
git clone https://github.com/aiknol/knol.git
cd knol
docker compose up -d
\`\`\`

That's it. Knol is now running on localhost:3000 with PostgreSQL, vector search, knowledge graphs, and the full context engineering stack.

## Connecting to Claude Desktop

Open your Claude Desktop MCP configuration:

\`\`\`bash
# macOS
code ~/Library/Application\\ Support/Claude/claude_desktop_config.json

# Windows
code %APPDATA%/Claude/claude_desktop_config.json
\`\`\`

Add the Knol MCP server:

\`\`\`json
{
  "mcpServers": {
    "knol-memory": {
      "command": "npx",
      "args": ["@aiknol/knol-mcp-server"],
      "env": {
        "KNOL_API_URL": "http://localhost:3000",
        "KNOL_API_KEY": "your-api-key"
      }
    }
  }
}
\`\`\`

Restart Claude Desktop. You should see the MCP tools icon appear, indicating Knol is connected.

## What Claude Can Do Now

With Knol connected, Claude has access to six memory tools:

**knol_store_memory** — Claude can save important facts from your conversation. "User prefers TypeScript over JavaScript" or "Working on a React dashboard for project Atlas."

**knol_search_memory** — Before answering, Claude can search past memories. "What do I know about this user's tech stack?" returns relevant context from previous sessions.

**knol_get_user_context** — Pull a complete summary of everything known about the current user. Preferences, projects, recent interactions, and relationships.

**knol_graph_query** — Traverse the knowledge graph. "What projects is this user connected to?" or "Who else works on project Atlas?"

## Real Example

Session 1 (Monday):

\`\`\`
You: I'm building a dashboard with Next.js and Tailwind. The backend is Rust with Axum.
Claude: [stores: user tech stack = Next.js, Tailwind, Rust, Axum]
\`\`\`

Session 2 (Wednesday):

\`\`\`
You: Can you help me add a new API endpoint?
Claude: [searches memory, finds Rust/Axum preference]
Claude: Sure! Since you are using Axum, here is the endpoint...
\`\`\`

No re-explanation needed. Claude already knows your stack.

## Beyond Claude Desktop

The same MCP server works with Cursor, Windsurf, and any MCP-compatible tool. Your memory is shared across all of them — context from a Claude conversation is available when you are coding in Cursor.

## Privacy and Self-Hosting

All memory stays on your machine. Knol runs locally, your data never leaves your infrastructure. For teams, deploy Knol on your own servers with multi-tenant isolation, encryption at rest, and full audit logging.

The MCP server is open-source. Star us on GitHub to follow development.`,
  },
  {
    title: 'Knol vs Mem0: A Technical Comparison for AI Memory',
    slug: 'knol-vs-mem0-comparison',
    date: 'March 29, 2026',
    tag: 'Comparison',
    description:
      'An honest technical comparison between Knol and Mem0 covering architecture, performance, features, and total cost of ownership for production AI memory.',
    href: '/blog/knol-vs-mem0-comparison',
    body: `## Two Different Approaches to AI Memory

Mem0 and Knol both solve the same problem: giving AI applications persistent memory. But they take fundamentally different architectural approaches, and those differences matter at scale.

This is an honest comparison. Both tools have strengths. The right choice depends on your requirements.

## Architecture

**Mem0** is a Python SDK that coordinates multiple backend services. In a typical production deployment, you need: Qdrant or Pinecone for vector search, Neo4j for knowledge graphs, Redis for caching, and a primary database for metadata. That is 4+ services to deploy, monitor, and maintain.

**Knol** is a single Rust binary backed by PostgreSQL with pgvector. Vector search, knowledge graphs, full-text search, and caching all run on one database. One service to deploy, one backup strategy, one set of credentials.

\`\`\`
Mem0 Production Stack:
  Python App -> Qdrant + Neo4j + Redis + PostgreSQL
  4 services, 3 languages, 2GB+ RAM minimum

Knol Production Stack:
  Rust Binary -> PostgreSQL (with pgvector)
  1 service, 50MB binary, 256MB RAM
\`\`\`

## Performance

Knol's Rust implementation delivers sub-5ms P95 latency on memory retrieval. Mem0's Python coordination layer adds overhead from cross-service communication, typically resulting in 50-200ms retrieval times depending on deployment.

For real-time applications like chatbots, that difference is noticeable to users. For batch processing, it means higher throughput per dollar.

## Feature Comparison

Both platforms support core memory operations: store, search, and retrieve. The differences are in advanced features.

Knol has memory decay (realistic forgetting), conflict detection (contradictory facts), bi-temporal modeling (valid time vs transaction time), and hybrid retrieval (vector + BM25 + graph fusion in a single query). These are built into the core engine.

Mem0 has a simpler model focused on vector-based memory with graph relationships. It is easier to get started with but has fewer knobs to tune for production workloads.

## Cost

The total cost of ownership differs significantly. Mem0's multi-service architecture means paying for Qdrant Cloud (or self-hosting), Neo4j Aura (or self-hosting), Redis, and your primary database. Each service has its own scaling curve.

Knol runs on PostgreSQL, which you probably already have. If you use Neon, Supabase, or AWS RDS, you are adding memory capability to an existing service rather than standing up new infrastructure.

## When to Choose Each

Choose Mem0 if you are already invested in the Qdrant/Neo4j ecosystem, your team is Python-first, or you need Mem0's managed cloud offering.

Choose Knol if you want minimal infrastructure (PostgreSQL only), sub-10ms latency, multi-tenant isolation, advanced features like memory decay and conflict detection, or you prefer self-hosting with open-source software.`,
  },
  {
    title: 'Deploy AI Memory on PostgreSQL in 60 Seconds',
    slug: 'ai-memory-postgresql-deploy',
    date: 'April 5, 2026',
    tag: 'Tutorial',
    description:
      'The fastest way to add persistent memory to your AI application. Three commands, one PostgreSQL database, sub-5ms latency. No Qdrant, no Neo4j, no complexity.',
    href: '/blog/ai-memory-postgresql-deploy',
    body: `## The 3-Command Deploy

Stop reading blog posts about complex AI memory architectures. Here is a working memory system in 60 seconds:

\`\`\`bash
git clone https://github.com/aiknol/knol.git
cd knol
docker compose up -d
\`\`\`

That is it. You now have a running memory system with vector search, knowledge graphs, full-text search, and 4 types of memory. Let us use it.

## Store Your First Memory

\`\`\`bash
curl -X POST http://localhost:3000/v1/memory \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer your-api-key" \\
  -d '{
    "content": "User prefers dark mode and uses VS Code",
    "user_id": "user_123"
  }'
\`\`\`

Knol automatically extracts structured facts from the content, generates vector embeddings, and updates the knowledge graph. All in one API call.

## Search Memories

\`\`\`bash
curl http://localhost:3000/v1/memory/search \\
  -H "Authorization: Bearer your-api-key" \\
  -d '{
    "query": "What editor does this user prefer?",
    "user_id": "user_123",
    "limit": 5
  }'
\`\`\`

The search uses hybrid retrieval — vector similarity, BM25 keyword matching, and knowledge graph traversal — fused together for the best results. Response time: under 5ms.

## Connect Your AI Application

\`\`\`python
from knol import KnolClient

client = KnolClient(
    api_url="http://localhost:3000",
    api_key="your-api-key"
)

# Store memory from a conversation
client.episodic.add(
    content="User asked about deploying to AWS ECS",
    user_id="user_123"
)

# Retrieve context for the next response
context = client.retrieve(
    query="What cloud platform does this user use?",
    user_id="user_123"
)
\`\`\`

## Why PostgreSQL Only?

Most AI memory systems require you to deploy 3-4 separate databases: one for vectors (Qdrant, Pinecone), one for graphs (Neo4j), one for search (Elasticsearch), and one for metadata (PostgreSQL). That is a lot of infrastructure for storing user preferences.

Knol uses PostgreSQL with the pgvector extension. Vectors, graphs, full-text search, and relational data all live in one database. One backup strategy, one set of credentials, one connection pool.

## What You Get Out of the Box

The Docker Compose includes everything: Gateway service for API routing, Write service for memory ingestion, Retrieve service for hybrid search, Graph service for entity extraction, PostgreSQL with pgvector for all data, Redis for caching, and NATS for async processing.

Total memory footprint: under 512MB. Compare that to running Qdrant + Neo4j + Redis + PostgreSQL separately.

## Next Steps

Once you have memories flowing, explore LangChain integration, the MCP server for Claude Desktop, knowledge graph queries, memory decay for automatic relevance scoring, and the admin dashboard for monitoring.

All documentation is at aiknol.com/docs. The project is open-source on GitHub.`,
  },
];


export const SDK_ECOSYSTEM = [
  { name: 'Python SDK', pkg: 'pip install knol', icon: '🐍' },
  { name: 'Async Python', pkg: 'from knol import AsyncKnolClient', icon: '⚡' },
  { name: 'TypeScript SDK', pkg: 'npm install @knol/sdk', icon: '📘' },
  { name: 'LangChain', pkg: 'from knol.langchain import KnolMemory', icon: '🦜' },
  { name: 'CrewAI', pkg: 'from knol.crewai import KnolMemory', icon: '🚢' },
  { name: 'MCP Server', pkg: 'npx @aiknol/knol-mcp-server', icon: '🔌' },
] as const;
