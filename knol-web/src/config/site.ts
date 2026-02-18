// ── Core site identity and navigation ────────────────────────────

export const SITE = {
  name: 'Knol',
  tagline: 'Context Engineering Infrastructure for AI',
  description:
    'Rust-native context engineering platform for LLM applications. One binary, one PostgreSQL database, sub-5ms latency. Hybrid retrieval, knowledge graphs, memory decay, and conflict detection. Deploy in 60 seconds.',
  url: 'https://aiknol.com',
  appUrl: 'https://app.aiknol.com',
  github: 'https://github.com/aiknol/knol',
  docsUrl: '/docs/',
  pypi: 'https://pypi.org/project/knol/',
  npm: 'https://www.npmjs.com/package/@knol-dev/sdk',
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
  { href: '/demo/', label: 'Demo' },
  { href: '/docs/', label: 'Docs' },
  { href: '/comparison', label: 'Compare' },
  { href: '/pricing/', label: 'Pricing' },
  { href: '/blog/', label: 'Blog' },
  { href: SITE.github, label: 'GitHub', external: true },
];

// ── Footer ──────────────────────────────────────────────────────

export const FOOTER_SECTIONS = [
  {
    title: 'Product',
    links: [
      { label: 'Live Demo', href: '/demo/' },
      { label: 'Features', href: '/#features' },
      { label: 'Pricing', href: '/pricing/' },
      { label: 'Comparison', href: '/comparison' },
      { label: 'Roadmap', href: `${SITE.github}/blob/main/ROADMAP.md`, external: true },
    ],
  },
  {
    title: 'Developers',
    links: [
      { label: 'Documentation', href: '/docs/' },
      { label: 'API Reference', href: '/docs/' },
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
      { label: 'Contact', href: 'mailto:hello@aiknol.com' },
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
