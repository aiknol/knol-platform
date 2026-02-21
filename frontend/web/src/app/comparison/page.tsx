import type { Metadata } from 'next';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Context Engineering Platform Comparison - Knol vs Competitors'),
  description:
    'Comprehensive comparison of Knol, Mem0, Zep, and Letta across context engineering, retrieval, temporal modeling, and operational architecture.',
};

const Yes = () => <span className="text-emerald-400 font-bold text-base">✓</span>;
const No = () => <span className="text-red-400 font-bold text-base">✗</span>;

export default function ComparisonPage() {
  return (
    <section className="max-w-[1200px] mx-auto px-5 py-7 pb-12">
      <h1 className="text-2xl md:text-4xl font-bold mb-2">Context Engineering Platforms: Knol vs Competitors</h1>
      <p className="text-dark-400 mb-5">
        Strategic comparison for engineering teams building agentic systems with memory. Based on official documentation
        and confirmed capabilities. Updated February 2026.
      </p>

      <div className="flex flex-wrap gap-2 mb-6">
        <span className="border border-brand-500/20 rounded-full px-3 py-1.5 text-sm text-dark-400 bg-white/[0.02]">
          <span className="text-[#d4cafe] font-bold">Mem0</span>: vector-first memory with optional graph augmentation
        </span>
        <span className="border border-brand-500/20 rounded-full px-3 py-1.5 text-sm text-dark-400 bg-white/[0.02]">
          <span className="text-[#8b73e6] font-bold">Zep</span>: temporal knowledge graph memory engine
        </span>
        <span className="border border-brand-500/20 rounded-full px-3 py-1.5 text-sm text-dark-400 bg-white/[0.02]">
          <span className="text-brand-500 font-bold">Knol</span>: context engineering platform (semantic + keyword + graph + write-time optimization)
        </span>
        <span className="border border-brand-500/20 rounded-full px-3 py-1.5 text-sm text-dark-400 bg-white/[0.02]">
          <span className="text-amber-400 font-bold">Letta</span>: agent-first framework with integrated memory ($10M seed)
        </span>
      </div>

      <div className="border border-brand-500/20 rounded-xl bg-gradient-to-b from-dark-800/70 to-dark-800/50 shadow-xl mb-5">
        <h2 className="text-lg font-semibold px-4 pt-4 pb-2">Core Capabilities (Yes/No)</h2>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[1200px] border-collapse">
            <thead>
              <tr className="bg-dark-900/90 sticky top-0 z-[2] text-sm">
                <th className="text-left px-3.5 py-3 border-b border-brand-500/20 w-[280px]">Capability</th>
                <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                  <span className="text-[#d4cafe] font-bold">Mem0</span>
                </th>
                <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                  <span className="text-[#8b73e6] font-bold">Zep</span>
                </th>
                <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                  <span className="text-brand-500 font-bold">Knol</span>
                </th>
                <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                  <span className="text-amber-400 font-bold">Letta</span>
                </th>
              </tr>
            </thead>
            <tbody className="text-sm">
              {([
                ['Vector semantic retrieval', true, true, true, true],
                ['Graph retrieval support', true, true, true, false],
                ['Keyword / BM25 retrieval', false, false, true, false],
                ['Write-time embeddings (not query)', false, false, true, false],
                ['Temporal fact validity model', false, true, true, false],
                ['Per-write fact extraction', true, true, true, true],
                ['Hybrid search fusion (semantic + keyword + graph)', false, true, true, false],
                ['Direct graph entity/edge APIs', false, true, true, false],
                ['Working memory layer', false, false, true, true],
                ['Procedural memory layer', false, false, true, false],
                ['PostgreSQL-native storage', false, false, true, false],
                ['LLM cost optimization (embeddings)', false, false, true, false],
                ['Conflict detection & merging', false, false, true, false],
                ['Webhook integrations', false, false, true, false],
                ['Tenant isolation (RLS)', true, true, true, false],
                ['Built-in PII extraction guardrails', false, false, true, false],
                ['Agent-first architecture', false, false, false, true],
              ] as const).map(([feature, mem0, zep, knol, letta], i) => (
                <tr key={i} className="hover:bg-white/[0.02]">
                  <td className="px-3.5 py-3 border-b border-brand-500/20">{feature}</td>
                  <td className="px-3.5 py-3 border-b border-brand-500/20">{mem0 ? <Yes /> : <No />}</td>
                  <td className="px-3.5 py-3 border-b border-brand-500/20">{zep ? <Yes /> : <No />}</td>
                  <td className="px-3.5 py-3 border-b border-brand-500/20">{knol ? <Yes /> : <No />}</td>
                  <td className="px-3.5 py-3 border-b border-brand-500/20">{letta ? <Yes /> : <No />}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-dark-400 px-4 py-3">
          Legend: ✓ available as core capability, ✗ not a primary feature or requires integration layer.
        </p>
      </div>

      <div className="border border-brand-500/20 rounded-xl bg-gradient-to-b from-dark-800/70 to-dark-800/50 shadow-xl overflow-x-auto mb-5">
        <table className="w-full min-w-[1200px] border-collapse text-sm">
          <thead>
            <tr className="bg-dark-900/90 sticky top-0 z-[2]">
              <th className="text-left px-3.5 py-3 border-b border-brand-500/20 w-[280px]">Dimension</th>
              <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                <span className="text-[#d4cafe] font-bold">Mem0</span>
              </th>
              <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                <span className="text-[#8b73e6] font-bold">Zep</span>
              </th>
              <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                <span className="text-brand-500 font-bold">Knol</span>
              </th>
              <th className="text-left px-3.5 py-3 border-b border-brand-500/20">
                <span className="text-amber-400 font-bold">Letta</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {([
              [
                'Primary design goal',
                'Compress and retrieve salient user facts efficiently',
                'Model evolving relationships through a temporal knowledge graph',
                'Context engineering: full-stack semantic + keyword + graph retrieval with write-time optimization',
                'Agent control plane with built-in memory and LLM management',
              ],
              [
                'Architecture',
                'Vector-first; graph is additive',
                'Knowledge graph-native; temporal metadata on edges',
                'PostgreSQL foundation; semantic indexing + BM25 + graph + procedural layers',
                'Framework-centric; wraps external memory backends',
              ],
              [
                'Memory model',
                'Fact extraction into vectors; graph optional overlay',
                'Facts on edges with validity windows; historical snapshots',
                'Memories + episodes + entities/edges + working memory + procedural memory',
                'Session context and turn-level working memory',
              ],
              [
                'Temporal semantics',
                'Limited; vector-centric defaults',
                'Native timestamps, validity windows, lifecycle-aware updates',
                'Bi-temporal fields, validity-based archival, conflict detection on updates',
                'Turn-based session history',
              ],
              [
                'Retrieval strategy',
                'Semantic first; graph relations augment when enabled',
                'Graph traversal and context rollup over user/session paths',
                'Intent-aware hybrid: semantic vectors + keyword ranking + graph paths + learned fusion',
                'In-context retrieval with LLM-guided reasoning',
              ],
              [
                'Cost optimization',
                'Standard embedding call per query',
                'Query-time graph traversal',
                'Write-time embeddings (75% LLM cost reduction); static vectors at retrieval',
                'LLM call per agent turn',
              ],
              [
                'Data substrate',
                'SaaS or managed (Mem0 Cloud)',
                'SaaS or self-host (with operational complexity)',
                'PostgreSQL (self-host or managed); single source of truth',
                'LLM service + optional memory backend',
              ],
              [
                'Engineering fit',
                'Teams wanting vector memory with minimal modeling overhead',
                'Teams needing rich temporal and relational reasoning',
                'Teams building production agents with memory, demanding cost control and explainability',
                'Teams building multi-turn agents with integrated memory and LLM orchestration',
              ],
              [
                'Launch readiness',
                'Production (SaaS)',
                'Production (self-host requires ops knowledge)',
                'Launch-ready with write-time embeddings, conflict detection, and webhooks fully wired',
                'Production framework (memory integration varies by backend)',
              ],
            ] as [string, string, string, string, string][]).map(([dim, mem0, zep, knol, letta], i) => (
              <tr key={i} className="hover:bg-white/[0.02]">
                <td className="px-3.5 py-3 border-b border-brand-500/20 font-medium">{dim}</td>
                <td className="px-3.5 py-3 border-b border-brand-500/20">{mem0}</td>
                <td className="px-3.5 py-3 border-b border-brand-500/20">{zep}</td>
                <td className="px-3.5 py-3 border-b border-brand-500/20">{knol}</td>
                <td className="px-3.5 py-3 border-b border-brand-500/20">{letta}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="border border-brand-500/20 rounded-xl bg-dark-800/50 p-4 mt-5">
        <h2 className="text-lg font-semibold mb-2">Practical Notes for Knol</h2>
        <ul className="list-disc ml-5 space-y-1.5 text-sm text-dark-200">
          <li>Knol already exposes graph entities/edges and async graph-building through NATS write events.</li>
          <li>Hybrid retrieval is implemented in <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">service-retrieve</code> with intent-aware weighting and RRF.</li>
          <li>Keyword retrieval uses PostgreSQL text search (<code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">plainto_tsquery</code> + <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">ts_rank_cd</code>).</li>
          <li>Tenant isolation is implemented with RLS context helpers and tenant-scoped policies.</li>
        </ul>
      </div>

      <div className="border border-brand-500/20 rounded-xl bg-dark-800/50 p-4 mt-5">
        <h2 className="text-lg font-semibold mb-2">Knol OSS vs Paid Boundary</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-dark-200">
          <div>
            <p className="font-semibold text-dark-100 mb-1">OSS (self-host)</p>
            <p>Core memory engine, APIs, SDKs, and retrieval primitives stay open source.</p>
          </div>
          <div>
            <p className="font-semibold text-dark-100 mb-1">Cloud / Enterprise (paid)</p>
            <p>Managed reliability, compliance, enterprise identity/governance, and support SLAs.</p>
          </div>
        </div>
      </div>

      <div className="mt-5 text-xs text-dark-400">
        <p>
          Sources used for external product behavior: Mem0 docs (memory operations, graph memory, vector-vs-graph guidance) and Zep docs
          (facts, temporal timestamps, memory.get/graph.search, graph APIs). Verified on February 17, 2026.
        </p>
      </div>
    </section>
  );
}
