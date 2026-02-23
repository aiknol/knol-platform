import { Metadata } from 'next';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('About Knol'),
  description: 'Learn about Knol — an open-source context engineering platform for building AI applications with persistent memory.',
};

export default function AboutPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl md:text-4xl font-bold text-dark-50 mb-4">About Knol</h1>

        {/* Mission */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Our Mission</h2>
          <p className="text-dark-300 text-lg leading-relaxed">
            Knol is an open-source context engineering platform built to give AI applications persistent memory, understanding, and reasoning.
            We believe that AI systems should be able to learn, remember, and build knowledge over time — not just process information in isolation.
            Our mission is to make it simple for developers to integrate intelligent memory systems into their applications with minimal
            infrastructure overhead and maximum performance.
          </p>
        </section>

        {/* What is Knol */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">What is Knol?</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              Knol is a Rust-native context engineering infrastructure that provides:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-2">
              <li><span className="font-semibold text-dark-100">Persistent Memory</span> — Store, search, and retrieve user memories with sub-5ms latency</li>
              <li><span className="font-semibold text-dark-100">Knowledge Graphs</span> — Automatically extract entities and relationships from your data</li>
              <li><span className="font-semibold text-dark-100">Hybrid Search</span> — Vector similarity + semantic search + knowledge graph traversal</li>
              <li><span className="font-semibold text-dark-100">Memory Decay</span> — Implement realistic forgetting with configurable decay curves</li>
              <li><span className="font-semibold text-dark-100">Conflict Detection</span> — Identify and resolve inconsistencies in user memories</li>
              <li><span className="font-semibold text-dark-100">Async Processing</span> — Extract and embed at scale without blocking your application</li>
            </ul>
            <p className="mt-4">
              Deploy Knol as a single binary with one PostgreSQL database, or integrate it into your existing stack.
              It&apos;s designed for developers who want intelligent memory without managing complex infrastructure.
            </p>
          </div>
        </section>

        {/* Why Knol */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Why Knol?</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Open Source</h3>
              <p className="text-dark-300 text-sm">
                Knol is fully open source. Review the code, contribute, and deploy on your own infrastructure
                without vendor lock-in or proprietary restrictions.
              </p>
            </div>

            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Production-Ready</h3>
              <p className="text-dark-300 text-sm">
                Built in Rust for performance and reliability. Deploy in Docker, Kubernetes, or bare metal.
                One binary, minimal dependencies, maximum efficiency.
              </p>
            </div>

            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Developer First</h3>
              <p className="text-dark-300 text-sm">
                Simple REST API and SDKs for Python and TypeScript. Integrates seamlessly with LangChain, CrewAI,
                and other AI frameworks.
              </p>
            </div>

            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Privacy Focused</h3>
              <p className="text-dark-300 text-sm">
                Your data stays your data. Deploy on your infrastructure, use your LLM provider, no telemetry
                or analytics tracking.
              </p>
            </div>
          </div>
        </section>

        {/* Values */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Our Values</h2>
          <div className="space-y-4 text-dark-300">
            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Simplicity</h3>
              <p>
                Great infrastructure should be invisible. Deploy and integrate in minutes, not weeks.
                No complex configuration required.
              </p>
            </div>

            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Performance</h3>
              <p>
                Sub-5ms latency on search, async extraction and embedding, optimized Rust implementation.
                Speed matters for real-time AI applications.
              </p>
            </div>

            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Transparency</h3>
              <p>
                Open source code, clear documentation, honest about limitations. You should understand
                exactly what you&apos;re running.
              </p>
            </div>

            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Developer Empowerment</h3>
              <p>
                Tools should enable experimentation and creativity. We provide the infrastructure layer
                so you can focus on building amazing applications.
              </p>
            </div>
          </div>
        </section>

        {/* Technology */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Technology Stack</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              Knol is built with cutting-edge technologies designed for scale, reliability, and performance:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-2">
              <li><span className="font-semibold text-dark-100">Rust</span> — Memory safety, performance, and low overhead</li>
              <li><span className="font-semibold text-dark-100">PostgreSQL + pgvector</span> — Reliable data storage with native vector support</li>
              <li><span className="font-semibold text-dark-100">NATS JetStream</span> — Scalable async processing and message streaming</li>
              <li><span className="font-semibold text-dark-100">Redis</span> — Fast caching and rate limiting</li>
              <li><span className="font-semibold text-dark-100">Multi-LLM Support</span> — Anthropic Claude, OpenAI, Google Gemini, and more</li>
            </ul>
          </div>
        </section>

        {/* Get Involved */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Get Involved</h2>
          <p className="text-dark-300 mb-6">
            Knol is an open-source project and we welcome contributions, feedback, and ideas from the community.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">GitHub</h3>
              <p className="text-dark-400 text-sm">Star, fork, and contribute code</p>
            </a>

            <a
              href={`${SITE.github}/discussions`}
              target="_blank"
              rel="noopener noreferrer"
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Discussions</h3>
              <p className="text-dark-400 text-sm">Share ideas and ask questions</p>
            </a>

            <a
              href={`${SITE.github}/issues`}
              target="_blank"
              rel="noopener noreferrer"
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Issues</h3>
              <p className="text-dark-400 text-sm">Report bugs and request features</p>
            </a>
          </div>
        </section>

        {/* Resources */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Resources</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <a
              href={SITE.docsUrl}
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Documentation</h3>
              <p className="text-dark-400 text-sm">API reference, guides, and tutorials</p>
            </a>

            <a
              href={SITE.demoUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Live Demo</h3>
              <p className="text-dark-400 text-sm">Try Knol in your browser</p>
            </a>

            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 hover:border-brand-500/50 transition text-center"
            >
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Open Source</h3>
              <p className="text-dark-400 text-sm">View source code on GitHub</p>
            </a>
          </div>
        </section>

        {/* Contact */}
        <section>
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Contact</h2>
          <p className="text-dark-300 mb-6">
            Have questions or want to learn more? We&apos;d love to hear from you.
          </p>
          <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 text-dark-300">
            <p className="mb-2">Email: <a href={`mailto:${SITE.contactEmail}`} className="text-brand-400 hover:text-brand-300">{SITE.contactEmail}</a></p>
            <p className="mb-2">Phone: <a href={`tel:${SITE.contactPhone}`} className="text-brand-400 hover:text-brand-300">{SITE.contactPhoneDisplay}</a></p>
            <p>GitHub: <a href={SITE.github} className="text-brand-400 hover:text-brand-300" target="_blank" rel="noopener noreferrer">github.com/aiknol/knol</a></p>
          </div>
        </section>
      </div>
    </div>
  );
}
