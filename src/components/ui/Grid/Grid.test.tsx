import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Grid, GridItem } from './Grid';

describe('Grid', () => {
  it('renders children correctly', () => {
    render(
      <Grid>
        <span>Item</span>
      </Grid>
    );
    expect(screen.getByText('Item')).toBeInTheDocument();
  });

  it('applies default cols class', () => {
    const { container } = render(<Grid>Content</Grid>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('cols-1'));
  });

  it('applies cols class', () => {
    const { container } = render(<Grid cols={3}>Content</Grid>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('cols-3'));
  });

  it('applies gap classes', () => {
    const { container } = render(<Grid gap="lg">Content</Grid>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('gap-lg'));
  });

  it('applies gapX and gapY classes', () => {
    const { container } = render(
      <Grid gapX="sm" gapY="md">
        Content
      </Grid>
    );
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('gap-x-sm'));
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('gap-y-md'));
  });
});

describe('GridItem', () => {
  it('renders children correctly', () => {
    render(
      <GridItem>
        <span>Cell</span>
      </GridItem>
    );
    expect(screen.getByText('Cell')).toBeInTheDocument();
  });

  it('applies span class', () => {
    const { container } = render(<GridItem span={6}>Content</GridItem>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('span-6'));
  });

  it('applies start class', () => {
    const { container } = render(<GridItem start={3}>Content</GridItem>);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('start-3'));
  });
});
