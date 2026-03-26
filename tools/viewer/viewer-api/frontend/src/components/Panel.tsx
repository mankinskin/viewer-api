import { JSX, ComponentChildren } from 'preact';
import { useState, useCallback, useRef } from 'preact/hooks';
import { ResizeHandle } from './ResizeHandle';

export type PanelPlacement = 'left' | 'right' | 'top' | 'bottom';

export interface PanelProps {
  /** Where the panel sits relative to the main content */
  placement: PanelPlacement;
  /** Panel content */
  children: ComponentChildren;
  /** Optional class name */
  class?: string;
  /** Initial size in px (width for left/right, height for top/bottom) */
  initialSize?: number;
  /** Min size in px (default: 0) */
  minSize?: number;
  /** Max size in px (default: unlimited) */
  maxSize?: number;
  /** Enable drag-to-resize (default: true) */
  resizable?: boolean;
  /** Whether the panel is visible (default: true) */
  visible?: boolean;
}

const RESIZE_EDGE: Record<PanelPlacement, 'left' | 'right' | 'top' | 'bottom'> = {
  left: 'right',
  right: 'left',
  top: 'bottom',
  bottom: 'top',
};

const RESIZE_DELTA_SIGN: Record<PanelPlacement, 1 | -1> = {
  left: 1,
  right: -1,
  top: 1,
  bottom: -1,
};

const RESIZE_DIRECTION: Record<PanelPlacement, 'horizontal' | 'vertical'> = {
  left: 'horizontal',
  right: 'horizontal',
  top: 'vertical',
  bottom: 'vertical',
};

/**
 * Resizable panel container for viewer tools.
 *
 * Supports four placement positions (left, right, top, bottom) with smooth
 * drag-to-resize via `ResizeHandle`.  Uses direct DOM mutation during drag
 * to avoid React re-renders, committing the final size to state on drag end.
 *
 * ```
 * <Panel placement="right" initialSize={320}>
 *   <CodeViewer />
 * </Panel>
 * ```
 */
export function Panel({
  placement,
  children,
  class: className = '',
  initialSize = 260,
  minSize = 0,
  maxSize,
  resizable = true,
  visible = true,
}: PanelProps): JSX.Element | null {
  const [size, setSize] = useState(initialSize);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const liveSizeRef = useRef(initialSize);

  const clampSize = useCallback((value: number) => {
    let next = value;
    if (minSize !== undefined) next = Math.max(minSize, next);
    if (maxSize !== undefined) next = Math.min(maxSize, next);
    return next;
  }, [minSize, maxSize]);

  const handleResizeStart = useCallback(() => {
    const el = panelRef.current;
    if (!el) return;
    el.classList.add('panel-resizing');
    const isHorizontal = placement === 'left' || placement === 'right';
    const rect = el.getBoundingClientRect();
    liveSizeRef.current = clampSize(isHorizontal ? rect.width : rect.height);
  }, [placement, clampSize]);

  const handleResize = useCallback((delta: number) => {
    const el = panelRef.current;
    if (!el) return;
    const next = clampSize(liveSizeRef.current + delta);
    liveSizeRef.current = next;
    const prop = (placement === 'left' || placement === 'right') ? 'width' : 'height';
    el.style[prop] = `${next}px`;
  }, [placement, clampSize]);

  const handleResizeEnd = useCallback(() => {
    const el = panelRef.current;
    if (el) el.classList.remove('panel-resizing');
    setSize(liveSizeRef.current);
  }, []);

  if (!visible) return null;

  const isHorizontal = placement === 'left' || placement === 'right';
  const sizeStyle = isHorizontal
    ? { width: `${size}px` }
    : { height: `${size}px` };

  return (
    <div
      ref={panelRef}
      class={`panel panel-${placement} ${className}`.trim()}
      style={sizeStyle}
    >
      {resizable && (
        <ResizeHandle
          direction={RESIZE_DIRECTION[placement]}
          edge={RESIZE_EDGE[placement]}
          deltaSign={RESIZE_DELTA_SIGN[placement]}
          onResizeStart={handleResizeStart}
          onResize={handleResize}
          onResizeEnd={handleResizeEnd}
        />
      )}
      {children}
    </div>
  );
}
