import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Card, CardHeader, CardContent, CardFooter } from './Card';

describe('Card', () => {
  it('renders children correctly', () => {
    const html = renderToStaticMarkup(<Card>Card content</Card>);
    expect(html).toContain('Card content');
  });

  it('applies padding classes', () => {
    const html = renderToStaticMarkup(<Card padding="lg">Content</Card>);
    expect(html).toMatch(/padding-lg/);
  });

  it('applies interactive class', () => {
    const html = renderToStaticMarkup(<Card interactive>Interactive</Card>);
    expect(html).toMatch(/interactive/);
  });

  it('renders CardHeader with title', () => {
    const html = renderToStaticMarkup(
      <Card>
        <CardHeader title="Card Title" />
      </Card>
    );
    expect(html).toContain('Card Title');
  });

  it('renders CardHeader with description', () => {
    const html = renderToStaticMarkup(
      <Card>
        <CardHeader title="Title" description="Card description" />
      </Card>
    );
    expect(html).toContain('Card description');
  });

  it('renders CardContent', () => {
    const html = renderToStaticMarkup(
      <Card>
        <CardContent>Content here</CardContent>
      </Card>
    );
    expect(html).toContain('Content here');
  });

  it('renders CardFooter', () => {
    const html = renderToStaticMarkup(
      <Card>
        <CardFooter>Footer content</CardFooter>
      </Card>
    );
    expect(html).toContain('Footer content');
  });
});
