import { JSX, ComponentChildren } from 'preact';
import { TreeView, type TreeNode, type TreeViewProps } from './TreeView';

export type SortDirection = 'asc' | 'desc';

export interface SortOption<K extends string = string> {
  key: K;
  label: string;
  /** Direction applied on first click; defaults to 'asc' for 'title', 'desc' for time-based keys. */
  defaultDirection?: SortDirection;
}

export interface SortState<K extends string = string> {
  key: K;
  direction: SortDirection;
}

export interface FilterOption<K extends string = string> {
  key: K;
  label: string;
  /** Optional icon rendered before the label. */
  icon?: ComponentChildren;
  /** Count shown after the label (e.g. number of matching items). */
  count?: number;
  /** Active-state accent color (CSS value). Falls back to --accent-color. */
  activeColor?: string;
}

export interface FileTreeProps<T = unknown> {
  nodes: TreeNode<T>[];
  selectedId?: string;
  onSelect?: (node: TreeNode<T>) => void;
  onContextMenu?: (node: TreeNode<T>, event: MouseEvent) => void;
  defaultExpanded?: string[];
  expanded?: Set<string>;
  onToggle?: (id: string) => void;
  loading?: boolean;
  emptyMessage?: string;
  class?: string;
  sortOptions?: SortOption[];
  sortState?: SortState;
  onSortChange?: (state: SortState) => void;
  /** Toggle-style category filter buttons rendered above the tree. */
  filterOptions?: FilterOption[];
  /** Currently active filter key, or null/undefined for "show all". */
  activeFilter?: string | null;
  /** Called when a filter button is clicked. Receives the key, or null to clear. */
  onFilterChange?: (key: string | null) => void;
}

function SortHeader({
  options,
  state,
  onChange,
}: {
  options: SortOption[];
  state: SortState;
  onChange: (state: SortState) => void;
}): JSX.Element {
  return (
    <div class="file-tree__sort-header">
      {options.map((opt) => {
        const active = state.key === opt.key;
        const nextDir: SortDirection = active
          ? state.direction === 'asc'
            ? 'desc'
            : 'asc'
          : (opt.defaultDirection ?? 'asc');
        return (
          <button
            key={opt.key}
            class={`file-tree__sort-btn${active ? ' file-tree__sort-btn--active' : ''}`}
            onClick={() => onChange({ key: opt.key, direction: nextDir })}
            title={`Sort by ${opt.label}`}
          >
            {opt.label}
            {active && (state.direction === 'asc' ? ' ↑' : ' ↓')}
          </button>
        );
      })}
    </div>
  );
}

function FilterHeader({
  options,
  activeKey,
  onChange,
}: {
  options: FilterOption[];
  activeKey: string | null | undefined;
  onChange: (key: string | null) => void;
}): JSX.Element {
  return (
    <div class="file-tree__filter-header">
      {options.map((opt) => {
        if (opt.count != null && opt.count <= 0) return null;
        const active = activeKey === opt.key;
        const style = active && opt.activeColor
          ? { borderColor: opt.activeColor, color: opt.activeColor }
          : undefined;
        return (
          <button
            key={opt.key}
            class={`file-tree__filter-btn${active ? ' file-tree__filter-btn--active' : ''}`}
            style={style}
            onClick={() => onChange(active ? null : opt.key)}
            title={active ? 'Show all' : `Show only ${opt.label}`}
          >
            {opt.icon && <span class="file-tree__filter-icon">{opt.icon}</span>}
            <span>{opt.label}{opt.count != null ? ` (${opt.count})` : ''}</span>
          </button>
        );
      })}
    </div>
  );
}

/**
 * Shared file tree shell for viewer tools.
 *
 * This wraps `TreeView` with common loading/empty states so viewers can keep
 * their own filtering/data logic while sharing a consistent tree container.
 */
export function FileTree<T = unknown>({
  nodes,
  selectedId,
  onSelect,
  onContextMenu,
  defaultExpanded = [],
  expanded,
  onToggle,
  loading = false,
  emptyMessage = 'No files found',
  class: className = '',
  sortOptions,
  sortState,
  onSortChange,
  filterOptions,
  activeFilter,
  onFilterChange,
}: FileTreeProps<T>): JSX.Element {
  const treeProps: TreeViewProps<T> = {
    nodes,
    selectedId,
    onSelect,
    onContextMenu,
    defaultExpanded,
    expanded,
    onToggle,
  };

  return (
    <div class={`file-tree ${className}`.trim()}>
      {filterOptions && filterOptions.length > 0 && onFilterChange && (
        <FilterHeader options={filterOptions} activeKey={activeFilter} onChange={onFilterChange} />
      )}
      {sortOptions && sortOptions.length > 0 && sortState && onSortChange && (
        <SortHeader options={sortOptions} state={sortState} onChange={onSortChange} />
      )}
      {loading ? (
        <div class="file-tree__loading">Loading...</div>
      ) : nodes.length === 0 ? (
        <div class="file-tree__empty">{emptyMessage}</div>
      ) : (
        <TreeView<T> {...treeProps} />
      )}
    </div>
  );
}

