import { useRef, useCallback } from 'preact/hooks';

export interface ResizeHandleProps {
  onResize: (delta: number) => void;
  onResizeStart?: () => void;
  onResizeEnd?: () => void;
  direction: 'horizontal' | 'vertical';
  /** Which edge this handle controls. Defaults to right/bottom by direction. */
  edge?: 'left' | 'right' | 'top' | 'bottom';
  /** Multiply drag delta by this value (use -1 for left/top anchored panes). */
  deltaSign?: 1 | -1;
}

export function ResizeHandle({
  onResize,
  onResizeStart,
  onResizeEnd,
  direction,
  edge,
  deltaSign = 1,
}: ResizeHandleProps) {
  const isDragging = useRef(false);
  const lastPos = useRef(0);
  const pendingDelta = useRef(0);
  const frameId = useRef<number | null>(null);

  const resolvedEdge = edge ?? (direction === 'horizontal' ? 'right' : 'bottom');

  const handleMouseDown = useCallback((e: MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    lastPos.current = direction === 'horizontal' ? e.clientX : e.clientY;
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';

    onResizeStart?.();

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
      const delta = currentPos - lastPos.current;
      lastPos.current = currentPos;

      pendingDelta.current += delta * deltaSign;
      if (frameId.current === null) {
        frameId.current = requestAnimationFrame(() => {
          frameId.current = null;
          const nextDelta = pendingDelta.current;
          pendingDelta.current = 0;
          if (nextDelta !== 0) {
            onResize(nextDelta);
          }
        });
      }
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      if (frameId.current !== null) {
        cancelAnimationFrame(frameId.current);
        frameId.current = null;
      }
      const nextDelta = pendingDelta.current;
      pendingDelta.current = 0;
      if (nextDelta !== 0) {
        onResize(nextDelta);
      }
      onResizeEnd?.();
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [onResize, onResizeStart, onResizeEnd, direction, deltaSign]);

  return (
    <div
      class={`resize-handle resize-handle-${direction} resize-handle-edge-${resolvedEdge}`}
      onMouseDown={handleMouseDown}
    />
  );
}
