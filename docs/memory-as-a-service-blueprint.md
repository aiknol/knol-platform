# Memory Infrastructure for AI: Production-Grade Technical Blueprint

*Kubernetes-ready, multi-tenant, Rust+Python microservice architecture for long-term LLM memory.*
*Synthesizes research (Mem0, Zep/Graphiti, MemGPT, A-MEM, HippoRAG) with production microservice design.*

Note: this is a long-form architecture blueprint and retains historical `memory-*` naming in multiple sections. For current runtime names and ports in this repository, use `docs/docker-stack.md`.

---

## 1. Problem Statement

LLMs reset after each context window. Even 200K+ token windows merely *delay* the limitation — they cannot maintain persistent state across sessions, evolve knowledge over time, or consolidate contradictory information. A **Memory Infrastructure** layer solves this by sitting between any LLM application and its storage backends, providing unified APIs for storing, consolidating, and retrieving long-term memory.

This document is the technical blueprint: exact microservices, data models with SQL, Kubernetes deployment, request flows, security enforcement, and build order.

---

## 2. Research Foundations

| System | Core Idea | Paper |
|--------|-----------|-------|
| **Mem0** | LLM-driven memory extraction + vector/graph hybrid storage | Chhikara et al., arXiv:2504.19413 (2025) |
| **Zep / Graphiti** | Bi-temporal knowledge graph with episodic ingestion | Rasmussen et al., arXiv:2501.13956 (2025) |
| **MemGPT / Letta** | OS-inspired hierarchical memory (core → recall → archival) | Packer et al., arXiv:2310.08560 (2023) |
| **A-MEM** | Zettelkasten-inspired agentic self-evolving memory | Xu et al., arXiv:2502.12110 (2025), NeurIPS 2025 |
| **HippoRAG** | Hippocampal indexing with KG + Personalized PageRank | Gutiérrez et al., arXiv:2405.14831 (2024), NeurIPS 2024 |
| **Survey** | Comprehensive taxonomy of agent memory mechanisms | Zhang et al., arXiv:2404.13501 (2024), ACM TOIS |
| **LangChain LangGraph** | Long-term memory via persistent store + vector search | LangChain Docs, docs.langchain.com |

### Key Design Principles from the Literature

1. **Memory types map to human cognition**: Episodic (raw events), Semantic (facts/entities), Procedural (how-to), Working (active context).
2. **Graph beats flat vectors for relational queries**: Zep +18.5% on LongMemEval; HippoRAG +20% on multi-hop QA.
3. **Memory must self-manage**: LLM-driven extraction, dedup, importance scoring (A-MEM, Mem0).
4. **Bi-temporality is non-negotiable**: event-time vs ingestion-time enables point-in-time queries, conflict resolution, audit (Zep).
5. **Hybrid retrieval wins**: vector + BM25 + graph traversal combined always outperforms any single signal.
6. **Async write, sync read**: Write path can be async (low latency ack) while read path must be fast and deterministic.

---

## 3. Microservice Architecture

### 3.1 Service Map

```
┌──────────────────────────────────────────────────────────────────────────┐
│                            CLIENT APPS                                    │
│  (Chatbots, Agent Frameworks, Copilots, Multi-Agent Systems, SDKs)       │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │  REST / gRPC / WebSocket
                               ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                    1. MEMORY-GATEWAY (Rust / Axum)                        │
│                                                                           │
│  ┌──────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐  │
│  │ AuthN/   │ │ Rate      │ │ Tenant    │ │ Scope     │ │ Billing   │  │
│  │ AuthZ    │ │ Limiter   │ │ Router    │ │ Resolver  │ │ Meter     │  │
│  │ (JWT +   │ │ (Redis    │ │ (RLS      │ │ (user/    │ │ (usage    │  │
│  │  API Key │ │  sliding  │ │  tenant   │ │  team/    │ │  hooks)   │  │
│  │  + OIDC) │ │  window)  │ │  context) │ │  project/ │ │           │  │
│  └──────────┘ └───────────┘ └───────────┘ │  org)     │ └───────────┘  │
│                                            └───────────┘                  │
│  External endpoints:                                                      │
│    /v1/memory/*  /v1/graph/*  /v1/admin/*  /v1/simulate/*                │
│                                                                           │
│  P99 gateway latency target: <5ms                                         │
│  Why Rust: 10x less memory than Python/Go, sub-ms routing, Tower          │
│  middleware composability. Critical for a service in the hot path          │
│  of every LLM call.                                                       │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │ Internal gRPC / HTTP
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐
│ 2. MEMORY-WRITE │  │ 3. MEMORY-      │  │ 5. MEMORY-ADMIN         │
│ (Rust/Axum +    │  │    RETRIEVE     │  │ (Rust/Axum)             │
│  NATS publisher)│  │ (Rust/Axum)     │  │                         │
│                 │  │                 │  │ • Memory CRUD UI API    │
│ • Idempotent    │  │ • Query intent  │  │ • Governance rules      │
│   ingestion     │  │   classifier    │  │ • Audit log browser     │
│ • Episode store │  │ • Adaptive      │  │ • Retention policies    │
│ • Emit events   │  │   retrieval     │  │ • Merge operations      │
│   to NATS       │  │   router        │  │ • Simulation/replay     │
│ • Fast ACK      │  │ • Vector search │  │                         │
│   (async proc)  │  │ • Graph expand  │  │ PATCH /v1/memory/{id}   │
│                 │  │ • BM25 search   │  │ POST  /v1/memory/merge  │
│ POST /internal/ │  │ • Temporal      │  │ GET   /v1/audit         │
│   ingest        │  │   filter        │  │ POST  /v1/simulate/     │
│ POST /internal/ │  │ • RRF rerank    │  │   replay                │
│   ingest/batch  │  │ • RL ranker     │  │                         │
│                 │  │ • Token budget  │  └─────────────────────────┘
│                 │  │   compression   │
│                 │  │                 │
│                 │  │ POST /internal/ │
│                 │  │   search        │
│                 │  │ POST /internal/ │
│                 │  │   context-pack  │
└────────┬────────┘  └─────────┬──────┘
         │                     │
         │  NATS JetStream     │  Direct DB reads
         ▼                     ▼
┌─────────────────┐  ┌─────────────────┐
│ 4. MEMORY-GRAPH │  │ 6. MEMORY-JOBS  │
│ (Rust/Axum)     │  │ (Rust workers)  │
│                 │  │                 │
│ • Entity CRUD   │  │ • LLM extractor │
│ • Edge CRUD     │  │ • Classifier    │
│ • BFS/DFS       │  │ • Conflict      │
│   traversal     │  │   resolver      │
│ • Personalized  │  │ • Embedder      │
│   PageRank      │  │ • Graph linker  │
│ • Temporal      │  │ • Compaction    │
│   validity      │  │ • Summarization │
│   queries       │  │ • TTL expiry    │
│ • Graph         │  │ • Re-embed on   │
│   summarization │  │   model change  │
│                 │  │ • Decay scoring │
│ POST /internal/ │  │ • A-MEM evolve  │
│   graph/upsert  │  │ • Metric rollup │
│ POST /internal/ │  │                 │
│   graph/expand  │  │ Runs as:        │
│ POST /internal/ │  │ • K8s CronJobs  │
│   graph/        │  │ • NATS consumers│
│   summarize     │  │                 │
└─────────────────┘  └─────────────────┘

┌─────────────────┐  ┌─────────────────┐
│ 7. BILLING-     │  │ 8. CONNECTOR-   │
│    METER        │  │    INGEST       │
│ (Rust, optional)│  │ (Rust, optional)│
│                 │  │                 │
│ • Usage event   │  │ • Webhook       │
│   aggregation   │  │   receiver      │
│ • Stripe/Orb    │  │ • File ingest   │
│   integration   │  │   (PDF, CSV)    │
│ • Quota         │  │ • Slack/Gmail/  │
│   enforcement   │  │   Notion/       │
│                 │  │   Salesforce    │
│                 │  │   connectors    │
└─────────────────┘  └─────────────────┘
```

