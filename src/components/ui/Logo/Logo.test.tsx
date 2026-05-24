import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Logo, LogoCompact } from './Logo';

vi.mock('../../brand/DAZPilotLogo', () => ({
  DAZPilotLogo: ({ size }: { size: number }) => <span data-testid="logo">{size}px</span>,
  DAZPilotLogoCompact: ({ size }: { size: number }) => (
    <span data-testid="logo-compact">{size}px</span>
  ),
}));

describe('Logo', () => {
  it('renders with default size', () => {
    render(<Logo />);
    expect(screen.getByTestId('logo')).toHaveTextContent('32px');
  });

  it('renders with custom size', () => {
    render(<Logo size={48} />);
    expect(screen.getByTestId('logo')).toHaveTextContent('48px');
  });
});

describe('LogoCompact', () => {
  it('renders with default size', () => {
    render(<LogoCompact />);
    expect(screen.getByTestId('logo-compact')).toHaveTextContent('24px');
  });

  it('renders with custom size', () => {
    render(<LogoCompact size={36} />);
    expect(screen.getByTestId('logo-compact')).toHaveTextContent('36px');
  });
});
