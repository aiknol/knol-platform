<p align="center">
  <h1 align="center">Knol Platform</h1>
  <p align="center">
    <strong>Context engineering infrastructure for AI applications</strong>
  </p>
  <p align="center">
    Built in Rust. Open core. Self-hostable.
  </p>
</p>

<p align="center">
  <a href="https://github.com/aiknol/knol-platform/actions"><img src="https://github.com/aiknol/knol-platform/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License"></a>
  <a href="https://github.com/aiknol/knol-platform"><img src="https://img.shields.io/badge/rust-1.77+-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#why-knol">Why Knol</a> &middot;
  <a href="#features">Features</a> &middot;
  <a href="#sdks">SDKs</a> &middot;
  <a href="#architecture">Architecture</a> &middot;
  <a href="https://docs.aiknol.com">Docs</a>
</p>

---

Knol gives your AI agents **persistent, structured memory** — not just vector search. Every conversation turn is processed through an intelligent extraction pipeline that builds a **knowledge graph** of entities, relationships, and facts alongside traditional semantic embeddings. Memories are grounded with source citations, verified for accuracy, and automatically resolved when they conflict.

> **Looking for the OSS-only repo?** See [github.com/aiknol/knol](https://github.com/aiknol/knol) — a standalone copy of `knol-oss/` that you can run without the enterprise services.

```python
from memory_sdk import MemoryClient

client = MemoryClient(api_key="your-key", base_url="http://localhost:3000")

# Store a memory
client.add("I work at Acme Corp as a senior engineer. I prefer dark mode.", user_id="user-123")

# Search with hybrid retrieval (vector + graph + temporal)
results = client.search("What does the user do for work?", user_id="user-123")
# -> [{"content": "User works at Acme Corp as a senior engineer", "confidence": 0.95, ...}]
```

## Quick Start

**One command to run the OSS core:**

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

**Full stack (OSS + Enterprise services):**

```bash
cp .env.example .env
# Edit .env with your secrets (ADMIN_JWT_SECRET, ADMIN_ENCRYPTION_KEY, etc.)

docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

**Frontend development:**

```bash
cd frontend && npm install
./scripts/frontend-services.sh start
```

| Service           | URL                           |
|-------------------|-------------------------------|
| API Gateway       | `http://localhost:3000`       |
| Admin API Health  | `http://localhost:3001/health`|
| Main Website      | `http://localhost:3005`       |
| Admin Panel       | `http://localhost:3006`       |
| Cloud Dashboard   | `http://localhost:3007`       |
| Demo UI           | `http://localhost:3008`       |
| Documentation     | `http://localhost:3009`       |

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
- **Conflict detection** — Automatic contradiction, duplicate, and refinement detection
- **Memory decay** — Configurable importance decay (exponential, linear, step) with access boost
- **Citation grounding** — Source quotes linked to every extracted memory
- **Content triage** — Skip trivial messages (greetings, acks) without LLM calls
- **Multi-provider LLM** — Anthropic Claude, OpenAI GPT-4o, Google Gemini with hot-swappable config
- **Webhook events** — Subscribe to memory.created, entity.created, conflict.detected, and more
- **Guardrails** — Input validation, prompt injection detection, PII filtering

### Enterprise (Source-Available)

- **Admin panel** — Web UI for managing LLM providers, API keys, guardrails, and system config
- **Multi-tenant** — Workspace isolation with per-tenant API keys and billing
- **Billing & usage** — Usage tracking, plan enforcement, and Stripe integration
- **Background jobs** — NATS-based async processing for heavy operations
- **Ingest pipeline** — Bulk memory ingestion with deduplication
- **Marketing automation** — Scheduled social media and content campaigns

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
│   (:8081)   │  │    (:8082)      │  │   (:8084)      │
│             │  │                 │  │                │
│ Episodes -> │  │ Vector search + │  │ Config, keys,  │
│ NATS queue  │  │ Graph traversal │  │ guardrails     │
└──────┬──────┘  │ + Temporal rank │  └────────────────┘
       │         └────────┬────────┘
       ▼                  │
┌──────────────┐          │
│Graph Service │          │
│  (:8083)     │◀─────────┘
│              │
│ LLM extract  │    ┌────────────┐  ┌───────┐  ┌───────┐
│ Embed vectors│───>│ PostgreSQL │  │ Redis │  │ NATS  │
│ Detect       │    │ + pgvector │  │ Cache │  │  JS   │
│  conflicts   │    └────────────┘  └───────┘  └───────┘
│ Fire webhooks│
└──────────────┘
```

**Write path:** API Gateway -> Write Service -> NATS -> Graph Service -> LLM extraction -> Embedding generation -> Conflict detection -> PostgreSQL + pgvector -> Webhook dispatch

**Read path:** API Gateway -> Retrieve Service -> Query embedding -> Vector search + Graph traversal + Temporal ranking -> Decay-adjusted results

## Project Structure

```
knol-platform/
├── knol-oss/                  # Open source core (Apache 2.0)
│   ├── crates/
│   │   ├── memory-common/     # Shared types, config, webhook definitions
│   │   ├── memory-db/         # Database pool, migrations, tenant isolation
│   │   ├── memory-cache/      # Redis client wrapper
│   │   ├── memory-queue/      # NATS JetStream producer/consumer
│   │   ├── memory-llm/        # LLM providers, extraction, embedding, conflict
│   │   ├── memory-vector/     # pgvector storage and similarity search
│   │   ├── memory-graph/      # Knowledge graph CRUD and traversal
│   │   ├── service-gateway/   # API gateway with auth and routing
│   │   ├── service-write/     # Write service (episodes -> NATS)
│   │   ├── service-retrieve/  # Search service (hybrid retrieval)
│   │   └── service-graph/     # Graph builder (extraction + embedding + webhooks)
│   └── sdk/                   # Python, TypeScript, MCP SDKs
├── knol-enterprise/           # Enterprise extensions (source-available)
│   └── crates/
│       ├── service-admin/     # Admin API + demo endpoints
│       ├── service-tenant/    # Multi-tenant workspace management
│       ├── service-billing/   # Usage tracking + Stripe
│       ├── service-jobs/      # Background job processing
│       ├── service-ingest/    # Bulk memory ingestion
│       └── service-marketing/ # Marketing automation
├── frontend/                  # Next.js web applications
│   ├── web/                   # Marketing site
│   ├── cloud/                 # Cloud dashboard
│   ├── admin/                 # Admin panel
│   ├── demo/                  # Interactive demo
│   └── docs/                  # Documentation site
├── deploy/                    # Production deployment (Docker Compose, Caddy)
├── tests/                     # E2E integration tests
└── scripts/                   # Dev and CI utilities
```

## Production Deployment

See [deploy/](deploy/) for the production Docker Compose setup with:
- Caddy reverse proxy with automatic TLS via Let's Encrypt
- Rolling deployments with health checks
- Resource limits tuned for a 4 vCPU / 8 GB VPS
- Neon Postgres + Upstash Redis (external managed services)

```bash
# On your VPS
cp deploy/.env.production.example deploy/.env.production
# Fill in real values
./deploy/deploy.sh v1.0.9
```

## Configuration

All configuration is managed through the admin panel or environment variables:

| Setting | Env Var | Default | Description |
|---------|---------|---------|-------------|
| LLM Provider | `LLM_PROVIDER` | `anthropic` | anthropic, openai, gemini |
| LLM Model | `LLM_MODEL` | `claude-haiku-4-5-20251001` | Model identifier |
| Embedding Provider | `EMBEDDING_PROVIDER` | `openai` | openai, voyage, gemini, local |
| Conflict Detection | `CONFLICT_DETECTION_ENABLED` | `true` | Auto-detect contradictions |
| Webhooks | `WEBHOOKS_ENABLED` | `true` | Fire webhook events |
| Content Triage | `TRIAGE_ENABLED` | `true` | Skip trivial messages |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Run `cargo fmt` and `cargo clippy` before committing
4. Submit a pull request

## License

- **OSS Core** (`knol-oss/`, `frontend/`, `deploy/`, `tests/`, `scripts/`): [Apache License 2.0](LICENSE)
- **Enterprise** (`knol-enterprise/`): [Source-available](knol-enterprise/LICENSE) — free to read and self-host with a license

## Links

- [Website](https://aiknol.com)
- [Documentation](https://docs.aiknol.com)
- [Cloud Dashboard](https://cloud.aiknol.com)
- [Live Demo](https://demo.aiknol.com)
- [OSS Core Repo](https://github.com/aiknol/knol)
