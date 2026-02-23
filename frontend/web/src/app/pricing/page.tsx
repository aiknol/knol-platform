import { Metadata } from 'next';
import PricingCard from '@/components/ui/PricingCard';
import { PRICING_TIERS } from '@/config';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Pricing'),
  description: 'Competitive, transparent pricing for Knol. Open source at the core, predictable plans for production growth.',
};

export default function PricingPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h1 className="text-3xl md:text-4xl font-bold text-dark-50">Competitive Pricing Built for Production</h1>
          <p className="mt-4 text-lg text-dark-300 max-w-2xl mx-auto">
            Open-source first. One pricing metric. Predictable cloud pricing for production AI memory workloads.
          </p>
        </div>

        <div className="card mb-12">
          <h2 className="text-xl font-semibold text-dark-100 mb-3">Context Engineering for Enterprise AI</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-dark-200">
            <div>
              <p className="font-semibold text-dark-100 mb-1">Structured Memory Management</p>
              <p>Store, search, and retrieve contextual information with semantic understanding. Reduce LLM hallucinations through reliable memory retrieval.</p>
            </div>
            <div>
              <p className="font-semibold text-dark-100 mb-1">Graph-based Context Engineering</p>
              <p>Model complex relationships between entities, memories, and context. Support n-hop traversal for deep contextual reasoning.</p>
            </div>
            <div>
              <p className="font-semibold text-dark-100 mb-1">Cost-Optimized Intelligence</p>
              <p>Reduce LLM token usage by 75% through efficient context retrieval. Only send relevant information to language models.</p>
            </div>
            <div>
              <p className="font-semibold text-dark-100 mb-1">Open-core, Enterprise-ready</p>
              <p>Self-host for control, use managed platform for scale. Same APIs everywhere—build locally, deploy globally.</p>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {PRICING_TIERS.map((plan) => (
            <PricingCard key={plan.name} {...plan} />
          ))}
        </div>

        <div className="card mt-10">
          <h2 className="text-xl font-semibold text-dark-100 mb-3">Simple Billing: One Unit</h2>
          <p className="text-sm text-dark-300 mb-3">
            Knol bills cloud usage using one metric: <strong>operations (ops)</strong>.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-dark-200">
            <div><strong>1 write</strong> (<code>memory.add</code>) = 1 op</div>
            <div><strong>1 retrieval</strong> (<code>memory.search</code>) = 1 op</div>
            <div><strong>1 context build</strong> (<code>memory.context</code>) = 1 op</div>
          </div>
        </div>

        {/* Feature comparison */}
        <div className="mt-20">
          <h2 className="text-2xl font-bold text-dark-50 text-center mb-8">Feature Comparison</h2>
          <div className="overflow-x-auto -mx-4 px-4 sm:mx-0 sm:px-0">
            <table className="w-full text-sm min-w-[540px]">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-3 sm:px-4 text-dark-300 font-medium">Feature</th>
                  <th className="text-center py-3 px-2 sm:px-4 text-dark-300 font-medium">OSS</th>
                  <th className="text-center py-3 px-4 text-dark-300 font-medium">Builder</th>
                  <th className="text-center py-3 px-4 text-dark-300 font-medium">Growth</th>
                  <th className="text-center py-3 px-4 text-dark-300 font-medium">Enterprise</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['Vector Search', true, true, true, true],
                  ['BM25 Full-text Search', true, true, true, true],
                  ['Knowledge Graph', true, true, true, true],
                  ['RRF Fusion', true, true, true, true],
                  ['PII Detection', true, true, true, true],
                  ['Multi-tenant RLS', true, true, true, true],
                  ['Python SDK', true, true, true, true],
                  ['TypeScript SDK', true, true, true, true],
                  ['Memory Decay', true, true, true, true],
                  ['Conflict Detection', true, true, true, true],
                  ['Webhook Events', true, true, true, true],
                  ['Write-time Embeddings', true, true, true, true],
                  ['N-hop Graph Traversal', true, true, true, true],
                  ['Managed Infrastructure', false, true, true, true],
                  ['Auto-scaling', false, true, true, true],
                  ['Memory Consolidation', true, true, true, true],
                  ['Admin Dashboard', true, true, true, true],
                  ['Audit Logging', true, true, true, true],
                  ['Custom Connectors', false, false, false, true],
                  ['SSO / SAML', false, false, false, true],
                  ['SCIM Provisioning', false, false, false, true],
                  ['Dedicated Support', false, false, false, true],
                  ['Compliance & governance packs', false, false, false, true],
                  ['Predictable overage pricing', false, true, true, true],
                ].map(([feature, ...tiers]) => (
                  <tr key={feature as string} className="border-b border-dark-600/20">
                    <td className="py-3 px-4 text-dark-200">{feature as string}</td>
                    {(tiers as boolean[]).map((available, i) => (
                      <td key={i} className="text-center py-3 px-4">
                        {available ? (
                          <svg className="w-5 h-5 text-brand-500 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                          </svg>
                        ) : (
                          <span className="text-dark-500">&mdash;</span>
                        )}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        <div className="mt-20">
          <h2 className="text-2xl font-bold text-dark-50 text-center mb-8">Migrate from Mem0 or Zep</h2>
          <div className="card max-w-4xl mx-auto">
            <p className="text-dark-300 text-sm mb-4">
              Knol includes migration tooling and API-compatible patterns to reduce switching risk.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-dark-200">
              <div>Schema + payload mapping checks</div>
              <div>Backfill + replay utilities</div>
              <div>Validation report before cutover</div>
            </div>
          </div>
        </div>

        <div className="mt-20">
          <h2 className="text-2xl font-bold text-dark-50 text-center mb-8">ROI Snapshot</h2>
          <div className="card max-w-4xl mx-auto">
            <p className="text-sm text-dark-300">
              Teams typically reduce memory-related LLM spend by optimizing context construction and avoiding redundant retrieval calls.
              Track three KPIs: retrieval hit rate, tokens per request, and p95 retrieval latency.
            </p>
          </div>
        </div>

        {/* FAQ */}
        <div className="mt-20 max-w-3xl mx-auto">
          <h2 className="text-2xl font-bold text-dark-50 text-center mb-8">FAQ</h2>
          <div className="space-y-6">
            {[
              {
                q: 'What is context engineering?',
                a: 'Context engineering is the practice of structuring and managing relevant information to optimize language model outputs. Knol enables this through semantic search, knowledge graphs, and intelligent memory management—reducing hallucinations and token usage while improving reasoning accuracy.',
              },
              {
                q: 'Can I self-host everything?',
                a: 'Yes. Knol keeps core memory APIs, core services, SDKs, and self-host deployment open source.',
              },
              {
                q: 'How do I migrate from Mem0 or Zep?',
                a: 'Knol provides migration tooling, mapping checks, replay utilities, and validation reports for both Mem0 and Zep workflows. Growth and Enterprise plans include migration assistance.',
              },
              {
                q: 'What happens if I exceed my plan limits?',
                a: 'We notify you before usage limits are reached. Usage above included volume is billed as transparent overage per 1K ops.',
              },
              {
                q: 'Is there a free trial for paid plans?',
                a: 'Yes, all paid plans include a 14-day free trial with full access to all features.',
              },
              {
                q: 'What LLM is used for extraction?',
                a: 'The default LLM provider is configurable from the admin UI. Knol supports Gemini, Anthropic, and OpenAI-compatible providers for extraction and reasoning workflows.',
              },
              {
                q: 'Why PostgreSQL-only architecture?',
                a: 'PostgreSQL with pgvector and native JSON support provides exceptional reliability, security, and compliance. This eliminates vendor lock-in, simplifies self-hosting, and makes governance easier.',
              },
            ].map(({ q, a }) => (
              <div key={q} className="card">
                <h3 className="font-semibold text-dark-100 mb-2">{q}</h3>
                <p className="text-dark-300 text-sm">{a}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
