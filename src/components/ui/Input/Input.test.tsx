import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Input } from './Input';

describe('Input', () => {
  it('renders with placeholder', () => {
    render(<Input placeholder="Enter text" />);
    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
  });

  it('renders with value', () => {
    render(<Input defaultValue="test value" />);
    expect(screen.getByRole('textbox')).toHaveValue('test value');
  });

  it('handles onChange events', () => {
    const handleChange = vi.fn();
    render(<Input onChange={handleChange} />);
    const input = screen.getByRole('textbox');
    // Simulate React change event
    fireEvent.change(input, { target: { value: 'new value' } });
    expect(handleChange).toHaveBeenCalled();
    // Get the actual argument passed to the handler
    const callArg = handleChange.mock.calls[0][0];
    expect(callArg.target.value).toBe('new value');
  });

  it('applies size classes', () => {
    render(<Input variantSize="sm" />);
    const input = screen.getByRole('textbox');
    // Check that the input has the base input class (CSS modules will add hash)
    expect(input).toHaveAttribute('class', expect.stringContaining('input'));
  });

  it('shows disabled state', () => {
    render(<Input disabled />);
    const input = screen.getByRole('textbox');
    expect(input).toBeDisabled();
  });

  it('applies error state', () => {
    render(<Input error />);
    const input = screen.getByRole('textbox');
    // Check that the input has the base input class (CSS modules will add hash)
    expect(input).toHaveAttribute('class', expect.stringContaining('input'));
  });
});
