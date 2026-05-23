import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { StatusBar } from './StatusBar';

vi.mock('../../../store', () => ({
  useConnectionStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({
      status: 'connected',
      aiModel: { loaded: true, name: 'GPT-4' },
    }),
  useSceneStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({
      figures: [{ id: 'fig1', name: 'Genesis', selected: true }],
    }),
  useViewportStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({
      timeline: { fps: 30 },
    }),
}));

describe('StatusBar', () => {
  it('shows connected status', () => {
    render(<StatusBar />);
    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('shows AI model name when loaded', () => {
    render(<StatusBar />);
    expect(screen.getByText('GPT-4')).toBeInTheDocument();
  });

  it('shows active figure name', () => {
    render(<StatusBar />);
    expect(screen.getByText('Genesis')).toBeInTheDocument();
  });

  it('shows FPS', () => {
    render(<StatusBar />);
    expect(screen.getByText('30 fps')).toBeInTheDocument();
  });
});
