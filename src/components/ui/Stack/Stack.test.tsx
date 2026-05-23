import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Stack, HStack, VStack } from './Stack';

describe('Stack', () => {
  it('renders children correctly', () => {
    render(
      <Stack>
        <span>Content</span>
      </Stack>
    );
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders row direction by default', () => {
    const { container } = render(<Stack>Content</Stack>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('direction-row'));
  });

  it('renders column direction', () => {
    const { container } = render(<Stack direction="column">Content</Stack>);
    expect(container.firstChild).toHaveAttribute(
      'class',
      expect.stringContaining('direction-column')
    );
  });

  it('applies gap classes', () => {
    const { container } = render(<Stack gap="xl">Content</Stack>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('gap-xl'));
  });

  it('applies align classes', () => {
    const { container } = render(<Stack align="center">Content</Stack>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('align-center'));
  });

  it('applies justify classes', () => {
    const { container } = render(<Stack justify="between">Content</Stack>);
    expect(container.firstChild).toHaveAttribute(
      'class',
      expect.stringContaining('justify-between')
    );
  });

  it('applies wrap class', () => {
    const { container } = render(<Stack wrap="wrap">Content</Stack>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('wrap'));
  });
});

describe('HStack', () => {
  it('renders children and defaults to row direction', () => {
    const { container } = render(<HStack>Content</HStack>);
    expect(screen.getByText('Content')).toBeInTheDocument();
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('direction-row'));
  });
});

describe('VStack', () => {
  it('renders children and defaults to column direction', () => {
    const { container } = render(<VStack>Content</VStack>);
    expect(screen.getByText('Content')).toBeInTheDocument();
    expect(container.firstChild).toHaveAttribute(
      'class',
      expect.stringContaining('direction-column')
    );
  });
});
