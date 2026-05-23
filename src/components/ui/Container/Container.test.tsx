import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Container } from './Container';

describe('Container', () => {
  it('renders children correctly', () => {
    render(
      <Container>
        <span>Content</span>
      </Container>
    );
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('applies default size class (lg)', () => {
    const { container } = render(<Container>Content</Container>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('lg'));
  });

  it('applies size classes', () => {
    const { container } = render(<Container size="sm">Content</Container>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('sm'));
  });

  it('applies centered class when centered is true', () => {
    const { container } = render(<Container centered>Content</Container>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('centered'));
  });

  it('renders all size variants', () => {
    const sizes = ['sm', 'md', 'lg', 'xl', 'full'] as const;
    sizes.forEach((size) => {
      const { container } = render(<Container size={size}>{size}</Container>);
      expect(container.firstChild).toHaveAttribute('class', expect.stringContaining(size));
    });
  });

  it('applies custom className', () => {
    const { container } = render(<Container className="custom">Content</Container>);
    expect(container.firstChild).toHaveClass('custom');
  });
});
