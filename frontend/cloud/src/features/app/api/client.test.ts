import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock session module
const sessionMocks = vi.hoisted(() => ({
  getAppAuthToken: vi.fn().mockReturnValue(null),
  clearAppAuthSession: vi.fn(),
}));

// Mock urls module
const urlMocks = vi.hoisted(() => ({
  resolveAppApiUrl: vi.fn().mockReturnValue('http://localhost:8085'),
}));

vi.mock('@/config/urls', () => ({
  resolveAppApiUrl: urlMocks.resolveAppApiUrl,
}));

vi.mock('./session', () => ({
  getAppAuthToken: sessionMocks.getAppAuthToken,
  clearAppAuthSession: sessionMocks.clearAppAuthSession,
}));

// Must import after mocks
import { apiFetch } from './client';

function mockFetchResponse(status: number, body: any, contentType = 'application/json') {
  const headers = new Headers({ 'content-type': contentType });
  return vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    headers,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(typeof body === 'string' ? body : JSON.stringify(body)),
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('apiFetch', () => {
  it('GET success parses JSON', async () => {
    globalThis.fetch = mockFetchResponse(200, { data: 'hello' });
    const result = await apiFetch('/app/test');
    expect(result).toEqual({ data: 'hello' });
  });

  it('POST sends JSON body', async () => {
    globalThis.fetch = mockFetchResponse(200, { ok: true });
    await apiFetch('/app/test', {
      method: 'POST',
      body: JSON.stringify({ name: 'test' }),
    });
    expect(fetch).toHaveBeenCalledTimes(1);
    const call = (fetch as any).mock.calls[0];
    expect(call[0]).toBe('http://localhost:8085/app/test');
  });

  it('401 clears session and redirects', async () => {
    globalThis.fetch = mockFetchResponse(401, { error: 'Unauthorized' });
    // Mock window.location
    const originalLocation = window.location;
    Object.defineProperty(window, 'location', {
      writable: true,
      value: { href: '/' },
    });
    await expect(apiFetch('/app/test')).rejects.toThrow('Unauthorized');
    expect(sessionMocks.clearAppAuthSession).toHaveBeenCalled();
    expect(window.location.href).toBe('/login');
    Object.defineProperty(window, 'location', {
      writable: true,
      value: originalLocation,
    });
  });

  it('401 with skipRedirect does not redirect', async () => {
    globalThis.fetch = mockFetchResponse(401, { error: 'Unauthorized' });
    await expect(apiFetch('/app/test', { skipRedirect: true })).rejects.toThrow('Unauthorized');
    expect(sessionMocks.clearAppAuthSession).not.toHaveBeenCalled();
  });

  it('500 extracts JSON error message', async () => {
    globalThis.fetch = mockFetchResponse(500, { error: 'Internal failure' });
    await expect(apiFetch('/app/test')).rejects.toThrow('Internal failure');
  });

  it('500 extracts text error message', async () => {
    globalThis.fetch = mockFetchResponse(500, 'Server error text', 'text/plain');
    await expect(apiFetch('/app/test')).rejects.toThrow('Server error text');
  });

  it('sends credentials include', async () => {
    globalThis.fetch = mockFetchResponse(200, {});
    await apiFetch('/app/test');
    const call = (fetch as any).mock.calls[0];
    expect(call[1].credentials).toBe('include');
  });

  it('sets Content-Type to JSON', async () => {
    globalThis.fetch = mockFetchResponse(200, {});
    await apiFetch('/app/test');
    const call = (fetch as any).mock.calls[0];
    const headers = call[1].headers;
    expect(headers.get('Content-Type')).toBe('application/json');
  });

  it('returns text for non-JSON response', async () => {
    globalThis.fetch = mockFetchResponse(200, 'plain text', 'text/plain');
    const result = await apiFetch('/app/test');
    expect(result).toBe('plain text');
  });
});
