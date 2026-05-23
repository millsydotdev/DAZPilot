import React, { useState, useEffect, useRef } from 'react';
import {
  Search,
  Grid,
  List,
  Package,
  User,
  Scissors,
  Sparkles,
  RefreshCw,
  Star,
  Clock,
  Heart,
  Sliders,
  Filter,
  Camera,
  X,
} from 'lucide-react';
import { useAssetsStore, type AssetFilter } from '../../store';
import { Button, Input, VStack, HStack, Text } from '../ui';
import styles from './AssetBrowser.module.css';

type ViewTab = 'categories' | 'favorites' | 'recent';

const categories = [
  { id: 'all', label: 'All Assets' },
  { id: 'figures', label: 'Figures', icon: User },
  { id: 'clothing', label: 'Clothing', icon: Package },
  { id: 'hair', label: 'Hair', icon: Sparkles },
  { id: 'hair', label: 'Hair', icon: Sparkles },
  { id: 'poses', label: 'Poses', icon: Scissors },
  { id: 'materials', label: 'Materials', icon: Sliders },
  { id: 'environments', label: 'Environments', icon: Package },
  { id: 'lights', label: 'Lights', icon: Sparkles },
  { id: 'cameras', label: 'Cameras', icon: Camera },
  { id: 'animations', label: 'Animations', icon: Clock },
];

const viewTabs = [
  { id: 'categories', label: 'Browse', icon: Package },
  { id: 'favorites', label: 'Favorites', icon: Star },
  { id: 'recent', label: 'Recent', icon: Clock },
] as const;

