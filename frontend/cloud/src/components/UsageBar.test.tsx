import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import React from 'react';
import UsageBar from './UsageBar';

// Ensure React is available for JSX transform
globalThis.React = React;

describe('UsageBar', () => {
  it('renders low usage with emerald/green', () => {
    const { container } = render(<UsageBar used={20} limit={100} />);
    const bar = container.querySelector('.bg-emerald-500');
    expect(bar).toBeTruthy();
  });

  it('renders medium usage with amber', () => {
    const { container } = render(<UsageBar used={60} limit={100} />);
    const bar = container.querySelector('.bg-amber-500');
    expect(bar).toBeTruthy();
  });

  it('renders high usage with red', () => {
    const { container } = render(<UsageBar used={85} limit={100} />);
    const bar = container.querySelector('.bg-red-500');
    expect(bar).toBeTruthy();
  });

  it('renders unlimited plan when limit is null', () => {
    render(<UsageBar used={10} limit={null} />);
    expect(screen.getByText('Unlimited plan')).toBeTruthy();
  });

  it('renders zero limit as unlimited', () => {
    render(<UsageBar used={10} limit={0} />);
    expect(screen.getByText('Unlimited plan')).toBeTruthy();
  });

  it('clamps percentage at 100', () => {
    render(<UsageBar used={150} limit={100} />);
    expect(screen.getByText('100%')).toBeTruthy();
  });

  it('shows label when showLabel is true', () => {
    render(<UsageBar used={500} limit={1000} showLabel={true} />);
    expect(screen.getByText(/500/)).toBeTruthy();
    expect(screen.getByText(/1,000/)).toBeTruthy();
  });
});
