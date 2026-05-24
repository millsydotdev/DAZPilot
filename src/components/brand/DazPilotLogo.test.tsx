import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { DAZPilotLogo, DAZPilotLogoCompact } from './DAZPilotLogo';

describe('DAZPilotLogo', () => {
  it('renders the SVG element', () => {
    const { container } = render(<DAZPilotLogo />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('renders with default size', () => {
    const { container } = render(<DAZPilotLogo />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '48');
    expect(svg).toHaveAttribute('height', '48');
  });

  it('renders with custom size', () => {
    const { container } = render(<DAZPilotLogo size={80} />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '80');
    expect(svg).toHaveAttribute('height', '80');
  });

  it('renders wordmark when showWordmark is true', () => {
    const { container } = render(<DAZPilotLogo showWordmark />);
    expect(screen.getByText('Scene Control')).toBeInTheDocument();
    expect(container.textContent).toContain('Daz');
    expect(container.textContent).toContain('Pilot');
  });

  it('hides wordmark by default', () => {
    render(<DAZPilotLogo />);
    expect(screen.queryByText('DAZPilot')).not.toBeInTheDocument();
  });

  it('hides wordmark when showWordmark is false', () => {
    render(<DAZPilotLogo showWordmark={false} />);
    expect(screen.queryByText('DAZPilot')).not.toBeInTheDocument();
  });

  it('hides wordmark when compact and showWordmark are true', () => {
    render(<DAZPilotLogo showWordmark compact />);
    expect(screen.queryByText('DAZPilot')).not.toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<DAZPilotLogo className="custom-class" />);
    const div = container.firstChild as HTMLElement;
    expect(div.className).toContain('custom-class');
  });
});

describe('DAZPilotLogoCompact', () => {
  it('renders SVG with compact size', () => {
    const { container } = render(<DAZPilotLogoCompact />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('width', '32');
    expect(svg).toHaveAttribute('height', '32');
  });

  it('renders with custom size', () => {
    const { container } = render(<DAZPilotLogoCompact size={48} />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '48');
    expect(svg).toHaveAttribute('height', '48');
  });
});
