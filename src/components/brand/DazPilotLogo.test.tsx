import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { DazPilotLogo, DazPilotLogoCompact } from './DazPilotLogo';

describe('DazPilotLogo', () => {
  it('renders the SVG element', () => {
    const { container } = render(<DazPilotLogo />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('renders with default size', () => {
    const { container } = render(<DazPilotLogo />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '48');
    expect(svg).toHaveAttribute('height', '48');
  });

  it('renders with custom size', () => {
    const { container } = render(<DazPilotLogo size={80} />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '80');
    expect(svg).toHaveAttribute('height', '80');
  });

  it('renders wordmark when showWordmark is true', () => {
    const { container } = render(<DazPilotLogo showWordmark />);
    expect(screen.getByText('Scene Control')).toBeInTheDocument();
    expect(container.textContent).toContain('Daz');
    expect(container.textContent).toContain('Pilot');
  });

  it('hides wordmark by default', () => {
    render(<DazPilotLogo />);
    expect(screen.queryByText('DazPilot')).not.toBeInTheDocument();
  });

  it('hides wordmark when showWordmark is false', () => {
    render(<DazPilotLogo showWordmark={false} />);
    expect(screen.queryByText('DazPilot')).not.toBeInTheDocument();
  });

  it('hides wordmark when compact and showWordmark are true', () => {
    render(<DazPilotLogo showWordmark compact />);
    expect(screen.queryByText('DazPilot')).not.toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<DazPilotLogo className="custom-class" />);
    const div = container.firstChild as HTMLElement;
    expect(div.className).toContain('custom-class');
  });
});

describe('DazPilotLogoCompact', () => {
  it('renders SVG with compact size', () => {
    const { container } = render(<DazPilotLogoCompact />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('width', '32');
    expect(svg).toHaveAttribute('height', '32');
  });

  it('renders with custom size', () => {
    const { container } = render(<DazPilotLogoCompact size={48} />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '48');
    expect(svg).toHaveAttribute('height', '48');
  });
});
