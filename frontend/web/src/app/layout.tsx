import type { Metadata } from 'next';
import './globals.css';
import Navbar from '@/components/layout/Navbar';
import Footer from '@/components/layout/Footer';
import HMRErrorHandler from '@/components/HMRErrorHandler';
import { SITE, pageTitle } from '@/config/site';

export const metadata: Metadata = {
  metadataBase: new URL(SITE.url),
  title: pageTitle(),
  description: SITE.description,
  keywords: ['context engineering', 'AI memory', 'LLM memory', 'knowledge graph', 'vector search', 'RAG', 'Rust', 'PostgreSQL', 'pgvector', SITE.name],
  icons: {
    icon: '/favicon.svg',
    shortcut: '/favicon.svg',
    apple: '/favicon.svg',
  },
  openGraph: {
    title: SITE.name,
    description: SITE.tagline,
    url: SITE.url,
    siteName: SITE.name,
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: SITE.name,
    description: SITE.tagline,
  },
};

const softwareAppJsonLd = {
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  name: 'DoAide Cortex',
  applicationCategory: 'DeveloperApplication',
  operatingSystem: 'Linux, macOS, Windows',
  description: SITE.description,
  url: SITE.url,
  downloadUrl: 'https://github.com/doaide/cortex-platform',
  softwareVersion: '0.2',
  releaseNotes: 'https://github.com/doaide/cortex-platform/releases',
  license: 'https://github.com/doaide/cortex-platform/blob/main/LICENSE',
  offers: {
    '@type': 'Offer',
    price: '0',
    priceCurrency: 'USD',
    description: 'Open-source self-hosted deployment (Apache 2.0)',
  },
  author: {
    '@type': 'Organization',
    name: 'DoAide',
    url: 'https://doaide.com',
    email: SITE.contactEmail,
    logo: `${SITE.url}/favicon.svg`,
  },
  featureList: [
    'Hybrid retrieval (vector + BM25 + graph)',
    'Knowledge graph storage on PostgreSQL',
    'Memory decay and consolidation',
    'Conflict detection and resolution',
    'Sub-5ms gateway latency',
    'LLM cost optimization (75% reduction)',
    'Python and TypeScript SDKs',
    'LangChain and CrewAI integrations',
    'MCP server for Claude Desktop',
    'Multi-tenant row-level security',
  ],
};

const organizationJsonLd = {
  '@context': 'https://schema.org',
  '@type': 'Organization',
  name: 'DoAide',
  url: 'https://doaide.com',
  logo: `${SITE.url}/favicon.svg`,
  email: SITE.contactEmail,
  telephone: SITE.contactPhone,
  sameAs: [
    'https://github.com/doaide',
    'https://www.linkedin.com/in/sumaninster/',
  ],
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="font-sans bg-gradient-dark min-h-screen">
        <HMRErrorHandler />
        <Navbar />
        <main className="pt-16">{children}</main>
        <Footer />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareAppJsonLd) }}
        />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationJsonLd) }}
        />
      </body>
    </html>
  );
}