### 3.2 Service Responsibilities Summary

| # | Service | Language | Scaling | Public? |
|---|---------|----------|---------|---------|
| 1 | **memory-gateway** | Rust/Axum | HPA on CPU + RPS | Yes (Ingress) |
| 2 | **memory-write** | Rust/Axum + NATS pub | HPA on queue depth | No (ClusterIP) |
| 3 | **memory-retrieve** | Rust/Axum | HPA on CPU + RPS | No (ClusterIP) |
| 4 | **memory-graph** | Rust/Axum | HPA on CPU | No (ClusterIP) |
| 5 | **memory-admin** | Rust/Axum | HPA on CPU | Yes (Ingress) |
| 6 | **memory-jobs** | Rust workers | Scaled by NATS lag + CronJob | No |
| 7 | **billing-meter** | Rust | StatefulSet (1 replica) | No |
| 8 | **connector-ingest** | Rust | HPA per connector type | No |

---

## 4. Data Model (Production SQL)

### 4.1 Multi-Tenancy Strategy

Postgres Row-Level Security (RLS) with `tenant_id`. Every request carries `tenant_id` from the auth layer (never client-provided), set via `SET app.tenant_id = ...` on DB session. All tables include `tenant_id`.

```sql
-- Enable RLS on every table
ALTER TABLE memories ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON memories
  USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### 4.2 Core Tables

```sql
-- ================================================================
-- MEMORIES: Canonical source-of-truth for all extracted memories
-- ================================================================
CREATE TABLE memories (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  user_id         UUID,                          -- nullable for team/org memory
  scope           TEXT NOT NULL DEFAULT 'user'    -- user/team/project/agent/org
                  CHECK (scope IN ('user','team','project','agent','org')),
  kind            TEXT NOT NULL                   -- preference/fact/task/event/
                  CHECK (kind IN (                --   relationship/summary/procedure
                    'preference','fact','task','event',
                    'relationship','summary','procedure'
                  )),
  content         TEXT NOT NULL,
  content_json    JSONB,                          -- normalized structured fields
  confidence      REAL NOT NULL DEFAULT 0.8       -- 0.0–1.0
                  CHECK (confidence >= 0 AND confidence <= 1),
  importance      REAL NOT NULL DEFAULT 0.5       -- decayable importance score
                  CHECK (importance >= 0 AND importance <= 1),
  status          TEXT NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active','superseded','archived','deleted')),

  -- Bi-temporal fields (Zep model)
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),  -- when fact became true
  valid_to        TIMESTAMPTZ,                         -- null = still true
  event_time      TIMESTAMPTZ,                         -- when it actually happened
  ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),   -- when we learned about it

  -- Provenance
  source_episode_id UUID REFERENCES episodes(id),
  created_by      TEXT NOT NULL DEFAULT 'system'  -- system/user/admin/connector
                  CHECK (created_by IN ('system','user','admin','connector')),

  -- Metadata
  tags            TEXT[] DEFAULT '{}',
  metadata        JSONB DEFAULT '{}',

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Performance indexes
CREATE INDEX idx_memories_tenant_user ON memories(tenant_id, user_id);
CREATE INDEX idx_memories_scope ON memories(tenant_id, scope);
CREATE INDEX idx_memories_kind ON memories(tenant_id, kind);
CREATE INDEX idx_memories_status ON memories(tenant_id, status) WHERE status = 'active';
CREATE INDEX idx_memories_valid ON memories(tenant_id, valid_from, valid_to);
CREATE INDEX idx_memories_tags ON memories USING GIN(tags);
CREATE INDEX idx_memories_metadata ON memories USING GIN(metadata);

