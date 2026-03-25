import { JSX, ComponentChildren } from 'preact';
import { useState, useCallback, useRef, useEffect } from 'preact/hooks';

export interface TreeNode<T = unknown> {
  id: string;
  label: string;
  icon?: ComponentChildren | 'folder' | 'file' | 'doc';
  children?: TreeNode<T>[];
  data?: T;
  /** Optional tooltip content shown on hover */
  tooltip?: ComponentChildren;
  /** Optional badge text/count shown on the right side */
  badge?: string | number;
}

export interface TreeViewProps<T = unknown> {
  nodes: TreeNode<T>[];
  selectedId?: string;
  onSelect?: (node: TreeNode<T>) => void;
  onContextMenu?: (node: TreeNode<T>, event: MouseEvent) => void;
  defaultExpanded?: string[];
  /** Controlled expanded state — if provided, TreeView won't manage its own expanded set */
  expanded?: Set<string>;
  /** Callback for controlled expansion toggle */
  onToggle?: (id: string) => void;
}

export function TreeView<T = unknown>({ nodes, selectedId, onSelect, onContextMenu, defaultExpanded = [], expanded: controlledExpanded, onToggle: controlledOnToggle }: TreeViewProps<T>): JSX.Element {
  const [internalExpanded, setInternalExpanded] = useState<Set<string>>(new Set(defaultExpanded));

  const expanded = controlledExpanded ?? internalExpanded;

  const toggleExpanded = useCallback((id: string) => {
    if (controlledOnToggle) {
      controlledOnToggle(id);
    } else {
      setInternalExpanded(prev => {
        const next = new Set(prev);
        if (next.has(id)) {
          next.delete(id);
        } else {
          next.add(id);
        }
        return next;
      });
    }
  }, [controlledOnToggle]);

  return (
    <div class="tree-view">
      {nodes.map(node => (
        <TreeItem
          key={node.id}
          node={node}
          selectedId={selectedId}
          expanded={expanded}
          onToggle={toggleExpanded}
          onSelect={onSelect}
          onContextMenu={onContextMenu}
          depth={0}
        />
      ))}
    </div>
  );
}

interface TreeItemProps<T = unknown> {
  node: TreeNode<T>;
  selectedId?: string;
  expanded: Set<string>;
  onToggle: (id: string) => void;
  onSelect?: (node: TreeNode<T>) => void;
  onContextMenu?: (node: TreeNode<T>, event: MouseEvent) => void;
  depth: number;
}

function TreeItem<T = unknown>({ node, selectedId, expanded, onToggle, onSelect, onContextMenu, depth }: TreeItemProps<T>): JSX.Element {
  const hasChildren = node.children && node.children.length > 0;
  const isExpanded = expanded.has(node.id);
  const isSelected = node.id === selectedId;
  const rowRef = useRef<HTMLDivElement>(null);
  const [tooltipVisible, setTooltipVisible] = useState(false);
  const [tooltipPos, setTooltipPos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const tooltipTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleClick = () => {
    if (hasChildren) {
      onToggle(node.id);
    }
    onSelect?.(node);
  };

  const handleContextMenu = (e: JSX.TargetedMouseEvent<HTMLDivElement>) => {
    if (onContextMenu) {
      e.preventDefault();
      onContextMenu(node, e as unknown as MouseEvent);
    }
  };

  const handleToggle = (e: Event) => {
    e.stopPropagation();
    onToggle(node.id);
  };

  // Tooltip hover handlers
  const handleMouseEnter = (e: JSX.TargetedMouseEvent<HTMLDivElement>) => {
    if (!node.tooltip) return;
    const target = e.currentTarget as HTMLElement;
    tooltipTimer.current = setTimeout(() => {
      const rect = target.getBoundingClientRect();
      setTooltipPos({ x: rect.right + 8, y: rect.top });
      setTooltipVisible(true);
    }, 400);
  };

  const handleMouseLeave = () => {
    if (tooltipTimer.current) {
      clearTimeout(tooltipTimer.current);
      tooltipTimer.current = null;
    }
    setTooltipVisible(false);
  };

  // Resolve icon: string shorthand or custom ComponentChildren
  const iconContent = (() => {
    const icon = node.icon;
    if (!icon) {
      return hasChildren ? <FolderIcon /> : <FileIcon />;
    }
    if (icon === 'doc') return <DocIcon />;
    if (icon === 'folder') return <FolderIcon />;
    if (icon === 'file') return <FileIcon />;
    return icon; // ComponentChildren
  })();

  const iconClass = typeof node.icon === 'string' ? node.icon : (hasChildren ? 'folder' : 'file');

  return (
    <div class="tree-item" style={{ paddingLeft: `${depth * 8}px` }}>
      <div
        ref={rowRef}
        class={`tree-item-row ${isSelected ? 'selected' : ''}`}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <span
          class={`tree-toggle ${isExpanded ? 'expanded' : ''} ${!hasChildren ? 'empty' : ''}`}
          onClick={hasChildren ? handleToggle : undefined}
        >
          <ChevronIcon />
        </span>
        <span class={`tree-icon ${iconClass}`}>
          {iconContent}
        </span>
        <span class="tree-label">{node.label}</span>
        {node.badge != null && <span class="tree-badge">{node.badge}</span>}
        {!node.badge && hasChildren && <span class="tree-badge">{node.children!.length}</span>}
      </div>
      {tooltipVisible && node.tooltip && (
        <TreeTooltip x={tooltipPos.x} y={tooltipPos.y}>
          {node.tooltip}
        </TreeTooltip>
      )}
      {hasChildren && isExpanded && (
        <div class="tree-children">
          {node.children!.map(child => (
            <TreeItem
              key={child.id}
              node={child}
              selectedId={selectedId}
              expanded={expanded}
              onToggle={onToggle}
              onSelect={onSelect}
              onContextMenu={onContextMenu}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ── Tooltip ──

interface TreeTooltipProps {
  x: number;
  y: number;
  children: ComponentChildren;
}

function TreeTooltip({ x, y, children }: TreeTooltipProps): JSX.Element {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    // Clamp tooltip to viewport
    const rect = el.getBoundingClientRect();
    if (rect.right > window.innerWidth) {
      el.style.left = `${x - rect.width - 16}px`;
    }
    if (rect.bottom > window.innerHeight) {
      el.style.top = `${window.innerHeight - rect.height - 8}px`;
    }
  }, [x, y]);

  return (
    <div
      ref={ref}
      class="tree-tooltip"
      style={{ position: 'fixed', left: `${x}px`, top: `${y}px` }}
    >
      {children}
    </div>
  );
}

function ChevronIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M10 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V8a2 2 0 00-2-2h-8l-2-2z" />
    </svg>
  );
}

function FileIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" />
      <polyline points="14 2 14 8 20 8" />
    </svg>
  );
}

function DocIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <line x1="10" y1="9" x2="8" y2="9" />
    </svg>
  );
}
