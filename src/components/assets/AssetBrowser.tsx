import { useState, useEffect, useRef } from 'react';
import { Search, Grid, List, Package, User, Scissors, Sparkles, RefreshCw } from 'lucide-react';
import { useAssetsStore, type AssetFilter } from '../../store';
import { Button, Input, VStack, HStack } from '../ui';
import styles from './AssetBrowser.module.css';

const categories = [
  { id: 'all', label: 'All Assets' },
  { id: 'figures', label: 'Figures', icon: User },
  { id: 'clothing', label: 'Clothing', icon: Package },
  { id: 'hair', label: 'Hair', icon: Sparkles },
  { id: 'poses', label: 'Poses', icon: Scissors },
];

const getTypeIcon = (category: string) => {
  const icons: Record<string, typeof User> = {
    figures: User,
    clothing: Package,
    hair: Sparkles,
    poses: Scissors,
  };
  const Icon = icons[category] || Package;
  return <Icon size={20} />;
};

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

export default function AssetBrowser() {
  const {
    files,
    contentPaths,
    filter,
    search,
    isLoading,
    isScanning,
    loadContentPaths,
    scanLibrary,
    setFilter,
    setSearch,
  } = useAssetsStore();

  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const loadPathsRef = useRef(loadContentPaths);

  useEffect(() => {
    loadPathsRef.current();
  }, []);

  const filteredFiles = files.filter((f) => {
    const matchesFilter = filter === 'all' || f.type === filter;
    const matchesSearch = !search || f.name.toLowerCase().includes(search.toLowerCase());
    return matchesFilter && matchesSearch;
  });

  const handleScan = () => {
    scanLibrary();
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <HStack gap="sm" className={styles.searchBox}>
          <Input
            className={styles.searchInput}
            placeholder="Search assets..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            icon={<Search size={16} />}
          />
        </HStack>

        <HStack gap="xs" className={styles.controls}>
          <button
            className={`${styles.viewButton} ${viewMode === 'grid' ? styles.active : ''}`}
            onClick={() => setViewMode('grid')}
          >
            <Grid size={16} />
          </button>
          <button
            className={`${styles.viewButton} ${viewMode === 'list' ? styles.active : ''}`}
            onClick={() => setViewMode('list')}
          >
            <List size={16} />
          </button>
          <Button
            onClick={handleScan}
            disabled={isScanning}
            icon={
              isScanning ? <RefreshCw className={styles.spin} size={16} /> : <RefreshCw size={16} />
            }
          >
            {isScanning ? 'Scanning...' : 'Scan Library'}
          </Button>
        </HStack>
      </div>

      <div className={styles.contentPaths}>
        {contentPaths.map((path) => (
          <button key={path.id} className={styles.pathChip}>
            <span>{path.name}</span>
          </button>
        ))}
      </div>

      <div className={styles.content}>
        <div className={styles.sidebar}>
          <VStack gap="xs">
            {categories.map((cat) => (
              <button
                key={cat.id}
                className={`${styles.filterButton} ${filter === cat.id ? styles.active : ''}`}
                onClick={() => setFilter(cat.id as AssetFilter)}
              >
                {cat.label}
              </button>
            ))}
          </VStack>
        </div>

        {isLoading ? (
          <div className={styles.loading}>
            <RefreshCw className={styles.spin} size={24} />
            <span>Loading assets...</span>
          </div>
        ) : filteredFiles.length === 0 ? (
          <div className={styles.emptyState}>
            <Package size={48} />
            <p>Click &quot;Scan Library&quot; to discover assets</p>
            <Button onClick={handleScan} icon={<RefreshCw size={16} />}>
              Scan Now
            </Button>
          </div>
        ) : (
          <div className={styles.assetGrid}>
            {filteredFiles.map((file) => (
              <div key={file.id} className={styles.assetCard}>
                <div className={styles.assetIcon}>{getTypeIcon(file.type)}</div>
                <div className={styles.assetName}>{file.name}</div>
                <div className={styles.assetMeta}>{formatSize(file.size)}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
