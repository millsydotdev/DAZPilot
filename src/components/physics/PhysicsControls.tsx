import { useState, useEffect } from 'react';
import { useViewportStore } from '../../store';
import { Play, Pause, RefreshCw, Settings, Info } from 'lucide-react';
import { Button, Input, VStack, HStack, Text, Separator } from '../ui';
import styles from './PhysicsControls.module.css';

export function PhysicsControls() {
  const { selectedFigure } = useViewportStore();

  const [simulationActive, setSimulationActive] = useState(false);
  const [simulationProgress, setSimulationProgress] = useState(0);
  const [currentFrame, setCurrentFrame] = useState(0);
  const totalFrames = 250;
  const [simulationSettings, setSimulationSettings] = useState({
    stiffness: 12.0,
    damping: 4.0,
    mass: 0.5,
    quality: 'medium',
    selfCollision: true,
    collisionOffset: 0.5,
    friction: 0.3,
  });

  const [isConfiguring, setIsConfiguring] = useState(false);
  const [simulationLogs, setSimulationLogs] = useState<string[]>([]);

  // Initialize simulation from viewport state or defaults
  useEffect(() => {
    // In a full implementation, we'd load physics state from the store
    // For now, we'll use local state
  }, []);

  const addLog = (message: string) => {
    setSimulationLogs((prev) => [
      `[${new Date().toLocaleTimeString()}] ${message}`,
      ...prev.slice(0, 9), // Keep last 10 logs
    ]);
  };

  const startSimulation = async () => {
    if (!selectedFigure) {
      addLog('Please select a figure first');
      return;
    }

    try {
      setSimulationActive(true);
      setSimulationProgress(0);
      setCurrentFrame(0);
      addLog(`Starting dForce simulation for ${selectedFigure}`);

      // In a real implementation, we'd call the physics simulation
      // For demo purposes, we'll simulate progress
      const interval = setInterval(() => {
        setSimulationProgress((prev) => Math.min(100, prev + 2));
        setCurrentFrame((prev) => Math.min(totalFrames, prev + 5));

        if (simulationProgress >= 100) {
          clearInterval(interval);
          setSimulationActive(false);
          addLog('Simulation completed successfully');
        }
      }, 100);
    } catch (error) {
      addLog(`Simulation failed: ${(error as Error).message}`);
      setSimulationActive(false);
    }
  };

  const stopSimulation = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('stop_simulation');
      setSimulationActive(false);
      addLog('Simulation stopped by user');
    } catch (error) {
      addLog(`Failed to stop simulation: ${(error as Error).message}`);
    }
  };

  const resetSimulation = async () => {
    setSimulationActive(false);
    setSimulationProgress(0);
    setCurrentFrame(0);
    setSimulationLogs([]);
    addLog('Simulation reset');
  };

  const handleSettingsChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const target = e.target as HTMLInputElement;
    const { name, value, type } = target;
    const checked = type === 'checkbox' ? target.checked : undefined;
    setSimulationSettings((prev) => {
      if (type === 'checkbox') {
        return { ...prev, [name]: checked };
      } else if (name === 'stiffness' || name === 'damping' || name === 'mass') {
        return { ...prev, [name]: parseFloat(value) };
      } else {
        return { ...prev, [name]: value };
      }
    });
  };

  const applySettings = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_phy_modifier_params', {
        node_id: selectedFigure || '',
        stiffness: simulationSettings.stiffness.toString(),
        damping: simulationSettings.damping.toString(),
        mass: simulationSettings.mass.toString(),
      });
      addLog('Physics settings applied');
      setIsConfiguring(false);
    } catch (error) {
      addLog(`Failed to apply settings: ${(error as Error).message}`);
    }
  };

  return (
    <div className={styles.physicsControlsContainer}>
      <div className={styles.header}>
        <h2>Physics Simulation Controls</h2>
        <div className={styles.headerActions}>
          <Button
            icon={<Info size={16} />}
            variant="secondary"
            size="sm"
            onClick={() => setIsConfiguring(true)}
          >
            Settings
          </Button>
        </div>
      </div>

      <Separator />

      {/* Simulation Status */}
      <div className={styles.statusSection}>
        <div className={styles.statusRow}>
          <Text variant="small" className={styles.statusLabel}>
            Active Figure:
          </Text>
          <Text variant="small" className={styles.statusValue}>
            {selectedFigure || 'None selected'}
          </Text>
        </div>

        <div className={styles.statusRow}>
          <Text variant="small" className={styles.statusLabel}>
            Simulation Status:
          </Text>
          <span className={`${styles.statusIndicator} ${simulationActive ? styles.active : ''}`}>
            {simulationActive ? 'Running' : 'Idle'}
          </span>
        </div>

        <div className={styles.statusRow}>
          <Text variant="small" className={styles.statusLabel}>
            Progress:
          </Text>
          <div className={styles.progressBar}>
            <div className={styles.progressFill} style={{ width: `${simulationProgress}%` }} />
          </div>
          <Text variant="small" className={styles.progressText}>
            {simulationProgress}% ({currentFrame}/${totalFrames} frames)
          </Text>
        </div>
      </div>

      <Separator />

      {/* Simulation Controls */}
      {simulationActive ? (
        <div className={styles.controlsSection}>
          <HStack justify="center" gap="sm">
            <Button variant="secondary" onClick={stopSimulation} className={styles.stopButton}>
              <Pause size={16} />
              Stop
            </Button>
            <Button variant="secondary" onClick={resetSimulation} className={styles.resetButton}>
              <RefreshCw size={16} />
              Reset
            </Button>
          </HStack>
        </div>
      ) : (
        <div className={styles.controlsSection}>
          <HStack justify="center" gap="sm">
            <Button
              onClick={startSimulation}
              disabled={!selectedFigure}
              className={styles.startButton}
            >
              <Play size={16} />
              Start Simulation
            </Button>
            <Button variant="secondary" onClick={() => setIsConfiguring(true)}>
              <Settings size={16} />
              Configure
            </Button>
          </HStack>
        </div>
      )}

      <Separator />

      {/* Configuration Panel */}
      {isConfiguring && (
        <div className={styles.configPanel}>
          <h3>Physics Properties</h3>
          <VStack gap="sm">
            <HStack>
              <Text variant="small" style={{ width: '80px' }}>
                Stiffness:
              </Text>
              <Input
                type="number"
                placeholder="12.0"
                value={simulationSettings.stiffness}
                onChange={handleSettingsChange}
                name="stiffness"
              />
              <Text variant="small" className={styles.unit}>
                {' '}
                (0.1 - 50.0)
              </Text>
            </HStack>

            <HStack>
              <Text variant="small" style={{ width: '80px' }}>
                Damping:
              </Text>
              <Input
                type="number"
                placeholder="4.0"
                value={simulationSettings.damping}
                onChange={handleSettingsChange}
                name="damping"
              />
              <Text variant="small" className={styles.unit}>
                {' '}
                (0.1 - 20.0)
              </Text>
            </HStack>

            <HStack>
              <Text variant="small" style={{ width: '80px' }}>
                Mass:
              </Text>
              <Input
                type="number"
                placeholder="0.5"
                value={simulationSettings.mass}
                onChange={handleSettingsChange}
                name="mass"
              />
              <Text variant="small" className={styles.unit}>
                {' '}
                (0.1 - 5.0)
              </Text>
            </HStack>

            <HStack>
              <Text variant="small" style={{ width: '80px' }}>
                Quality:
              </Text>
              <select
                value={simulationSettings.quality}
                onChange={(e) =>
                  setSimulationSettings((prev) => ({ ...prev, quality: e.target.value }))
                }
                className={styles.select}
              >
                <option value="preview">Preview</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="ultra">Ultra</option>
              </select>
            </HStack>

            <HStack>
              <Text variant="small" style={{ width: '80px' }}>
                Self Collision:
              </Text>
              <input
                type="checkbox"
                checked={simulationSettings.selfCollision}
                onChange={(e) =>
                  setSimulationSettings((prev) => ({ ...prev, selfCollision: e.target.checked }))
                }
                name="selfCollision"
              />
            </HStack>
          </VStack>

          <HStack justify="end" gap="sm">
            <Button variant="secondary" onClick={() => setIsConfiguring(false)}>
              Cancel
            </Button>
            <Button onClick={applySettings}>Apply Settings</Button>
          </HStack>
        </div>
      )}

      <Separator />

      {/* Simulation Logs */}
      {simulationLogs.length > 0 && (
        <div className={styles.logsSection}>
          <h3>Simulation Logs</h3>
          <div className={styles.logsContainer}>
            {simulationLogs.map((log, index) => (
              <div key={index} className={styles.logEntry}>
                <Text variant="small" className={styles.logText}>
                  {log}
                </Text>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
