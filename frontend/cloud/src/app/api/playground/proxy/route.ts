import { NextRequest, NextResponse } from 'next/server';

/**
 * Server-side proxy for Playground gateway requests.
 *
 * The Playground needs to call the tenant gateway (e.g. https://api.aiknol.com)
 * with a Bearer token.  Browsers block these cross-origin requests when the
 * gateway doesn't return Access-Control-Allow-Origin.  By routing through this
 * Next.js API route the request happens server-side where CORS doesn't apply.
 *
 * In development the gateway typically runs locally (docker-compose maps it to
 * port 3000).  Set GATEWAY_INTERNAL_URL to point the proxy at the local
 * instance instead of the public URL the client sends.
 *
 * Accepts POST with JSON body:
 *   { url: string, method: string, headers: Record<string,string>, body?: string }
 *
 * Returns JSON:
 *   { status: number, body: string }
 */

/**
 * Optional internal gateway URL.
 * When set, the proxy rewrites the *origin* of incoming gateway URLs to this
 * value while preserving path + query string.
 *
 * Example: set to "http://localhost:3000" to route requests to a local gateway.
 */
const GATEWAY_INTERNAL_URL = process.env.GATEWAY_INTERNAL_URL?.trim() || '';

/** Timeout for upstream gateway requests (ms). */
const UPSTREAM_TIMEOUT_MS = 15_000;

/** Max upstream response size to buffer (bytes). */
const MAX_UPSTREAM_BYTES = 1_000_000;

async function readTextLimited(res: Response, limitBytes: number): Promise<string> {
  const body = res.body;
  if (!body) return await res.text();

  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  let truncated = false;

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      if (!value) continue;

      if (total + value.byteLength > limitBytes) {
        const remaining = Math.max(0, limitBytes - total);
        if (remaining > 0) chunks.push(value.slice(0, remaining));
        truncated = true;
        break;
      }

      chunks.push(value);
      total += value.byteLength;
    }
  } finally {
    try {
      reader.releaseLock();
    } catch {
      // ignore
    }
  }

  const merged = new Uint8Array(chunks.reduce((acc, c) => acc + c.byteLength, 0));
  let offset = 0;
  for (const c of chunks) {
    merged.set(c, offset);
    offset += c.byteLength;
  }

  const text = new TextDecoder().decode(merged);
  return truncated ? `${text}\n\n[truncated: upstream response exceeded ${limitBytes} bytes]` : text;
}

/**
 * Rewrite the public gateway URL to the internal one by preserving the path
 * and query string.  If no internal URL is configured the original is returned.
 */
function rewriteUrl(parsed: URL): string {
  if (!GATEWAY_INTERNAL_URL) return parsed.toString();
  return `${GATEWAY_INTERNAL_URL}${parsed.pathname}${parsed.search}`;
}

function isLocalHttpUrl(parsed: URL): boolean {
  if (parsed.protocol !== 'http:') return false;
  const host = parsed.hostname;
  return host === 'localhost' || host === '127.0.0.1' || host === '::1';
}

export async function POST(request: NextRequest) {
  try {
    const { url, method, headers, body } = await request.json();

    if (!url || typeof url !== 'string') {
      return NextResponse.json({ error: 'Missing url' }, { status: 400 });
    }

    let parsed: URL;
    try {
      parsed = new URL(url);
    } catch {
      return NextResponse.json({ error: 'Invalid url (must be absolute)' }, { status: 400 });
    }

    const targetUrl = rewriteUrl(parsed);

    // When NOT rewriting, only allow HTTPS to prevent SSRF to internal services.
    // When rewriting to a known internal URL the scheme check is skipped.
    if (targetUrl === url) {
      if (parsed.protocol !== 'https:' && !isLocalHttpUrl(parsed)) {
        return NextResponse.json(
          { error: 'Only HTTPS URLs are allowed (except http://localhost in dev)' },
          { status: 400 },
        );
      }
    }

    // Forward the client IP so the gateway can rate-limit per real client
    // instead of lumping all proxy requests under a single "unknown" IP.
    const clientIp =
      request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() ||
      request.headers.get('x-real-ip') ||
      '127.0.0.1';

    const upstreamHeaders: Record<string, string> = { ...(headers || {}) };
    upstreamHeaders['X-Forwarded-For'] = clientIp;

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), UPSTREAM_TIMEOUT_MS);

    try {
      const res = await fetch(targetUrl, {
        method: method || 'GET',
        headers: upstreamHeaders,
        body: body || undefined,
        signal: controller.signal,
      });

      const text = await readTextLimited(res, MAX_UPSTREAM_BYTES);
      return NextResponse.json({ status: res.status, body: text });
    } finally {
      clearTimeout(timer);
    }
  } catch (err) {
    if (err instanceof DOMException && err.name === 'AbortError') {
      return NextResponse.json(
        { error: `Gateway did not respond within ${UPSTREAM_TIMEOUT_MS / 1000}s` },
        { status: 504 },
      );
    }
    const message = err instanceof Error ? err.message : 'Proxy request failed';
    return NextResponse.json({ error: message }, { status: 502 });
  }
}
