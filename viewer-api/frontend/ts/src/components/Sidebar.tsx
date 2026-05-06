import { JSX, ComponentChildren } from 'preact';
import { useState, useCallback, useRef } from 'preact/hooks';
import { ResizeHandle } from './ResizeHandle';

export interface SidebarProps {
  /** Header title */
  title: string;
  /** Optional badge/count to display in header */
  badge?: string | number;
  /** Main content of the sidebar */
  children: ComponentChildren;
  /** Optional class name */
  class?: string;
  /** Optional loading state */
  loading?: boolean;
  /** Optional empty state message */
  emptyMessage?: string;
  /** Whether content is empty */
  isEmpty?: boolean;
  /** Enable collapse/expand toggle button */
  collapsible?: boolean;
  /** Controlled collapsed state (if provided, sidebar is controlled) */
  collapsed?: boolean;
  /** Callback when collapsed state changes */
  onCollapsedChange?: (collapsed: boolean) => void;
  /** Enable drag-to-resize handle on the right edge */
  resizable?: boolean;
  /** Enable/disable specific resize edges. By default, right edge is enabled. */
  resizeEdges?: {
    left?: boolean;
    right?: boolean;
  };
  /** Initial width in px (default: 260) */
  initialWidth?: number;
  /** Min width in px (default: 0) */
  minWidth?: number;
  /** Max width in px (default: unlimited) */
  maxWidth?: number;
  /** Mobile overlay mode: when true, sidebar is shown as overlay with backdrop */
  mobileOpen?: boolean;
  /** Callback to close mobile overlay */
  onMobileClose?: () => void;
}

/**
 * Common sidebar shell component for viewer tools.
 * 
 * Provides a consistent sidebar layout with:
 * - Header row with title and optional badge
 * - Content area with optional loading/empty states
 * - Optional collapse/expand toggle
 * - Optional drag-to-resize handle
 */
export function Sidebar({ 
  title, 
  badge, 
  children, 
  class: className = '',
  loading = false,
  emptyMessage = 'No items found',
  isEmpty = false,
  collapsible = false,
  collapsed: controlledCollapsed,
  onCollapsedChange,
  resizable = true,
  resizeEdges,
  initialWidth = 260,
  minWidth = 0,
  maxWidth,
  mobileOpen,
  onMobileClose,
}: SidebarProps): JSX.Element {
  const [internalCollapsed, setInternalCollapsed] = useState(false);
  const [width, setWidth] = useState(initialWidth);
  const sidebarRef = useRef<HTMLElement | null>(null);
  const liveWidthRef = useRef(initialWidth);

  const isCollapsed = controlledCollapsed ?? internalCollapsed;

  const toggleCollapse = useCallback(() => {
    const next = !isCollapsed;
    if (onCollapsedChange) {
      onCollapsedChange(next);
    } else {
      setInternalCollapsed(next);
    }
  }, [isCollapsed, onCollapsedChange]);

  const clampWidth = useCallback((value: number) => {
    let next = value;
    if (minWidth !== undefined) next = Math.max(minWidth, next);
    if (maxWidth !== undefined) next = Math.min(maxWidth, next);
    return next;
  }, [minWidth, maxWidth]);

  const handleResizeStart = useCallback(() => {
    const el = sidebarRef.current;
    if (!el) return;
    el.classList.add('sidebar-resizing');
    liveWidthRef.current = clampWidth(el.getBoundingClientRect().width);
  }, [clampWidth]);

  const handleResize = useCallback((delta: number) => {
    const el = sidebarRef.current;
    if (!el) return;
    const next = clampWidth(liveWidthRef.current + delta);
    liveWidthRef.current = next;
    el.style.width = `${next}px`;
  }, [clampWidth]);

  const handleResizeEnd = useCallback(() => {
    const el = sidebarRef.current;
    if (el) {
      el.classList.remove('sidebar-resizing');
    }
    setWidth(liveWidthRef.current);
  }, []);

  const resolvedResizeEdges = resizeEdges ?? { right: true };

  const sidebarStyle = isCollapsed
    ? { width: '0px', minWidth: '0px', overflow: 'hidden' as const }
    : resizable
      ? {
          width: `${width}px`,
          ...(minWidth !== undefined ? { minWidth: `${minWidth}px` } : {}),
          ...(maxWidth !== undefined ? { maxWidth: `${maxWidth}px` } : {}),
        }
      : {};

  // Mobile overlay class
  const mobileClass = mobileOpen !== undefined
    ? (mobileOpen ? 'sidebar-mobile-open' : 'sidebar-mobile-closed')
    : '';

  return (
    <>
      {/* Backdrop for mobile overlay */}
      {mobileOpen && onMobileClose && (
        <div class="sidebar-overlay visible" onClick={onMobileClose} />
      )}
      <aside
        ref={sidebarRef as any}
        class={`sidebar ${isCollapsed ? 'sidebar-collapsed' : ''} ${mobileClass} ${className}`}
        style={sidebarStyle}
      >
        {!isCollapsed && (
          <>
            <div class="sidebar-header">
              <h2>{title}</h2>
              {badge !== undefined && <span class="sidebar-badge">{badge}</span>}
              {collapsible && (
                <button class="sidebar-collapse-btn" onClick={toggleCollapse} title="Collapse sidebar">
                  <CollapseIcon />
                </button>
              )}
              {onMobileClose && (
                <button class="sidebar-close-btn" onClick={onMobileClose} title="Close sidebar">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" width="16" height="16">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              )}
            </div>
            
            <div class="sidebar-content">
              {loading ? (
                <div class="sidebar-loading">Loading...</div>
              ) : isEmpty ? (
                <div class="sidebar-empty">{emptyMessage}</div>
              ) : (
                children
              )}
            </div>

            {resizable && resolvedResizeEdges.left && (
              <ResizeHandle
                direction="horizontal"
                edge="left"
                deltaSign={-1}
                onResizeStart={handleResizeStart}
                onResize={handleResize}
                onResizeEnd={handleResizeEnd}
              />
            )}
            {resizable && resolvedResizeEdges.right && (
              <ResizeHandle
                direction="horizontal"
                edge="right"
                onResizeStart={handleResizeStart}
                onResize={handleResize}
                onResizeEnd={handleResizeEnd}
              />
            )}
          </>
        )}
        {isCollapsed && collapsible && (
          <button class="sidebar-expand-btn" onClick={toggleCollapse} title="Expand sidebar">
            <ExpandIcon />
          </button>
        )}
      </aside>
    </>
  );
}

// ── Icons ──

function CollapseIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}

function ExpandIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}
