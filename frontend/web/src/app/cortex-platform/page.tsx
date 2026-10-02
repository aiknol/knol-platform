import { Metadata } from 'next';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('DoAide Cortex Platform — Context Engineering Infrastructure'),
  description:
    'The full Cortex platform: Rust-native context engineering infrastructure with hybrid retrieval, knowledge graphs, memory decay, and microservice architecture. Open core, self-hostable.',
  keywords: [
    'cortex platform', 'context engineering', 'AI memory', 'knowledge graph',
    'hybrid retrieval', 'Rust', 'PostgreSQL', 'pgvector', 'open core',
  ],
};

export default function CortexPlatformPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-5xl mx-auto">

        {/* Hero */}
        <section className="mb-20 text-center">
          <span className="text-xs px-3 py-1 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20 mb-4 inline-block font-mono">
            Open Core &middot; Apache 2.0 + Enterprise
          </span>
          <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold text-dark-50 mb-6">
            Cortex Platform
          </h1>
          <p className="text-lg md:text-xl text-dark-300 max-w-2xl mx-auto mb-8">
            The complete context engineering infrastructure for AI applications.
            Rust-native microservices, PostgreSQL-only storage, sub-5ms latency.
            Self-host or use our managed cloud.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              GitHub &rarr;
            </a>
            <a href={SITE.docsUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Documentation
            </a>
            <a href="/pricing/" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Cloud Plans
            </a>
          </div>
        </section>

        {/* What is Cortex Platform */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">What is Cortex Platform?</h2>
          <p className="text-dark-300 mb-4 leading-relaxed">
            Cortex Platform is the monorepo that contains the full Cortex stack: the open-source
            core (<code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">cortex-oss/</code>) plus
            enterprise services (<code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">cortex-enterprise/</code>).
            It includes everything needed to build, test, and deploy production-grade context engineering infrastructure.
          </p>
          <p className="text-dark-300 leading-relaxed">
            The platform gives your AI agents <strong className="text-dark-100">persistent, structured memory</strong> &mdash;
            not just vector search. Every conversation turn is processed through an intelligent extraction pipeline that builds
            a knowledge graph of entities, relationships, and facts alongside traditional semantic embeddings.
          </p>
        </section>

        {/* Architecture Diagram */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">Microservice Architecture</h2>
          <div className="card font-mono text-sm text-dark-300 overflow-x-auto hidden md:block">
            <pre>{`
  ┌─────────────┐      ┌──────────────┐      ┌────────────────┐
  │   Gateway    │─────▶│  Write Svc   │─────▶│  NATS Stream   │
  │  auth/rate   │      │  episodes    │      │  extraction    │
  └──────┬──────┘      └──────────────┘      └───────┬────────┘
         │                                            │
         │             ┌──────────────┐      ┌────────▼────────┐
         └────────────▶│ Retrieve Svc │      │   Graph Svc     │
                       │ vector+BM25  │      │  entities/edges │
                       │  RRF fusion  │      │  LLM extraction │
                       │  graph walk  │      │  conflict detect│
                       └──────────────┘      │  embedding gen  │
                                             └─────────────────┘

  All services → single PostgreSQL (pgvector) database
`}</pre>
          </div>
          <div className="md:hidden space-y-3">
            {[
              { name: 'Gateway', desc: 'Auth, rate limiting, API routing' },
              { name: 'Write Service', desc: 'Episodes, webhooks, ingestion' },
              { name: 'NATS Stream', desc: 'Async extraction pipeline' },
              { name: 'Retrieve Service', desc: 'Vector + BM25 + RRF fusion' },
              { name: 'Graph Service', desc: 'Entities, edges, LLM extraction, conflict detection' },
              { name: 'PostgreSQL', desc: 'Single database with pgvector for everything' },
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

        {/* Key Capabilities */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">Key Capabilities</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {[
              {
                title: 'Hybrid Retrieval',
                badge: 'Sub-5ms',
                desc: 'Vector similarity + BM25 full-text + N-hop graph traversal, fused with Reciprocal Rank Fusion. Intent-aware routing picks the optimal strategy per query.',
              },
              {
                title: 'Knowledge Graph',
                badge: 'Auto-extract',
                desc: 'LLM-powered entity and relationship extraction from conversations. Bi-temporal modeling tracks when facts were true vs when they were learned.',
              },
              {
                title: 'Memory Decay & Conflict Detection',
                badge: 'Cognitive',
                desc: 'Age-based decay scoring mirrors human forgetting. Contradictory facts are detected automatically and resolved with temporal validity windows.',
              },
              {
                title: '7-Layer Cost Optimization',
                badge: '75% savings',
                desc: 'Prompt caching, semantic deduplication, intent-based model routing, batching, working memory bypass, conflict caching, and cross-tenant pooling.',
              },
              {
                title: 'Four Memory Types',
                badge: 'Cognitive model',
                desc: 'Episodic (raw events), Semantic (distilled facts), Working (session context), and Procedural (learned patterns). Mirrors human cognition.',
              },
              {
                title: 'PII Guardrails & Webhooks',
                badge: 'Production',
                desc: 'Built-in detection and redaction for emails, phones, SSNs, credit cards. Event webhooks for memory created, updated, and conflicts detected.',
              },
            ].map((item) => (
              <div key={item.title} className="card">
                <span className="inline-block mb-2 px-2 py-0.5 rounded text-xs font-mono text-brand-300 bg-brand-500/10 border border-brand-500/20">
                  {item.badge}
                </span>
                <h3 className="text-lg font-semibold text-dark-100 mb-2">{item.title}</h3>
                <p className="text-dark-400 text-sm leading-relaxed">{item.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Repository Structure */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Repository Structure</h2>
          <div className="card font-mono text-sm text-dark-300 overflow-x-auto">
            <pre>{`cortex-platform/
├── cortex-oss/                    # Open source (Apache 2.0)
│   ├── crates/
│   │   ├── gateway/             # API gateway & auth
│   │   ├── write-service/       # Episode ingestion
│   │   ├── retrieve-service/    # Hybrid search engine
│   │   ├── graph-service/       # Knowledge graph & extraction
│   │   └── shared/              # Common types & utilities
│   ├── docker-compose.yml       # One-command deploy
│   └── migrations/              # PostgreSQL schema
│
├── cortex-enterprise/             # Commercial (BSL)
│   ├── crates/
│   │   ├── admin-dashboard/     # Management UI
│   │   ├── sso-saml/            # SSO/SAML/SCIM
│   │   ├── audit-log/           # Compliance logging
│   │   └── backup-service/      # Automated backups
│   └── ...
│
├── sdks/                        # Client SDKs
│   ├── python/                  # pip install doaide-cortex
│   ├── typescript/              # npm install @doaide/cortex-sdk
│   ├── langchain/               # LangChain adapter
│   └── crewai/                  # CrewAI adapter
│
├── frontend/                    # Web & cloud dashboards
└── docs/                        # Documentation`}</pre>
          </div>
        </section>

        {/* Open Core Model */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Open Core Model</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="card">
              <h3 className="text-lg font-semibold text-dark-100 mb-3">Always Open Source (Apache 2.0)</h3>
              <ul className="text-sm text-dark-300 space-y-2">
                <li>All core services: gateway, write, retrieve, graph</li>
                <li>Full SDKs: Python, TypeScript, LangChain, CrewAI, MCP</li>
                <li>Hybrid retrieval, knowledge graphs, memory decay</li>
                <li>Conflict detection, PII guardrails, webhooks</li>
                <li>Docker Compose one-command deploy</li>
              </ul>
            </div>
            <div className="card">
              <h3 className="text-lg font-semibold text-dark-100 mb-3">Enterprise Add-ons</h3>
              <ul className="text-sm text-dark-300 space-y-2">
                <li>Admin dashboard with analytics</li>
                <li>SSO/SAML/SCIM identity management</li>
                <li>SOC 2 / HIPAA compliance controls</li>
                <li>Automated backups and point-in-time recovery</li>
                <li>SLA commitments and dedicated support</li>
              </ul>
            </div>
          </div>
        </section>

        {/* Quick Start */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Quick Start</h2>
          <div className="card font-mono text-sm overflow-x-auto">
            <pre className="text-dark-300">{`# Clone the platform repo
git clone https://github.com/doaide/cortex-platform.git
cd cortex-platform

# Start the full stack (OSS services + PostgreSQL + NATS)
docker compose -f docker-compose.oss.yml up -d

# Verify everything is running
curl http://localhost:3000/health

# Install the Python SDK
pip install doaide-cortex

# Or TypeScript
npm install @doaide/cortex-sdk`}</pre>
          </div>
        </section>

        {/* Related Products */}
        <section className="mb-20">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Related Products</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <a href="/cortex-oss/" className="card group hover:border-brand-500/40 transition-colors">
              <h3 className="text-lg font-semibold text-dark-100 mb-2 group-hover:text-brand-400 transition-colors">Cortex OSS</h3>
              <p className="text-dark-400 text-sm leading-relaxed">
                Standalone open-source repo with just the core services. No enterprise code, no extra dependencies. Perfect for contributors and self-hosters.
              </p>
            </a>
            <a href="/cortex-local/" className="card group hover:border-brand-500/40 transition-colors">
              <h3 className="text-lg font-semibold text-dark-100 mb-2 group-hover:text-brand-400 transition-colors">Cortex Local</h3>
              <p className="text-dark-400 text-sm leading-relaxed">
                Lightweight SQLite-backed MCP server for personal use. Zero setup, zero cloud. Install with npm for Claude, Cursor, and Claude Code.
              </p>
            </a>
          </div>
        </section>

        {/* CTA */}
        <section className="bg-dark-700/30 border border-dark-600 rounded-xl p-8 text-center">
          <h2 className="text-2xl font-bold text-dark-50 mb-3">
            Ready to deploy context engineering?
          </h2>
          <p className="text-dark-300 mb-6 max-w-xl mx-auto">
            Self-host the open-source stack or start with managed cloud. Full platform, one PostgreSQL database.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              Star on GitHub
            </a>
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Try Live Demo
            </a>
            <a href="/pricing/" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              See Plans
            </a>
          </div>
        </section>

      </div>
    </div>
  );
}
