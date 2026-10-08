import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import FAQ from '@/components/marketing/FAQ';

describe('FAQ', () => {
  it('renders all FAQ questions', () => {
    render(<FAQ />);
    expect(screen.getByText('What is DoAide Cortex?')).toBeInTheDocument();
    expect(screen.getByText('How is Cortex different from a vector database?')).toBeInTheDocument();
    expect(screen.getByText('Do I need Neo4j or Qdrant to use Cortex?')).toBeInTheDocument();
    expect(screen.getByText('How do I deploy Cortex?')).toBeInTheDocument();
    expect(screen.getByText('What programming languages are supported?')).toBeInTheDocument();
    expect(screen.getByText('Is Cortex suitable for enterprise use?')).toBeInTheDocument();
    expect(screen.getByText('What kind of latency can I expect?')).toBeInTheDocument();
    expect(screen.getByText('Can I migrate from Mem0 or Zep?')).toBeInTheDocument();
  });

  it('expands an answer when clicking a question', () => {
    render(<FAQ />);
    const question = screen.getByText('What is DoAide Cortex?');
    fireEvent.click(question);
    expect(screen.getByText(/open-source, Rust-native context engineering platform/)).toBeInTheDocument();
  });

  it('collapses an answer when clicking the same question again', () => {
    render(<FAQ />);
    const question = screen.getByText('What is DoAide Cortex?');
    fireEvent.click(question);
    expect(screen.getByText(/open-source, Rust-native context engineering platform/)).toBeInTheDocument();
    fireEvent.click(question);
    expect(screen.queryByText(/open-source, Rust-native context engineering platform/)).not.toBeInTheDocument();
  });

  it('renders JSON-LD FAQPage schema', () => {
    const { container } = render(<FAQ />);
    const script = container.querySelector('script[type="application/ld+json"]');
    expect(script).toBeInTheDocument();
    const json = JSON.parse(script!.textContent!);
    expect(json['@type']).toBe('FAQPage');
    expect(json.mainEntity).toHaveLength(8);
    expect(json.mainEntity[0]['@type']).toBe('Question');
    expect(json.mainEntity[0].acceptedAnswer['@type']).toBe('Answer');
  });
});
