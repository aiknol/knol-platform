import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import Navbar from '@/components/layout/Navbar';
import Footer from '@/components/layout/Footer';
import CodeBlock from '@/components/ui/CodeBlock';

describe('mobile touch targets', () => {
  it('menu toggle is 48x48 and reports its state', () => {
    render(<Navbar />);
    const toggle = screen.getByRole('button', { name: 'Toggle menu' });
    expect(toggle).toHaveClass('min-h-[48px]', 'min-w-[48px]');
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
  });

  it('every link in the open mobile menu is at least 48px tall', () => {
    const { container } = render(<Navbar />);
    fireEvent.click(screen.getByRole('button', { name: 'Toggle menu' }));
    const menu = container.querySelector('header > div.md\\:hidden')!;
    const links = menu.querySelectorAll('a');
    expect(links.length).toBeGreaterThan(5);
    links.forEach((a) => expect(a.className).toContain('min-h-[48px]'));
  });

  it('footer links are 48px on mobile', () => {
    const { container } = render(<Footer />);
    container.querySelectorAll('nav a').forEach((a) => expect(a.className).toContain('min-h-[48px]'));
  });

  it('code copy button is visible and tappable on touch screens', () => {
    render(<CodeBlock code="npm i" language="bash" />);
    const btn = screen.getByRole('button', { name: 'Copy' });
    expect(btn).toHaveClass('opacity-100', 'min-h-[48px]', 'min-w-[48px]');
  });
});
