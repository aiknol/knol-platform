import { describe, it, expect, vi, beforeEach } from 'vitest';

// Helper to import env module fresh (bypasses cached `validated` flag)
async function importEnvFresh() {
  vi.resetModules();
  return import('./env');
}

function setEnv(overrides: Record<string, string>) {
  // Clear all NEXT_PUBLIC_ vars first
  for (const key of Object.keys(process.env)) {
    if (key.startsWith('NEXT_PUBLIC_')) {
      delete process.env[key];
    }
  }
  for (const [key, value] of Object.entries(overrides)) {
    process.env[key] = value;
  }
}

beforeEach(() => {
  // Clear all relevant env vars
  for (const key of Object.keys(process.env)) {
    if (key.startsWith('NEXT_PUBLIC_')) {
      delete process.env[key];
    }
  }
});

describe('ensurePublicEnvIsValid', () => {
  it('passes with no env vars set', async () => {
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('accepts valid scheme http', async () => {
    setEnv({ NEXT_PUBLIC_URL_SCHEME: 'http' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('accepts valid scheme https', async () => {
    setEnv({ NEXT_PUBLIC_URL_SCHEME: 'https' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('rejects invalid scheme', async () => {
    setEnv({ NEXT_PUBLIC_URL_SCHEME: 'ftp' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must be "http" or "https"/);
  });

  it('accepts valid host', async () => {
    setEnv({ NEXT_PUBLIC_BASE_DOMAIN: 'localhost' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('rejects host with protocol', async () => {
    setEnv({ NEXT_PUBLIC_BASE_DOMAIN: 'http://foo' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must be a host without protocol/);
  });

  it('rejects host with slash', async () => {
    setEnv({ NEXT_PUBLIC_BASE_DOMAIN: 'foo/bar' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must be a host without protocol/);
  });

  it('accepts valid port', async () => {
    setEnv({ NEXT_PUBLIC_MAIN_PORT: '3005' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('rejects non-numeric port', async () => {
    setEnv({ NEXT_PUBLIC_MAIN_PORT: 'abc' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must be numeric/);
  });

  it('rejects port out of range', async () => {
    setEnv({ NEXT_PUBLIC_MAIN_PORT: '99999' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must be between 1 and 65535/);
  });

  it('accepts valid URL', async () => {
    setEnv({ NEXT_PUBLIC_SITE_URL: 'https://example.com' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).not.toThrow();
  });

  it('rejects URL without protocol', async () => {
    setEnv({ NEXT_PUBLIC_SITE_URL: 'example.com' });
    const { ensurePublicEnvIsValid } = await importEnvFresh();
    expect(() => ensurePublicEnvIsValid()).toThrow(/must start with http/);
  });
});
