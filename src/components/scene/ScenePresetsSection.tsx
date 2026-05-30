import { useState, useEffect } from 'react';
import { Save, Trash2, Loader2, Bookmark } from 'lucide-react';
import { usePresetStore } from '../../store';
import { Button, Input, Select } from '../ui';
import styles from './ScenePresetsSection.module.css';

export default function ScenePresetsSection() {
  const {
    presets,
    isSaving,
    isLoading,
    error,
    saveCurrentSceneAsPreset,
    loadPreset,
    deletePreset,
    loadPersistedPresets,
    setError,
  } = usePresetStore();

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [category, setCategory] = useState('scene');
  const [showForm, setShowForm] = useState(false);

  useEffect(() => {
    void loadPersistedPresets();
  }, [loadPersistedPresets]);

  const handleSave = async () => {
    if (!name.trim()) {
      setError('Enter a preset name');
      return;
    }
    await saveCurrentSceneAsPreset(name.trim(), description.trim(), category);
    setName('');
    setDescription('');
    setShowForm(false);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Bookmark size={16} />
        <span>Scene Presets</span>
        <Button size="sm" onClick={() => setShowForm(!showForm)}>
          <Save size={14} />
          Save Current
        </Button>
      </div>

      {error && <p className={styles.error}>{error}</p>}

      {showForm && (
        <div className={styles.form}>
          <Input placeholder="Preset name" value={name} onChange={(e) => setName(e.target.value)} />
          <Input
            placeholder="Description (optional)"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
          <Select
            value={category}
            onChange={setCategory}
            options={[
              { value: 'scene', label: 'Scene' },
              { value: 'lighting', label: 'Lighting' },
              { value: 'camera', label: 'Camera' },
              { value: 'figure', label: 'Figure' },
              { value: 'custom', label: 'Custom' },
            ]}
          />
          <div className={styles.formActions}>
            <Button size="sm" onClick={handleSave} disabled={isSaving}>
              {isSaving ? <Loader2 size={14} className={styles.spin} /> : 'Save'}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setShowForm(false)}>
              Cancel
            </Button>
          </div>
        </div>
      )}

      {isLoading && presets.length === 0 ? (
        <div className={styles.centered}>
          <Loader2 size={20} className={styles.spin} />
        </div>
      ) : presets.length === 0 ? (
        <p className={styles.empty}>No saved presets. Save the current scene to create one.</p>
      ) : (
        <ul className={styles.list}>
          {presets.map((preset) => (
            <li key={preset.id} className={styles.item}>
              <button
                type="button"
                className={styles.itemBody}
                onClick={() => void loadPreset(preset.id)}
              >
                <span className={styles.itemName}>{preset.name}</span>
                <span className={styles.itemMeta}>
                  {preset.category} · {preset.description || 'No description'}
                </span>
              </button>
              <button
                type="button"
                className={styles.deleteBtn}
                aria-label={`Delete ${preset.name}`}
                onClick={() => void deletePreset(preset.id)}
              >
                <Trash2 size={14} />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
