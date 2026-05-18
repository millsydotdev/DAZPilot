import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Button } from './Button';

describe('Button', () => {
  it('renders children correctly', () => {
    const html = renderToStaticMarkup(<Button>Click me</Button>);
    expect(html).toContain('Click me');
  });

  it('applies variant classes', () => {
    const html = renderToStaticMarkup(<Button variant="primary">Primary</Button>);
    expect(html).toMatch(/primary/);
  });

  it('applies size classes', () => {
    const html = renderToStaticMarkup(<Button size="sm">Small</Button>);
    expect(html).toMatch(/sm/);
  });

  it('shows loading state', () => {
    const html = renderToStaticMarkup(<Button loading>Loading</Button>);
    expect(html).toContain('disabled');
    expect(html).toMatch(/loading/);
  });

  it('handles disabled state', () => {
    const html = renderToStaticMarkup(<Button disabled>Disabled</Button>);
    expect(html).toContain('disabled');
  });

  it('applies fullWidth class', () => {
    const html = renderToStaticMarkup(<Button fullWidth>Full width</Button>);
    expect(html).toMatch(/fullWidth/);
  });
});
