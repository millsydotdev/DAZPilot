import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Badge } from './Badge';

describe('Badge', () => {
  it('renders children correctly', () => {
    const html = renderToStaticMarkup(<Badge>New</Badge>);
    expect(html).toContain('New');
  });

  it('applies variant classes', () => {
    const html = renderToStaticMarkup(<Badge variant="success">Success</Badge>);
    expect(html).toMatch(/success/);
  });

  it('applies size classes', () => {
    const html = renderToStaticMarkup(<Badge size="lg">Large</Badge>);
    expect(html).toMatch(/lg/);
  });

  it('shows dot when dot prop is true', () => {
    const html = renderToStaticMarkup(<Badge dot>With dot</Badge>);
    expect(html).toMatch(/dot/);
  });

  it('renders different variants', () => {
    const variants = ['info', 'success', 'warning', 'error'] as const;
    variants.forEach((variant) => {
      const html = renderToStaticMarkup(<Badge variant={variant}>{variant}</Badge>);
      expect(html).toMatch(new RegExp(variant));
    });
  });
});
