import { useState, useEffect } from 'react';
import { useViewportStore, type Keyframe } from '../../store';
import { Trash2, Edit } from 'lucide-react';
import { Button, Input, HStack, VStack, Text, Separator } from '../ui';
import styles from './KeyframeEditor.module.css';

export function KeyframeEditor() {
  const { timeline, selectedPose, togglePoseLibrary } = useViewportStore();

  const [editingKeyframe, setEditingKeyframe] = useState<{
    nodeId: string;
    property: string;
    frame: number;
    value: number;
  } | null>(null);

  const [newKeyframe, setNewKeyframe] = useState({
    nodeId: '',
    property: '',
    frame: 0,
    value: 0,
  });

  useEffect(() => {
    // Reset editing state when timeline changes
    if (timeline.currentFrame !== editingKeyframe?.frame) {
      setEditingKeyframe(null);
    }
  }, [timeline.currentFrame, editingKeyframe]);

  const handleAddKeyframe = async () => {
    if (!newKeyframe.nodeId || !newKeyframe.property) return;

    // Import the animation module to create keyframe
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('create_keyframe', {
        node_id: newKeyframe.nodeId,
        property: newKeyframe.property,
        frame: newKeyframe.frame,
        value: newKeyframe.value,
        interpolation: 'linear',
      });

      // Reset form
      setNewKeyframe({ nodeId: '', property: '', frame: 0, value: 0 });
    } catch (error) {
      console.error('Failed to create keyframe:', error);
    }
  };

  const handleUpdateKeyframe = async () => {
    if (!editingKeyframe) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('create_keyframe', {
        node_id: editingKeyframe.nodeId,
        property: editingKeyframe.property,
        frame: editingKeyframe.frame,
        value: editingKeyframe.value,
        interpolation: 'linear',
      });

      setEditingKeyframe(null);
    } catch (error) {
      console.error('Failed to update keyframe:', error);
    }
  };

  const handleDeleteKeyframe = async (keyframe: Keyframe) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_keyframe', {
        node_id: keyframe.nodeId,
        property: keyframe.property,
        frame: keyframe.frame,
      });
    } catch (error) {
      console.error('Failed to delete keyframe:', error);
    }
  };

  const handleStartEdit = (keyframe: Keyframe) => {
    setEditingKeyframe({
      nodeId: keyframe.nodeId,
      property: keyframe.property,
      frame: keyframe.frame,
      value: keyframe.value,
    });
  };

  // Get keyframes for current frame
  const keyframesAtCurrentFrame =
    selectedPose?.keyframes.filter((kf) => kf.frame === Math.floor(timeline.currentFrame)) || [];

  return (
    <div className={styles.keyframeEditor}>
      <div className={styles.header}>
        <h3>Keyframe Editor</h3>
        <Button
          variant="outline"
          size="sm"
          onClick={togglePoseLibrary}
          className={`${styles.toggleButton} ${selectedPose ? styles.active : ''}`}
        >
          {selectedPose ? 'Close Pose' : 'Open Pose Library'}
        </Button>
      </div>

      <Separator />

      <div className={styles.timelineInfo}>
        <Text variant="muted" size="xs">
          Current Frame: {Math.floor(timeline.currentFrame)}
        </Text>
        <Text variant="muted" size="xs">
          FPS: {timeline.fps}
        </Text>
      </div>

      <Separator />

      {/* Keyframe Form */}
      <div className={styles.formSection}>
        <h4>Add/Edit Keyframe</h4>

        <VStack gap="sm">
          <HStack>
            <Text size="xs" width="60">
              Node:
            </Text>
            <Input
              placeholder="e.g., Genesis8Female:LeftHand"
              value={editingKeyframe ? editingKeyframe.nodeId : newKeyframe.nodeId}
              onChange={(e) => {
                if (editingKeyframe) {
                  setEditingKeyframe({ ...editingKeyframe, nodeId: e.target.value });
                } else {
                  setNewKeyframe((prev) => ({ ...prev, nodeId: e.target.value }));
                }
              }}
            />
          </HStack>

          <HStack>
            <Text size="xs" width="60">
              Property:
            </Text>
            <Input
              placeholder="e.g., scaleX, positionZ"
              value={editingKeyframe ? editingKeyframe.property : newKeyframe.property}
              onChange={(e) => {
                if (editingKeyframe) {
                  setEditingKeyframe({ ...editingKeyframe, property: e.target.value });
                } else {
                  setNewKeyframe((prev) => ({ ...prev, property: e.target.value }));
                }
              }}
            />
          </HStack>

          <HStack>
            <Text size="xs" width="60">
              Frame:
            </Text>
            <Input
              type="number"
              placeholder="Frame number"
              value={editingKeyframe ? editingKeyframe.frame : newKeyframe.frame}
              onChange={(e) => {
                const val = parseInt(e.target.value) || 0;
                if (editingKeyframe) {
                  setEditingKeyframe({ ...editingKeyframe, frame: val });
                } else {
                  setNewKeyframe((prev) => ({ ...prev, frame: val }));
                }
              }}
            />
          </HStack>

          <HStack>
            <Text size="xs" width="60">
              Value:
            </Text>
            <Input
              type="number"
              placeholder="Property value"
              value={editingKeyframe ? editingKeyframe.value : newKeyframe.value}
              onChange={(e) => {
                const val = parseFloat(e.target.value) || 0;
                if (editingKeyframe) {
                  setEditingKeyframe({ ...editingKeyframe, value: val });
                } else {
                  setNewKeyframe((prev) => ({ ...prev, value: val }));
                }
              }}
            />
          </HStack>

          <HStack justify="end">
            {editingKeyframe ? (
              <Button onClick={handleUpdateKeyframe} className={styles.updateButton}>
                Update Keyframe
              </Button>
            ) : (
              <Button
                onClick={handleAddKeyframe}
                className={styles.addButton}
                disabled={!(newKeyframe.nodeId && newKeyframe.property)}
              >
                Add Keyframe
              </Button>
            )}
            <Button
              variant="outline"
              onClick={() => setEditingKeyframe(null)}
              className={styles.cancelButton}
            >
              Cancel
            </Button>
          </HStack>
        </VStack>
      </div>

      <Separator />

      {/* Current Frame Keyframes */}
      {selectedPose && (
        <>
          <h4>Keyframes at Frame {Math.floor(timeline.currentFrame)}</h4>
          {keyframesAtCurrentFrame.length > 0 ? (
            <div className={styles.keyframesList}>
              {keyframesAtCurrentFrame.map((kf) => (
                <div
                  key={`${kf.nodeId}-${kf.property}-${kf.frame}`}
                  className={styles.keyframeItem}
                >
                  <div className={styles.keyframeInfo}>
                    <Text size="xs" className={styles.nodeName}>
                      {kf.nodeId.split(':').pop() || kf.nodeId}
                    </Text>
                    <Text size="xs" className={styles.propertyName}>
                      {kf.property}
                    </Text>
                    <Text size="xs" className={styles.keyframeValue}>
                      {kf.value.toFixed(3)}
                    </Text>
                  </div>
                  <div className={styles.keyframeActions}>
                    <Button
                      icon={<Edit size={16} />}
                      variant="ghost"
                      size="xs"
                      onClick={() => handleStartEdit(kf)}
                      title="Edit"
                    />
                    <Button
                      icon={<Trash2 size={16} />}
                      variant="ghost"
                      size="xs"
                      onClick={() => handleDeleteKeyframe(kf)}
                      title="Delete"
                    />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <Text variant="muted" size="xs" className={styles.noKeyframes}>
              No keyframes at this frame
            </Text>
          )}
        </>
      )}

      <Separator />

      {/* All Keyframes for Selected Pose */}
      {selectedPose && selectedPose.keyframes.length > 0 && (
        <>
          <h4>All Keyframes</h4>
          <div className={styles.allKeyframes}>
            {selectedPose.keyframes
              .slice()
              .sort((a, b) => a.frame - b.frame)
              .map((kf) => (
                <div
                  key={`${kf.nodeId}-${kf.property}-${kf.frame}`}
                  className={styles.keyframeItem}
                >
                  <div className={styles.keyframeInfo}>
                    <Text size="xs" className={styles.nodeName}>
                      {kf.nodeId.split(':').pop() || kf.nodeId}
                    </Text>
                    <Text size="xs" className={styles.propertyName}>
                      {kf.property}
                    </Text>
                    <Text size="xs" className={styles.keyframeValue}>
                      {kf.value.toFixed(3)}
                    </Text>
                    <Text size="xs" className={styles.keyframeFrame}>
                      Frame {kf.frame}
                    </Text>
                  </div>
                  <div className={styles.keyframeActions}>
                    <Button
                      icon={<Edit size={16} />}
                      variant="ghost"
                      size="xs"
                      onClick={() => {
                        setEditingKeyframe({
                          nodeId: kf.nodeId,
                          property: kf.property,
                          frame: kf.frame,
                          value: kf.value,
                        });
                      }}
                      title="Edit"
                    />
                    <Button
                      icon={<Trash2 size={16} />}
                      variant="ghost"
                      size="xs"
                      onClick={() => handleDeleteKeyframe(kf)}
                      title="Delete"
                    />
                  </div>
                </div>
              ))}
          </div>
        </>
      )}
    </div>
  );
}
