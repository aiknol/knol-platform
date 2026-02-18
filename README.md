<p align="center">
  <h1 align="center">Knol</h1>
  <p align="center">
    <strong>Long-term memory infrastructure for LLM applications</strong>
  </p>
  <p align="center">
    Built in Rust. Open core. Self-hostable.
  </p>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#why-knol">Why Knol</a> &middot;
  <a href="#features">Features</a> &middot;
  <a href="#sdks">SDKs</a> &middot;
  <a href="#architecture">Architecture</a> &middot;
  <a href="https://aiknol.com/docs">Docs</a>
</p>

---

Knol gives your AI agents **persistent, structured memory** — not just vector search. Every conversation turn is processed through an intelligent extraction pipeline that builds a **knowledge graph** of entities, relationships, and facts alongside traditional semantic embeddings. Memories are grounded with source citations, verified for accuracy, and automatically resolved when they conflict.

```python
from memory_sdk import MemoryClient

client = MemoryClient(api_key="your-key", base_url="http://localhost:3000")

# Store a memory
client.add("I work at Acme Corp as a senior engineer. I prefer dark mode.", user_id="user-123")

# Search with hybrid retrieval (vector + graph + temporal)
results = client.search("What does the user do for work?", user_id="user-123")
# → [{"content": "User works at Acme Corp as a senior engineer", "confidence": 0.95, ...}]
```

## Quick Start

**One command to run everything:**

```bash
docker compose -f docker-compose.oss.yml up -d --build
```

The API is now live at `http://localhost:3000`. Try it:

```bash
# Add a memory
curl -X POST http://localhost:3000/v1/memory \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key" \
  -d '{"content": "User prefers TypeScript over JavaScript", "user_id": "user-123"}'

# Search memories
curl -X POST http://localhost:3000/v1/memory/search \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key" \
  -d '{"query": "programming preferences", "user_id": "user-123"}'
```

**Full stack (OSS + Admin Panel):**

```bash
export ADMIN_JWT_SECRET='replace-with-random-32-plus-char-secret'
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

| Service           | URL                          |
|-------------------|------------------------------|
| API Gateway       | `http://localhost:3000`      |
| Admin Panel       | `http://localhost:3006`      |
| Demo UI           | `http://localhost:8080`      |
| Admin API Health  | `http://localhost:3001/health`|

## Why Knol

Most memory solutions offer vector search and call it a day. Knol goes further:

**Hybrid retrieval** — Queries hit vector similarity, knowledge graph traversal, and temporal scoring simultaneously. The system classifies query intent and routes to the optimal strategy automatically.

**Knowledge graph, not just vectors** — Every extraction builds entities, relationships, and edges. Ask "who does Sarah work with?" and get graph-traversed answers, not just keyword matches.

**Write-time intelligence** — Embeddings are generated at write time (not query time). Conflicts are detected automatically. Duplicate memories are deduplicated. Outdated facts are superseded.

**Grounded and verifiable** — Every extracted memory includes a source citation linking back to the original text. Optional LLM-based verification scores how well the source supports each memory.

**Built for production** — Rust microservices. NATS JetStream for event processing. PostgreSQL with pgvector. Redis caching. Multi-tenant isolation. JWT auth. Webhook notifications.

## Features

### Core (OSS — Apache 2.0)

- **Intelligent extraction** — LLM-powered memory extraction with entity/relationship/fact detection
- **Hybrid search** — Vector similarity + knowledge graph + temporal scoring in one query
- **Knowledge graph** — N-hop traversal, path finding, entity neighbors, relationship typing
- **Embedding generation** — OpenAI, Voyage AI, Google Gemini, or local embeddings at write time
- **Conflict detection** — Automatic contradiction, duplicate, and refinement detection with configurable resolution (newest-wins, highest-confidence, manual review)
- **Memory decay** — Configurable importance decay (exponential, linear, step) with access boost
- **Citation grounding** — Source quotes linked to every extracted memory
- **Content triage** — Skip trivial messages (greetings, acks) without LLM calls
- **Multi-provider LLM** — Anthropic Claude, OpenAI GPT-4o, Google Gemini with hot-swappable config
- **Webhook events** — Subscribe to memory.created, entity.created, conflict.detected, and more
- **Export/Import** — Bulk memory export (JSON/CSV) and import with conflict strategies
- **Guardrails** — Input validation, prompt injection detection, PII filtering

### Enterprise (Commercial License)

- **Admin panel** — Web UI for managing LLM providers, API keys, guardrails, and system config
- **Dynamic provider switching** — Change LLM provider/model without restart (auto-refreshes from DB)
- **Token usage tracking** — Per-tenant, per-model cost monitoring with cache hit tracking
- **Audit logging** — Full audit trail of memory operations
- **SSO / RBAC** — Enterprise authentication and role-based access control

## SDKs

### Python

```bash
pip install memory-sdk
```

```python
from memory_sdk import MemoryClient, AsyncMemoryClient

# Sync
client = MemoryClient(api_key="key", base_url="http://localhost:3000")
client.add("User prefers dark mode", user_id="u1")
results = client.search("preferences", user_id="u1")

# Async
async with AsyncMemoryClient(api_key="key") as client:
    await client.add("User prefers dark mode", user_id="u1")
    results = await client.search("preferences", user_id="u1")
```

