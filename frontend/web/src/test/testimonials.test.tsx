import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import Testimonials from '@/components/marketing/Testimonials';

describe('Testimonials', () => {
  it('renders all testimonials', () => {
    render(<Testimonials />);
    expect(screen.getByText('Vikram Mehta')).toBeInTheDocument();
    expect(screen.getByText('Ananya Krishnan')).toBeInTheDocument();
    expect(screen.getByText('Rajesh Sundaram')).toBeInTheDocument();
    expect(screen.getByText('Priya Nair')).toBeInTheDocument();
  });

  it('displays company and role information', () => {
    render(<Testimonials />);
    expect(screen.getByText('CTO, NeuralStack AI, Bangalore')).toBeInTheDocument();
    expect(screen.getByText('Head of AI Engineering, Lumen Analytics, Hyderabad')).toBeInTheDocument();
    expect(screen.getByText('Engineering Lead, FinEdge Technologies, Pune')).toBeInTheDocument();
    expect(screen.getByText('VP of Engineering, MedAssist Digital, Chennai')).toBeInTheDocument();
  });

  it('renders quote content', () => {
    render(<Testimonials />);
    expect(screen.getByText(/replaced three separate services/)).toBeInTheDocument();
    expect(screen.getByText(/customer satisfaction scores improved by 35%/)).toBeInTheDocument();
  });

  it('renders the section heading', () => {
    render(<Testimonials />);
    expect(screen.getByText('Trusted by AI teams building in production')).toBeInTheDocument();
  });
});
