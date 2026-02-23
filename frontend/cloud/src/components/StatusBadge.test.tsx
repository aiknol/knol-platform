import React from 'react';
// Make React available globally so components that rely on the JSX transform
// (without an explicit React import) work correctly in jsdom.
globalThis.React = React;
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import StatusBadge from './StatusBadge';

describe('StatusBadge', () => {
  it('active renders with emerald/green classes', () => {
    const { container } = render(<StatusBadge status="active" />);
    const badge = container.firstElementChild!;
    expect(badge.className).toContain('emerald');
  });

  it('past_due renders with red classes', () => {
    const { container } = render(<StatusBadge status="past_due" />);
    const badge = container.firstElementChild!;
    expect(badge.className).toContain('red');
  });

  it('free plan renders with dark/neutral classes', () => {
    const { container } = render(<StatusBadge status="free" />);
    const badge = container.firstElementChild!;
    expect(badge.className).toContain('dark');
  });

  it('growth plan renders with brand classes', () => {
    const { container } = render(<StatusBadge status="growth" />);
    const badge = container.firstElementChild!;
    expect(badge.className).toContain('brand');
  });

  it('unknown status renders with default fallback', () => {
    const { container } = render(<StatusBadge status="unknown_status_xyz" />);
    const badge = container.firstElementChild!;
    expect(badge.className).toContain('dark');
  });

  it('badge contains the status text', () => {
    render(<StatusBadge status="active" />);
    expect(screen.getByText('active')).toBeTruthy();
  });

  it('replaces underscores with spaces in display text', () => {
    render(<StatusBadge status="past_due" />);
    expect(screen.getByText('past due')).toBeTruthy();
  });

  it('uses custom label when provided', () => {
    render(<StatusBadge status="active" label="Live" />);
    expect(screen.getByText('Live')).toBeTruthy();
  });

  it('renders as a span with badge styling', () => {
    const { container } = render(<StatusBadge status="active" />);
    const badge = container.firstElementChild!;
    expect(badge.tagName).toBe('SPAN');
    expect(badge.className).toContain('rounded-full');
    expect(badge.className).toContain('border');
  });
});
