import { JSX } from 'preact';
import { useCallback, useState } from 'preact/hooks';
import { ResizeHandle } from './ResizeHandle';

export interface Tab {
  id: string;
  label: string;
  icon?: JSX.Element;
  closeable?: boolean;
  modified?: boolean;
}

export interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose?: (id: string) => void;
  rightContent?: JSX.Element;
  /** Enable drag-resize on the bottom edge (default: false). */
  resizableBottom?: boolean;
  /** Initial tab bar height in px. */
  initialHeight?: number;
  /** Min tab bar height in px. */
  minHeight?: number;
  /** Max tab bar height in px. */
  maxHeight?: number;
}

export function TabBar({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  rightContent,
  resizableBottom = false,
  initialHeight = 32,
  minHeight = 24,
  maxHeight = 120,
}: TabBarProps): JSX.Element {
  const [height, setHeight] = useState(initialHeight);

  const onResizeBottom = useCallback((delta: number) => {
    setHeight((prev) => Math.max(minHeight, Math.min(maxHeight, prev + delta)));
  }, [minHeight, maxHeight]);

  return (
    <div class="tab-bar" style={{ height: `${height}px` }}>
      <div class="tabs">
        {tabs.map(tab => (
          <button
            key={tab.id}
            class={`tab ${activeTabId === tab.id ? 'active' : ''}`}
            onClick={() => onSelect(tab.id)}
            title={tab.label}
          >
            {tab.icon && <span class="tab-icon">{tab.icon}</span>}
            <span class="tab-label">{tab.label}</span>
            {tab.modified && <span class="tab-modified">•</span>}
            {tab.closeable && onClose && (
              <span 
                class="tab-close" 
                onClick={(e) => {
                  e.stopPropagation();
                  onClose(tab.id);
                }}
              >
                <CloseIcon />
              </span>
            )}
          </button>
        ))}
      </div>
      {rightContent && <div class="tab-info">{rightContent}</div>}
      {resizableBottom && (
        <ResizeHandle direction="vertical" edge="bottom" onResize={onResizeBottom} />
      )}
    </div>
  );
}

function CloseIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}
