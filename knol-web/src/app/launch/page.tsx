import { Metadata } from 'next';
import Link from 'next/link';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Knol — Context Engineering Infrastructure for AI'),
  description:
    'Rust-native context engineering platform for LLM applications. One binary, one PostgreSQL database, sub-5ms latency. Deploy in 60 seconds. Open source.',
  keywords: ['AI memory', 'context engineering', 'LLM memory', 'Rust', 'PostgreSQL', 'open source', 'Mem0 alternative', 'Zep alternative'],
};

export default function LaunchPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        {/* Hero */}
        <section className="text-center mb-16">
          <h1 className="text-5xl font-bold text-dark-50 mb-6">
            The Context Engineering Platform for AI
          </h1>
          <p className="text-xl text-dark-300 max-w-2xl mx-auto mb-8">
            One Rust binary. One PostgreSQL database. Sub-5ms latency.
            Hybrid retrieval, knowledge graphs, memory decay, and conflict detection.
            Deploy in 60 seconds. Apache 2.0.
          </p>
          <div className="flex justify-center gap-4 flex-wrap mb-8">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-8 py-3 rounded-lg font-medium text-lg"
            >
              Star on GitHub
            </a>
            <Link href="/demo/" className="btn-secondary px-8 py-3 rounded-lg font-medium text-lg">
              Try the Demo
            </Link>
          </div>
        </section>

        {/* 3-command quick start */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6 text-center">Deploy in 3 Commands</h2>
          <div className="bg-dark-800 border border-dark-700 rounded-lg p-6 overflow-x-auto">
            <pre className="text-sm font-mono">
              <span className="text-dark-400">$</span> <span className="text-dark-200">git clone https://github.com/aiknol/knol.git && cd knol</span>{'\n'}
              <span className="text-dark-400">$</span> <span className="text-dark-200">docker compose up -d</span>{'\n'}
              <span className="text-dark-400">$</span> <span className="text-dark-200">curl -X POST http://localhost:3000/v1/memory \</span>{'\n'}
              <span className="text-dark-200">{'    '}-H &quot;Content-Type: application/json&quot; \</span>{'\n'}
              <span className="text-dark-200">{'    '}-d &apos;{'{"content": "User prefers dark mode", "user_id": "u1"}'}&apos;</span>{'\n'}
              <span className="text-brand-400">{'{"id": "mem_abc123", "status": "stored", "latency_ms": 3}'}</span>
            </pre>
          </div>
        </section>

        {/* Benchmarks */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6 text-center">Benchmarks</h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-700">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Metric</th>
                  <th className="text-center py-3 px-4 text-dark-400">Mem0</th>
                  <th className="text-center py-3 px-4 text-dark-400">Zep</th>
                  <th className="text-center py-3 px-4 text-brand-400 font-semibold">Knol</th>
                </tr>
              </thead>
              <tbody className="text-dark-300">
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Retrieval latency (P95)</td>
                  <td className="text-center py-3 px-4">50-200ms</td>
                  <td className="text-center py-3 px-4">30-100ms</td>
                  <td className="text-center py-3 px-4 text-brand-400 font-semibold">&lt;5ms</td>
                </tr>
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Binary size</td>
                  <td className="text-center py-3 px-4">~500MB (Python)</td>
                  <td className="text-center py-3 px-4">~300MB (Go+Python)</td>
                  <td className="text-center py-3 px-4 text-brand-400 font-semibold">50MB</td>
                </tr>
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Databases required</td>
                  <td className="text-center py-3 px-4">3-4</td>
                  <td className="text-center py-3 px-4">2-3</td>
                  <td className="text-center py-3 px-4 text-brand-400 font-semibold">1 (PostgreSQL)</td>
                </tr>
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Memory decay</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-brand-400">Yes</td>
                </tr>
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Conflict detection</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-brand-400">Yes</td>
                </tr>
                <tr className="border-b border-dark-800">
                  <td className="py-3 px-4">Hybrid retrieval (vector+BM25+graph)</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-dark-500">No</td>
                  <td className="text-center py-3 px-4 text-brand-400">Yes</td>
                </tr>
                <tr>
                  <td className="py-3 px-4">Open-source self-host</td>
                  <td className="text-center py-3 px-4 text-dark-500">Partial</td>
                  <td className="text-center py-3 px-4 text-dark-500">Partial</td>
                  <td className="text-center py-3 px-4 text-brand-400">Full (Apache 2.0)</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        {/* Key differentiators */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Key Differentiators</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Bi-Temporal Memory Model</h3>
              <p className="text-dark-300 text-sm">
                Every fact has two timestamps: when it was true in the real world (valid time) and when you learned it
                (transaction time). Query memories at any point in history.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">7-Layer LLM Cost Optimization</h3>
              <p className="text-dark-300 text-sm">
                Semantic deduplication, prompt caching, model routing, batch processing, and more.
                75% reduction in LLM extraction costs without sacrificing quality.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">HMAC-Signed Webhooks</h3>
              <p className="text-dark-300 text-sm">
                React to memory events in real-time. New facts, conflicts detected, memories decayed — all
                delivered via signed webhooks with retry logic.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">MCP Server Built-In</h3>
              <p className="text-dark-300 text-sm">
                Give Claude, Cursor, or any MCP-compatible tool persistent memory.
                One command: <code className="text-dark-200">npx @aiknol/knol-mcp-server</code>
              </p>
            </div>
          </div>
        </section>

        {/* Architecture */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Architecture</h2>
          <div className="bg-dark-800 border border-dark-700 rounded-lg p-6 overflow-x-auto">
            <pre className="text-xs text-dark-300 font-mono leading-relaxed">{`┌─────────────────────────────────────────────────────────────┐
│                     Your Application                         │
│              (Python SDK / TypeScript SDK / REST)             │
└─────────────────────┬───────────────────────────────────────┘
                      │
              ┌───────▼───────┐
              │    Gateway    │  Auth, rate limiting, routing
              └───┬───────┬───┘
          ┌───────▼──┐  ┌─▼──────────┐
          │  Write   │  │  Retrieve   │  Hybrid search engine
          │  Service │  │  Service    │  Vector + BM25 + Graph
          └───┬──────┘  └─┬──────────┘
              │           │
          ┌───▼───────────▼───┐    ┌──────────────┐
          │    PostgreSQL      │    │  Graph        │
          │    + pgvector      │◄───│  Service      │
          │                    │    └──────────────┘
          └────────────────────┘
                One Database`}</pre>
          </div>
        </section>

        {/* SDK examples */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Works With Your Stack</h2>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {[
              { name: 'Python', cmd: 'pip install knol' },
              { name: 'TypeScript', cmd: 'npm install @knol/sdk' },
              { name: 'LangChain', cmd: 'from knol.langchain import KnolMemory' },
              { name: 'CrewAI', cmd: 'from knol.crewai import KnolMemory' },
              { name: 'MCP Server', cmd: 'npx @aiknol/knol-mcp-server' },
              { name: 'REST API', cmd: 'curl localhost:3000/v1/memory' },
            ].map((sdk) => (
              <div key={sdk.name} className="bg-dark-700/30 border border-dark-600 rounded-lg p-4">
                <h3 className="text-sm font-semibold text-dark-100 mb-1">{sdk.name}</h3>
                <code className="text-xs text-dark-400 font-mono">{sdk.cmd}</code>
              </div>
            ))}
          </div>
        </section>

        {/* Final CTA */}
        <section className="bg-dark-700/30 border border-dark-600 rounded-lg p-8 text-center">
          <h2 className="text-3xl font-bold text-dark-50 mb-4">
            The Nginx of AI Memory
          </h2>
          <p className="text-dark-300 mb-6 max-w-xl mx-auto">
            While others build Python SDKs on top of three databases, we built a single Rust binary
            that handles vector search, knowledge graphs, and temporal memory on just PostgreSQL.
            Sub-5ms latency. 50MB footprint. Apache 2.0.
          </p>
          <div className="flex justify-center gap-4 flex-wrap">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-8 py-3 rounded-lg font-medium text-lg"
            >
              Star on GitHub
            </a>
            <Link href="/docs/" className="btn-secondary px-8 py-3 rounded-lg font-medium text-lg">
              Documentation
            </Link>
            <Link href="/mcp/" className="btn-secondary px-8 py-3 rounded-lg font-medium text-lg">
              MCP Server
            </Link>
          </div>
        </section>
      </div>
    </div>
  );
}
