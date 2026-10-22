import Link from 'next/link';
import { SITE } from '@/config/site';

const PATHS = [
  { href: '/demo/', title: 'Try the live demo', desc: 'Watch Cortex learn, remember and build a knowledge graph — right in your browser.' },
  { href: '/cortex-local/', title: 'Run Cortex Local', desc: 'Local MCP memory for Claude Desktop, Cursor and Windsurf. One npm install, no account.' },
  { href: '/cortex-oss/', title: 'Self-host Cortex OSS', desc: 'Apache 2.0. One binary and one PostgreSQL database on your own infrastructure.' },
  { href: '/docs/', title: 'Read the docs', desc: 'API reference, Python and TypeScript SDKs, and integration guides.' },
] as const;

export default function NoAccountNeeded({ heading }: { heading: string }) {
  return (
    <section className="px-4 sm:px-6 lg:px-8 py-16 md:py-24">
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl md:text-4xl font-semibold text-dark-50 tracking-tight text-center">{heading}</h1>
        <p className="mt-4 text-dark-300 text-center">
          You don&apos;t need an account to use {SITE.name}. Hosted cloud accounts are opening soon —
          everything below works today, free and without signing up.
        </p>
        <div className="mt-10 grid grid-cols-1 sm:grid-cols-2 gap-4">
          {PATHS.map((p) => (
            <Link key={p.href} href={p.href} className="card block min-h-[48px] hover:border-brand-500/50 transition-colors">
              <p className="text-dark-50 font-semibold">{p.title} &rarr;</p>
              <p className="mt-2 text-dark-400 text-sm leading-relaxed">{p.desc}</p>
            </Link>
          ))}
        </div>
        <p className="mt-10 text-center text-dark-400 text-sm">
          Want early access to the hosted cloud?{' '}
          <a href={`mailto:${SITE.contactEmail}?subject=Cortex%20cloud%20early%20access`} className="text-brand-400 hover:text-brand-300 inline-flex items-center min-h-[48px]">
            Email {SITE.contactEmail}
          </a>
        </p>
      </div>
    </section>
  );
}
