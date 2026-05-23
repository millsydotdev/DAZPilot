import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Modal } from './Modal';

describe('Modal', () => {
  it('does not render when isOpen is false', () => {
    const { container } = render(
      <Modal isOpen={false} onClose={() => {}}>
        Content
      </Modal>
    );
    expect(container.innerHTML).toBe('');
  });

  it('renders children when isOpen is true', () => {
    render(
      <Modal isOpen={true} onClose={() => {}}>
        <span>Modal content</span>
      </Modal>
    );
    expect(screen.getByText('Modal content')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(
      <Modal isOpen={true} onClose={() => {}} title="My Title">
        Content
      </Modal>
    );
    expect(screen.getByText('My Title')).toBeInTheDocument();
  });

  it('calls onClose when clicking close button', () => {
    const handleClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={handleClose} title="Title">
        Content
      </Modal>
    );
    const closeBtn = screen.getByRole('button');
    fireEvent.click(closeBtn);
    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when pressing Escape', () => {
    const handleClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={handleClose}>
        Content
      </Modal>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('applies custom className to content wrapper', () => {
    const { container } = render(
      <Modal isOpen={true} onClose={() => {}} className="my-modal">
        Content
      </Modal>
    );
    const contentDiv = container.querySelector('.my-modal');
    expect(contentDiv).toBeInTheDocument();
  });
});
