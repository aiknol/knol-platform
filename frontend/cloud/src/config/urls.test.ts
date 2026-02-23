import { describe, it, expect, vi, beforeEach } from 'vitest';

function clearEnv() {
  for (const key of Object.keys(process.env)) {
    if (key.startsWith('NEXT_PUBLIC_')) {
      delete process.env[key];
    }
  }
}

async function importUrlsFresh() {
  vi.resetModules();
  return import('./urls');
}

beforeEach(() => {
  clearEnv();
});

describe('URL resolvers', () => {
  it('resolveAppApiUrl returns explicit env', async () => {
    process.env.NEXT_PUBLIC_APP_API_URL = 'http://custom:9000';
    const { resolveAppApiUrl } = await importUrlsFresh();
    expect(resolveAppApiUrl()).toBe('http://custom:9000');
  });

  it('resolveAppApiUrl returns dev default', async () => {
    // NODE_ENV is 'test' which counts as not production (IS_DEV = true)
    const { resolveAppApiUrl } = await importUrlsFresh();
    expect(resolveAppApiUrl()).toBe('http://localhost:8085');
  });

  it('resolveAdminApiUrl returns explicit env', async () => {
    process.env.NEXT_PUBLIC_ADMIN_API_URL = 'http://api:3001';
    const { resolveAdminApiUrl } = await importUrlsFresh();
    expect(resolveAdminApiUrl()).toBe('http://api:3001');
  });

  it('resolveAdminApiUrl returns dev default', async () => {
    const { resolveAdminApiUrl } = await importUrlsFresh();
    expect(resolveAdminApiUrl()).toBe('http://localhost:3001');
  });

  it('resolveSiteUrl returns default', async () => {
    const { resolveSiteUrl } = await importUrlsFresh();
    // In dev with default BASE_DOMAIN=localhost, SITE_ORIGIN resolves to http://localhost:3005
    const url = resolveSiteUrl();
    expect(url).toBeTruthy();
  });

  it('resolveAppSignupUrl contains /signup/', async () => {
    const { resolveAppSignupUrl } = await importUrlsFresh();
    expect(resolveAppSignupUrl()).toContain('/signup/');
  });

  it('resolveAppLoginUrl contains /login/', async () => {
    const { resolveAppLoginUrl } = await importUrlsFresh();
    expect(resolveAppLoginUrl()).toContain('/login/');
  });

  it('resolveDemoUrl returns path in dev', async () => {
    const { resolveDemoUrl } = await importUrlsFresh();
    const url = resolveDemoUrl();
    expect(url).toBeTruthy();
  });

  it('resolveDocsUrl returns a URL', async () => {
    const { resolveDocsUrl } = await importUrlsFresh();
    const url = resolveDocsUrl();
    expect(url).toMatch(/^https?:\/\//);
  });

  it('respects custom base domain', async () => {
    process.env.NEXT_PUBLIC_BASE_DOMAIN = 'myapp.com';
    process.env.NEXT_PUBLIC_URL_SCHEME = 'https';
    const { resolveAppApiUrl } = await importUrlsFresh();
    const url = resolveAppApiUrl();
    expect(url).toContain('myapp.com');
  });
});
