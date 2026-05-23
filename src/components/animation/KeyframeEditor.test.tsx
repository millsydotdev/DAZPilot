import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { KeyframeEditor } from './KeyframeEditor';

const mockPose = {
  id: 'pose-1',
  name: 'Test Pose',
  category: 'basic' as const,
  file_path: '/test.pose',
  keyframes: [
    { nodeId: 'arm', property: 'rotationX', frame: 1, value: 45, interpolation: 'linear' as const },
    { nodeId: 'arm', property: 'rotationX', frame: 2, value: 90, interpolation: 'linear' as const },
  ],
};

type StoreData = {
  timeline: { currentFrame: number; fps: number; duration: number; totalFrames: number };
  selectedPose: typeof mockPose | null;
  togglePoseLibrary: ReturnType<typeof vi.fn>;
  poses: (typeof mockPose)[];
  setSelectedPose: ReturnType<typeof vi.fn>;
  selectedFigure: string | null;
  setSelectedFigure: ReturnType<typeof vi.fn>;
};

let mockStoreData: StoreData = {
  timeline: { currentFrame: 1, fps: 30, duration: 10, totalFrames: 10 },
  selectedPose: mockPose,
  togglePoseLibrary: vi.fn(),
  poses: [mockPose],
  setSelectedPose: vi.fn(),
  selectedFigure: null,
  setSelectedFigure: vi.fn(),
};

vi.mock('../../store', () => ({
  useViewportStore: (selector?: (...args: unknown[]) => unknown) =>
    selector ? selector(mockStoreData) : mockStoreData,
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('KeyframeEditor', () => {
  beforeEach(() => {
    mockStoreData = {
      timeline: { currentFrame: 1, fps: 30, duration: 10, totalFrames: 10 },
      selectedPose: mockPose,
      togglePoseLibrary: vi.fn(),
      poses: [mockPose],
      setSelectedPose: vi.fn(),
      selectedFigure: null,
      setSelectedFigure: vi.fn(),
    };
  });

  it('renders the component title', () => {
    render(<KeyframeEditor />);
    expect(screen.getByText('Keyframe Editor')).toBeInTheDocument();
  });

  it('shows current frame and FPS', () => {
    render(<KeyframeEditor />);
    expect(screen.getByText(/Current Frame: 1/)).toBeInTheDocument();
    expect(screen.getByText(/FPS: 30/)).toBeInTheDocument();
  });

  it('shows toggle pose library button', () => {
    render(<KeyframeEditor />);
    expect(screen.getByText('Close Pose')).toBeInTheDocument();
  });

  it('calls togglePoseLibrary when button is clicked', () => {
    render(<KeyframeEditor />);
    fireEvent.click(screen.getByText('Close Pose'));
    expect(mockStoreData.togglePoseLibrary).toHaveBeenCalled();
  });

  it('shows keyframes at current frame', () => {
    render(<KeyframeEditor />);
    expect(screen.getAllByText('arm').length).toBeGreaterThan(0);
    expect(screen.getAllByText('rotationX').length).toBeGreaterThan(0);
    expect(screen.getAllByText('45.000').length).toBeGreaterThan(0);
  });

  it('shows all keyframes section', () => {
    render(<KeyframeEditor />);
    expect(screen.getByText('All Keyframes')).toBeInTheDocument();
    expect(screen.getByText('Frame 1')).toBeInTheDocument();
    expect(screen.getByText('Frame 2')).toBeInTheDocument();
  });

  it('shows keyframe count for the frame', () => {
    render(<KeyframeEditor />);
    expect(screen.getByText(/Keyframes at Frame 1/)).toBeInTheDocument();
  });

  it('shows empty state when no selectedPose', () => {
    mockStoreData.selectedPose = null;
    render(<KeyframeEditor />);
    expect(screen.queryByText('All Keyframes')).not.toBeInTheDocument();
    expect(screen.queryByText('90.000')).not.toBeInTheDocument();
  });
});
