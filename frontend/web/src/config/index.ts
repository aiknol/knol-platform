// Barrel export — import from '@/config' or '@/config/site' etc.
export { SITE, NAV_LINKS, FOOTER_SECTIONS, pageTitle } from './site';
export type { NavItem } from './site';
export { TECH_STACK, KEY_METRICS, MEMORY_TYPES, USE_CASES, COMPARISON_FEATURES, BLOG_POSTS, SDK_ECOSYSTEM } from './marketing';
export type { BlogPost } from './marketing';
export { PRICING_TIERS } from './pricing';
export type { PricingTier } from './pricing';
export { resolveSiteUrl, resolveAppSignupUrl, resolveAppLoginUrl, resolveDemoUrl, resolveAdminApiUrl, resolveAppApiUrl } from './urls';
