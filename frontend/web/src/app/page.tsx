import FeatureGrid from '@/components/marketing/FeatureGrid';
import CodeBlock from '@/components/ui/CodeBlock';
import ComparisonTable from '@/components/marketing/ComparisonTable';
import { SITE, TECH_STACK, KEY_METRICS, MEMORY_TYPES, USE_CASES, SDK_ECOSYSTEM } from '@/config';

const quickStartCode = `# Start the full stack in one command
docker compose up -d

# Or install the Python SDK
pip install knol

# TypeScript? We've got you covered
npm install @knol-dev/sdk`;

const pythonExample = `from knol import KnolClient

client = KnolClient(
    base_url="http://localhost:3000",
    api_key="your-api-key"
)

# Store a memory — extraction happens automatically
client.add(
    content="User prefers dark mode and concise responses",
    user_id="user-123"
)

# Hybrid retrieval: vector + BM25 + knowledge graph
results = client.search(
    query="What are the user's preferences?",
    user_id="user-123"
)

# Access the knowledge graph directly
entities = client.list_entities(user_id="user-123")`;

const tsExample = `import { KnolClient } from '@knol-dev/sdk';

const knol = new KnolClient({
  baseUrl: 'http://localhost:3000',
  apiKey: 'your-api-key',
});

// Store and search with the same clean API
await knol.add({
  content: 'User prefers TypeScript and functional patterns',
  userId: 'user-123',
});

const results = await knol.search({
  query: 'programming preferences',
  userId: 'user-123',
});`;

