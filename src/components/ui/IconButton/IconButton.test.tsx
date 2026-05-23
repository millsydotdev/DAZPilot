import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { IconButton } from './IconButton';

describe('IconButton', () => {
  it('renders with label as aria-label and title', () => {
    render(<IconButton icon={<span>X</span>} label="Close" />);
    const btn = screen.getByRole('button');
    expect(btn).toHaveAttribute('aria-label', 'Close');
    expect(btn).toHaveAttribute('title', 'Close');
  });

  it('renders icon content', () => {
    render(<IconButton icon={<span data-testid="icon">X</span>} label="Close" />);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('applies size classes', () => {
    const { container } = render(<IconButton icon={<span>X</span>} label="Close" size="lg" />);
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('lg'));
  });

  it('applies variant classes', () => {
    const { container } = render(
      <IconButton icon={<span>X</span>} label="Close" variant="primary" />
    );
    expect(container.firstChild).toHaveAttribute('class', expect.stringContaining('primary'));
  });

  it('shows spinner when loading', () => {
    const { container } = render(<IconButton icon={<span>X</span>} label="Close" loading />);
    expect(container.querySelector('[class*="spinner"]')).toBeInTheDocument();
    expect(container.querySelector('[class*="icon"]')).not.toBeInTheDocument();
  });

  it('is disabled when loading', () => {
    render(<IconButton icon={<span>X</span>} label="Close" loading />);
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('is disabled when disabled prop is set', () => {
    render(<IconButton icon={<span>X</span>} label="Close" disabled />);
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<IconButton icon={<span>X</span>} label="Close" onClick={handleClick} />);
    screen.getByRole('button').click();
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('does not call onClick when disabled', () => {
    const handleClick = vi.fn();
    render(<IconButton icon={<span>X</span>} label="Close" disabled onClick={handleClick} />);
    screen.getByRole('button').click();
    expect(handleClick).not.toHaveBeenCalled();
  });
});
