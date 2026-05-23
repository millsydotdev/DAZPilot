import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Collapsible } from './Collapsible';

describe('Collapsible', () => {
  it('renders title', () => {
    render(<Collapsible title="Settings">Content</Collapsible>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('shows children by default', () => {
    render(
      <Collapsible title="Settings">
        <span>Inner content</span>
      </Collapsible>
    );
    expect(screen.getByText('Inner content')).toBeInTheDocument();
  });

  it('hides children when defaultOpen is false', () => {
    render(
      <Collapsible title="Settings" defaultOpen={false}>
        <span>Hidden</span>
      </Collapsible>
    );
    expect(screen.queryByText('Hidden')).not.toBeInTheDocument();
  });

  it('toggles content visibility on header click', () => {
    render(
      <Collapsible title="Settings">
        <span>Toggle me</span>
      </Collapsible>
    );
    expect(screen.getByText('Toggle me')).toBeInTheDocument();
    fireEvent.click(screen.getByText('Settings'));
    expect(screen.queryByText('Toggle me')).not.toBeInTheDocument();
    fireEvent.click(screen.getByText('Settings'));
    expect(screen.getByText('Toggle me')).toBeInTheDocument();
  });

  it('has aria-expanded attribute', () => {
    render(<Collapsible title="Settings">Content</Collapsible>);
    expect(screen.getByText('Settings').closest('button')).toHaveAttribute('aria-expanded', 'true');
  });
});
