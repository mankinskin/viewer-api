import { JSX } from 'preact';
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

