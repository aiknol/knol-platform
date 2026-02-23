import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import React from 'react';
import UsageChart from './UsageChart';

// Ensure React is available for JSX transform
globalThis.React = React;

const MOCK_DATA = [
  { month: '2026-01', ops_count: 5000, plan: 'free', usage_limit: 10000 },
  { month: '2026-02', ops_count: 42000, plan: 'builder', usage_limit: 100000 },
  { month: '2025-12', ops_count: 1500000, plan: 'growth', usage_limit: 500000 },
];

describe('UsageChart', () => {
  it('renders empty state', () => {
    render(<UsageChart data={[]} />);
    expect(screen.getByText('No usage history yet')).toBeTruthy();
  });

  it('renders bars for data', () => {
    const { container } = render(<UsageChart data={MOCK_DATA} />);
    const rects = container.querySelectorAll('rect');
    expect(rects.length).toBe(3);
  });

  it('formats month labels correctly', () => {
    render(<UsageChart data={[{ month: '2026-02', ops_count: 100, plan: 'free', usage_limit: null }]} />);
    expect(screen.getByText('Feb')).toBeTruthy();
  });

  it('formats all months', () => {
    const janData = [{ month: '2026-01', ops_count: 100, plan: 'free', usage_limit: null }];
    const decData = [{ month: '2026-12', ops_count: 100, plan: 'free', usage_limit: null }];

    const { unmount } = render(<UsageChart data={janData} />);
    expect(screen.getByText('Jan')).toBeTruthy();
    unmount();

    render(<UsageChart data={decData} />);
    expect(screen.getByText('Dec')).toBeTruthy();
  });

  it('formats ops as millions', () => {
    render(<UsageChart data={[{ month: '2026-01', ops_count: 1500000, plan: 'free', usage_limit: null }]} />);
    expect(screen.getByText('1.5M')).toBeTruthy();
  });

  it('formats ops as thousands', () => {
    render(<UsageChart data={[{ month: '2026-01', ops_count: 42000, plan: 'free', usage_limit: null }]} />);
    expect(screen.getByText('42K')).toBeTruthy();
  });

  it('formats small numbers directly', () => {
    render(<UsageChart data={[{ month: '2026-01', ops_count: 500, plan: 'free', usage_limit: null }]} />);
    expect(screen.getByText('500')).toBeTruthy();
  });

  it('renders limit line when limit exists', () => {
    const { container } = render(<UsageChart data={MOCK_DATA} />);
    const lines = container.querySelectorAll('line');
    expect(lines.length).toBeGreaterThan(0);
    // Check dashed limit line attributes
    const limitLine = lines[0];
    expect(limitLine.getAttribute('stroke')).toBe('#71717A');
    expect(limitLine.getAttribute('stroke-dasharray')).toBe('4 3');
    // Check the "limit" text label
    expect(screen.getByText('limit')).toBeTruthy();
  });
});
