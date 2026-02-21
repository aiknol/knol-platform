import type { Metadata } from 'next';
import './globals.css';
import Navbar from '@/components/layout/Navbar';
import Footer from '@/components/layout/Footer';
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

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="font-sans bg-gradient-dark min-h-screen">
        <Navbar />
        <main className="pt-16">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
