import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Spinner } from './Spinner';

describe('Spinner', () => {
  it('renders with default size and color', () => {
    const { container } = render(<Spinner />);
    expect(container.firstChild).toBeInTheDocument();
    expect(container.firstChild).toHaveAttribute('aria-label', 'Loading');
  });

  it('applies size classes', () => {
    const { container } = render(<Spinner size="lg" />);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('lg'));
  });

  it('applies color classes', () => {
    const { container } = render(<Spinner color="white" />);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('white'));
  });

  it('applies custom className', () => {
    const { container } = render(<Spinner className="custom" />);
    expect(container.firstChild).toHaveClass('custom');
  });

  it('renders all size variants', () => {
    const sizes = ['sm', 'md', 'lg'] as const;
    sizes.forEach((size) => {
      const { container } = render(<Spinner size={size} />);
      expect(container.firstChild).toHaveAttribute('class', expect.stringContaining(size));
    });
  });

  it('renders all color variants', () => {
    const colors = ['primary', 'white', 'muted'] as const;
    colors.forEach((color) => {
      const { container } = render(<Spinner color={color} />);
      expect(container.firstChild).toHaveAttribute('class', expect.stringContaining(color));
    });
  });
});
