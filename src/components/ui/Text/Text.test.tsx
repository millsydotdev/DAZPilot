import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Text } from './Text';

describe('Text', () => {
  it('renders children correctly', () => {
    const html = renderToStaticMarkup(<Text>Hello World</Text>);
    expect(html).toContain('Hello World');
  });

  it('applies variant classes', () => {
    const html = renderToStaticMarkup(<Text variant="heading1">Heading</Text>);
    expect(html).toMatch(/heading1/);
  });

  it('applies bold class', () => {
    const html = renderToStaticMarkup(<Text bold>Bold text</Text>);
    expect(html).toMatch(/bold/);
  });

  it('applies italic class', () => {
    const html = renderToStaticMarkup(<Text italic>Italic text</Text>);
    expect(html).toMatch(/italic/);
  });

  it('applies underline class', () => {
    const html = renderToStaticMarkup(<Text underline>Underlined</Text>);
    expect(html).toMatch(/underline/);
  });

  it('applies truncate class', () => {
    const html = renderToStaticMarkup(<Text truncate>Truncated text</Text>);
    expect(html).toMatch(/truncate/);
  });

  it('renders all variants', () => {
    const variants = ['heading1', 'heading2', 'heading3', 'body', 'small', 'muted'] as const;
    variants.forEach((variant) => {
      const html = renderToStaticMarkup(<Text variant={variant}>{variant}</Text>);
      expect(html).toMatch(new RegExp(variant));
    });
  });
});
