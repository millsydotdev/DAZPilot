import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Tabs } from './Tabs';

describe('Tabs', () => {
  const tabs = [
    { id: 'tab1', label: 'First Tab' },
    { id: 'tab2', label: 'Second Tab', icon: <span data-testid="icon-tab2">*</span> },
  ];

  it('renders all tabs', () => {
    render(<Tabs tabs={tabs} activeTab="tab1" onTabChange={() => {}} />);
    expect(screen.getByText('First Tab')).toBeInTheDocument();
    expect(screen.getByText('Second Tab')).toBeInTheDocument();
  });

  it('calls onTabChange with tab id when clicked', () => {
    const handleChange = vi.fn();
    render(<Tabs tabs={tabs} activeTab="tab1" onTabChange={handleChange} />);
    fireEvent.click(screen.getByText('Second Tab'));
    expect(handleChange).toHaveBeenCalledWith('tab2');
  });

  it('renders icon when provided', () => {
    render(<Tabs tabs={tabs} activeTab="tab1" onTabChange={() => {}} />);
    expect(screen.getByTestId('icon-tab2')).toBeInTheDocument();
  });

  it('applies active styles to active tab', () => {
    render(<Tabs tabs={tabs} activeTab="tab2" onTabChange={() => {}} />);
    const activeBtn = screen.getByText('Second Tab').closest('button');
    expect(activeBtn).toHaveClass('text-accent');
  });

  it('applies custom className', () => {
    const { container } = render(
      <Tabs tabs={tabs} activeTab="tab1" onTabChange={() => {}} className="my-tabs" />
    );
    expect(container.firstChild).toHaveClass('my-tabs');
  });
});
