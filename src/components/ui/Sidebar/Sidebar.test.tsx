import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  const items = [
    { id: 'home', label: 'Home', icon: <span data-testid="icon-home">H</span> },
    { id: 'settings', label: 'Settings', icon: <span data-testid="icon-settings">S</span> },
  ];

  it('renders all items', () => {
    render(<Sidebar items={items} activeId="home" onItemClick={() => {}} />);
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders icons for each item', () => {
    render(<Sidebar items={items} activeId="home" onItemClick={() => {}} />);
    expect(screen.getByTestId('icon-home')).toBeInTheDocument();
    expect(screen.getByTestId('icon-settings')).toBeInTheDocument();
  });

  it('calls onItemClick when an item is clicked', () => {
    const handleClick = vi.fn();
    render(<Sidebar items={items} activeId="home" onItemClick={handleClick} />);
    fireEvent.click(screen.getByText('Settings'));
    expect(handleClick).toHaveBeenCalledWith('settings');
  });

  it('highlights active item', () => {
    const { container } = render(
      <Sidebar items={items} activeId="settings" onItemClick={() => {}} />
    );
    const buttons = container.querySelectorAll('button');
    expect(buttons[1]).toHaveClass('text-accent');
  });
});
