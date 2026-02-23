import { Metadata } from 'next';
import CodeBlock from '@/components/ui/CodeBlock';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Documentation'),
  description: 'Complete API reference and SDK guides for Knol — context engineering infrastructure for AI applications.',
};

const endpoints = [
  { method: 'POST', path: '/v1/memory', desc: 'Store a new memory (async extraction)' },
  { method: 'POST', path: '/v1/memory/batch', desc: 'Store multiple memories in one call' },
  { method: 'POST', path: '/v1/memory/search', desc: 'Hybrid search (vector + BM25 + graph)' },
  { method: 'GET', path: '/v1/memory/:id', desc: 'Get a specific memory by ID' },
  { method: 'PUT', path: '/v1/memory/:id', desc: 'Update a memory' },
  { method: 'DELETE', path: '/v1/memory/:id', desc: 'Delete a memory' },
  { method: 'GET', path: '/v1/graph/entities', desc: 'List entities in knowledge graph' },
  { method: 'GET', path: '/v1/graph/entities/:id/edges', desc: 'Get entity relationships (N-hop)' },
  { method: 'POST', path: '/v1/memory/export', desc: 'Export memories (JSON)' },
  { method: 'POST', path: '/v1/memory/import', desc: 'Import memories (JSON)' },
  { method: 'GET', path: '/v1/admin/memories', desc: 'List all memories (admin)' },
  { method: 'GET', path: '/v1/webhooks', desc: 'List webhooks' },
  { method: 'POST', path: '/v1/webhooks', desc: 'Create a webhook' },
  { method: 'DELETE', path: '/v1/webhooks/:id', desc: 'Delete a webhook' },
  { method: 'GET', path: '/v1/admin/audit', desc: 'Browse audit log' },
  { method: 'GET', path: '/health', desc: 'Health check' },
  { method: 'GET', path: '/metrics', desc: 'Prometheus metrics' },
];

const storeExample = `curl -X POST http://localhost:3000/v1/memory \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "content": "User prefers dark mode and concise responses",
    "user_id": "user-123",
    "metadata": {
      "source": "settings",
      "session_id": "sess-456"
    }
  }'

# Response
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "accepted",
  "message": "Memory queued for processing"
}`;

const searchExample = `curl -X POST http://localhost:3000/v1/memory/search \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "What does the user prefer?",
    "user_id": "user-123",
    "limit": 5,
    "memory_types": ["semantic", "episodic"]
  }'

# Response — hybrid retrieval fuses vector + BM25 + graph
{
  "results": [
    {
      "id": "550e8400-...",
      "content": "User prefers dark mode and concise responses",
      "memory_type": "semantic",
      "score": 0.94,
      "created_at": "2026-02-15T10:30:00Z"
    }
  ],
  "query_intent": "preference",
  "retrieval_strategy": "vector_primary"
}`;

const pythonSdkExample = `from knol import KnolClient

client = KnolClient(
    base_url="http://localhost:3000",
    api_key="your-api-key"
)

# Store — extraction & embedding happen automatically
memory = client.add(
    content="User prefers dark mode and concise responses",
    user_id="user-123",
    metadata={"source": "settings"}
)

# Search — hybrid retrieval (vector + BM25 + graph)
results = client.search(
    query="user preferences",
    user_id="user-123",
    limit=5
)

# Knowledge graph
entities = client.list_entities(user_id="user-123")

# CRUD
memory = client.get(memory_id="550e8400-...")
client.update(memory_id="550e8400-...", content="Updated content")
client.delete(memory_id="550e8400-...")`;

const asyncPythonExample = `from knol import AsyncKnolClient
import asyncio

async def main():
    client = AsyncKnolClient(
        base_url="http://localhost:3000",
        api_key="your-api-key"
    )

    # Async store + search
    await client.add(
        content="User prefers TypeScript and functional patterns",
        user_id="user-456"
    )

    results = await client.search(
        query="programming preferences",
        user_id="user-456"
    )

asyncio.run(main())`;

const tsSdkExample = `import { KnolClient } from '@knol-dev/sdk';

const knol = new KnolClient({
  baseUrl: 'http://localhost:3000',
  apiKey: 'your-api-key',
});

// Store
await knol.add({
  content: 'User prefers TypeScript and functional patterns',
  userId: 'user-123',
});

// Search
const results = await knol.search({
  query: 'programming preferences',
  userId: 'user-123',
});

// Knowledge graph
const entities = await knol.listEntities({ userId: 'user-123' });`;

const langchainExample = `from knol.langchain import KnolMemory
from langchain.agents import AgentExecutor

memory = KnolMemory(
    base_url="http://localhost:3000",
    api_key="your-api-key",
    user_id="user-123"
)

# Use as LangChain memory backend
agent = AgentExecutor(
    ...,
    memory=memory
)`;

