import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ContextMenu } from './ContextMenu';

describe('ContextMenu', () => {
  const items = [
    { label: 'Open', onClick: vi.fn(), icon: <span data-testid="icon-open">O</span> },
    { label: 'Delete', onClick: vi.fn(), danger: true },
  ];

  it('renders children', () => {
    render(
      <ContextMenu items={items}>
        <span>Right-click me</span>
      </ContextMenu>
    );
    expect(screen.getByText('Right-click me')).toBeInTheDocument();
  });

  it('shows menu on context menu event', () => {
    render(
      <ContextMenu items={items}>
        <span>Right-click me</span>
      </ContextMenu>
    );
    fireEvent.contextMenu(screen.getByText('Right-click me'));
    expect(screen.getByText('Open')).toBeInTheDocument();
    expect(screen.getByText('Delete')).toBeInTheDocument();
  });

  it('hides menu on outside click', () => {
    render(
      <div>
        <ContextMenu items={items}>
          <span>Right-click me</span>
        </ContextMenu>
        <span data-testid="outside">Outside</span>
      </div>
    );
    fireEvent.contextMenu(screen.getByText('Right-click me'));
    expect(screen.getByText('Open')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('outside'));
    expect(screen.queryByText('Open')).not.toBeInTheDocument();
  });

  it('calls onClick when a menu item is clicked', () => {
    const handleOpen = vi.fn();
    render(
      <ContextMenu items={[{ label: 'Open', onClick: handleOpen }]}>
        <span>Right-click</span>
      </ContextMenu>
    );
    fireEvent.contextMenu(screen.getByText('Right-click'));
    fireEvent.click(screen.getByText('Open'));
    expect(handleOpen).toHaveBeenCalledTimes(1);
  });
});
