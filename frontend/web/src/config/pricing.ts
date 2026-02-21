import { SITE } from './site';

// ── Pricing tier definitions ────────────────────────────────────

export interface PricingTier {
  name: string;
  price: string;
  period: string;
  description: string;
  features: string[];
  highlighted?: boolean;
  cta: string;
  ctaLink: string;
}

export const PRICING_TIERS: PricingTier[] = [
  {
    name: 'Open Source',
    price: 'Free',
    period: 'forever',
    description: 'Full production stack on your infrastructure',
    features: [
      'All core services (gateway, write, retrieve, graph)',
      'Unlimited ops on your own infrastructure',
      'Hybrid retrieval (vector + BM25 + graph)',
      'Knowledge graph with N-hop traversal',
      'Memory decay, conflict detection',
      'Working + procedural memory',
      'Python, TypeScript, LangChain, CrewAI SDKs',
      'MCP server included',
      'Docker Compose one-command deploy',
      'Apache 2.0 license',
    ],
    cta: 'Get Started',
    ctaLink: SITE.github,
  },
  {
    name: 'Builder',
    price: '$29',
    period: 'month',
    description: 'Fast start for production pilots and POCs',
    features: [
      '100K ops/month included',
      '5 projects',
      'Unlimited end users',
      'Managed hosting & auto-scaling',
      'Transparent overage: $0.50 per 1K ops',
      'Email support (48h SLA)',
    ],
    cta: 'Start Free Trial',
    ctaLink: SITE.appUrl,
  },
  {
    name: 'Growth',
    price: '$199',
    period: 'month',
    description: 'Scaling products with operational control and SLOs',
    features: [
      '500K ops/month included',
      '20 projects',
      'Admin dashboard + audit logs',
      'Memory consolidation + conflict resolution',
      'PII guardrails & data governance',
      'Webhook event system',
      'Transparent overage: $0.40 per 1K ops',
      'Priority support (24h SLA)',
      '99.9% uptime SLA',
    ],
    highlighted: true,
    cta: 'Start Free Trial',
    ctaLink: SITE.appUrl,
  },
  {
    name: 'Enterprise',
    price: 'Custom',
    period: 'annual contract',
    description: 'Compliance, control, and high-throughput',
    features: [
      'Committed ops volume + discounted overage',
      'Dedicated VPC / BYOC deployment',
      'SSO / SAML + SCIM provisioning',
      'SOC 2 / HIPAA compliance path',
      'Custom SLA & dedicated support',
      'Security reviews + architecture consulting',
      'Multi-region deployment',
      'Custom integrations & connectors',
      'On-prem deployment option',
    ],
    cta: 'Contact Sales',
    ctaLink: `mailto:${SITE.contactEmail}`,
  },
];
