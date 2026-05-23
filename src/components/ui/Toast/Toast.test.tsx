import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Toast } from './Toast';

vi.mock('../../../store/toastStore', () => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  useToastStore: (selector?: any) => {
    const state = { removeToast: vi.fn() };
    return selector ? selector(state) : state;
  },
}));

const createToast = (overrides = {}) => ({
  id: 'test-1',
  message: 'Test message',
  type: 'info' as const,
  duration: 0,
  ...overrides,
});

describe('Toast', () => {
  it('renders message text', () => {
    render(<Toast toast={createToast()} />);
    expect(screen.getByText('Test message')).toBeInTheDocument();
  });

  it('renders default title based on type', () => {
    render(<Toast toast={createToast({ type: 'info' })} />);
    expect(screen.getByText('Notification')).toBeInTheDocument();

    render(<Toast toast={createToast({ type: 'success', id: 'test-2' })} />);
    expect(screen.getByText('Success')).toBeInTheDocument();

    render(<Toast toast={createToast({ type: 'error', id: 'test-3' })} />);
    expect(screen.getByText('Error')).toBeInTheDocument();

    render(<Toast toast={createToast({ type: 'warning', id: 'test-4' })} />);
    expect(screen.getByText('Warning')).toBeInTheDocument();
  });

  it('renders custom title when provided', () => {
    render(<Toast toast={createToast({ title: 'Custom Title' })} />);
    expect(screen.getByText('Custom Title')).toBeInTheDocument();
  });

  it('renders close button', () => {
    render(<Toast toast={createToast()} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('renders progress bar when duration > 0', () => {
    const { container } = render(<Toast toast={createToast({ duration: 4000 })} />);
    expect(container.querySelector('[class*="progressBar"]')).toBeInTheDocument();
  });

  it('does not render progress bar when duration is 0', () => {
    const { container } = render(<Toast toast={createToast({ duration: 0 })} />);
    expect(container.querySelector('[class*="progressBar"]')).not.toBeInTheDocument();
  });

  it('applies exiting class when close button is clicked', () => {
    const { container } = render(<Toast toast={createToast()} />);
    fireEvent.click(screen.getByRole('button'));
    expect(container.firstElementChild?.className).toContain('exiting');
  });
});
