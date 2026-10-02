import { Metadata } from 'next';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('DoAide Cortex OSS — Open-Source Memory Infrastructure'),
  description:
    'Cortex OSS is the standalone open-source memory infrastructure for AI applications. Hybrid search, knowledge graphs, memory decay — all Apache 2.0 licensed.',
  keywords: [
    'cortex oss', 'open source', 'AI memory', 'knowledge graph', 'vector search',
    'Rust', 'PostgreSQL', 'pgvector', 'Apache 2.0', 'self-host',
  ],
};

export default function CortexOssPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-5xl mx-auto">

        {/* Hero */}
        <section className="mb-20 text-center">
          <span className="text-xs px-3 py-1 rounded-full bg-green-500/10 text-green-400 border border-green-500/20 mb-4 inline-block font-mono">
            100% Open Source &middot; Apache 2.0
          </span>
          <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold text-dark-50 mb-6">
            Cortex OSS
          </h1>
          <p className="text-lg md:text-xl text-dark-300 max-w-2xl mx-auto mb-8">
            Open-source memory infrastructure for AI applications.
            Give your agents persistent, searchable, context-aware memory.
            No enterprise code, no license restrictions.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.githubOss}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              GitHub &rarr;
            </a>
            <a href={SITE.docsUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Documentation
            </a>
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Live Demo
            </a>
          </div>
        </section>

        {/* What is DoAide Cortex OSS */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">What is DoAide Cortex OSS?</h2>
          <p className="text-dark-300 mb-4 leading-relaxed">
            Cortex OSS is a standalone copy of the open-source core
            from <a href="/cortex-platform/" className="text-brand-400 hover:text-brand-300 transition-colors">Cortex Platform</a>.
            It contains everything you need to run a production memory system without any enterprise dependencies.
          </p>
          <p className="text-dark-300 leading-relaxed">
            Write a memory in plain text. Cortex automatically extracts entities, builds a knowledge graph,
            detects conflicts with existing memories, and makes everything searchable via vector + full-text
            hybrid retrieval.
          </p>
        </section>

        {/* Quick Start */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Deploy in 60 Seconds</h2>
          <div className="card font-mono text-sm overflow-x-auto">
            <pre className="text-dark-300">{`git clone https://github.com/doaide/cortex.git
cd cortex
docker compose up -d

# Verify
curl http://localhost:3000/health
# → {"status":"ok"}`}</pre>
          </div>
          <p className="text-dark-400 text-sm mt-4">
            That&apos;s it. PostgreSQL with pgvector, all Rust services, and NATS streaming &mdash; running locally.
          </p>
        </section>

        {/* Features */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">Features</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {[
              {
                title: 'Hybrid Search',
                desc: 'Vector similarity + BM25 full-text with Reciprocal Rank Fusion. Intent-aware routing picks the best strategy per query.',
              },
              {
                title: 'Knowledge Graph',
                desc: 'Automatic entity and relationship extraction from conversations. N-hop traversal for rich contextual retrieval.',
              },
              {
                title: 'Multi-Scope Memory',
                desc: 'User, session, agent, team, and org-level scoping. Row-level security isolation between tenants.',
              },
              {
                title: 'Memory Types',
                desc: 'Episodic, semantic, procedural, and working memory. Four cognitive layers that mirror how humans remember.',
              },
              {
                title: 'PII Redaction',
                desc: 'Built-in detection and redaction for emails, phones, SSNs, credit cards, and more. Privacy by default.',
              },
              {
                title: 'Conflict Detection',
                desc: 'Automatic detection of contradictions and duplicates. Temporal validity windows resolve conflicting facts.',
              },
              {
                title: 'Decay Scoring',
                desc: 'Older memories gracefully fade; recently accessed ones stay relevant. Configurable decay curves per memory type.',
              },
              {
                title: 'Policy Engine',
                desc: 'Retention limits, access control, content filtering, and auto-redaction. Fine-grained control over memory lifecycle.',
              },
              {
                title: 'Webhooks',
                desc: 'Subscribe to memory events: created, updated, conflicts detected. Build reactive workflows on top of memory.',
              },
            ].map((f) => (
              <div key={f.title} className="card">
                <h3 className="text-sm font-semibold text-dark-100 mb-2">{f.title}</h3>
                <p className="text-dark-400 text-xs leading-relaxed">{f.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Architecture */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">Architecture</h2>
          <div className="card font-mono text-sm text-dark-300 overflow-x-auto hidden md:block">
            <pre>{`
  Client SDK                     Cortex OSS Services
  ──────────                     ─────────────────
                    ┌──────────────────────────────────────────┐
                    │                                          │
  Python SDK ──────▶│   Gateway   ──▶  Write Service           │
  TypeScript SDK ──▶│   :3000        │                         │
  REST / cURL ─────▶│                ▼                         │
  MCP Server ──────▶│            NATS Stream ──▶ Graph Service │
                    │                            │  extraction │
                    │   Retrieve  ◀──────────────┘  entities   │
                    │   Service      conflict detect            │
                    │   vector+BM25  embedding gen              │
                    │   graph walk                              │
                    │                                          │
                    └──────────────┬───────────────────────────┘
                                   │
                                   ▼
                            ┌─────────────┐
                            │ PostgreSQL   │
                            │  + pgvector  │
                            │  (single DB) │
                            └─────────────┘`}</pre>
          </div>
          <div className="md:hidden space-y-3">
            {[
              { name: 'Gateway (:3000)', desc: 'API routing, auth, rate limiting' },
              { name: 'Write Service', desc: 'Episode ingestion & webhooks' },
              { name: 'NATS Stream', desc: 'Async extraction pipeline' },
              { name: 'Graph Service', desc: 'Entity extraction, conflict detection, embeddings' },
              { name: 'Retrieve Service', desc: 'Vector + BM25 + graph fusion (RRF)' },
              { name: 'PostgreSQL + pgvector', desc: 'Single database for all data' },
            ].map((svc) => (
              <div key={svc.name} className="card !p-4 flex items-center gap-3">
                <span className="text-brand-500 text-lg">&blacktriangleright;</span>
                <div>
                  <p className="text-sm font-semibold text-dark-100">{svc.name}</p>
                  <p className="text-xs text-dark-400">{svc.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* SDK Integration */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Integrate in Minutes</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="card">
              <h3 className="text-sm font-semibold text-dark-100 mb-3">Python</h3>
              <div className="font-mono text-xs text-dark-300 overflow-x-auto">
                <pre>{`from cortex import CortexClient

client = CortexClient(
    base_url="http://localhost:3000",
    api_key="your-api-key"
)

# Store a memory
client.add(
    content="User prefers dark mode",
    user_id="user-123"
)

# Hybrid retrieval
results = client.search(
    query="user preferences",
    user_id="user-123"
)`}</pre>
              </div>
            </div>
            <div className="card">
              <h3 className="text-sm font-semibold text-dark-100 mb-3">TypeScript</h3>
              <div className="font-mono text-xs text-dark-300 overflow-x-auto">
                <pre>{`import { CortexClient } from '@doaide/cortex-sdk';

const cortex = new CortexClient({
  baseUrl: 'http://localhost:3000',
  apiKey: 'your-api-key',
});

// Store a memory
await cortex.add({
  content: 'User prefers TypeScript',
  userId: 'user-123',
});

// Hybrid retrieval
const results = await cortex.search({
  query: 'programming preferences',
  userId: 'user-123',
});`}</pre>
              </div>
            </div>
          </div>
        </section>

        {/* Cortex OSS vs Platform */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Cortex OSS vs Cortex Platform</h2>
          <div className="overflow-x-auto rounded-xl border border-dark-600/30">
            <table className="w-full text-sm min-w-[480px]">
              <thead>
                <tr className="bg-dark-800/80">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium"></th>
                  <th className="text-center py-3 px-4 text-green-400 font-semibold">Cortex OSS</th>
                  <th className="text-center py-3 px-4 text-brand-400 font-semibold">Cortex Platform</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['License', 'Apache 2.0', 'Apache 2.0 + BSL (enterprise)'],
                  ['Core services', 'All included', 'All included'],
                  ['Knowledge graph', '✓', '✓'],
                  ['Hybrid retrieval', '✓', '✓'],
                  ['Memory decay', '✓', '✓'],
                  ['SDKs (Python, TS, etc.)', '✓', '✓'],
                  ['Admin dashboard', '—', '✓'],
                  ['SSO / SAML / SCIM', '—', '✓'],
                  ['Audit logging', '—', '✓'],
                  ['Automated backups', '—', '✓'],
                  ['Best for', 'Self-hosters & contributors', 'Teams & enterprise'],
                ].map(([feature, oss, platform]) => (
                  <tr key={feature} className="border-t border-dark-600/20 hover:bg-dark-800/30 transition-colors">
                    <td className="py-3 px-4 text-dark-200">{feature}</td>
                    <td className="py-3 px-4 text-center text-dark-300 text-xs bg-green-500/5">{oss}</td>
                    <td className="py-3 px-4 text-center text-dark-300 text-xs bg-brand-500/5">{platform}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Related Products */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Related Products</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <a href="/cortex-platform/" className="card group hover:border-brand-500/40 transition-colors">
              <h3 className="text-lg font-semibold text-dark-100 mb-2 group-hover:text-brand-400 transition-colors">Cortex Platform</h3>
              <p className="text-dark-400 text-sm leading-relaxed">
                The full monorepo with open-source core plus enterprise services. SSO, admin dashboard, audit logging, and managed cloud.
              </p>
            </a>
            <a href="/cortex-local/" className="card group hover:border-brand-500/40 transition-colors">
              <h3 className="text-lg font-semibold text-dark-100 mb-2 group-hover:text-brand-400 transition-colors">Cortex Local</h3>
              <p className="text-dark-400 text-sm leading-relaxed">
                Lightweight SQLite-backed MCP server for personal use. Zero setup, no Docker required. Just npm install.
              </p>
            </a>
          </div>
        </section>

        {/* CTA */}
        <section className="bg-dark-700/30 border border-dark-600 rounded-xl p-8 text-center">
          <h2 className="text-2xl font-bold text-dark-50 mb-3">
            Start building with Cortex OSS
          </h2>
          <p className="text-dark-300 mb-6 max-w-xl mx-auto">
            Three commands to a running memory system. Fully open source, no strings attached.
          </p>
          <div className="bg-dark-800 border border-dark-600 rounded-lg px-6 py-4 font-mono text-brand-300 text-sm mb-6 inline-block">
            git clone https://github.com/doaide/cortex.git && cd knol && docker compose up -d
          </div>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.githubOss}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              Star on GitHub
            </a>
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Try Live Demo
            </a>
          </div>
        </section>

      </div>
    </div>
  );
}
