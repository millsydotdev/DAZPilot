import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Tooltip, TooltipTrigger, TooltipContent } from './Tooltip';

describe('Tooltip', () => {
  it('shows tooltip content on mouse enter', () => {
    render(
      <Tooltip>
        <TooltipTrigger>Hover me</TooltipTrigger>
        <TooltipContent>Tooltip text</TooltipContent>
      </Tooltip>
    );
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    fireEvent.mouseEnter(screen.getByText('Hover me'));
    expect(screen.getByRole('tooltip')).toBeInTheDocument();
    expect(screen.getByText('Tooltip text')).toBeInTheDocument();
  });

  it('hides tooltip on mouse leave', () => {
    render(
      <Tooltip>
        <TooltipTrigger>Hover me</TooltipTrigger>
        <TooltipContent>Tooltip text</TooltipContent>
      </Tooltip>
    );
    fireEvent.mouseEnter(screen.getByText('Hover me'));
    expect(screen.getByRole('tooltip')).toBeInTheDocument();
    fireEvent.mouseLeave(screen.getByText('Hover me'));
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('shows tooltip on focus', () => {
    render(
      <Tooltip>
        <TooltipTrigger>Focus me</TooltipTrigger>
        <TooltipContent>Focused tooltip</TooltipContent>
      </Tooltip>
    );
    fireEvent.focus(screen.getByText('Focus me'));
    expect(screen.getByText('Focused tooltip')).toBeInTheDocument();
  });

  it('hides tooltip on blur', () => {
    render(
      <Tooltip>
        <TooltipTrigger>Focus me</TooltipTrigger>
        <TooltipContent>Focused tooltip</TooltipContent>
      </Tooltip>
    );
    fireEvent.focus(screen.getByText('Focus me'));
    expect(screen.getByText('Focused tooltip')).toBeInTheDocument();
    fireEvent.blur(screen.getByText('Focus me'));
    expect(screen.queryByText('Focused tooltip')).not.toBeInTheDocument();
  });

  it('sets data-side attribute on tooltip', () => {
    render(
      <Tooltip side="bottom">
        <TooltipTrigger>Hover</TooltipTrigger>
        <TooltipContent>Bottom tooltip</TooltipContent>
      </Tooltip>
    );
    fireEvent.mouseEnter(screen.getByText('Hover'));
    expect(screen.getByRole('tooltip')).toHaveAttribute('data-side', 'bottom');
  });

  it('throws error if TooltipContent is used outside Tooltip', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const errorHandler = (e: Event) => {
      e.preventDefault();
    };
    window.addEventListener('error', errorHandler);
    expect(() => render(<TooltipContent>Orphan</TooltipContent>)).toThrow(
      'Tooltip components must be used within a Tooltip provider'
    );
    window.removeEventListener('error', errorHandler);
    consoleSpy.mockRestore();
  });
});
