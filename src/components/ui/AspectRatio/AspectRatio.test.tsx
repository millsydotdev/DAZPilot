import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { AspectRatio } from './AspectRatio';

describe('AspectRatio', () => {
  it('renders children correctly', () => {
    render(
      <AspectRatio>
        <span>Content</span>
      </AspectRatio>
    );
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders with default ratio', () => {
    const { container } = render(<AspectRatio>Content</AspectRatio>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('aspectRatio'));
  });

  it('renders with different ratios', () => {
    const ratios = ['1:1', '4:3', '16:9', '21:9'] as const;
    ratios.forEach((ratio) => {
      const { container } = render(<AspectRatio ratio={ratio}>{ratio}</AspectRatio>);
      expect(container.firstChild).toBeInTheDocument();
    });
  });

  it('wraps children in content div', () => {
    const { container } = render(
      <AspectRatio>
        <span>Inner</span>
      </AspectRatio>
    );
    expect(container.querySelector('[class*="content"]')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<AspectRatio className="custom-ratio">Content</AspectRatio>);
    expect(container.firstChild).toHaveClass('custom-ratio');
  });
});
