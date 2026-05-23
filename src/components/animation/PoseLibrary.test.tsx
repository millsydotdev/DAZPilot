import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { PoseLibrary } from './PoseLibrary';

const mockPoses = [
  {
    id: 'pose-1',
    name: 'Casual Stand',
    category: 'basic' as const,
    file_path: '/poses/casual_stand.duf',
    keyframes: [
      {
        nodeId: 'hip',
        property: 'rotationY',
        frame: 1,
        value: 0,
        interpolation: 'linear' as const,
      },
    ],
  },
  {
    id: 'pose-2',
    name: 'Running',
    category: 'action' as const,
    file_path: '/poses/running.duf',
    keyframes: [],
  },
];

let mockStoreData = {
  poses: mockPoses,
  selectedPose: null,
  setSelectedPose: vi.fn(),
  togglePoseLibrary: vi.fn(),
  selectedFigure: null,
  setSelectedFigure: vi.fn(),
  timeline: { currentFrame: 0, fps: 30, duration: 10, totalFrames: 10 },
};

vi.mock('../../store', () => ({
  useViewportStore: (selector?: (...args: unknown[]) => unknown) =>
    selector ? selector(mockStoreData) : mockStoreData,
}));

describe('PoseLibrary', () => {
  beforeEach(() => {
    mockStoreData = {
      poses: mockPoses,
      selectedPose: null,
      setSelectedPose: vi.fn(),
      togglePoseLibrary: vi.fn(),
      selectedFigure: null,
      setSelectedFigure: vi.fn(),
      timeline: { currentFrame: 0, fps: 30, duration: 10, totalFrames: 10 },
    };
  });

  it('renders component title', () => {
    render(<PoseLibrary />);
    expect(screen.getByText('Pose Library')).toBeInTheDocument();
  });

  it('renders pose names', () => {
    render(<PoseLibrary />);
    expect(screen.getByText('Casual Stand')).toBeInTheDocument();
    expect(screen.getByText('Running')).toBeInTheDocument();
  });

  it('renders category badges', () => {
    render(<PoseLibrary />);
    expect(screen.getByText('basic')).toBeInTheDocument();
    expect(screen.getByText('action')).toBeInTheDocument();
  });

  it('shows keyframe count for poses with keyframes', () => {
    render(<PoseLibrary />);
    expect(screen.getByText('1 keyframes')).toBeInTheDocument();
  });

  it('shows pose count', () => {
    render(<PoseLibrary />);
    expect(screen.getByText(/2 poses/)).toBeInTheDocument();
  });

  it('calls setSelectedPose and togglePoseLibrary when a pose is clicked', () => {
    render(<PoseLibrary />);
    fireEvent.click(screen.getByText('Casual Stand'));
    expect(mockStoreData.setSelectedPose).toHaveBeenCalledWith(mockPoses[0]);
    expect(mockStoreData.togglePoseLibrary).toHaveBeenCalled();
  });

  it('filters poses by search term', () => {
    render(<PoseLibrary />);
    const searchInput = screen.getByPlaceholderText('Search poses...');
    fireEvent.change(searchInput, { target: { value: 'Run' } });
    expect(screen.queryByText('Casual Stand')).not.toBeInTheDocument();
    expect(screen.getByText('Running')).toBeInTheDocument();
  });

  it('shows empty state when no poses match', () => {
    render(<PoseLibrary />);
    const searchInput = screen.getByPlaceholderText('Search poses...');
    fireEvent.change(searchInput, { target: { value: 'zzz' } });
    expect(screen.getByText('No poses found')).toBeInTheDocument();
  });

  it('toggles upload form when upload button is clicked', () => {
    render(<PoseLibrary />);
    fireEvent.click(screen.getByText('Upload Pose'));
    expect(screen.getByText('Upload New Pose')).toBeInTheDocument();
    const cancelButtons = screen.getAllByText('Cancel');
    fireEvent.click(cancelButtons[1]);
    expect(screen.queryByText('Upload New Pose')).not.toBeInTheDocument();
  });

  it('calls close button to toggle library', () => {
    render(<PoseLibrary />);
    fireEvent.click(screen.getByText('Close'));
    expect(mockStoreData.togglePoseLibrary).toHaveBeenCalled();
  });
});
