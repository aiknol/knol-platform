import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

function mockNextRequest(payload: unknown, headers: Record<string, string> = {}) {
  return {
    json: async () => payload,
    headers: new Headers(headers),
  } as any;
}

describe('Playground proxy route', () => {
  const originalEnv = { ...process.env };

  beforeEach(() => {
    vi.restoreAllMocks();
    process.env = { ...originalEnv };
  });

  afterEach(() => {
    process.env = { ...originalEnv };
  });

  it('rejects non-https URLs when not rewriting', async () => {
    process.env.GATEWAY_INTERNAL_URL = '';
    vi.resetModules();
    const { POST } = await import('./route');

    const res = await POST(
      mockNextRequest({ url: 'http://example.com/test', method: 'GET', headers: {} }),
    );
    expect(res.status).toBe(400);
    const json = await res.json();
    expect(json.error).toContain('Only HTTPS URLs are allowed');
  });

  it('allows localhost http URLs when not rewriting', async () => {
    process.env.GATEWAY_INTERNAL_URL = '';
    vi.resetModules();
    const { POST } = await import('./route');

    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response('ok', { status: 200 }));

    const res = await POST(
      mockNextRequest({ url: 'http://localhost:3000/health', method: 'GET', headers: {} }),
    );

    expect(res.status).toBe(200);
    const json = await res.json();
    expect(json.status).toBe(200);
    expect(json.body).toBe('ok');
  });

  it('forwards client IP and returns upstream response', async () => {
    process.env.GATEWAY_INTERNAL_URL = '';
    vi.resetModules();
    const { POST } = await import('./route');

    const fetchSpy = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValue(new Response('ok', { status: 200 }));

    const res = await POST(
      mockNextRequest(
        { url: 'https://example.com/test', method: 'GET', headers: { Authorization: 'Bearer x' } },
        { 'x-forwarded-for': '1.2.3.4, 5.6.7.8' },
      ),
    );

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const [, opts] = fetchSpy.mock.calls[0];
    expect((opts as any).headers['X-Forwarded-For']).toBe('1.2.3.4');

    expect(res.status).toBe(200);
    const json = await res.json();
    expect(json.status).toBe(200);
    expect(json.body).toBe('ok');
  });

  it('truncates large upstream responses', async () => {
    process.env.GATEWAY_INTERNAL_URL = '';
    vi.resetModules();
    const { POST } = await import('./route');

    const big = 'a'.repeat(1_000_000 + 10);
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(big, { status: 200 }));

    const res = await POST(
      mockNextRequest({ url: 'https://example.com/big', method: 'GET', headers: {} }),
    );
    const json = await res.json();
    expect(json.status).toBe(200);
    expect(typeof json.body).toBe('string');
    expect(json.body).toContain('[truncated: upstream response exceeded');
    expect(json.body.startsWith('a')).toBe(true);
    expect(json.body.length).toBeGreaterThan(1_000_000);
  });

  it('returns 504 on AbortError', async () => {
    process.env.GATEWAY_INTERNAL_URL = '';
    vi.resetModules();
    const { POST } = await import('./route');

    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new DOMException('Aborted', 'AbortError'));

    const res = await POST(
      mockNextRequest({ url: 'https://example.com/slow', method: 'GET', headers: {} }),
    );
    expect(res.status).toBe(504);
    const json = await res.json();
    expect(json.error).toContain('Gateway did not respond');
  });
});
