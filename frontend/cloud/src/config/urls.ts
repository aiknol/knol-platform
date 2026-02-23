import { ensurePublicEnvIsValid } from './env';

ensurePublicEnvIsValid();

const HTTP_URL_RE = /^https?:\/\//i;
const IS_DEV = process.env.NODE_ENV !== 'production';

function readEnv(name: string): string | null {
  const value = process.env[name]?.trim();
  return value ? value : null;
}

function stripTrailingSlash(value: string): string {
  return value.replace(/\/+$/, '');
}

function ensureTrailingSlash(value: string): string {
  return `${stripTrailingSlash(value)}/`;
}

function ensureSignupUrl(value: string): string {
  const normalized = stripTrailingSlash(value);
  if (normalized.endsWith('/signup')) {
    return `${normalized}/`;
  }
  return `${normalized}/signup/`;
}

function normalizeScheme(value: string | null, hostHint: string | null): string {
  if (!value) {
    if (hostHint && isLocalHostDomain(hostHint)) return 'http';
    return IS_DEV ? 'http' : 'https';
  }
  return value.replace(':', '');
}

function isLocalHostDomain(value: string): boolean {
  return value === 'localhost' || value === '127.0.0.1';
}

function isIpv4Host(value: string): boolean {
  return /^\d{1,3}(?:\.\d{1,3}){3}$/.test(value);
}

function defaultHostFor(baseDomain: string | null, servicePrefix?: string): string | null {
  if (!baseDomain) return null;
  if (!servicePrefix || isLocalHostDomain(baseDomain)) {
    return baseDomain;
  }
  return `${servicePrefix}.${baseDomain}`;
}

function buildOrigin(hostValue: string, scheme: string, portValue: string | null): string {
  if (HTTP_URL_RE.test(hostValue)) {
    return stripTrailingSlash(hostValue);
  }

  const normalizedPort = portValue?.trim();
  const portSuffix = normalizedPort ? `:${normalizedPort}` : '';
  return `${scheme}://${hostValue}${portSuffix}`;
}

function defaultPort(portValue: string | null, fallbackPort: string): string | null {
  if (portValue) return portValue;
  return IS_DEV ? fallbackPort : null;
}

function resolveOrigin(
  explicitUrlEnv: string | null,
  explicitHostEnv: string | null,
  explicitPortEnv: string | null,
  fallbackHost: string | null,
  scheme: string,
): string | null {
  if (explicitUrlEnv) {
    return stripTrailingSlash(explicitUrlEnv);
  }

  const host = explicitHostEnv || fallbackHost;
  if (!host) return null;
  return buildOrigin(host, scheme, explicitPortEnv);
}

const BASE_DOMAIN = readEnv('NEXT_PUBLIC_BASE_DOMAIN') || (IS_DEV ? 'localhost' : null);
const URL_SCHEME = normalizeScheme(readEnv('NEXT_PUBLIC_URL_SCHEME'), BASE_DOMAIN);

const SITE_ORIGIN = resolveOrigin(
  readEnv('NEXT_PUBLIC_SITE_URL'),
  readEnv('NEXT_PUBLIC_MAIN_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_MAIN_PORT'), '3005'),
  defaultHostFor(BASE_DOMAIN),
  URL_SCHEME,
);

const APP_ORIGIN = resolveOrigin(
  null,
  readEnv('NEXT_PUBLIC_APP_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_APP_PORT'), '3007'),
  defaultHostFor(BASE_DOMAIN, 'cloud'),
  URL_SCHEME,
);

const DEMO_ORIGIN = resolveOrigin(
  null,
  readEnv('NEXT_PUBLIC_DEMO_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_DEMO_PORT'), '3008'),
  defaultHostFor(BASE_DOMAIN, 'demo'),
  URL_SCHEME,
);

const DOCS_ORIGIN = resolveOrigin(
  readEnv('NEXT_PUBLIC_DOCS_URL'),
  readEnv('NEXT_PUBLIC_DOCS_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_DOCS_PORT'), '3009'),
  defaultHostFor(BASE_DOMAIN, 'docs'),
  URL_SCHEME,
);

