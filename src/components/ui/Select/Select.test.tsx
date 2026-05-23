import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Select } from './Select';

describe('Select', () => {
  const options = [
    { value: 'preview', label: 'Preview' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
  ];

  it('renders with selected label', () => {
    render(<Select options={options} value="preview" onChange={() => {}} />);
    expect(screen.getByText('Preview')).toBeInTheDocument();
  });

  it('shows placeholder when value does not match any option', () => {
    render(<Select options={options} value="unknown" onChange={() => {}} />);
    expect(screen.getByText('Select an option')).toBeInTheDocument();
  });

  it('opens dropdown on click', () => {
    render(<Select options={options} value="preview" onChange={() => {}} />);
    fireEvent.click(screen.getByRole('button'));
    expect(screen.getByText('Medium')).toBeInTheDocument();
    expect(screen.getByText('High')).toBeInTheDocument();
  });

  it('calls onChange with selected value', () => {
    const handleChange = vi.fn();
    render(<Select options={options} value="preview" onChange={handleChange} />);
    fireEvent.click(screen.getByRole('button'));
    fireEvent.click(screen.getByText('High'));
    expect(handleChange).toHaveBeenCalledWith('high');
  });

  it('closes dropdown after selection', () => {
    render(<Select options={options} value="preview" onChange={() => {}} />);
    fireEvent.click(screen.getByRole('button'));
    fireEvent.click(screen.getByText('High'));
    expect(screen.queryByText('Medium')).not.toBeInTheDocument();
  });

  it('renders with label', () => {
    render(<Select options={options} value="preview" onChange={() => {}} label="Quality" />);
    expect(screen.getByText('Quality')).toBeInTheDocument();
  });

  it('applies fullWidth class', () => {
    const { container } = render(
      <Select options={options} value="preview" onChange={() => {}} fullWidth />
    );
    expect(container.firstChild).toHaveClass('w-full');
  });

  it('closes dropdown when clicking outside', () => {
    render(
      <div>
        <Select options={options} value="preview" onChange={() => {}} />
        <span data-testid="outside">Outside</span>
      </div>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(screen.getByText('Medium')).toBeInTheDocument();
    fireEvent.mouseDown(screen.getByTestId('outside'));
    expect(screen.queryByText('Medium')).not.toBeInTheDocument();
  });
});