const crewaiExample = `from knol.crewai import KnolMemory
from crewai import Crew

memory = KnolMemory(
    base_url="http://localhost:3000",
    api_key="your-api-key"
)

crew = Crew(
    agents=[...],
    tasks=[...],
    memory=memory
)`;

const webhookExample = `curl -X POST http://localhost:3000/v1/webhooks \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://your-app.com/webhook",
    "events": ["memory.created", "conflict.detected"],
    "active": true
  }'

# Response
{
  "id": "webhook-550e8400-e29b",
  "url": "https://your-app.com/webhook",
  "events": ["memory.created", "conflict.detected"],
  "active": true,
  "created_at": "2026-02-15T10:30:00Z"
}`;

const batchExample = `curl -X POST http://localhost:3000/v1/memory/batch \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "memories": [
      {
        "content": "User prefers dark mode",
        "user_id": "user-123",
        "metadata": {"source": "settings"}
      },
      {
        "content": "User knows TypeScript and Rust",
        "user_id": "user-123",
        "metadata": {"source": "profile"}
      }
    ]
  }'

# Response
{
  "inserted": 2,
  "ids": ["550e8400-e29b-41d4-...", "550e8400-e29c-41d4-..."],
  "status": "accepted"
}`;

export default function DocsPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl md:text-4xl font-bold text-dark-50 mb-4">Documentation</h1>
        <p className="text-dark-300 text-lg mb-12">
          Complete reference for the Knol REST API, Python SDK, TypeScript SDK, and framework integrations.
        </p>

        {/* Quick Start */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Quick Start</h2>
          <CodeBlock
            code={`# Option 1: Docker Compose (recommended)
docker compose up -d

# Option 2: pip install
pip install knol

# Option 3: npm install
npm install @knol-dev/sdk

# Option 4: Build from source
cd knol-oss
cargo build --workspace --release`}
            language="bash"
            title="Setup"
          />
        </section>

        {/* Authentication */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Authentication</h2>
          <p className="text-dark-300 mb-4">
            All API requests require an API key in the Authorization header. Keys are SHA-256 hashed and stored securely.
            Create keys via the admin dashboard or admin API.
          </p>
          <div className="code-block">
            <code>Authorization: Bearer YOUR_API_KEY</code>
          </div>
        </section>

        {/* Endpoints */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">API Endpoints</h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Method</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Path</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                {endpoints.map((ep) => (
                  <tr key={ep.path + ep.method} className="border-b border-dark-600/20">
                    <td className="py-3 px-4">
                      <span className={`font-mono text-xs px-2 py-0.5 rounded ${
                        ep.method === 'GET' ? 'bg-green-900/30 text-green-400' :
                        ep.method === 'POST' ? 'bg-blue-900/30 text-blue-400' :
                        ep.method === 'PUT' ? 'bg-yellow-900/30 text-yellow-400' :
                        'bg-red-900/30 text-red-400'
                      }`}>{ep.method}</span>
                    </td>
                    <td className="py-3 px-4 font-mono text-dark-200">{ep.path}</td>
                    <td className="py-3 px-4 text-dark-300">{ep.desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Store Memory */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Store a Memory</h2>
          <p className="text-dark-300 mb-4">
            Memories are accepted instantly and processed asynchronously. The pipeline extracts entities,
            generates embeddings, detects conflicts, and fires webhook events — all in the background.
          </p>
          <CodeBlock code={storeExample} language="bash" title="POST /v1/memory" />
        </section>

        {/* Search */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Search Memories</h2>
          <p className="text-dark-300 mb-4">
            The search endpoint uses adaptive hybrid retrieval. It classifies query intent
            (preference, temporal, relational, or general) and fuses vector, BM25, and graph signals
            via Reciprocal Rank Fusion for optimal results.
          </p>
          <CodeBlock code={searchExample} language="bash" title="POST /v1/memory/search" />
        </section>

        {/* Python SDK */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Python SDK</h2>
          <p className="text-dark-300 mb-4">
            Install with <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">pip install knol</code>.
            Both sync and async clients are included.
          </p>
          <div className="space-y-6">
            <CodeBlock code={pythonSdkExample} language="python" title="Sync Client" />
            <CodeBlock code={asyncPythonExample} language="python" title="Async Client" />
          </div>
        </section>

        {/* TypeScript SDK */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">TypeScript SDK</h2>
          <p className="text-dark-300 mb-4">
            Install with <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">npm install @knol-dev/sdk</code>.
            Fully typed with TypeScript generics.
          </p>
          <CodeBlock code={tsSdkExample} language="typescript" title="TypeScript SDK" />
        </section>

        {/* Framework Integrations */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Framework Integrations</h2>
          <p className="text-dark-300 mb-6">
            Drop-in memory backends for popular AI agent frameworks.
          </p>
          <div className="space-y-6">
            <CodeBlock code={langchainExample} language="python" title="LangChain" />
            <CodeBlock code={crewaiExample} language="python" title="CrewAI" />
          </div>
        </section>

        {/* Error Codes */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Error Codes</h2>
          <p className="text-dark-300 mb-4">
            All errors are returned as JSON with a consistent format. The response body always contains an <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">{"error"}</code> field with a descriptive message.
          </p>
          <div className="overflow-x-auto mb-4">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Status</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Code</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['400', 'Bad Request', 'Validation error (malformed JSON, missing required fields)'],
                  ['401', 'Unauthorized', 'Missing or invalid API key'],
                  ['403', 'Forbidden', 'API key role insufficient for this operation'],
                  ['404', 'Not Found', 'Resource does not exist'],
                  ['402', 'Payment Required', 'Plan limit exceeded'],
                  ['429', 'Too Many Requests', 'Rate limit exceeded'],
                  ['500', 'Internal Server Error', 'Unexpected server error'],
                ].map(([status, code, desc]) => (
                  <tr key={status} className="border-b border-dark-600/20">
                    <td className="py-3 px-4 font-mono text-brand-400">{status}</td>
                    <td className="py-3 px-4 font-medium text-dark-200">{code}</td>
                    <td className="py-3 px-4 text-dark-300">{desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <CodeBlock code={`{\n  "error": "Missing required field: content"\n}`} language="json" title="Error Response Format" />
        </section>

        {/* Rate Limiting */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Rate Limiting</h2>
          <p className="text-dark-300 mb-4">
            Rate limits are enforced per-tenant at the gateway level. When you exceed the limit, the API returns HTTP 429 with an error response.
          </p>
          <div className="overflow-x-auto mb-4">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Plan</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Requests/Minute</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['Free', '10'],
                  ['Developer', '100'],
                  ['Pro', '500'],
                  ['Team', '2,000'],
                  ['Enterprise', '10,000'],
                ].map(([plan, limit]) => (
                  <tr key={plan} className="border-b border-dark-600/20">
                    <td className="py-3 px-4 font-medium text-dark-200">{plan}</td>
                    <td className="py-3 px-4 text-dark-300">{limit}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <CodeBlock code={`{\n  "error": "Rate limit exceeded. Maximum 10 requests per minute on Free plan"\n}`} language="json" title="Rate Limit Exceeded Response" />
        </section>

        {/* Webhooks */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Webhooks</h2>
          <p className="text-dark-300 mb-4">
            Webhooks allow you to receive real-time events from Knol. Events are sent as HTTP POST requests to your configured URL with HMAC-SHA256 signatures for verification.
          </p>
          <div className="mb-6">
            <h3 className="text-lg font-semibold text-dark-100 mb-3">Webhook Management</h3>
            <ul className="list-disc list-inside text-dark-300 space-y-2 mb-4">
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">GET /v1/webhooks</code> — List all webhooks for your tenant</li>
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">POST /v1/webhooks</code> — Create a new webhook subscription</li>
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">DELETE /v1/webhooks/:id</code> — Remove a webhook</li>
            </ul>
          </div>
          <div className="mb-6">
            <h3 className="text-lg font-semibold text-dark-100 mb-3">Event Types</h3>
            <ul className="list-disc list-inside text-dark-300 space-y-1 mb-4">
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">memory.created</code> — New memory stored and accepted</li>
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">entity.created</code> — New entity extracted and added to graph</li>
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">edge.created</code> — New relationship discovered between entities</li>
              <li><code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">conflict.detected</code> — Memory conflict identified and flagged for review</li>
            </ul>
          </div>
          <div className="mb-6">
            <h3 className="text-lg font-semibold text-dark-100 mb-3">Signature Verification</h3>
            <p className="text-dark-300 mb-4">
              Each webhook request includes an <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">X-Webhook-Signature</code> header containing an HMAC-SHA256 signature of the request body using your webhook secret. Verify this signature to confirm the request came from Knol.
            </p>
          </div>
          <CodeBlock code={webhookExample} language="bash" title="POST /v1/webhooks" />
        </section>

        {/* Batch Operations */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Batch Operations</h2>
          <p className="text-dark-300 mb-4">
            Store multiple memories in a single request. Batch operations are processed asynchronously and return immediately with accepted IDs. This is more efficient than multiple individual requests.
          </p>
          <CodeBlock code={batchExample} language="bash" title="POST /v1/memory/batch" />
        </section>

        {/* Configuration */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Configuration</h2>
          <p className="text-dark-300 mb-4">
            Configuration follows a three-tier hierarchy: database (system_config table) &rarr; environment variable &rarr; compiled default.
            Runtime changes via the admin API take effect without restarts.
          </p>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Variable</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Default</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['DATABASE_URL', 'postgres://memory:memory_dev@localhost/memory', 'PostgreSQL + pgvector connection'],
                  ['REDIS_URL', 'redis://localhost:6379', 'Redis for rate limiting & caching'],
                  ['NATS_URL', 'nats://localhost:4222', 'NATS JetStream for async pipeline'],
                  ['LLM_PROVIDER', 'anthropic', 'LLM provider (anthropic, openai, gemini)'],
                  ['LLM_API_KEY', '(required)', 'API key for LLM extraction'],
                  ['LLM_MODEL', 'claude-haiku', 'Model for entity extraction'],
                  ['JWT_SECRET', '(required)', 'JWT signing key (min 32 chars)'],
                  ['GATEWAY_PORT', '3000', 'Gateway listen port'],
                  ['WEBHOOK_ENABLED', 'true', 'Enable webhook event dispatch'],
                  ['RUST_LOG', 'info', 'Log level (trace, debug, info, warn, error)'],
                ].map(([name, def, desc]) => (
                  <tr key={name} className="border-b border-dark-600/20">
                    <td className="py-3 px-4 font-mono text-brand-400 text-xs">{name}</td>
                    <td className="py-3 px-4 font-mono text-dark-400 text-xs">{def}</td>
                    <td className="py-3 px-4 text-dark-300">{desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Architecture */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Architecture</h2>
          <p className="text-dark-300 mb-4">
            Knol uses a microservice architecture with each concern isolated in its own Rust binary.
            All services share a single PostgreSQL database with pgvector for vector storage.
          </p>
          {/* Desktop diagram */}
          <div className="card font-mono text-sm text-dark-300 overflow-x-auto hidden md:block">
            <pre>{`┌─────────────┐      ┌──────────────┐      ┌────────────────┐
│   Gateway    │─────▶│  Write Svc   │─────▶│  NATS Stream   │
│  auth/rate   │      │  episodes    │      │  extraction    │
│  /metrics    │      │  webhooks    │      │  embedding     │
└──────┬──────┘      └──────────────┘      └───────┬────────┘
       │                                            │
       │             ┌──────────────┐      ┌────────▼────────┐
       └────────────▶│ Retrieve Svc │      │   Graph Svc     │
                     │ vector+BM25  │      │  LLM extraction │
                     │  RRF fusion  │      │  conflict detect│
                     │  graph walk  │      │  embedding gen  │
                     └──────────────┘      └─────────────────┘
                            │
                     ┌──────▼──────┐
                     │ PostgreSQL  │
                     │  + pgvector │
                     └─────────────┘`}</pre>
          </div>
          {/* Mobile architecture - card list */}
          <div className="md:hidden space-y-2">
            {[
              { name: 'Gateway', desc: 'Auth, rate limiting, metrics' },
              { name: 'Write Service', desc: 'Episodes, webhooks' },
              { name: 'NATS Stream', desc: 'Extraction, embedding pipeline' },
              { name: 'Retrieve Service', desc: 'Vector + BM25 + RRF + graph walk' },
              { name: 'Graph Service', desc: 'LLM extraction, conflict detection' },
              { name: 'PostgreSQL + pgvector', desc: 'Single database for all data' },
            ].map((svc) => (
              <div key={svc.name} className="card !p-3 flex items-center gap-3">
                <span className="text-brand-500 text-sm">▸</span>
                <div className="min-w-0">
                  <p className="text-sm font-semibold text-dark-100">{svc.name}</p>
                  <p className="text-xs text-dark-400">{svc.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Links */}
        <section className="mb-16">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <a href={SITE.github} className="card text-center hover:border-brand-500/50" target="_blank" rel="noopener noreferrer">
              <h3 className="text-lg font-semibold text-dark-100 mb-1">GitHub</h3>
              <p className="text-dark-400 text-sm">Source code, issues, discussions</p>
            </a>
            <a href={SITE.pypi} className="card text-center hover:border-brand-500/50" target="_blank" rel="noopener noreferrer">
              <h3 className="text-lg font-semibold text-dark-100 mb-1">PyPI</h3>
              <p className="text-dark-400 text-sm">pip install knol</p>
            </a>
            <a href={SITE.npm} className="card text-center hover:border-brand-500/50" target="_blank" rel="noopener noreferrer">
              <h3 className="text-lg font-semibold text-dark-100 mb-1">npm</h3>
              <p className="text-dark-400 text-sm">npm install @knol-dev/sdk</p>
            </a>
          </div>
        </section>
      </div>
    </div>
  );
}
