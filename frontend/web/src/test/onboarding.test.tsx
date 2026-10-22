import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

let mockPath = '/';
vi.mock('next/navigation', () => ({ usePathname: () => mockPath }));

import OnboardingGuide, { ONBOARDING_KEY, STEPS } from '@/components/ui/OnboardingGuide';

// jsdom here has no usable localStorage; use an in-memory one.
const store = new Map<string, string>();
Object.defineProperty(globalThis, 'localStorage', {
  configurable: true,
  value: {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, String(v)),
    clear: () => store.clear(),
  },
});

beforeEach(() => {
  mockPath = '/';
  localStorage.clear();
});

describe('OnboardingGuide', () => {
  it('shows on the first homepage visit and walks through all steps', () => {
    render(<OnboardingGuide />);
    expect(screen.getByRole('dialog')).toHaveTextContent(STEPS[0].title);
    for (let i = 1; i < STEPS.length; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Next' }));
      expect(screen.getByRole('dialog')).toHaveTextContent(STEPS[i].title);
    }
    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    expect(screen.getByRole('dialog')).toHaveTextContent(STEPS[1].title);
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(localStorage.getItem(ONBOARDING_KEY)).toBe('1');
  });

  it('links each step to a page that needs no login', () => {
    render(<OnboardingGuide />);
    expect(screen.getByRole('link', { name: /open the demo/i }).getAttribute('href')).toMatch(/^\/demo\/?$/);
  });

  it('stays hidden once dismissed', () => {
    localStorage.setItem(ONBOARDING_KEY, '1');
    render(<OnboardingGuide />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('does not show on other pages', () => {
    mockPath = '/pricing/';
    render(<OnboardingGuide />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('closes on Escape and via the close button', () => {
    const { unmount } = render(<OnboardingGuide />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).toBeNull();
    unmount();
    localStorage.clear();
    render(<OnboardingGuide />);
    const close = screen.getByRole('button', { name: 'Close guide' });
    expect(close).toHaveClass('min-h-[48px]', 'min-w-[48px]');
    fireEvent.click(close);
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
