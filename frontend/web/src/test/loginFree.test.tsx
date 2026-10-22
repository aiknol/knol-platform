import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, afterEach } from 'vitest';
import NoAccountNeeded from '@/components/marketing/NoAccountNeeded';

async function prodUrls() {
  vi.resetModules();
  vi.stubEnv('NODE_ENV', 'production');
  return import('@/config/urls');
}

afterEach(() => {
  vi.unstubAllEnvs();
});

describe('login-free defaults (production, no env overrides)', () => {
  it('serves demo and docs on-site instead of unresolvable subdomains', async () => {
    const urls = await prodUrls();
    expect(urls.resolveDemoUrl()).toBe('/demo/');
    expect(urls.resolveDocsUrl()).toBe('/docs/');
  });

  it('keeps login/signup on-site when no hosted app is configured', async () => {
    const urls = await prodUrls();
    expect(urls.resolveAppLoginUrl()).toBe('/login/');
    expect(urls.resolveAppSignupUrl()).toBe('/signup/');
  });

  it('still honours an explicit hosted app URL', async () => {
    vi.stubEnv('NEXT_PUBLIC_APP_URL', 'https://app.example.com');
    const urls = await prodUrls();
    expect(urls.resolveAppSignupUrl()).toBe('https://app.example.com/signup/');
  });
});

describe('NoAccountNeeded', () => {
  it('links to every path that works without an account', () => {
    render(<NoAccountNeeded heading="No login needed" />);
    expect(screen.getByRole('heading', { name: 'No login needed' })).toBeInTheDocument();
    const hrefs = screen.getAllByRole('link').map((a) => a.getAttribute('href')?.replace(/\/$/, ''));
    for (const href of ['/demo', '/cortex-local', '/cortex-oss', '/docs']) {
      expect(hrefs).toContain(href);
    }
  });
});
