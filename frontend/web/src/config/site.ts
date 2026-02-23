// ── Core site identity and navigation ────────────────────────────

import { resolveAppSignupUrl, resolveDemoUrl, resolveDocsUrl, resolveSiteUrl } from './urls';

export const SITE = {
  name: 'Knol',
  tagline: 'Context Engineering Infrastructure for AI',
  description:
    'Rust-native context engineering platform for LLM applications. One binary, one PostgreSQL database, sub-5ms latency. Hybrid retrieval, knowledge graphs, memory decay, and conflict detection. Deploy in 60 seconds.',
  url: resolveSiteUrl(),
  appUrl: resolveAppSignupUrl(),
  demoUrl: resolveDemoUrl(),
  github: 'https://github.com/aiknol/knol',
  docsUrl: resolveDocsUrl(),
  pypi: 'https://pypi.org/project/knol/',
  npm: 'https://www.npmjs.com/package/@knol-dev/sdk',
  contactEmail: 'aiknolcontact@gmail.com',
  contactPhone: '+14155055990',
  contactPhoneDisplay: '(415) 505-5990',
} as const;

export function pageTitle(title?: string): string {
  return title ? `${title} | ${SITE.name}` : `${SITE.name} - ${SITE.tagline}`;
}

// ── Navigation ──────────────────────────────────────────────────

export interface NavItem {
  href: string;
  label: string;
  external?: boolean;
}

export const NAV_LINKS: NavItem[] = [
  { href: SITE.demoUrl, label: 'Demo' },
  { href: SITE.docsUrl, label: 'Docs' },
  { href: '/mcp/', label: 'MCP' },
  { href: '/comparison', label: 'Compare' },
  { href: '/pricing/', label: 'Pricing' },
  { href: '/blog/', label: 'Blog' },
  { href: `mailto:${SITE.contactEmail}`, label: 'Contact' },
  { href: SITE.github, label: 'GitHub', external: true },
];

// ── Footer ──────────────────────────────────────────────────────

export const FOOTER_SECTIONS = [
  {
    title: 'Product',
    links: [
      { label: 'Live Demo', href: SITE.demoUrl },
      { label: 'Features', href: '/#features' },
      { label: 'Pricing', href: '/pricing/' },
      { label: 'Comparison', href: '/comparison' },
      { label: 'Roadmap', href: `${SITE.github}/issues`, external: true },
    ],
  },
  {
    title: 'Developers',
    links: [
      { label: 'Documentation', href: SITE.docsUrl },
      { label: 'API Reference', href: SITE.docsUrl },
      { label: 'MCP Server', href: '/mcp/' },
      { label: 'GitHub', href: SITE.github, external: true },
      { label: 'PyPI', href: SITE.pypi, external: true },
      { label: 'npm', href: SITE.npm, external: true },
    ],
  },
  {
    title: 'Company',
    links: [
      { label: 'Blog', href: '/blog/' },
      { label: 'About', href: '/about/' },
      { label: 'Contact', href: `mailto:${SITE.contactEmail}` },
      { label: SITE.contactPhoneDisplay, href: `tel:${SITE.contactPhone}` },
    ],
  },
  {
    title: 'Legal',
    links: [
      { label: 'Privacy', href: '/privacy/' },
      { label: 'Terms', href: '/terms/' },
      { label: 'License', href: `${SITE.github}/blob/main/LICENSE` },
    ],
  },
] as const;
