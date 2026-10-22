import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';

let mockPath = '/';
vi.mock('next/navigation', () => ({ usePathname: () => mockPath }));

import WhatsAppShare, { whatsAppShareUrl } from '@/components/ui/WhatsAppShare';
import { SITE } from '@/config/site';

describe('WhatsAppShare', () => {
  it('builds an encoded wa.me link', () => {
    expect(whatsAppShareUrl('Hi there', 'https://x.test/a/')).toBe(
      'https://wa.me/?text=Hi%20there%20https%3A%2F%2Fx.test%2Fa%2F',
    );
  });

  it('shares the current page URL', () => {
    mockPath = '/mcp/';
    render(<WhatsAppShare />);
    const link = screen.getByRole('link', { name: /share on whatsapp/i });
    expect(decodeURIComponent(link.getAttribute('href')!)).toContain(`${SITE.url}/mcp/`);
    expect(link).toHaveClass('min-h-[48px]');
  });

  it('normalises paths without a trailing slash', () => {
    mockPath = '/blog/hybrid-retrieval';
    render(<WhatsAppShare />);
    const href = screen.getByRole('link', { name: /share on whatsapp/i }).getAttribute('href')!;
    expect(decodeURIComponent(href)).toContain('/blog/hybrid-retrieval/');
  });

  it.each(['/demo/', '/login/', '/signup/'])('is hidden on %s', (path) => {
    mockPath = path;
    const { container } = render(<WhatsAppShare />);
    expect(container).toBeEmptyDOMElement();
  });
});