### TypeScript / JavaScript

```bash
npm install @knol/sdk
```

```typescript
import { KnolClient } from '@knol/sdk';

const client = new KnolClient({ apiKey: 'key', baseUrl: 'http://localhost:3000' });

await client.addMemory({ content: 'User prefers dark mode', userId: 'u1' });
const results = await client.search({ query: 'preferences', userId: 'u1' });
```

### Framework Integrations

**LangChain:**
```python
from memory_sdk.integrations.langchain import KnolMemory, KnolRetriever

memory = KnolMemory(api_key="key", user_id="u1")
retriever = KnolRetriever(api_key="key", user_id="u1")
```

**CrewAI:**
```python
from memory_sdk.integrations.crewai import KnolCrewMemory

memory = KnolCrewMemory(api_key="key", user_id="u1")
```

**MCP Server:**
```bash
npm install -g @knol/mcp-server
KNOL_API_KEY=your-key knol-mcp
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      API Gateway (:3000)                     │
│                  Auth · Routing · Rate Limiting               │
└──────┬──────────────────┬───────────────────┬────────────────┘
       │                  │                   │
┌──────▼──────┐  ┌────────▼────────┐  ┌───────▼───────┐
│Write Service│  │Retrieve Service │  │ Admin Service  │
│   (:8081)   │  │    (:8082)      │  │   (:3001)      │
│             │  │                 │  │                │
│ Episodes →  │  │ Vector search + │  │ Config, keys,  │
│ NATS queue  │  │ Graph traversal │  │ guardrails     │
└──────┬──────┘  │ + Temporal rank │  └────────────────┘
       │         └────────┬────────┘
       ▼                  │
┌──────────────┐          │
│Graph Service │          │
│  (:8083)     │◄─────────┘
│              │
│ LLM extract  │    ┌────────────┐  ┌───────┐  ┌───────┐
│ Embed vectors│───►│ PostgreSQL │  │ Redis │  │ NATS  │
│ Detect       │    │ + pgvector │  │ Cache │  │  JS   │
│  conflicts   │    └────────────┘  └───────┘  └───────┘
│ Fire webhooks│
└──────────────┘
```

**Write path:** API Gateway → Write Service → NATS → Graph Service → LLM extraction → Embedding generation → Conflict detection → PostgreSQL + pgvector → Webhook dispatch

**Read path:** API Gateway → Retrieve Service → Query embedding → Vector search + Graph traversal + Temporal ranking → Decay-adjusted results

## Configuration

All configuration is managed through the admin panel or environment variables. Key settings:

| Setting | Env Var | Default | Description |
|---------|---------|---------|-------------|
| LLM Provider | `LLM_PROVIDER` | `anthropic` | anthropic, openai, gemini |
| LLM Model | `LLM_MODEL` | `claude-haiku-4-5-20251001` | Model identifier |
| Embedding Provider | `EMBEDDING_PROVIDER` | `openai` | openai, voyage, gemini, local |
| Conflict Detection | `CONFLICT_DETECTION_ENABLED` | `true` | Auto-detect contradictions |
| Webhooks | `WEBHOOKS_ENABLED` | `true` | Fire webhook events |
| Content Triage | `TRIAGE_ENABLED` | `true` | Skip trivial messages |
| LLM Cache | `LLM_CACHE_ENABLED` | `true` | Redis-backed response cache |

## Documentation

- [Architecture Deep Dive](knol-oss/ARCHITECTURE.html)
- [Docker Stack Guide](docs/docker-stack.md)
- [OSS vs Commercial Boundary](docs/oss-vs-commercial.md)
- [API Documentation](https://aiknol.com/docs)

## Project Structure

```
memorylayer/
├── knol-oss/                  # Open source (Apache 2.0)
│   ├── crates/
│   │   ├── memory-common/     # Shared types, config, webhook definitions
│   │   ├── memory-db/         # Database pool, migrations, tenant isolation
│   │   ├── memory-cache/      # Redis client wrapper
│   │   ├── memory-queue/      # NATS JetStream producer/consumer
│   │   ├── memory-llm/        # LLM providers, extraction, embedding, decay, conflict
│   │   ├── memory-vector/     # pgvector storage and similarity search
│   │   ├── memory-graph/      # Knowledge graph CRUD and traversal
│   │   ├── service-gateway/   # API gateway with auth and routing
│   │   ├── service-write/     # Write service (episodes → NATS)
│   │   ├── service-retrieve/  # Search service (hybrid retrieval)
│   │   └── service-graph/     # Graph builder (extraction + embedding + webhooks)
│   └── sdk/
│       ├── python/            # Python SDK + LangChain + CrewAI integrations
│       ├── typescript/        # TypeScript/JavaScript SDK
│       └── mcp/               # MCP server for Claude, Cursor, etc.
├── knol-enterprise/           # Commercial features
├── knol-web/                  # Admin panel (Next.js)
├── knol-demo/                 # Interactive demo UI
└── docker-compose.oss.yml     # One-command deployment
```

## License

- `knol-oss/` — [Apache License 2.0](knol-oss/LICENSE)
- `knol-enterprise/` — Commercial License
- `knol-web/` — All Rights Reserved

## Contributing

Contributions to `knol-oss/` are welcome under the Apache 2.0 license. Please open an issue to discuss significant changes before submitting a PR.
