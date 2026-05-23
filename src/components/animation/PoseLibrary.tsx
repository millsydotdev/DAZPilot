import { useState } from 'react';
import { useViewportStore, type Pose } from '../../store';
import { Search, Plus, Trash2, Edit, Copy, Download, Play } from 'lucide-react';
import { Button, Input, VStack, HStack, Text, Badge, Separator, ScrollArea } from '../ui';
import styles from './PoseLibrary.module.css';

export function PoseLibrary() {
  const {
    poses,
    selectedPose,
    setSelectedPose,
    togglePoseLibrary,
    selectedFigure,
    setSelectedFigure,
  } = useViewportStore();

  const [searchTerm, setSearchTerm] = useState('');
  const [filterCategory, setFilterCategory] = useState<string | null>(null);
  const [showUpload, setShowUpload] = useState(false);
  const [uploadData, setUploadData] = useState<{ name: string; file: File | null }>({
    name: '',
    file: null,
  });

  // Available pose categories
  const categories = [
    { id: 'all', label: 'All', count: poses.length },
    { id: 'basic', label: 'Basic', count: poses.filter((p) => p.category === 'basic').length },
    { id: 'action', label: 'Action', count: poses.filter((p) => p.category === 'action').length },
    {
      id: 'expression',
      label: 'Expression',
      count: poses.filter((p) => p.category === 'expression').length,
    },
    { id: 'hand', label: 'Hand Poses', count: poses.filter((p) => p.category === 'hand').length },
  ];

  // Filtered poses based on search and category
  const filteredPoses = poses.filter((pose) => {
    const matchesSearch = pose.name.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesCategory = !filterCategory || pose.category === filterCategory;
    return matchesSearch && matchesCategory;
  });

  const handleSelectPose = (pose: Pose) => {
    setSelectedPose(pose);
    setSelectedFigure(pose.name); // Set as active figure for reference
    togglePoseLibrary(); // Close library after selection
  };

  const handleApplyPose = async (pose: Pose) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('apply_pose', {
        pose_file: pose.file_path,
        figure_id: selectedFigure || 'genesis_8_female', // Default figure
      });

      // Also select the pose in UI
      setSelectedPose(pose);
    } catch (error) {
      console.error('Failed to apply pose:', error);
    }
  };

  const handleUploadPose = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploadData((prev) => ({
      ...prev,
      file,
    }));
  };

  const handleSaveUploadedPose = async () => {
    if (!uploadData.name || !uploadData.file) return;

    try {
      // In a real implementation, we would upload the file and get a path back
      // For now, we'll simulate by adding to the pose library

      // Update local state (in reality, this would come from backend)
      setTimeout(() => {
        setShowUpload(false);
        setUploadData({ name: '', file: null });
        // Would normally refresh pose list from backend
      }, 500);
    } catch (error) {
      console.error('Failed to save pose:', error);
    }
  };

  const handleDeletePose = async (pose: Pose) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_pose', {
        pose_file: pose.file_path,
      });

      // Remove from local state
      // Would normally refresh pose list from backend
    } catch (error) {
      console.error('Failed to delete pose:', error);
    }
  };

  return (
    <div className={styles.poseLibraryContainer}>
      <div className={styles.header}>
        <h2>Pose Library</h2>
        <div className={styles.headerActions}>
          <Button
            icon={<Search size={16} />}
            variant="outline"
            size="sm"
            onClick={() => setShowUpload(!showUpload)}
            disabled={showUpload}
          >
            {showUpload ? 'Cancel' : 'Upload Pose'}
          </Button>
          <Button icon={<Plus size={16} />} variant="ghost" size="sm" onClick={togglePoseLibrary}>
            Close
          </Button>
        </div>
      </div>

      <Separator />

      {/* Search and Filter */}
      <div className={styles.controlsSection}>
        <VStack gap="sm">
          <HStack>
            <Input
              placeholder="Search poses..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              icon={<Search size={16} />}
            />
          </HStack>

          <HStack>
            <Text size="xs" className={styles.filterLabel}>
              Filter by Category:
            </Text>
            <select
              value={filterCategory || 'all'}
              onChange={(e) => setFilterCategory(e.target.value === 'all' ? null : e.target.value)}
              className={styles.filterSelect}
            >
              {categories.map((cat) => (
                <option key={cat.id} value={cat.id}>
                  {cat.label} ({cat.count})
                </option>
              ))}
            </select>
          </HStack>
        </VStack>
      </div>

      <Separator />

      {/* Upload Form */}
      {showUpload && (
        <div className={styles.uploadSection}>
          <h3>Upload New Pose</h3>
          <VStack gap="sm">
            <Input
              placeholder="Pose name (e.g., 'Casual Stand')"
              value={uploadData.name}
              onChange={(e) => setUploadData((prev) => ({ ...prev, name: e.target.value }))}
            />
            <HStack>
              <Text size="xs">File:</Text>
              <input
                type="file"
                accept=".dsf,.duf,.pose"
                onChange={handleUploadPose}
                className={styles.fileInput}
              />
              {uploadData.file && (
                <span className={styles.uploadedFile}>{uploadData.file.name}</span>
              )}
            </HStack>
            <HStack justify="end">
              <Button variant="outline" onClick={() => setShowUpload(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleSaveUploadedPose}
                disabled={!(uploadData.name && uploadData.file)}
              >
                Save Pose
              </Button>
            </HStack>
          </VStack>
        </div>
      )}

      <Separator />

      {/* Poses List */}
      <div className={styles.posesSection}>
        <Text size="sm" className={styles.posesCount}>
          {filteredPoses.length} poses
        </Text>

        {filteredPoses.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No poses found</p>
            {searchTerm || filterCategory ? (
              <p className={styles.muted}>Try adjusting your search or filter</p>
            ) : (
              <p className={styles.muted}>
                No poses available. Use the upload feature to add custom poses.
              </p>
            )}
          </div>
        ) : (
          <ScrollArea className={styles.posesList}>
            {filteredPoses.map((pose) => (
              <div
                key={pose.id}
                className={`${styles.poseItem} ${selectedPose && selectedPose.id === pose.id ? styles.active : ''}`}
                onClick={() => handleSelectPose(pose)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') handleSelectPose(pose);
                }}
                role="button"
                tabIndex={0}
              >
                <div className={styles.poseInfo}>
                  <div className={styles.poseName}>{pose.name}</div>
                  <div className={styles.poseMeta}>
                    <Badge variant="secondary" size="xs">
                      {pose.category}
                    </Badge>
                    {pose.keyframes.length > 0 && (
                      <span className={styles.keyframeCount}>
                        {pose.keyframes.length} keyframes
                      </span>
                    )}
                  </div>
                </div>
                <div className={styles.poseActions}>
                  <Button
                    icon={<Play size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleApplyPose(pose);
                    }}
                    title="Apply Pose"
                  />
                  <Button
                    icon={<Copy size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      // Copy pose functionality would go here
                    }}
                    title="Copy Pose"
                  />
                  <Button
                    icon={<Download size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      // Download pose functionality would go here
                    }}
                    title="Download Pose"
                  />
                  <Button
                    icon={<Edit size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      // Edit pose functionality would go here
                    }}
                    title="Edit Pose"
                  />
                  <Button
                    icon={<Trash2 size={14} />}
                    variant="destructive"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeletePose(pose);
                    }}
                    title="Delete Pose"
                  />
                </div>
              </div>
            ))}
          </ScrollArea>
        )}
      </div>
    </div>
  );
}
