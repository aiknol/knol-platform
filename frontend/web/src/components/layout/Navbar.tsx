'use client';

import Link from 'next/link';
import { useState } from 'react';
import { SITE, NAV_LINKS } from '@/config/site';
import KnolLogo from '@/components/layout/KnolLogo';

const GITHUB_ICON = (
  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24" aria-label="GitHub" role="img">
    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
  </svg>
);

export default function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <header className="fixed top-0 w-full bg-dark-900/95 backdrop-blur-md border-b border-dark-600/30 z-50">
      <nav className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between h-16">
        <Link href="/" className="flex items-center gap-3">
          <KnolLogo className="w-8 h-8" label="Knol home" />
          <span className="text-lg font-medium tracking-tight text-dark-50">{SITE.name}</span>
        </Link>

        <ul className="hidden md:flex items-center gap-8">
          {NAV_LINKS.map((link) => (
            <li key={link.href}>
              {link.external ? (
                <a href={link.href} target="_blank" rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 text-dark-300 hover:text-dark-50 transition-colors text-sm border border-dark-600/40 rounded-lg px-3 py-1.5 hover:border-dark-500/60">
                  {GITHUB_ICON}
                  {link.label}
                </a>
              ) : link.href.startsWith('http://') || link.href.startsWith('https://') ? (
                <a href={link.href} target="_blank" rel="noopener noreferrer" className="text-dark-300 hover:text-dark-50 transition-colors text-sm">
                  {link.label}
                </a>
              ) : (
                <Link href={link.href} className="text-dark-300 hover:text-dark-50 transition-colors text-sm">
                  {link.label}
                </Link>
              )}
            </li>
          ))}
          <li>
            <a href={SITE.appUrl} target="_blank" rel="noopener noreferrer" className="btn-primary text-sm !py-2 !px-4">Get Started Free</a>
          </li>
        </ul>

        <button className="md:hidden text-dark-100" onClick={() => setMobileOpen(!mobileOpen)} aria-label="Toggle menu">
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            {mobileOpen
              ? <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              : <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />}
          </svg>
        </button>
      </nav>

      {mobileOpen && (
        <div className="md:hidden bg-dark-800 border-t border-dark-600/30 px-4 py-4 space-y-3">
          {NAV_LINKS.map((link) => {
            const isExternal = 'external' in link && link.external;
            const isAbsolute = link.href.startsWith('http://') || link.href.startsWith('https://');
            return isExternal ? (
              <a key={link.href} href={link.href} target="_blank" rel="noopener noreferrer" className="block text-dark-300 hover:text-dark-50" onClick={() => setMobileOpen(false)}>
                {link.label}
              </a>
            ) : isAbsolute ? (
              <a key={link.href} href={link.href} target="_blank" rel="noopener noreferrer" className="block text-dark-300 hover:text-dark-50" onClick={() => setMobileOpen(false)}>
                {link.label}
              </a>
            ) : (
              <Link key={link.href} href={link.href} className="block text-dark-300 hover:text-dark-50" onClick={() => setMobileOpen(false)}>
                {link.label}
              </Link>
            );
          })}
          <a href={SITE.github} className="block text-dark-300 hover:text-dark-50">GitHub</a>
          <a href={SITE.appUrl} target="_blank" rel="noopener noreferrer" className="block btn-primary text-sm text-center mt-2" onClick={() => setMobileOpen(false)}>Get Started Free</a>
        </div>
      )}
    </header>
  );
}
