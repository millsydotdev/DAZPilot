import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Input } from './Input';

describe('Input', () => {
  it('renders with placeholder', () => {
    const html = renderToStaticMarkup(<Input placeholder="Enter text" />);
    expect(html).toContain('placeholder="Enter text"');
  });

  it('renders with label', () => {
    const html = renderToStaticMarkup(<Input label="Email" />);
    expect(html).toContain('Email');
  });

  it('shows error message', () => {
    const html = renderToStaticMarkup(<Input error errorMessage="This field is required" />);
    expect(html).toMatch(/error/);
    expect(html).toContain('This field is required');
  });

  it('shows hint text', () => {
    const html = renderToStaticMarkup(<Input hint="Enter your email address" />);
    expect(html).toContain('Enter your email address');
  });

  it('applies disabled state', () => {
    const html = renderToStaticMarkup(<Input disabled />);
    expect(html).toContain('disabled');
  });

  it('applies size variant', () => {
    const html = renderToStaticMarkup(<Input variantSize="sm" />);
    expect(html).toMatch(/sm/);
  });
});
