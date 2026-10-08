const TESTIMONIALS = [
  {
    quote: 'We replaced three separate services — Qdrant, Neo4j, and a custom caching layer — with Cortex running on a single PostgreSQL instance. Deployment complexity dropped by 80% and retrieval latency went from 45ms to under 5ms.',
    name: 'Vikram Mehta',
    role: 'CTO',
    company: 'NeuralStack AI, Bangalore',
  },
  {
    quote: 'Our AI agents used to lose context between conversations. After integrating Cortex, customer satisfaction scores improved by 35% because the agents actually remember previous interactions and preferences.',
    name: 'Ananya Krishnan',
    role: 'Head of AI Engineering',
    company: 'Lumen Analytics, Hyderabad',
  },
  {
    quote: 'As a team lead managing 40 engineers, I was spending hours answering the same architecture questions. Cortex automatically captures decisions from our Slack channels and makes them searchable. It paid for itself in the first week.',
    name: 'Rajesh Sundaram',
    role: 'Engineering Lead',
    company: 'FinEdge Technologies, Pune',
  },
  {
    quote: 'Data residency was non-negotiable for our healthcare clients. With Cortex self-hosted on AWS Mumbai, we got AI-powered knowledge management without any data leaving Indian borders. The Apache 2.0 license made procurement straightforward.',
    name: 'Priya Nair',
    role: 'VP of Engineering',
    company: 'MedAssist Digital, Chennai',
  },
] as const;

export default function Testimonials() {
  return (
    <section className="px-4 sm:px-6 lg:px-8 py-20 bg-dark-800/30">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-14">
          <h2 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight">
            Trusted by AI teams building in production
          </h2>
          <p className="mt-4 text-dark-400 max-w-2xl mx-auto">
            Engineering leaders who switched to Cortex for context engineering.
          </p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {TESTIMONIALS.map((t) => (
            <div key={t.name} className="card">
              <p className="text-dark-200 text-sm leading-relaxed italic mb-6">
                &ldquo;{t.quote}&rdquo;
              </p>
              <div>
                <p className="text-dark-100 font-semibold text-sm">{t.name}</p>
                <p className="text-dark-400 text-xs">
                  {t.role}, {t.company}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
