import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Separator } from './Separator';

describe('Separator', () => {
  it('renders a horizontal separator by default', () => {
    const html = renderToStaticMarkup(<Separator />);
    expect(html).toMatch(/separator/);
  });

  it('renders vertical orientation', () => {
    const html = renderToStaticMarkup(<Separator orientation="vertical" />);
    expect(html).toMatch(/vertical/);
  });

  it('renders dashed variant', () => {
    const html = renderToStaticMarkup(<Separator dashed />);
    expect(html).toMatch(/dashed/);
  });

  it('applies spacing classes', () => {
    const html = renderToStaticMarkup(<Separator spacing="lg" />);
    expect(html).toMatch(/spacing-lg/);
  });

  it('applies custom className', () => {
    const html = renderToStaticMarkup(<Separator className="custom-class" />);
    expect(html).toContain('custom-class');
  });
});
