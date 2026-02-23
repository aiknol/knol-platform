import type { Metadata } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import { DOCS_SITE } from '@/config/site';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL(DOCS_SITE.siteUrl),
  title: DOCS_SITE.name,
  description: 'Public tenant-service and OSS documentation for Knol.',
  icons: {
    icon: '/favicon.svg',
    shortcut: '/favicon.svg',
    apple: '/favicon.svg',
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <header className="topbar">
          <div className="container topbarInner">
            <Link href="/" className="brand" aria-label="Knol Docs Home">
              <Image src="/favicon.svg" alt="Knol" width={30} height={30} priority />
              <div>
                <p className="brandName">{DOCS_SITE.name}</p>
                <p className="brandSub">{DOCS_SITE.tagline}</p>
              </div>
            </Link>
            <nav className="nav">
              <Link href="/api/">API Reference</Link>
              <Link href="/library/">Documentation Library</Link>
              <a href={DOCS_SITE.githubRepoUrl} target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
            </nav>
          </div>
        </header>
        <main className="container page">{children}</main>
      </body>
    </html>
  );
}
