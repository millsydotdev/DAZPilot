import { useState, useEffect } from 'react';
import { usePresetStore } from '../../store';
import { Button, Input, Text, Select } from '../ui';
import { Trash2, Save, Loader2 } from 'lucide-react';
import styles from './PresetPanel.module.css';

export default function PresetPanel() {
  const {
    presets,
    selectedPreset,
    isSaving,
    isLoading,
    error,
    saveCurrentSceneAsPreset,
    loadPreset,
    deletePreset,
    loadPersistedPresets,
    getPresetsByCategory,
    setIsSaving,
    setIsLoading,
    setError,
  } = usePresetStore();

  const [newPresetName, setNewPresetName] = useState('');
  const [newPresetDescription, setNewPresetDescription] = useState('');
  const [newPresetCategory, setNewPresetCategory] = useState('scene');
  const [activeTab, setActiveTab] = useState<
    'all' | 'lighting' | 'camera' | 'figure' | 'scene' | 'custom'
  >('all');
  const [showSaveDialog, setShowSaveDialog] = useState(false);

  useEffect(() => {
    void loadPersistedPresets();
  }, [loadPersistedPresets]);

  const handleSavePreset = async () => {
    if (!newPresetName.trim()) {
      setError('Please enter a preset name');
      return;
    }

    setIsSaving(true);
    setError(null);
    try {
      await saveCurrentSceneAsPreset(
        newPresetName.trim(),
        newPresetDescription.trim(),
        newPresetCategory
      );
      setNewPresetName('');
      setNewPresetDescription('');
      setShowSaveDialog(false);
    } catch (e) {
      console.error('Error saving preset:', e);
      setError('Failed to save preset');
    } finally {
      setIsSaving(false);
    }
  };

  const handleLoadPreset = async (id: string) => {
    setIsLoading(true);
    setError(null);
    try {
      await loadPreset(id);
    } catch (e) {
      console.error('Error loading preset:', e);
      setError('Failed to load preset');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeletePreset = async (id: string) => {
    setIsSaving(true); // Reuse isSaving for deletion feedback
    setError(null);
    try {
      const preset = presets.find((p) => p.id === id);
      if (!preset) {
        throw new Error(`Preset with ID ${id} not found`);
      }

      // Confirm deletion
      const confirmed = window.confirm(
        `Are you sure you want to delete the preset "${preset.name}"?`
      );
      if (!confirmed) {
        setIsSaving(false);
        return;
      }

      await deletePreset(id);
    } catch (e) {
      console.error('Error deleting preset:', e);
      setError('Failed to delete preset');
    } finally {
      setIsSaving(false);
    }
  };

  const filteredPresets = presets.filter(
    (preset) => activeTab === 'all' || preset.category === activeTab
  );

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>Scene Presets</h2>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowSaveDialog(true)}
          className={styles.saveButton}
        >
          <Save size={16} />
          Save Current Scene
        </Button>
      </div>

      {/* Save Preset Dialog */}
      {showSaveDialog && (
        <div
          className={styles.dialogOverlay}
          onClick={() => setShowSaveDialog(false)}
          role="presentation"
          onKeyDown={(e) => e.key === 'Escape' && setShowSaveDialog(false)}
        >
          <div
            className={styles.dialogContent}
            onClick={(e) => e.stopPropagation()}
            role="presentation"
            onKeyDown={(e) => e.stopPropagation()}
          >
            <h3 className={styles.dialogTitle}>Save Scene as Preset</h3>
            <div className={styles.dialogBody}>
              <Input
                placeholder="Preset name (e.g., '3-Point Lighting')"
                value={newPresetName}
                onChange={(e) => setNewPresetName(e.target.value)}
                className={styles.input}
              />
              <Input
                placeholder="Description (optional)"
                value={newPresetDescription}
                onChange={(e) => setNewPresetDescription(e.target.value)}
                className={styles.input}
                style={{ marginTop: '8px' }}
              />
              <Select
                value={newPresetCategory}
                onChange={(value) => setNewPresetCategory(value)}
                options={[
                  { value: 'lighting', label: 'Lighting Setup' },
                  { value: 'camera', label: 'Camera Angle' },
                  { value: 'figure', label: 'Figure Pose' },
                  { value: 'scene', label: 'Full Scene' },
                  { value: 'custom', label: 'Custom' },
                ]}
                className={styles.select}
                style={{ marginTop: '8px' }}
              />
            </div>
            <div className={styles.dialogFooter}>
              <Button variant="outline" size="sm" onClick={() => setShowSaveDialog(false)}>
                Cancel
              </Button>
              <Button
                variant="primary"
                size="sm"
                onClick={handleSavePreset}
                loading={isSaving}
                disabled={isSaving}
              >
                {isSaving ? 'Saving...' : 'Save Preset'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${activeTab === 'all' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('all')}
        >
          All ({presets.length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'lighting' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('lighting')}
        >
          Lighting ({getPresetsByCategory('lighting').length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'camera' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('camera')}
        >
          Camera ({getPresetsByCategory('camera').length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'figure' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('figure')}
        >
          Figure ({getPresetsByCategory('figure').length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'scene' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('scene')}
        >
          Scene ({getPresetsByCategory('scene').length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'custom' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('custom')}
        >
          Custom ({getPresetsByCategory('custom').length})
        </button>
      </div>

      {/* Error Message */}
      {error && (
        <div className={styles.errorMessage}>
          <Text color="error">{error}</Text>
        </div>
      )}

      {/* Presets List */}
      <div className={styles.presetsList}>
        {filteredPresets.length === 0 ? (
          <div className={styles.emptyState}>
            <Text color="muted">No presets found</Text>
            {activeTab !== 'all' && (
              <Text color="muted" style={{ marginTop: '4px' }}>
                Try switching to &quot;All&quot; tab or create a new preset
              </Text>
            )}
          </div>
        ) : (
          filteredPresets.map((preset) => (
            <div key={preset.id} className={styles.presetCard}>
              <div className={styles.presetHeader}>
                <h4 className={styles.presetTitle}>{preset.name}</h4>
                <div className={styles.presetMeta}>
                  <span className={styles.categoryTag}>
                    {preset.category.charAt(0).toUpperCase() + preset.category.slice(1)}
                  </span>
                  <span className={styles.presetDate}>
                    {new Date(preset.createdAt).toLocaleDateString()}
                  </span>
                </div>
              </div>
              <p className={styles.presetDescription}>
                {preset.description || 'No description provided'}
              </p>
              <div className={styles.presetActions}>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleLoadPreset(preset.id)}
                  loading={isLoading && selectedPreset?.id === preset.id}
                  disabled={isLoading && selectedPreset?.id === preset.id}
                >
                  {isLoading && selectedPreset?.id === preset.id ? <Loader2 size={14} /> : 'Load'}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleDeletePreset(preset.id)}
                  loading={isSaving}
                  disabled={isSaving}
                  className={styles.deleteButton}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