-- ================================================================
-- MEMORY_VECTORS: Embedding index (pgvector)
-- ================================================================
CREATE TABLE memory_vectors (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id       UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
  tenant_id       UUID NOT NULL,
  user_id         UUID,
  scope           TEXT NOT NULL,
  kind            TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'active',
  valid_from      TIMESTAMPTZ NOT NULL,
  valid_to        TIMESTAMPTZ,
  embedding       vector(1536) NOT NULL,          -- text-embedding-3-small
  content_hash    TEXT NOT NULL,                   -- for dedup detection

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- HNSW index for fast ANN search
CREATE INDEX idx_vectors_embedding ON memory_vectors
  USING hnsw (embedding vector_cosine_ops)
  WITH (m = 16, ef_construction = 200);

CREATE INDEX idx_vectors_tenant ON memory_vectors(tenant_id, status);
CREATE INDEX idx_vectors_dedup ON memory_vectors(tenant_id, content_hash);

-- ================================================================
-- EPISODES: Raw conversation/event log (pointers to S3/MinIO)
-- ================================================================
CREATE TABLE episodes (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  user_id         UUID,
  session_id      TEXT,
  agent_id        TEXT,

  content         TEXT NOT NULL,                   -- raw message text
  role            TEXT NOT NULL                     -- user/assistant/system/tool
                  CHECK (role IN ('user','assistant','system','tool')),
  event_time      TIMESTAMPTZ NOT NULL DEFAULT now(),
  ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

  -- S3 pointer for full payload (large events, tool outputs, attachments)
  storage_key     TEXT,                            -- s3://{tenant}/{user}/{session}/{ts}
  content_hash    TEXT NOT NULL,                   -- SHA256 for integrity

  metadata        JSONB DEFAULT '{}',

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_episodes_tenant_user ON episodes(tenant_id, user_id);
CREATE INDEX idx_episodes_session ON episodes(tenant_id, session_id);
CREATE INDEX idx_episodes_time ON episodes(tenant_id, event_time DESC);

-- ================================================================
-- ENTITIES: Knowledge graph nodes (temporal)
-- ================================================================
CREATE TABLE entities (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  name            TEXT NOT NULL,
  entity_type     TEXT NOT NULL,                   -- person/org/concept/location/product
  summary         TEXT,                            -- LLM-generated entity summary
  attributes      JSONB DEFAULT '{}',
  embedding       vector(1536),

  -- Bi-temporal
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to        TIMESTAMPTZ,
  status          TEXT NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active','merged','deleted')),
  merged_into     UUID REFERENCES entities(id),    -- if merged, points to canonical

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_entities_tenant ON entities(tenant_id);
CREATE INDEX idx_entities_type ON entities(tenant_id, entity_type);
CREATE INDEX idx_entities_name ON entities(tenant_id, name);
CREATE INDEX idx_entities_embedding ON entities
  USING hnsw (embedding vector_cosine_ops);
CREATE UNIQUE INDEX idx_entities_dedup ON entities(tenant_id, name, entity_type)
  WHERE status = 'active';

-- ================================================================
-- EDGES: Knowledge graph relationships (temporal)
-- ================================================================
CREATE TABLE edges (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  source_entity_id UUID NOT NULL REFERENCES entities(id),
  target_entity_id UUID NOT NULL REFERENCES entities(id),
  rel_type        TEXT NOT NULL,                   -- works_at/prefers/manages/etc.
  properties      JSONB DEFAULT '{}',
  weight          REAL NOT NULL DEFAULT 1.0,       -- importance/confidence
  source_episode_id UUID REFERENCES episodes(id),  -- provenance

  -- Bi-temporal
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to        TIMESTAMPTZ,
  status          TEXT NOT NULL DEFAULT 'active',

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_edges_source ON edges(tenant_id, source_entity_id) WHERE status = 'active';
CREATE INDEX idx_edges_target ON edges(tenant_id, target_entity_id) WHERE status = 'active';
CREATE INDEX idx_edges_rel ON edges(tenant_id, rel_type);
CREATE INDEX idx_edges_temporal ON edges(tenant_id, valid_from, valid_to);

-- ================================================================
-- WORKING_MEMORY: Per-session active context (Redis-backed, DB for persistence)
-- ================================================================
CREATE TABLE working_memory (
  session_id      TEXT NOT NULL,
  tenant_id       UUID NOT NULL,
  user_id         UUID,
  agent_id        TEXT,
  summary         TEXT,                            -- running conversation summary
  active_facts    UUID[] DEFAULT '{}',             -- currently relevant memory IDs
  active_procs    UUID[] DEFAULT '{}',             -- currently relevant procedures
  scratchpad      JSONB DEFAULT '{}',              -- agent-managed free-form state
  turn_count      INTEGER NOT NULL DEFAULT 0,
  last_updated    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, session_id)
);

-- ================================================================
-- PROCEDURAL_MEMORY: Learned workflows and tool-use patterns
-- ================================================================
CREATE TABLE procedural_memories (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  scope           TEXT NOT NULL DEFAULT 'user',
  scope_id        TEXT,
  user_id         UUID,
  agent_id        TEXT,

  description     TEXT NOT NULL,                   -- "User prefers CSV over PDF"
  trigger_condition TEXT,                           -- when to activate
  procedure_steps TEXT NOT NULL,                    -- step-by-step or tool call seq
  success_count   INTEGER NOT NULL DEFAULT 0,
  fail_count      INTEGER NOT NULL DEFAULT 0,
  embedding       vector(1536),

  status          TEXT NOT NULL DEFAULT 'active',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used       TIMESTAMPTZ
);

CREATE INDEX idx_procs_tenant ON procedural_memories(tenant_id, status);
CREATE INDEX idx_procs_embedding ON procedural_memories
  USING hnsw (embedding vector_cosine_ops);

-- ================================================================
-- MEMORY_AUDIT: Full governance and compliance trail
-- ================================================================
CREATE TABLE memory_audit (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  memory_id       UUID NOT NULL,                   -- references memories or entities
  target_table    TEXT NOT NULL,                    -- 'memories', 'entities', 'edges'
  action          TEXT NOT NULL                     -- create/update/delete/merge/
                  CHECK (action IN (               --   supersede/restore/archive
                    'create','update','delete','merge',
                    'supersede','restore','archive','decay'
                  )),
  actor_type      TEXT NOT NULL                     -- system/user/admin/connector
                  CHECK (actor_type IN ('system','user','admin','connector')),
  actor_id        TEXT,
  diff            JSONB,                            -- before/after delta
  reason          TEXT,                             -- why this change happened
  timestamp       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_memory ON memory_audit(tenant_id, memory_id);
CREATE INDEX idx_audit_time ON memory_audit(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_actor ON memory_audit(tenant_id, actor_type, actor_id);

-- ================================================================
-- TENANTS: Multi-tenant configuration
-- ================================================================
CREATE TABLE tenants (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name            TEXT NOT NULL,
  slug            TEXT NOT NULL UNIQUE,
  plan            TEXT NOT NULL DEFAULT 'free'
                  CHECK (plan IN ('free','developer','pro','team','enterprise')),

  -- Per-tenant configuration
  config          JSONB NOT NULL DEFAULT '{
    "extraction_model": "gpt-4.1-nano",
    "embedding_model": "text-embedding-3-small",
    "embedding_dim": 1536,
    "decay_lambda": 0.01,
    "importance_threshold": 0.1,
    "max_memories_per_user": 100000,
    "retention_days": null,
    "pii_redaction": false,
    "custom_ontology": null
  }'::jsonb,

  -- Billing
  api_key_hash    TEXT NOT NULL,
  usage_ops_month INTEGER NOT NULL DEFAULT 0,
  usage_limit     INTEGER,                         -- null = unlimited (enterprise)

  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ================================================================
-- MEMORY_POLICIES: Governance rules per tenant
-- ================================================================
CREATE TABLE memory_policies (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  name            TEXT NOT NULL,
  rule_type       TEXT NOT NULL                    -- retention/redaction/scope_access/
                  CHECK (rule_type IN (            --   auto_classify/auto_expire
                    'retention','redaction','scope_access',
                    'auto_classify','auto_expire','pii_filter'
                  )),
  config          JSONB NOT NULL,                  -- rule-specific configuration
  enabled         BOOLEAN NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 4.3 Upgrade Path: Postgres Graph → Neo4j

The OSS version uses Postgres `entities` + `edges` tables. For enterprise customers needing heavy graph workloads (multi-hop traversal, PageRank, community detection), the `memory-graph` service abstracts the backend behind a trait/interface:

```rust
#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn upsert_entity(&self, tenant_id: Uuid, entity: Entity) -> Result<Uuid>;
    async fn upsert_edge(&self, tenant_id: Uuid, edge: Edge) -> Result<Uuid>;
    async fn expand(&self, tenant_id: Uuid, req: ExpandRequest) -> Result<Vec<GraphNode>>;
    async fn pagerank(&self, tenant_id: Uuid, seed: Uuid, depth: u32) -> Result<Vec<RankedNode>>;
    async fn temporal_query(&self, tenant_id: Uuid, point_in_time: DateTime) -> Result<GraphSnapshot>;
}

// Implementations:
// - PostgresGraphStore (OSS default)
// - Neo4jGraphStore (enterprise)
// - FalkorDBGraphStore (alternative)
```

---

## 5. Request Flows (Exact Sequences)

### 5.1 Write Flow: Ingest → Extract → Store → Index

```
Client/Agent
    │
    │  POST /v1/memory/add
    │  { messages: [...], user_id, scope, metadata }
    ▼
┌─ memory-gateway ──────────────────────────────────────────┐
│  1. Authenticate (JWT/API key)                             │
│  2. Resolve tenant_id from token                           │
│  3. Validate request schema                                │
│  4. Check rate limit + quota                               │
│  5. Route to memory-write                                  │
│  6. Meter usage event (async)                              │
└───────────────────────────┬───────────────────────────────┘
                            │  gRPC /internal/ingest
                            ▼
┌─ memory-write ────────────────────────────────────────────┐
│  1. Generate idempotency key (hash of content + user + ts) │
│  2. Check Redis for duplicate (skip if seen)               │
│  3. Store raw episode(s) in episodes table                 │
│  4. Upload full payload to S3 (if large)                   │
│  5. Emit MemoryIngested event to NATS JetStream            │
│  6. Return ingest_id immediately (async processing)        │
│                                                            │
│  Latency target: <50ms (sync portion)                      │
└───────────────────────────┬───────────────────────────────┘
                            │  NATS JetStream
                            ▼
┌─ memory-jobs (workers) ───────────────────────────────────┐
│                                                            │
│  STEP 1: CLASSIFY                                          │
│  • LLM scores: importance × novelty × user-relevance       │
│  • If score < threshold → store episodic only, emit done   │
│                                                            │
│  STEP 2: EXTRACT                                           │
│  • LLM extracts structured data:                           │
│    - Entities (name, type, attributes)                     │
│    - Relations (entity-A rel_type entity-B)                │
│    - Temporal markers ("next Thursday" → abs date)          │
│    - Procedures (tool-use patterns, workflow steps)         │
│    - User preferences / corrections                        │
│  • Classify kind: preference/fact/task/event/relationship  │
│  • Normalize to content_json schema                        │
│                                                            │
│  STEP 3: RECONCILE (conflict resolution)                   │
│  • For each extracted entity/relation:                     │
│    1. Semantic search existing (embedding similarity)       │
│    2. LLM decides: NEW | UPDATE | MERGE | CONTRADICT       │
│    3. If CONTRADICT → check temporal ordering:             │
│       - Newer event_time wins (mark old valid_to = now)    │
│       - Or flag for user resolution (if confidence < 0.6)  │
│    4. Update graph edges and weights                       │
│  • Dedup via content_hash on memory_vectors table          │
│                                                            │
│  STEP 4: EMBED & STORE                                     │
│  • Generate embeddings (text-embedding-3-small)            │
│  • Write to memories table (canonical record)              │
│  • Write to memory_vectors table (pgvector index)          │
│  • Emit GraphUpsert events → memory-graph service          │
│  • Write to memory_audit table (provenance)                │
│  • Emit MemoryUpserted event (for webhooks)                │
│                                                            │
│  STEP 5: GRAPH LINKING                                     │
│  • memory-graph receives entities/edges                    │
│  • Dedup against existing entities (name + type + tenant)  │
│  • Upsert to entities + edges tables                       │
│  • Update embeddings on entity summaries                   │
│                                                            │
└───────────────────────────────────────────────────────────┘
```

### 5.2 Retrieve Flow: Adaptive Hybrid Retrieval

```
Client/Agent
    │
    │  POST /v1/memory/search
    │  { query, user_id, scope, time_range?, top_k?, token_budget? }
    ▼
┌─ memory-gateway → memory-retrieve ────────────────────────┐
│                                                            │
│  STEP 1: CLASSIFY QUERY INTENT                             │
│  Lightweight classifier (rules + small model) determines:  │
│                                                            │
│  ┌─ Query Type ───────── Strategy ────────────────────┐   │
│  │ "What does user prefer?"  → PREFERENCE path         │   │
│  │   → Vector search on kind=preference                │   │
│  │   → Weight: recency HIGH, graph LOW                 │   │
│  │                                                     │   │
│  │ "What happened last week?" → TEMPORAL path          │   │
│  │   → Time-range filter + temporal graph traversal    │   │
│  │   → Weight: temporal HIGH, vector LOW               │   │
│  │                                                     │   │
│  │ "Who works with Dr. Chen?" → RELATIONAL path        │   │
│  │   → Graph traversal (BFS depth 2) + PageRank        │   │
│  │   → Weight: graph HIGH, vector MEDIUM               │   │
│  │                                                     │   │
│  │ "Tell me about the project" → BROAD RECALL path     │   │
│  │   → Full hybrid: vector + BM25 + graph + temporal   │   │
│  │   → Weight: balanced across all signals             │   │
│  │                                                     │   │
│  │ "How did we solve X before?" → PROCEDURAL path      │   │
│  │   → Search procedural_memories + success_count rank │   │
│  │   → Weight: procedural HIGH, episodic MEDIUM        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                            │
│  STEP 2: PARALLEL SEARCH (run concurrently)                │
│  a) Vector search (pgvector / Qdrant)                      │
│     - query embedding → ANN search                         │
│     - Filter: tenant_id + scope + status='active'          │
│     - Filter: valid_from <= now, valid_to IS NULL or > now │
│  b) BM25 keyword search (Postgres FTS / dedicated index)   │
│  c) Graph expansion (memory-graph service)                 │
│     - Start from active entities in working memory         │
│     - BFS depth 1–2 with temporal validity filter          │
│     - Optional: Personalized PageRank from seed entities   │
│  d) Temporal scan (for timeline queries)                   │
│     - Range query on event_time or valid_from              │
│  e) Scope cascade filter                                   │
│     - Search: session → user → agent → org                 │
│     - Merge with scope-based priority weighting            │
│                                                            │
│  STEP 3: MERGE & RERANK                                    │
│  Reciprocal Rank Fusion (RRF):                             │
│                                                            │
│    score(m) = Σ_i  w_i / (k + rank_i(m))                  │
│                                                            │
│  Signal weights (configurable per tenant):                 │
│    relevance:       0.35 (vector similarity)               │
│    recency:         0.25 (decay-adjusted timestamp)        │
│    importance:      0.20 (extracted importance score)       │
│    graph_centrality: 0.10 (PageRank / edge weight)         │
│    access_frequency: 0.10 (reinforcement signal)           │
│                                                            │
│  Optional: RL-trained cross-encoder reranker (paid tier)   │
│  Optional: LLM rerank for top-20 candidates (paid tier)    │
│                                                            │
│  STEP 4: COMPRESS & FORMAT                                 │
│  • Apply token budget constraint                           │
│  • Format as structured context block:                     │
│    - Working memory summary (prepended)                    │
│    - Relevant facts as "Known facts: ..."                  │
│    - Relevant procedures as system instructions            │
│    - Each memory includes citation ID for provenance       │
│  • Cache result in Redis (TTL: session duration)           │
│                                                            │
│  Response includes:                                        │
│    memories: [{ id, kind, content, score, source }]        │
│    context_block: "formatted text for LLM injection"       │
│    token_count: 1847                                       │
│    search_metadata: { latency_ms, signals_used, ... }      │
│                                                            │
│  Latency target: P95 <200ms, P99 <500ms                   │
└───────────────────────────────────────────────────────────┘
```

### 5.3 Edit Flow: User/Admin Governance

```
Client / Admin UI
    │
    │  PATCH /v1/memory/{id}
    │  { content?, tags?, valid_to?, status? }
    ▼
┌─ memory-gateway → memory-admin ───────────────────────────┐
│                                                            │
│  1. Load current memory state                              │
│  2. Compute diff (old vs new)                              │
│  3. Write to memory_audit table:                           │
│     { memory_id, action='update', actor_type='user',       │
│       actor_id, diff: {before: {...}, after: {...}} }      │
│  4. Update memories table                                  │
│  5. Emit MemoryUpdated event to NATS                       │
│  6. Workers re-embed + update vector index                 │
│  7. Workers update graph nodes/edges if entity changed     │
│                                                            │
│  For DELETE:                                               │
│  • Soft delete only (status='deleted', valid_to=now)       │
│  • Audit trail preserved                                   │
│  • Vector index entry marked inactive                      │
│                                                            │
│  For MERGE:                                                │
│  POST /v1/memory/merge { source_ids: [...], target_id }   │
│  • Merge content from sources into target                  │
│  • Mark sources as status='superseded'                     │
│  • Update all graph edges to point to merged entity        │
│  • Full audit trail of the merge operation                 │
│                                                            │
└───────────────────────────────────────────────────────────┘
```

### 5.4 Simulation Flow: Point-in-Time Replay

```
POST /v1/simulate/replay
{ user_id, point_in_time: "2026-01-15T14:30:00Z",
  query: "What medications was the agent aware of?" }
    │
    ▼
┌─ memory-admin ────────────────────────────────────────────┐
│                                                            │
│  1. Query memories WHERE:                                  │
│     tenant_id = ? AND user_id = ?                          │
│     AND ingested_at <= '2026-01-15T14:30:00Z'              │
│     AND (valid_to IS NULL OR valid_to > '2026-01-15...')   │
│     AND status IN ('active','superseded')                  │
│                                                            │
│  2. Reconstruct entity graph at that point in time:        │
│     entities WHERE ingested_at <= point AND                │
│       (valid_to IS NULL OR valid_to > point)               │
│     edges WHERE ingested_at <= point AND                   │
│       (valid_to IS NULL OR valid_to > point)               │
│                                                            │
│  3. Run retrieval pipeline against reconstructed state     │
│     (vector search with historical embeddings)             │
│                                                            │
│  4. Return:                                                │
│     {                                                      │
│       memories_at_time: [...],                             │
│       entity_graph_at_time: {...},                         │
│       provenance_chain: [                                  │
│         { memory → source_episode → original_input }       │
│       ],                                                   │
│       query_result: "At that time, the agent knew about    │
│         medications X, Y, and Z..."                        │
│     }                                                      │
│                                                            │
│  Use cases:                                                │
│  • Healthcare: "What did the AI know when it recommended?" │
│  • Finance: "What facts existed before trade execution?"   │
│  • Legal: "What precedents was the agent aware of?"        │
│  • Debug: "Why did the agent hallucinate at 3:42pm?"       │
└───────────────────────────────────────────────────────────┘
```

---

## 6. Background Job Pipelines

### 6.1 Consolidation (memory-jobs)

```
┌─ PERIODIC JOBS (CronJobs) ────────────────────────────────┐
│                                                            │
│  EVERY 15 MINUTES:                                         │
│  • Decay scoring:                                          │
│    importance = base_importance × e^(-λ × days_since_use)  │
│    λ configurable per tenant (default 0.01)                │
│                                                            │
│  EVERY HOUR:                                               │
│  • Entity deduplication scan:                              │
│    - Find entities with high name/embedding similarity     │
│    - LLM confirms/denies merge candidates                  │
│    - Execute merges with full audit trail                  │
│                                                            │
│  EVERY 6 HOURS:                                            │
│  • Episodic compression:                                   │
│    - Sessions older than 7 days                            │
│    - Summarize episode chains → semantic memory            │
│    - Archive raw episodes to S3                            │
│    - Keep summary + pointers in Postgres                   │
│                                                            │
│  NIGHTLY:                                                  │
│  • Archive: Move memories with importance < threshold       │
│    from active tables to cold storage (S3 + parquet)       │
│  • TTL expiry: Honor retention policies                    │
│  • A-MEM evolution: Re-link notes as new connections       │
│    emerge across the graph                                 │
│  • Metric rollup: Aggregate per-tenant usage stats         │
│  • PII scan: Run redaction policies on new memories        │
│                                                            │
│  ON-DEMAND (event-triggered via NATS):                     │
│  • Contradiction detected → resolve via temporal order     │
│  • Entity merge candidate → LLM confirms/denies           │
│  • Memory count exceeds per-user budget → compress oldest  │
│  • Model migration: re-embed all memories with new model   │
│                                                            │
└───────────────────────────────────────────────────────────┘
```

### 6.2 Connector Ingest Pipeline (connector-ingest)

```
┌─ CONNECTOR SOURCES ───────────────────────────────────────┐
│                                                            │
│  Slack:     Webhook → parse messages → ingest as episodes  │
│  Gmail:     OAuth poll → parse threads → ingest            │
│  Notion:    Webhook → parse pages → ingest                 │
│  Salesforce: Change Data Capture → parse records → ingest  │
│  Custom:    Webhook endpoint + file upload (PDF, CSV)      │
│                                                            │
│  Each connector:                                           │
│  1. Receives raw data from external source                 │
│  2. Normalizes to standard Episode format                  │
│  3. Sets created_by = 'connector'                          │
│  4. Posts to memory-write /internal/ingest/batch           │
│  5. Same extraction pipeline processes it                  │
│                                                            │
│  Connectors run as separate K8s Deployments with           │
│  independent HPA scaling per connector type.               │
└───────────────────────────────────────────────────────────┘
```

---

## 7. API Design (Complete)

### 7.1 Core Memory Endpoints

```
POST   /v1/memory/add
  Headers: Authorization: Bearer <jwt> | X-API-Key: <key>
  Body: {
    messages: [
      { role: "user", content: "I just moved to San Francisco" },
      { role: "assistant", content: "Great! I'll remember that." }
    ],
    user_id: "user_123",
    session_id: "session_abc",       // optional
    agent_id: "support-bot-v2",      // optional
    scope: "user",                   // optional, default: user
    metadata: { source: "chat" }     // optional
  }
  → 202 Accepted
  → { ingest_id: "uuid", status: "processing", episode_ids: ["uuid", ...] }

POST   /v1/memory/search
  Body: {
    query: "Where does the user live?",
    user_id: "user_123",
    scope: "user",                   // optional, default: cascade all
    filters: {                       // optional
      kinds: ["fact", "preference"],
      since: "2025-01-01T00:00:00Z",
      tags: ["location"]
    },
    top_k: 10,                       // optional, default: 10
    score_threshold: 0.3,            // optional, default: 0.0
    include_graph: true              // optional, expand graph neighbors
  }
  → 200 OK
  → {
      memories: [
        { id, kind, content, score, confidence, valid_from, source_episode_id,
          graph_context: { related_entities: [...], edges: [...] } }
      ],
      search_metadata: { latency_ms, signals: ["vector","graph"], ... }
    }

POST   /v1/memory/context
  Body: {
    messages: [{ role: "user", content: "Book my usual restaurant" }],
    user_id: "user_123",
    session_id: "session_abc",
    token_budget: 2000               // max tokens for context block
  }
  → 200 OK
  → {
      context_block: "## Known about this user:\n- Lives in SF since March...",
      memories_used: [{ id, kind, content, score }],
      token_count: 1247,
      working_memory_summary: "Conversation about dinner plans..."
    }

GET    /v1/memory/{id}
PUT    /v1/memory/{id}              // manual correction (audit-logged)
DELETE /v1/memory/{id}              // soft delete (audit-logged)
GET    /v1/memory/all?user_id=&scope=&kind=&since=&until=&page=&limit=
POST   /v1/memory/merge             // merge duplicate memories
```

### 7.2 Graph Endpoints

```
GET    /v1/graph/entities?user_id=&type=&search=
GET    /v1/graph/entity/{id}/relations
POST   /v1/graph/traverse
  Body: { start_entity_id, depth: 2, rel_types: ["works_at"], temporal_filter: {...} }
GET    /v1/graph/timeline?user_id=&entity_id=&from=&to=
POST   /v1/graph/summarize
  Body: { entity_id }  → LLM-generated summary of entity's knowledge graph neighborhood
```

### 7.3 Admin & Governance Endpoints

```
GET    /v1/audit?memory_id=&actor_type=&since=&until=&page=&limit=
POST   /v1/config/memory-rules      // per-tenant extraction/scoring config
GET    /v1/stats?user_id=&tenant_id= // usage stats (memory count, storage, cost)
POST   /v1/memory/reset             // bulk scope deletion (audit-logged)
GET    /v1/policies                 // list governance policies
POST   /v1/policies                 // create retention/redaction/PII rules
POST   /v1/simulate/replay          // point-in-time memory reconstruction
```

### 7.4 Webhook & Streaming

```
POST   /v1/webhooks
  Body: { url, events: ["memory.created","memory.updated","memory.deleted"], secret }
  → Register webhook endpoint for real-time memory events

GET    /v1/events/stream             // SSE stream of memory events (WebSocket also supported)
```

---

## 8. Kubernetes Deployment

### 8.1 Namespace Layout

```
memory-system/               # Shared infrastructure
  ├── postgres (StatefulSet or managed: RDS/Cloud SQL)
  ├── redis (StatefulSet or managed: ElastiCache)
  ├── nats (StatefulSet, JetStream enabled)
  ├── qdrant (StatefulSet, optional — pgvector as default)
  ├── minio (StatefulSet or managed: S3)
  ├── otel-collector (DaemonSet)
  ├── prometheus (StatefulSet)
  ├── grafana (Deployment)
  └── loki (StatefulSet, optional)

memory-app/                  # Application services
  ├── memory-gateway (Deployment, HPA: CPU + RPS)
  │   └── Ingress (external: /v1/*)
  ├── memory-write (Deployment, HPA: queue depth)
  │   └── Service (ClusterIP only)
  ├── memory-retrieve (Deployment, HPA: CPU + RPS)
  │   └── Service (ClusterIP only)
  ├── memory-graph (Deployment, HPA: CPU)
  │   └── Service (ClusterIP only)
  ├── memory-admin (Deployment, HPA: CPU)
  │   └── Ingress (external: /v1/admin/*, /v1/audit/*, /v1/simulate/*)
  ├── memory-jobs (Deployment, scaled by NATS consumer lag)
  │   └── CronJobs (nightly compaction, TTL, metrics)
  ├── billing-meter (Deployment, 1 replica)
  │   └── Service (ClusterIP only)
  └── connector-ingest (Deployment per connector type, HPA)
      └── Service (ClusterIP only)
```

### 8.2 HPA Configuration

```yaml
# memory-gateway: latency-sensitive, scale on RPS
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: memory-gateway
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: memory-gateway
  minReplicas: 2
  maxReplicas: 50
  metrics:
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "500"
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        targetAverageUtilization: 70

# memory-jobs: scale on NATS consumer lag
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: memory-jobs
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: memory-jobs
  minReplicas: 1
  maxReplicas: 20
  metrics:
  - type: External
    external:
      metric:
        name: nats_consumer_pending_messages
      target:
        type: AverageValue
        averageValue: "100"
```

### 8.3 Networking & Security

```
Ingress:
  • memory-gateway and memory-admin only (all other services ClusterIP)
  • TLS termination at Ingress controller
  • WAF rules for API protection

Service mesh (enterprise):
  • Linkerd or Istio for mTLS between all services
  • Mutual TLS for zero-trust internal communication
  • Network policies restricting pod-to-pod traffic

Secrets management:
  • External Secrets Operator → AWS Secrets Manager / Vault
  • No secrets in ConfigMaps or env vars
  • API keys hashed with Argon2 in tenants table
```

---

## 9. Security & Multi-Tenancy Enforcement

### 9.1 Hard Requirements

| Layer | Enforcement |
|-------|------------|
| **API Gateway** | Every request must carry `tenant_id` from auth layer. Never trust client-provided tenant_id. JWT claims include `tenant_id`, `user_id`, `scopes`. |
| **Postgres** | RLS on ALL tables. `SET app.tenant_id` on every connection from pool. |
| **Vector Store** | `tenant_id` in payload metadata. Filter on every query. Qdrant: use payload filtering. pgvector: WHERE clause. |
| **Graph Store** | `tenant_id` on every node and edge. Enforced at query layer in memory-graph service. |
| **S3/MinIO** | Bucket-per-tenant or prefix-per-tenant with IAM policy enforcement. |
| **Redis** | Key prefix: `{tenant_id}:{key}`. No cross-tenant cache hits possible. |
| **NATS** | Subject prefix: `memory.{tenant_id}.*`. Per-tenant authorization. |

### 9.2 Access Scopes

```
Scope hierarchy (most restrictive → broadest):
  session  → only this session's working memory
  user     → all of this user's memories across sessions
  team     → all team members' memories
  project  → app/agent-specific namespace
  agent    → shared across all users of this agent
  org      → global tenant-wide memory (admin-managed)

Retrieval cascade: session → user → agent → org
Each scope adds results with decreasing priority weight.
```

### 9.3 Data Protection

```
PII handling:
  • Optional PII detection during extraction (regex + NER model)
  • Auto-redaction policies per tenant (memory_policies table)
  • GDPR right-to-forget: DELETE cascades to vectors, graph, episodes, audit
  • Audit trail preserved even after deletion (records the delete event)

Encryption:
  • At rest: Postgres TDE or volume encryption
  • In transit: TLS everywhere (mTLS with service mesh)
  • BYOK: Enterprise customers provide KMS key ARN
    Postgres: pgcrypto column-level encryption
    S3: SSE-KMS with customer-managed key

Air-gapped deployment (enterprise):
  • All services run in customer's VPC
  • No external network calls except to customer's LLM endpoint
  • BYOM: LLM calls routed to customer's model (vLLM, Azure OpenAI, etc.)
  • Container images signed and scanned
```

---

## 10. Observability

### 10.1 Per-Tenant Metrics (the sellable dashboard)

```
Performance:
  • memory_hit_rate          — % of retrievals that returned ≥1 result used by LLM
  • retrieval_latency_p95    — breakdown: gateway / vector / graph / rerank / format
  • write_pipeline_lag       — NATS consumer lag (seconds behind real-time)
  • token_savings_ratio      — tokens saved vs. raw conversation context

Quality:
  • hallucination_proxy      — "answer grounded by memory?" signal from LLM feedback
  • contradiction_rate       — % of memories that conflicted with existing facts
  • auto_resolve_rate        — % of contradictions resolved automatically
  • stale_memory_rate        — % of memories with importance below threshold

Governance:
  • memories_created_by_type — breakdown: system / user / admin / connector
  • pii_detections_count     — PII auto-redacted this period
  • retention_expirations    — memories expired by policy
  • audit_events_count       — total governance actions

Cost:
  • llm_extraction_cost      — $ spent on memory extraction LLM calls
  • embedding_cost           — $ spent on embedding generation
  • storage_cost             — $ for Postgres + vector + S3 storage
  • total_cost_per_1k_ops    — blended cost metric
  • projected_savings        — estimated token cost savings from memory compression
```

### 10.2 Implementation Stack

```
• OpenTelemetry SDK in every Rust service (tracing + metrics + logs)
• Traces: Distributed trace across gateway → write → workers → graph
  Every span carries: tenant_id, user_id, request_id, memory_id
• Metrics: Prometheus scrape from /metrics on every service
• Logs: Structured JSON logs → Loki or ELK
  Fields: timestamp, level, tenant_id, user_id, request_id, service, message
• Dashboards: Grafana
  - Operational dashboard (SRE)
  - Per-tenant dashboard (customer-facing, paid tier)
  - Cost attribution dashboard (internal + enterprise customers)
```

---

## 11. Technology Stack Summary

| Component | OSS Default | Enterprise Upgrade | Why |
|-----------|------------|-------------------|-----|
| **Gateway** | Rust / Axum | Same | P99 <5ms, low memory, composable middleware |
| **Services** | Rust / Axum | Same | Consistent stack, memory safety, performance |
| **Async Queue** | NATS JetStream | Same (or Kafka) | Lightweight, fast, built-in persistence |
| **Relational DB** | PostgreSQL + RLS | Same (managed) | Battle-tested, RLS for tenant isolation |
| **Vector Store** | pgvector (in Postgres) | Qdrant / Pinecone | Start simple, upgrade when scale demands |
| **Graph Store** | Postgres tables (entities/edges) | Neo4j / FalkorDB | Start simple, upgrade for heavy graph workloads |
| **Cache** | Redis / Valkey | Same (managed) | Hot memory, rate limiting, idempotency |
| **Object Store** | MinIO | S3 / GCS / Azure Blob | Episodic logs, archived memories |
| **Event Stream** | NATS JetStream | Kafka / RedPanda | Webhooks, CDC, event sourcing |
| **LLM** | gpt-4.1-nano via LiteLLM | BYOM (any provider) | Cheapest for extraction ops |
| **Embeddings** | text-embedding-3-small | BYOM (Cohere, Voyage, local) | Good quality, low cost |
| **Observability** | OpenTelemetry + Prometheus + Grafana | Datadog / Langfuse | Full stack observability |
| **Deployment** | Docker Compose (dev) | Kubernetes + Helm | OSS → production path |
| **Service Mesh** | None (dev) | Linkerd / Istio | mTLS for enterprise |

### Why This Stack

The "Postgres for everything" approach for OSS means **one dependency to self-host** (plus Redis and NATS). This is critical for developer adoption — `docker compose up` and you have a working memory platform. The upgrade path to dedicated vector/graph stores is clean because the storage layer is abstracted behind traits.

---

## 12. SDK Design (Framework-Agnostic)

### 12.1 Minimal API Surface

```python
# Python SDK — "one import, five methods"
from memoryinfra import MemoryClient

memory = MemoryClient(api_key="mk_...", base_url="https://api.memoryinfra.dev")

# Add memories (async processing)
result = memory.add(
    messages=[
        {"role": "user", "content": "I just moved to San Francisco"},
        {"role": "assistant", "content": "Welcome to SF!"}
    ],
    user_id="user_123",
    session_id="session_abc",
    metadata={"source": "chat"}
)

# Search memories
results = memory.search(
    query="Where does the user live?",
    user_id="user_123",
    top_k=5
)

# Get formatted context for LLM injection
context = memory.context(
    messages=[{"role": "user", "content": "Book my usual restaurant"}],
    user_id="user_123",
    token_budget=2000
)
# context.context_block → inject into system prompt

# List / update / delete
memories = memory.list(user_id="user_123", kind="preference")
memory.update(id="uuid", content="Lives in Oakland", tags=["location"])
memory.delete(id="uuid")
```

### 12.2 Framework Integrations

```python
# LangChain integration
from memoryinfra.integrations.langchain import MemoryInfraMemory
memory = MemoryInfraMemory(api_key="mk_...", user_id="user_123")
chain = ConversationChain(llm=llm, memory=memory)

# CrewAI integration
from memoryinfra.integrations.crewai import MemoryInfraStorage
crew = Crew(agents=[...], memory=True, memory_config={"provider": MemoryInfraStorage(...)})

# MCP Server (Claude, Cursor, Windsurf)
# Runs as: memory-mcp-server --api-key mk_... --user-id user_123
# Provides tools: memory_add, memory_search, memory_context
```

### 12.3 SDKs to Ship

| SDK | Priority | Timeline |
|-----|---------|---------|
| Python | P0 (launch) | Week 4 |
| TypeScript/Node | P0 (launch) | Week 4 |
| Rust | P1 | Week 8 |
| Go | P2 | Week 12 |
| MCP Server | P0 (launch) | Week 4 |
| LangChain plugin | P0 (launch) | Week 4 |
| LlamaIndex plugin | P1 | Week 6 |
| CrewAI plugin | P1 | Week 6 |
| AutoGen plugin | P2 | Week 10 |
| Strands SDK plugin | P2 | Week 10 |

---

## 13. MVP Build Order (Fast to Market)

### Sprint 1–2 (Weeks 1–4): Foundation

```
✅ memory-gateway (Rust/Axum)
   - JWT + API key auth
   - Tenant resolution + RLS setup
   - Rate limiting (Redis)
   - Request routing

✅ PostgreSQL schema
   - tenants, memories, episodes, memory_vectors tables
   - RLS policies on all tables
   - pgvector extension + HNSW index

✅ memory-write
   - Episodic store (episodes table + S3)
   - Basic LLM extraction (gpt-4.1-nano via LiteLLM)
   - Embedding generation + vector upsert
   - Idempotency (content hash dedup)

✅ memory-retrieve
   - Vector similarity search (pgvector)
   - Temporal validity filter
   - Basic scope cascade (user → org)
   - Token budget compression

✅ Python SDK + TypeScript SDK
   - add() / search() / context() / list() / update() / delete()

✅ Docker Compose for self-hosting
✅ MCP Server
✅ LangChain plugin
```

### Sprint 3–4 (Weeks 5–8): Graph + Intelligence

```
✅ entities + edges tables (Postgres graph)
✅ memory-graph service
   - Entity extraction + dedup
   - Edge upsert with temporal validity
   - BFS traversal + temporal filter
   - Basic PageRank implementation

✅ Hybrid retrieval in memory-retrieve
   - Vector + BM25 + graph expansion
   - RRF reranking
   - Query intent classifier (adaptive router)

✅ memory-jobs (NATS workers)
   - Async extraction pipeline
   - Entity dedup scanner
   - Basic decay scoring

✅ memory_audit table + basic audit logging
✅ LlamaIndex + CrewAI plugins
✅ Rust SDK
```

### Sprint 5–6 (Weeks 9–12): Governance + Observability

```
✅ memory-admin service
   - Memory CRUD UI API
   - Audit log browser
   - Merge operations
   - Memory policies (retention, PII redaction)
   - Simulation/replay endpoint

✅ Consolidation pipeline
   - Episodic compression (summarize old sessions)
   - TTL expiry
   - A-MEM evolution (re-link notes)
   - Contradiction detection + resolution

✅ Observability
   - OpenTelemetry tracing across all services
   - Prometheus metrics + Grafana dashboards
   - Per-tenant metrics (hit rate, latency, cost)

✅ Working memory (Redis-backed session state)
✅ Procedural memory extraction
✅ Webhook system (memory events → customer endpoints)
✅ Go SDK
```

### Sprint 7–8 (Weeks 13–16): Production Hardening

```
✅ Multi-tenancy at scale
   - Tenant config hot-reload
   - Usage metering + billing hooks
   - Quota enforcement

✅ Kubernetes Helm chart
   - HPA for all services
   - PodDisruptionBudgets
   - Resource requests/limits
   - Health checks (liveness, readiness, startup)

✅ Security hardening
   - mTLS option (Linkerd/Istio)
   - Secret rotation
   - Penetration testing

✅ Benchmarking
   - DMR benchmark: target ≥94%
   - LongMemEval: target +18.5% vs baseline
   - Multi-hop QA: target +20% vs standard RAG
   - Latency benchmarks: P95 <200ms retrieval

✅ Neo4j integration option (enterprise graph backend)
✅ Qdrant integration option (enterprise vector backend)
✅ BYOM support (LiteLLM → customer model endpoint)
✅ Documentation site + API reference
```

---

## 14. Benchmarking Strategy

| Benchmark | What It Tests | Target | Measurement |
|-----------|--------------|--------|-------------|
| **DMR** | Single-fact recall across sessions | ≥94% (match Zep) | Automated test suite |
| **LongMemEval** | Cross-session synthesis, temporal reasoning | +18.5% vs baseline | Automated test suite |
| **LOCCO** | 3,080 interactions across 100 users | Track accuracy + latency | Monthly regression |
| **Multi-hop QA** | Relational reasoning across graph memories | +20% vs standard RAG | Automated test suite |
| **Latency** | End-to-end retrieval performance | P95 <200ms, P99 <500ms | Continuous monitoring |
| **Write throughput** | Ingestion capacity | 10K ops/sec per replica | Load testing |
| **Cost efficiency** | Per-op extraction cost | <$0.003/op blended | Monthly cost review |

---

## 15. References

1. Chhikara et al. "Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory." arXiv:2504.19413, 2025.
2. Rasmussen et al. "Zep: A Temporal Knowledge Graph Architecture for Agent Memory." arXiv:2501.13956, 2025.
3. Packer et al. "MemGPT: Towards LLMs as Operating Systems." arXiv:2310.08560, 2023.
4. Xu et al. "A-MEM: Agentic Memory for LLM Agents." arXiv:2502.12110, 2025. NeurIPS 2025.
5. Gutiérrez et al. "HippoRAG: Neurobiologically Inspired Long-Term Memory for Large Language Models." arXiv:2405.14831, 2024. NeurIPS 2024.
6. Zhang et al. "A Survey on the Memory Mechanism of Large Language Model based Agents." arXiv:2404.13501, 2024. ACM TOIS.
7. "From Human Memory to AI Memory: A Survey on Memory Mechanisms in the Era of LLMs." arXiv:2504.15965, 2025.
8. LangChain. "Long-term memory." docs.langchain.com/docs/concepts/memory.
9. Mem0. "Platform Documentation." docs.mem0.ai.
10. Zep. "Graphiti: Build Real-Time Knowledge Graphs for AI Agents." github.com/getzep/graphiti.
