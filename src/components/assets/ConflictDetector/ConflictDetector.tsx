import { AlertTriangle, CheckCircle, XCircle, RefreshCw, Trash2 } from 'lucide-react';
import { useAssetFixerStore } from '../../../store';
import { Button, Card, CardContent, VStack, HStack } from '../../ui';
import styles from './ConflictDetector.module.css';

interface ConflictDetectorProps {
  onClose?: () => void;
}

export function ConflictDetector({ onClose }: ConflictDetectorProps) {
  const {
    isScanning,
    lastScanResult,
    lastFixResult,
    isFixing,
    scanConflicts,
    autoFixAll,
    clearResults,
  } = useAssetFixerStore();

  const hasConflicts = lastScanResult && lastScanResult.conflicts.length > 0;

  const handleScan = async () => {
    await scanConflicts('C:\\DAZ 3D\\Studio');
  };

  const handleAutoFix = async () => {
    await autoFixAll('C:\\DAZ 3D\\Studio', 'C:\\DAZ 3D\\Studio\\Fixed');
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'high':
        return <XCircle size={16} className={styles.severityHigh} />;
      case 'medium':
        return <AlertTriangle size={16} className={styles.severityMedium} />;
      default:
        return <AlertTriangle size={16} className={styles.severityLow} />;
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3>Asset Conflict Detector</h3>
        {onClose && (
          <Button variant="ghost" size="sm" onClick={onClose}>
            <XCircle size={16} />
          </Button>
        )}
      </div>

      <div className={styles.actions}>
        <Button
          onClick={handleScan}
          disabled={isScanning}
          icon={
            isScanning ? (
              <RefreshCw className={styles.spin} size={16} />
            ) : (
              <AlertTriangle size={16} />
            )
          }
        >
          {isScanning ? 'Scanning...' : 'Scan for Conflicts'}
        </Button>
        {hasConflicts && (
          <Button
            variant="secondary"
            onClick={handleAutoFix}
            disabled={isFixing}
            icon={isFixing ? <RefreshCw className={styles.spin} size={16} /> : <Trash2 size={16} />}
          >
            {isFixing ? 'Fixing...' : 'Auto-Fix All'}
          </Button>
        )}
        {(lastScanResult || lastFixResult) && (
          <Button variant="ghost" onClick={clearResults}>
            Clear
          </Button>
        )}
      </div>

      {lastScanResult && (
        <div className={styles.results}>
          <div className={styles.summary}>
            <span>Scanned: {lastScanResult.total_scanned} files</span>
            <span className={hasConflicts ? styles.hasConflicts : styles.noConflicts}>
              {hasConflicts ? `${lastScanResult.conflicts.length} conflicts found` : 'No conflicts'}
            </span>
          </div>

          {lastScanResult.warnings.length > 0 && (
            <div className={styles.warnings}>
              {lastScanResult.warnings.map((warning, i) => (
                <div key={i} className={styles.warningItem}>
                  <AlertTriangle size={14} />
                  <span>{warning}</span>
                </div>
              ))}
            </div>
          )}

          {hasConflicts && (
            <VStack gap="sm" className={styles.conflictList}>
              {lastScanResult.conflicts.map((conflict, i) => (
                <Card key={i} className={styles.conflictCard}>
                  <CardContent className={styles.conflictContent}>
                    <HStack gap="sm">
                      {getSeverityIcon(conflict.severity)}
                      <span className={styles.conflictName}>{conflict.name}</span>
                      <span className={styles.conflictType}>{conflict.conflict_type}</span>
                    </HStack>
                    <div className={styles.conflictFiles}>
                      {conflict.files.map((file, j) => (
                        <code key={j} className={styles.filePath}>
                          {file}
                        </code>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </VStack>
          )}
        </div>
      )}

      {lastFixResult && (
        <div className={styles.fixResults}>
          {lastFixResult.success ? (
            <div className={styles.successBox}>
              <CheckCircle size={20} />
              <span>Fixed {lastFixResult.fixed_files.length} files successfully</span>
            </div>
          ) : (
            <div className={styles.errorBox}>
              <XCircle size={20} />
              <span>Fix completed with errors</span>
            </div>
          )}
          {lastFixResult.errors.length > 0 && (
            <div className={styles.errorList}>
              {lastFixResult.errors.map((error, i) => (
                <div key={i} className={styles.errorItem}>
                  {error}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
