'use client';

import { useState } from 'react';

const FAQ_ITEMS = [
  {
    q: 'What is DoAide Cortex?',
    a: 'DoAide Cortex is an open-source, Rust-native context engineering platform for LLM applications. It provides persistent memory, knowledge graphs, and hybrid retrieval so your AI agents can remember and reason across conversations.',
  },
  {
    q: 'How is Cortex different from a vector database?',
    a: 'Vector databases handle one dimension of retrieval — semantic similarity. Cortex combines vector search, BM25 text matching, and knowledge graph traversal into a single hybrid retrieval system, all running on PostgreSQL. No separate infrastructure required.',
  },
  {
    q: 'Do I need Neo4j or Qdrant to use Cortex?',
    a: 'No. Cortex runs entirely on PostgreSQL with pgvector. There is no need for Neo4j, Qdrant, Redis, or any other external database. One PostgreSQL instance handles vectors, full-text search, and graph queries.',
  },
  {
    q: 'How do I deploy Cortex?',
    a: 'Run "docker compose up -d" to start the full stack locally. For production, deploy on any cloud (AWS, Azure, GCP) or on-premises infrastructure. The Rust-native binary starts in milliseconds and uses minimal RAM.',
  },
  {
    q: 'What programming languages are supported?',
    a: 'Cortex provides official SDKs for Python and TypeScript, plus integrations with LangChain, CrewAI, and MCP (Model Context Protocol). You can also use the REST API directly from any language.',
  },
  {
    q: 'Is Cortex suitable for enterprise use?',
    a: 'Yes. Cortex is Apache 2.0 licensed with no per-user fees. Self-host it for full data residency and compliance. The managed cloud plan adds SSO/SAML, audit logging, and SLA commitments for teams that need them.',
  },
  {
    q: 'What kind of latency can I expect?',
    a: 'Cortex delivers sub-5ms gateway latency for hybrid retrieval queries. The Rust core provides a 10x smaller memory footprint compared to Python-based alternatives, making it suitable for high-concurrency production workloads.',
  },
  {
    q: 'Can I migrate from Mem0 or Zep?',
    a: 'Yes. Cortex includes migration tooling for importing historical memory records from other platforms, replaying events to validate retrieval quality, and generating verification reports before cutover.',
  },
] as const;

export default function FAQ() {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <section className="px-4 sm:px-6 lg:px-8 py-20">
      <div className="max-w-3xl mx-auto">
        <div className="text-center mb-14">
          <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">
            Frequently asked questions
          </h2>
          <p className="mt-4 text-dark-400">
            Everything you need to know about DoAide Cortex.
          </p>
        </div>
        <div className="space-y-3">
          {FAQ_ITEMS.map((item, i) => (
            <div key={i} className="card !p-0 overflow-hidden">
              <button
                type="button"
                onClick={() => setOpen(open === i ? null : i)}
                className="w-full flex items-center justify-between px-6 py-5 text-left cursor-pointer"
                aria-expanded={open === i}
              >
                <span className="text-dark-100 font-medium text-sm pr-4">{item.q}</span>
                <span className="text-brand-400 shrink-0 text-lg">{open === i ? '−' : '+'}</span>
              </button>
              {open === i && (
                <div className="px-6 pb-5 text-sm text-dark-300 leading-relaxed border-t border-dark-600/20 pt-4">
                  {item.a}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify({
            '@context': 'https://schema.org',
            '@type': 'FAQPage',
            mainEntity: FAQ_ITEMS.map((item) => ({
              '@type': 'Question',
              name: item.q,
              acceptedAnswer: {
                '@type': 'Answer',
                text: item.a,
              },
            })),
          }),
        }}
      />
    </section>
  );
}
