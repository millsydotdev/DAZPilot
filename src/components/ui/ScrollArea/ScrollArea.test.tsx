import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ScrollArea } from './ScrollArea';

describe('ScrollArea', () => {
  it('renders children correctly', () => {
    render(
      <ScrollArea>
        <span>Scroll content</span>
      </ScrollArea>
    );
    expect(screen.getByText('Scroll content')).toBeInTheDocument();
  });

  it('renders vertical and horizontal scrollbars by default', () => {
    const { container } = render(<ScrollArea>Content</ScrollArea>);
    expect(container.querySelector('[class*="scrollbarVertical"]')).toBeInTheDocument();
    expect(container.querySelector('[class*="scrollbarHorizontal"]')).toBeInTheDocument();
  });

  it('hides vertical scrollbar when vertical=false', () => {
    const { container } = render(<ScrollArea vertical={false}>Content</ScrollArea>);
    expect(container.querySelector('[class*="scrollbarVertical"]')).not.toBeInTheDocument();
    expect(container.querySelector('[class*="scrollbarHorizontal"]')).toBeInTheDocument();
  });

  it('hides horizontal scrollbar when horizontal=false', () => {
    const { container } = render(<ScrollArea horizontal={false}>Content</ScrollArea>);
    expect(container.querySelector('[class*="scrollbarVertical"]')).toBeInTheDocument();
    expect(container.querySelector('[class*="scrollbarHorizontal"]')).not.toBeInTheDocument();
  });

  it('hides both scrollbars when both are false', () => {
    const { container } = render(
      <ScrollArea horizontal={false} vertical={false}>
        Content
      </ScrollArea>
    );
    expect(container.querySelector('[class*="scrollbarVertical"]')).not.toBeInTheDocument();
    expect(container.querySelector('[class*="scrollbarHorizontal"]')).not.toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<ScrollArea className="my-scroll">Content</ScrollArea>);
    expect(container.firstChild).toHaveClass('my-scroll');
  });
});