const getTypeIcon = (category: string) => {
  const icons: Record<string, typeof Package> = {
    figures: User,
    clothing: Package,
    hair: Sparkles,
    poses: Scissors,
    materials: Sliders,
    environments: Package,
    lights: Sparkles,
    cameras: Camera,
    animations: Clock,
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
    toggleFavourite,
    sortBy,
    setSortBy,
    sortDirection,
    setSortDirection,
    advancedFilters,
    setAdvancedFilters,
  } = useAssetsStore();

  const [viewTab, setViewTab] = useState<ViewTab>('categories');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false);
  const loadPathsRef = useRef(loadContentPaths);

  useEffect(() => {
    loadPathsRef.current();
  }, []);

  // Apply filters and sorting
  const displayFiles = React.useMemo(() => {
    let filteredFiles = [...files];

    // Apply view tab filters
    if (viewTab === 'favorites') {
      filteredFiles = filteredFiles.filter((f) => f.isFavourite);
    } else if (viewTab === 'recent') {
      filteredFiles = filteredFiles.sort((a, b) => b.modified - a.modified).slice(0, 20);
    } else {
      // Apply category filter
      if (filter !== 'all') {
        filteredFiles = filteredFiles.filter((f) => f.type === filter);
      }
    }

    // Apply search filter
    if (search) {
      const searchLower = search.toLowerCase();
      filteredFiles = filteredFiles.filter((f) => {
        const tags = f.metadata?.tags as string[] | undefined;
        return (
          f.name.toLowerCase().includes(searchLower) ||
          tags?.some((tag: string) => tag.toLowerCase().includes(searchLower)) ||
          false
        );
      });
    }

    // Apply advanced filters
    if (advancedFilters) {
      const ft = advancedFilters.fileTypes;
      if (ft && ft.length > 0) {
        filteredFiles = filteredFiles.filter((f) => ft.includes(f.type));
      }

      if (advancedFilters.dateRange) {
        const startDate = advancedFilters.dateRange.start || 0;
        const endDate = advancedFilters.dateRange.end || Number.MAX_SAFE_INTEGER;
        filteredFiles = filteredFiles.filter(
          (f) => f.modified >= startDate && f.modified <= endDate
        );
      }

      const sr = advancedFilters.sizeRange;
      if (sr) {
        const minSize = sr.min || 0;
        const maxSize = sr.max || Infinity;
        filteredFiles = filteredFiles.filter((f) => f.size >= minSize && f.size <= maxSize);
      }
    }

    // Apply sorting
    if (sortBy && sortBy !== 'none') {
      filteredFiles.sort((a, b) => {
        let comparison = 0;
        const dir = sortDirection === 'asc' ? 1 : -1;

        switch (sortBy) {
          case 'name':
            comparison = a.name.localeCompare(b.name);
            break;
          case 'size':
            comparison = a.size - b.size;
            break;
          case 'date':
            comparison = a.modified - b.modified;
            break;
          case 'type':
            comparison = a.type.localeCompare(b.type);
            break;
          default:
            comparison = 0;
        }

        return comparison * dir;
      });
    }

    return filteredFiles;
  }, [files, viewTab, filter, search, sortBy, sortDirection, advancedFilters]);

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
            aria-label="Search assets"
          />
        </HStack>

        <HStack gap="xs" className={styles.controls}>
          {viewTabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                className={`${styles.viewButton} ${viewTab === tab.id ? styles.active : ''}`}
                onClick={() => setViewTab(tab.id as ViewTab)}
                aria-label={tab.label}
                title={tab.label}
              >
                <Icon size={16} />
              </button>
            );
          })}
          <div className={styles.divider} />
          <button
            className={`${styles.viewButton} ${viewMode === 'grid' ? styles.active : ''}`}
            onClick={() => setViewMode('grid')}
            aria-label="Grid view"
          >
            <Grid size={16} />
          </button>
          <button
            className={`${styles.viewButton} ${viewMode === 'list' ? styles.active : ''}`}
            onClick={() => setViewMode('list')}
            aria-label="List view"
          >
            <List size={16} />
          </button>
          <Button
            variant="outline"
            onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
            className={`${showAdvancedFilters ? styles.active : ''}`}
          >
            <Filter size={16} />
            Filter
          </Button>
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
        {viewTab === 'categories' && (
          <div className={styles.sidebar}>
            <VStack gap="xs">
              {categories.map((cat) => (
                <button
                  key={cat.id}
                  className={`${styles.filterButton} ${filter === cat.id ? styles.active : ''}`}
                  onClick={() => setFilter(cat.id as AssetFilter)}
                  aria-label={cat.label}
                >
                  {cat.label}
                </button>
              ))}
            </VStack>
          </div>
        )}

        {isLoading ? (
          <div className={styles.loading}>
            <RefreshCw className={styles.spin} size={24} />
            <span>Loading assets...</span>
          </div>
        ) : displayFiles.length === 0 ? (
          <div className={styles.emptyState}>
            <Package size={48} />
            <p>
              {viewTab === 'favorites'
                ? 'No favorites yet. Star assets to add them here.'
                : viewTab === 'recent'
                  ? 'No recent assets.'
                  : 'Click "Scan Library" to discover assets'}
            </p>
          </div>
        ) : (
          <>
            <div className={viewMode === 'grid' ? styles.grid : styles.list}>
              {displayFiles.map((file) => (
                <div
                  key={file.id}
                  className={styles.assetCard}
                  role="listitem"
                  aria-label={file.name}
                >
                  <button
                    className={styles.favButton}
                    onClick={() => toggleFavourite(file.path)}
                    aria-label={file.isFavourite ? 'Remove from favorites' : 'Add to favorites'}
                  >
                    {file.isFavourite ? (
                      <Heart size={14} fill="currentColor" />
                    ) : (
                      <Heart size={14} />
                    )}
                  </button>
                  <div className={styles.assetIcon}>{getTypeIcon(file.type)}</div>
                  <div className={styles.assetName}>{file.name}</div>
                  <div className={styles.assetMeta}>{formatSize(file.size)}</div>
                </div>
              ))}
            </div>

            {/* Advanced Filters Panel */}
            {showAdvancedFilters && (
              <div className={styles.advancedFilters}>
                <div className={styles.advancedFiltersHeader}>
                  <h3>Advanced Filters</h3>
                  <button onClick={() => setShowAdvancedFilters(false)}>
                    <X size={16} />
                  </button>
                </div>
                <div className={styles.advancedFiltersContent}>
                  <VStack gap="sm">
                    <HStack>
                      <Text size="xs">Sort By:</Text>
                      <select
                        value={sortBy || 'none'}
                        onChange={(e) => setSortBy(e.target.value)}
                        className={styles.select}
                      >
                        <option value="none">None</option>
                        <option value="name">Name</option>
                        <option value="size">Size</option>
                        <option value="date">Date Modified</option>
                        <option value="type">Type</option>
                      </select>
                      <Text size="xs" className={styles.mx2}>
                        Order:
                      </Text>
                      <select
                        value={sortDirection || 'asc'}
                        onChange={(e) => setSortDirection(e.target.value as 'asc' | 'desc')}
                        className={styles.select}
                      >
                        <option value="asc">Ascending</option>
                        <option value="desc">Descending</option>
                      </select>
                    </HStack>

                    <HStack>
                      <Text size="xs">File Types:</Text>
                      <VStack gap="xs">
                        {[
                          'figures',
                          'clothing',
                          'hair',
                          'poses',
                          'materials',
                          'environments',
                          'lights',
                          'cameras',
                          'animations',
                        ].map((type) => (
                          <label key={type} className={styles.checkboxLabel}>
                            <input
                              type="checkbox"
                              checked={advancedFilters.fileTypes?.includes(type) || false}
                              onChange={(e) => {
                                const fileTypes = advancedFilters.fileTypes || [];
                                const updated = e.target.checked
                                  ? [...fileTypes, type]
                                  : fileTypes.filter((t: string) => t !== type);
                                setAdvancedFilters({ ...advancedFilters, fileTypes: updated });
                              }}
                            />
                            <span>{type.charAt(0).toUpperCase() + type.slice(1)}</span>
                          </label>
                        ))}
                      </VStack>
                    </HStack>

                    <HStack>
                      <VStack gap="xs">
                        <Text size="xs">Date Range:</Text>
                        <HStack>
                          <Input
                            type="date"
                            placeholder="Start Date"
                            value={advancedFilters.dateRange?.start?.toString().split('T')[0] || ''}
                            onChange={(e) => {
                              const date = e.target.value ? new Date(e.target.value) : null;
                              setAdvancedFilters({
                                ...advancedFilters,
                                dateRange: {
                                  start: date ? date.getTime() : 0,
                                  end: advancedFilters.dateRange?.end ?? null,
                                },
                              });
                            }}
                          />
                          <span className={styles.mx2}>to</span>
                          <Input
                            type="date"
                            placeholder="End Date"
                            value={advancedFilters.dateRange?.end?.toString().split('T')[0] || ''}
                            onChange={(e) => {
                              const date = e.target.value ? new Date(e.target.value) : null;
                              setAdvancedFilters({
                                ...advancedFilters,
                                dateRange: {
                                  start: advancedFilters.dateRange?.start ?? null,
                                  end: date ? date.getTime() : Date.now(),
                                },
                              });
                            }}
                          />
                        </HStack>
                      </VStack>
                    </HStack>

                    <HStack>
                      <VStack gap="xs">
                        <Text size="xs">File Size (MB):</Text>
                        <HStack>
                          <Input
                            type="number"
                            placeholder="Min"
                            value={
                              advancedFilters.sizeRange?.min != null
                                ? advancedFilters.sizeRange.min / (1024 * 1024)
                                : ''
                            }
                            onChange={(e) => {
                              const value = parseFloat(e.target.value) || 0;
                              setAdvancedFilters({
                                ...advancedFilters,
                                sizeRange: {
                                  min: value * 1024 * 1024,
                                  max: advancedFilters.sizeRange?.max ?? null,
                                },
                              });
                            }}
                          />
                          <span className={styles.mx2}>to</span>
                          <Input
                            type="number"
                            placeholder="Max"
                            value={
                              advancedFilters.sizeRange?.max != null
                                ? advancedFilters.sizeRange.max / (1024 * 1024)
                                : ''
                            }
                            onChange={(e) => {
                              const value = parseFloat(e.target.value) || 0;
                              setAdvancedFilters({
                                ...advancedFilters,
                                sizeRange: {
                                  min: advancedFilters.sizeRange?.min ?? null,
                                  max: value === 0 ? null : value * 1024 * 1024,
                                },
                              });
                            }}
                          />
                        </HStack>
                      </VStack>
                    </HStack>

                    <HStack justify="end">
                      <Button variant="outline" onClick={() => setShowAdvancedFilters(false)}>
                        Cancel
                      </Button>
                      <Button onClick={() => setShowAdvancedFilters(false)}>Apply Filters</Button>
                    </HStack>
                  </VStack>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
