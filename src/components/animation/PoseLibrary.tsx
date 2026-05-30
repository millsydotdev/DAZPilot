import { useState } from 'react';
import { useViewportStore, type Pose } from '../../store';
import { useToastStore } from '../../store/toastStore';
import { Search, Plus, Trash2, Edit, Copy, Download, Play } from 'lucide-react';
import { Button, Input, VStack, HStack, Text, Badge, Separator, ScrollArea } from '../ui';
import styles from './PoseLibrary.module.css';

interface PoseLibraryProps {
  embedded?: boolean;
}

export function PoseLibrary({ embedded = false }: PoseLibraryProps) {
  const {
    poses,
    selectedPose,
    setSelectedPose,
    togglePoseLibrary,
    selectedFigure,
    setSelectedFigure,
    loadState,
  } = useViewportStore();

  const toast = useToastStore();

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
    setSelectedFigure(pose.name);
    if (!embedded) {
      togglePoseLibrary();
    }
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
      const { invoke } = await import('@tauri-apps/api/core');
      const bytes = new Uint8Array(await uploadData.file.arrayBuffer());
      await invoke('save_uploaded_pose', {
        name: uploadData.name,
        category: 'basic',
        file_bytes: Array.from(bytes),
        original_filename: uploadData.file.name,
      });
      await loadState();
      setShowUpload(false);
      setUploadData({ name: '', file: null });
      toast.success(`Pose "${uploadData.name}" saved to library`);
    } catch (error) {
      console.error('Failed to save pose:', error);
      toast.error(`Failed to save pose: ${error}`);
    }
  };

  const handleCopyPose = async (pose: Pose) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const copyName = `${pose.name} Copy`;
      await invoke('duplicate_pose', {
        pose_file: pose.file_path,
        new_name: copyName,
      });
      await loadState();
      toast.success(`Copied pose as "${copyName}"`);
    } catch (error) {
      toast.error(`Failed to copy pose: ${error}`);
    }
  };

  const handleDownloadPose = async (pose: Pose) => {
    if (!pose.file_path) {
      toast.warning('No file path available for this pose');
      return;
    }
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const dest = await invoke<string | null>('select_directory', {
        title: 'Select folder to save pose file',
      });
      if (!dest) return;
      await invoke('copy_file_to_directory', {
        source_path: pose.file_path,
        dest_directory: dest,
      });
      toast.success(`Pose saved to ${dest}`);
    } catch (error) {
      toast.error(`Failed to download pose: ${error}`);
    }
  };

  const handleEditPose = async (pose: Pose) => {
    const newName = window.prompt('Enter new name for pose:', pose.name);
    if (!newName || newName === pose.name || !pose.file_path) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('rename_pose', {
        pose_file: pose.file_path,
        new_name: newName,
      });
      await loadState();
      toast.success(`Pose renamed to "${newName}"`);
    } catch (error) {
      toast.error(`Failed to rename pose: ${error}`);
    }
  };

  const handleDeletePose = async (pose: Pose) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_pose', {
        pose_file: pose.file_path,
      });

      await loadState();
      toast.success(`Deleted pose "${pose.name}"`);
    } catch (error) {
      console.error('Failed to delete pose:', error);
    }
  };

  return (
    <div className={embedded ? styles.embedded : styles.poseLibraryContainer}>
      {!embedded && (
        <>
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
              <Button
                icon={<Plus size={16} />}
                variant="ghost"
                size="sm"
                onClick={togglePoseLibrary}
              >
                Close
              </Button>
            </div>
          </div>
          <Separator />
        </>
      )}

      {embedded && (
        <div className={styles.embeddedToolbar}>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowUpload(!showUpload)}
            disabled={showUpload}
          >
            {showUpload ? 'Cancel upload' : 'Upload pose'}
          </Button>
        </div>
      )}

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
                      void handleCopyPose(pose);
                    }}
                    title="Copy Pose"
                  />
                  <Button
                    icon={<Download size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      void handleDownloadPose(pose);
                    }}
                    title="Download Pose"
                  />
                  <Button
                    icon={<Edit size={14} />}
                    variant="ghost"
                    size="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      void handleEditPose(pose);
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