export default function HomePage() {
  return (
    <>
      {/* Hero */}
      <section className="px-4 sm:px-6 lg:px-8 pt-24 pb-16 md:pt-36 md:pb-24 text-center relative overflow-hidden">
        {/* Gradient orb background */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[500px] bg-brand-500/10 rounded-full blur-[120px] pointer-events-none" />
        <div className="max-w-4xl mx-auto relative">
          <div className="inline-block mb-6 px-4 py-1.5 rounded-full border border-brand-500/20 bg-brand-500/5 text-brand-300 text-sm tracking-wide">
            Open Source &middot; Apache 2.0 &middot; Built in Rust &middot; PostgreSQL-native
          </div>
          <h1 className="text-4xl md:text-6xl lg:text-7xl font-semibold text-dark-50 leading-[1.1] tracking-tight">
            Context engineering
            <br />
            <span className="gradient-text">infrastructure for AI</span>
          </h1>
          <p className="mt-8 text-lg md:text-xl text-dark-300 max-w-2xl mx-auto leading-relaxed">
            Rust-native memory platform for LLM applications.
            One binary. One PostgreSQL database. Sub-5ms latency. Deploy in 60 seconds.
          </p>
          <div className="mt-10 flex flex-col sm:flex-row gap-4 justify-center">
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-primary text-base">
              Try the Live Demo
            </a>
            <a href={SITE.github} className="btn-secondary text-base">
              Star on GitHub
            </a>
            <a href="/pricing/" className="btn-secondary text-base">
              See Cloud Plans
            </a>
          </div>
          <p className="mt-6 text-sm text-dark-400">
            No Neo4j &middot; No Qdrant &middot; No Redis required &middot; Just PostgreSQL
          </p>
        </div>
      </section>

      {/* Tech Stack Logos */}
      <section className="px-4 sm:px-6 lg:px-8 py-12 border-y border-dark-600/20">
        <div className="max-w-5xl mx-auto text-center">
          <p className="text-sm text-dark-400 mb-8 uppercase tracking-wider">Built for the modern AI stack</p>
          <div className="flex flex-wrap justify-center items-center gap-x-12 gap-y-6 text-dark-400">
            {TECH_STACK.map((tech) => (
              <span key={tech} className="font-mono text-sm opacity-70">{tech}</span>
            ))}
          </div>
        </div>
      </section>

      {/* Key Metrics */}
      <section className="px-4 sm:px-6 lg:px-8 py-16 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
            {KEY_METRICS.map((stat) => (
              <div key={stat.label}>
                <div className="text-3xl md:text-4xl font-bold gradient-text">{stat.value}</div>
                <div className="text-sm text-dark-200 mt-2 font-medium">{stat.label}</div>
                <div className="text-xs text-dark-400 mt-1">{stat.sub}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Problem / Solution — How It Works */}
      <section className="px-4 sm:px-6 lg:px-8 py-20">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">
              LLMs forget everything between requests.
              <br />
              <span className="gradient-text">Knol gives them context.</span>
            </h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Context engineering is about assembling the right information at the right time.
              Knol automates this across every interaction.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {[
              {
                num: '01',
                title: 'Ingest',
                desc: 'Feed conversations through the write pipeline. Knol extracts entities, relationships, and facts using LLM-powered analysis with 75% cost optimization.',
              },
              {
                num: '02',
                title: 'Consolidate',
                desc: 'Memories evolve from episodic to semantic knowledge. Duplicates merge, conflicts resolve automatically, and importance decays naturally over time.',
              },
              {
                num: '03',
                title: 'Retrieve',
                desc: 'Hybrid search fuses vector similarity, BM25 text matching, and N-hop graph traversal with intent-aware routing — all in under 5ms.',
              },
            ].map((step) => (
              <div key={step.num} className="card">
                <span className="text-brand-500 font-mono text-sm font-bold">{step.num}</span>
                <h3 className="text-xl font-semibold text-dark-50 mt-3 mb-3">{step.title}</h3>
                <p className="text-dark-300 text-sm leading-relaxed">{step.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Why Knol — Competitive Differentiation */}
      <section className="px-4 sm:px-6 lg:px-8 py-20 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Why teams choose Knol</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              The only open-source memory platform that doesn&apos;t require Neo4j, Qdrant, or external vector databases.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {[
              {
                title: 'Radical Simplicity',
                desc: 'Everything runs on PostgreSQL + pgvector. No Neo4j for graphs, no Qdrant for vectors, no separate Redis for caching. One database to operate, backup, and scale.',
                highlight: 'PostgreSQL-only',
              },
              {
                title: 'Infrastructure-Grade Performance',
                desc: 'Rust-native core delivers sub-5ms gateway latency and a 10x smaller memory footprint than Python alternatives. Starts in milliseconds, barely uses RAM.',
                highlight: 'Sub-5ms latency',
              },
              {
                title: 'LLM Cost Intelligence',
                desc: '7-layer optimization pipeline with prompt caching, batching, model routing, and deduplication. Cut extraction costs by 75% without sacrificing quality.',
                highlight: '75% cost reduction',
              },
            ].map((item) => (
              <div key={item.title} className="card">
                <span className="inline-block mb-3 px-2 py-0.5 rounded text-xs font-mono text-brand-300 bg-brand-500/10 border border-brand-500/20">
                  {item.highlight}
                </span>
                <h3 className="text-lg font-semibold text-dark-50 mb-2">{item.title}</h3>
                <p className="text-dark-400 text-sm leading-relaxed">{item.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="px-4 sm:px-6 lg:px-8 py-20">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Everything you need, out of the box</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Production-grade context engineering infrastructure with no assembly required.
            </p>
          </div>
          <FeatureGrid />
        </div>
      </section>

      {/* Architecture */}
      <section className="px-4 sm:px-6 lg:px-8 py-16 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-10">
            <h2 className="text-3xl font-semibold text-dark-50 tracking-tight">Microservice architecture</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Each concern in its own Rust service. Scale the pieces that matter. All talking to one PostgreSQL database.
            </p>
          </div>
          {/* Desktop architecture diagram */}
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
`}</pre>
          </div>
          {/* Mobile architecture diagram - simplified card layout */}
          <div className="md:hidden space-y-3">
            {[
              { name: 'Gateway', desc: 'Auth & rate limiting' },
              { name: 'Write Service', desc: 'Episodes & webhooks' },
              { name: 'NATS Stream', desc: 'Async extraction' },
              { name: 'Retrieve Service', desc: 'Vector + BM25 + RRF fusion' },
              { name: 'Graph Service', desc: 'Entities, edges, LLM extraction' },
            ].map((svc) => (
              <div key={svc.name} className="card !p-4 flex items-center gap-3">
                <span className="text-brand-500 text-lg">▸</span>
                <div>
                  <p className="text-sm font-semibold text-dark-100">{svc.name}</p>
                  <p className="text-xs text-dark-400">{svc.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Memory Types */}
      <section className="px-4 sm:px-6 lg:px-8 py-20">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Cognitive memory model</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Inspired by human cognition. Four memory types that mirror how people actually remember.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {MEMORY_TYPES.map((m) => (
              <div key={m.name} className="card text-center">
                <h3 className={`text-lg font-semibold text-${m.color} mb-2`}>{m.name}</h3>
                <p className="text-dark-400 text-sm leading-relaxed">{m.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Comparison */}
      <section className="px-4 sm:px-6 lg:px-8 py-20 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">How Knol compares</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              The only open-source context engineering platform with hybrid retrieval, knowledge graphs, memory decay, and PostgreSQL-only architecture.
            </p>
          </div>
          <ComparisonTable />
          <div className="text-center mt-8">
            <a href="/comparison" className="text-brand-300 hover:text-brand-200 text-sm font-medium transition-colors">
              View detailed comparison &rarr;
            </a>
          </div>
        </div>
      </section>

      {/* SDK Ecosystem */}
      <section className="px-4 sm:px-6 lg:px-8 py-20">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Complete SDK ecosystem</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Six integration paths, all ready at launch. From raw REST to framework-native adapters.
            </p>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {SDK_ECOSYSTEM.map((sdk) => (
              <div key={sdk.name} className="card text-center py-5">
                <div className="text-2xl mb-2">{sdk.icon}</div>
                <h3 className="text-sm font-semibold text-dark-100 mb-1">{sdk.name}</h3>
                <code className="text-xs text-dark-400 font-mono">{sdk.pkg}</code>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Migration Wedge */}
      <section className="px-4 sm:px-6 lg:px-8 py-16 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-10">
            <h2 className="text-3xl font-semibold text-dark-50 tracking-tight">Switch from Mem0 or Zep with confidence</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              Use migration tooling built for low-risk cutovers: schema mapping checks, replay utilities, and validation reports.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="card text-sm text-dark-300">Import and normalize historical memory records</div>
            <div className="card text-sm text-dark-300">Replay events to validate retrieval quality before cutover</div>
            <div className="card text-sm text-dark-300">Generate a migration verification report for launch review</div>
          </div>
        </div>
      </section>

      {/* Commercial Boundary */}
      <section className="px-4 sm:px-6 lg:px-8 py-16 bg-dark-800/30">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-10">
            <h2 className="text-3xl font-semibold text-dark-50 tracking-tight">Open core, commercial operations</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              The full context engineering engine stays open source forever. We monetize managed reliability, security, and compliance.
            </p>
          </div>
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
              <h3 className="text-lg font-semibold text-dark-100 mb-3">Paid Cloud / Enterprise</h3>
              <ul className="text-sm text-dark-300 space-y-2">
                <li>Managed uptime, scaling, and automated backups</li>
                <li>SSO/SAML/SCIM identity management</li>
                <li>SOC 2 / HIPAA compliance controls</li>
                <li>Admin dashboard with audit logging</li>
                <li>SLA commitments and dedicated support</li>
              </ul>
            </div>
          </div>
          <div className="text-center mt-6">
            <a href="/pricing/" className="text-brand-300 hover:text-brand-200 text-sm font-medium transition-colors">
              Compare OSS vs paid plans &rarr;
            </a>
          </div>
        </div>
      </section>

      {/* Code Examples */}
      <section className="px-4 sm:px-6 lg:px-8 py-20">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Integrate in minutes</h2>
            <p className="mt-4 text-dark-400">
              Python SDK, TypeScript SDK, REST API, or deploy the full stack with Docker.
            </p>
          </div>

          <div className="space-y-8">
            <CodeBlock code={pythonExample} language="python" title="Python SDK" />
            <CodeBlock code={tsExample} language="typescript" title="TypeScript SDK" />
            <CodeBlock code={quickStartCode} language="bash" title="Quick Start" />
          </div>
        </div>
      </section>

      {/* Use Cases */}
      <section className="px-4 sm:px-6 lg:px-8 py-20 bg-dark-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-14">
            <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">Built for AI teams</h2>
            <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
              From solo developers to enterprise teams building production AI applications.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {USE_CASES.map((uc) => (
              <div key={uc.title} className="card">
                <h3 className="text-lg font-semibold text-dark-50 mb-2">{uc.title}</h3>
                <p className="text-dark-400 text-sm leading-relaxed">{uc.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="px-4 sm:px-6 lg:px-8 py-24 text-center relative overflow-hidden">
        <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[600px] h-[300px] bg-brand-500/10 rounded-full blur-[100px] pointer-events-none" />
        <div className="max-w-3xl mx-auto relative">
          <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">
            Ready to give your AI persistent context?
          </h2>
          <p className="mt-4 text-lg text-dark-300">
            Deploy the open-source stack in 60 seconds. Scale with managed cloud when you&apos;re ready.
          </p>
          <div className="mt-10 flex flex-col sm:flex-row gap-4 justify-center">
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-primary">
              Try the Live Demo
            </a>
            <a href={SITE.github} className="btn-secondary">
              Star on GitHub
            </a>
          </div>
        </div>
      </section>
    </>
  );
}
