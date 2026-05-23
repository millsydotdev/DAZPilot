import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Flex } from './Flex';

describe('Flex', () => {
  it('renders children correctly', () => {
    render(
      <Flex>
        <span>Item</span>
      </Flex>
    );
    expect(screen.getByText('Item')).toBeInTheDocument();
  });

  it('renders row direction by default', () => {
    const { container } = render(<Flex>Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('row'));
  });

  it('applies direction classes', () => {
    const { container } = render(<Flex direction="column">Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('column'));
  });

  it('applies gap classes', () => {
    const { container } = render(<Flex gap="lg">Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('gap-lg'));
  });

  it('applies align classes', () => {
    const { container } = render(<Flex align="center">Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('align-center'));
  });

  it('applies justify classes', () => {
    const { container } = render(<Flex justify="evenly">Content</Flex>);
    expect(container.firstChild).toHaveAttribute(
      'class',
      expect.stringContaining('justify-evenly')
    );
  });

  it('applies wrap class', () => {
    const { container } = render(<Flex wrap="nowrap">Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('nowrap'));
  });

  it('applies grow class when grow is true', () => {
    const { container } = render(<Flex grow>Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('grow'));
  });

  it('applies shrink-0 when shrink is false', () => {
    const { container } = render(<Flex shrink={false}>Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('shrink-0'));
  });

  it('applies inline class when inline is true', () => {
    const { container } = render(<Flex inline>Content</Flex>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('inline'));
  });
});
