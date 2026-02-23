const HTTP_URL_RE = /^https?:\/\//i;
const HOST_RE = /^[a-zA-Z0-9.-]+$/;
const PORT_RE = /^\d+$/;

function readEnv(name: string): string | null {
  const value = process.env[name]?.trim();
  return value ? value : null;
}

function assertHttpUrl(name: string): void {
  const value = readEnv(name);
  if (!value) return;
  if (!HTTP_URL_RE.test(value)) {
    throw new Error(`[env] ${name} must start with http:// or https://. Received: "${value}"`);
  }
}

function assertHost(name: string): void {
  const value = readEnv(name);
  if (!value) return;
  if (HTTP_URL_RE.test(value) || value.includes('/')) {
    throw new Error(`[env] ${name} must be a host without protocol/path. Received: "${value}"`);
  }
  if (!HOST_RE.test(value)) {
    throw new Error(`[env] ${name} contains invalid host characters. Received: "${value}"`);
  }
}

function assertPort(name: string): void {
  const value = readEnv(name);
  if (!value) return;
  if (!PORT_RE.test(value)) {
    throw new Error(`[env] ${name} must be numeric. Received: "${value}"`);
  }
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < 1 || parsed > 65535) {
    throw new Error(`[env] ${name} must be between 1 and 65535. Received: "${value}"`);
  }
}

function assertScheme(name: string): void {
  const value = readEnv(name);
  if (!value) return;
  if (value !== 'http' && value !== 'https') {
    throw new Error(`[env] ${name} must be "http" or "https". Received: "${value}"`);
  }
}

let validated = false;

export function ensurePublicEnvIsValid(): void {
  if (validated) return;

  assertScheme('NEXT_PUBLIC_URL_SCHEME');
  assertHost('NEXT_PUBLIC_BASE_DOMAIN');
  assertHttpUrl('NEXT_PUBLIC_SITE_URL');
  assertHost('NEXT_PUBLIC_MAIN_HOST');
  assertPort('NEXT_PUBLIC_MAIN_PORT');
  assertHost('NEXT_PUBLIC_APP_HOST');
  assertPort('NEXT_PUBLIC_APP_PORT');
  assertHost('NEXT_PUBLIC_DEMO_HOST');
  assertPort('NEXT_PUBLIC_DEMO_PORT');
  assertHost('NEXT_PUBLIC_DOCS_HOST');
  assertPort('NEXT_PUBLIC_DOCS_PORT');
  assertHost('NEXT_PUBLIC_ADMIN_API_HOST');
  assertPort('NEXT_PUBLIC_ADMIN_API_PORT');
  assertHttpUrl('NEXT_PUBLIC_ADMIN_API_URL');
  assertHttpUrl('NEXT_PUBLIC_APP_API_URL');
  assertHttpUrl('NEXT_PUBLIC_DEMO_URL');
  assertHttpUrl('NEXT_PUBLIC_DOCS_URL');
  assertHttpUrl('NEXT_PUBLIC_APP_URL');
  assertHttpUrl('NEXT_PUBLIC_APP_SIGNUP_URL');
  assertHttpUrl('NEXT_PUBLIC_APP_LOGIN_URL');

  validated = true;
}