const ADMIN_API_ORIGIN = resolveOrigin(
  null,
  readEnv('NEXT_PUBLIC_ADMIN_API_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_ADMIN_API_PORT'), '3001'),
  defaultHostFor(BASE_DOMAIN, 'api'),
  URL_SCHEME,
);

const TENANT_API_ORIGIN = resolveOrigin(
  null,
  readEnv('NEXT_PUBLIC_TENANT_API_HOST'),
  defaultPort(readEnv('NEXT_PUBLIC_TENANT_API_PORT'), '8085'),
  defaultHostFor(BASE_DOMAIN, 'cloud-api'),
  URL_SCHEME,
);

export function resolveSiteUrl(): string {
  return SITE_ORIGIN || 'https://aiknol.com';
}

export function resolveAppSignupUrl(): string {
  const explicitSignup = readEnv('NEXT_PUBLIC_APP_SIGNUP_URL');
  if (explicitSignup) return ensureSignupUrl(explicitSignup);

  const explicitApp = readEnv('NEXT_PUBLIC_APP_URL');
  if (explicitApp) return ensureSignupUrl(explicitApp);

  if (APP_ORIGIN) return `${APP_ORIGIN}/signup/`;
  return 'https://cloud.aiknol.com/signup/';
}

export function resolveAppLoginUrl(): string {
  const explicitLogin = readEnv('NEXT_PUBLIC_APP_LOGIN_URL');
  if (explicitLogin) return ensureTrailingSlash(explicitLogin);

  const explicitApp = readEnv('NEXT_PUBLIC_APP_URL');
  if (explicitApp) return `${stripTrailingSlash(explicitApp)}/login/`;

  if (APP_ORIGIN) return `${APP_ORIGIN}/login/`;
  return 'https://cloud.aiknol.com/login/';
}

export function resolveDemoUrl(): string {
  const explicit = readEnv('NEXT_PUBLIC_DEMO_URL');
  if (explicit) return ensureTrailingSlash(explicit);
  if (DEMO_ORIGIN) return `${DEMO_ORIGIN}/`;
  return '/demo/';
}

export function resolveDocsUrl(): string {
  if (DOCS_ORIGIN) return `${DOCS_ORIGIN}/`;
  return 'https://docs.aiknol.com/';
}

function inferApiOriginFromLocation(): string | null {
  if (typeof window === 'undefined') return null;

  const { protocol, hostname } = window.location;
  if (!hostname) return null;

  if (isLocalHostDomain(hostname) || isIpv4Host(hostname)) {
    const localApiPort = defaultPort(readEnv('NEXT_PUBLIC_ADMIN_API_PORT'), '3001') || '3001';
    return `${protocol}//${hostname}:${localApiPort}`;
  }

  const labels = hostname.split('.').filter(Boolean);
  if (labels.length < 2) return null;

  const apex = labels.length > 2 ? labels.slice(1).join('.') : labels.join('.');
  return `${protocol}//api.${apex}`;
}

function resolveApiFallbackOrigin(): string {
  if (ADMIN_API_ORIGIN) return ADMIN_API_ORIGIN;

  const inferred = inferApiOriginFromLocation();
  if (inferred) return stripTrailingSlash(inferred);

  return IS_DEV ? 'http://localhost:3001' : 'https://api.aiknol.com';
}

export function resolveAdminApiUrl(): string {
  const explicit = readEnv('NEXT_PUBLIC_ADMIN_API_URL');
  if (explicit) return stripTrailingSlash(explicit);
  return resolveApiFallbackOrigin();
}

export function resolveAppApiUrl(): string {
  const explicit = readEnv('NEXT_PUBLIC_APP_API_URL');
  if (explicit) return stripTrailingSlash(explicit);

  // In production, the tenant API is typically behind the same API gateway.
  // In development, the tenant service runs on its own port (default 8085).
  if (TENANT_API_ORIGIN) return TENANT_API_ORIGIN;

  if (IS_DEV) return 'http://localhost:8085';

  // Production fallback: tenant API shares the same origin as the admin API
  return resolveAdminApiUrl();
}
