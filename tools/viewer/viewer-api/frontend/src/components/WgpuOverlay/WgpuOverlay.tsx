/// <reference types="@webgpu/types" />
/**
 * WgpuOverlay — GPU-accelerated canvas rendered **behind** HTML.
 *
 * Single opaque canvas (z-index -1) renders everything:
 *   - Background effects (smoke, grain, CRT, vignette)
 *   - Element rect decorations (underglow, cinders)
 *   - 3D overlays (edges, grids, cubes via registered callbacks)
 *   - Particles (sparks, embers, beams, glitter via additive blend)
 *
 * Thin component shell.  All heavy lifting is delegated to:
 *   - element-scanner.ts   — reactive DOM scanning (no MAX_ELEMENTS limit)
 *   - gpu-init.ts          — device/pipeline/shader creation
 *   - gpu-buffers.ts       — dynamic buffer management
 *   - gpu-render-loop.ts   — rAF loop, uniform packing, GPU passes
 *   - overlay-api.ts       — signals, render callbacks, capture trigger
 *   - thumbnail-capture.ts — one-shot JPEG capture
 *
 * Selector metadata and app-specific signals (themeColors, effectSettings,
 * active view) are injected via the `schema` prop so this component is
 * not coupled to any specific application.
 */
import { useEffect, useRef } from 'preact/hooks';

import { initGpu, type GpuPipelines } from './gpu-init';
import { GpuBufferManager } from './gpu-buffers';
import { ElementScanner } from './element-scanner';
import { RenderLoop } from './gpu-render-loop';
import { gpuOverlayEnabled, fxEnabled, overlayGpu, setScanInvalidator, setParticleResetter } from './overlay-api';
import type { AppSchema, SchemaSelectorEntry } from './schemas';
import type { SelectorEntry } from './element-types';

// ---------------------------------------------------------------------------
// Re-exports — preserve the public API surface so that external consumers
// (HypergraphView, ThemeSettings, Header, App) can keep importing from
// this single barrel module.
// ---------------------------------------------------------------------------

export {
    gpuOverlayEnabled,
    fxEnabled,
    overlayGpu,
    captureOverlayThumbnail,
    registerOverlayRenderer,
    unregisterOverlayRenderer,
    markOverlayScanDirty,
    resetOverlayParticles,
    setOverlayParticleVP,
    setOverlayParticleViewport,
    setOverlayRefDepth,
    setOverlayWorldScale,
    setOverlayCameraPos,
    type OverlayRenderCallback,
} from './overlay-api';

// ---------------------------------------------------------------------------
// Selector helpers
// ---------------------------------------------------------------------------

function buildSelectorMeta(selectors: ReadonlyArray<SchemaSelectorEntry>): SelectorEntry[] {
    return selectors.map(({ sel, hue, kind }) => ({ sel, hue, kind }));
}

function buildPriorityIndices(selectors: ReadonlyArray<SchemaSelectorEntry>): Set<number> {
    const set = new Set<number>();
    selectors.forEach(({ priority }, i) => { if (priority) set.add(i); });
    return set;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface GpuSession {
    pipelines: GpuPipelines;
    buffers:   GpuBufferManager;
    scanner:   ElementScanner;
    loop:      RenderLoop;
}

interface WgpuOverlayProps {
    schema: AppSchema;
}

export function WgpuOverlay({ schema }: WgpuOverlayProps) {
    const canvasRef  = useRef<HTMLCanvasElement>(null);
    const sessionRef = useRef<GpuSession | null>(null);

    // --- track mouse position for 3D hover effect -------------------------
    useEffect(() => {
        const onMove = (e: MouseEvent) => {
            sessionRef.current?.loop.setMouse(e.clientX, e.clientY);
        };
        const onLeave = () => {
            sessionRef.current?.loop.setMouse(-9999, -9999);
        };
        window.addEventListener('mousemove', onMove);
        window.addEventListener('mouseleave', onLeave);
        return () => {
            window.removeEventListener('mousemove', onMove);
            window.removeEventListener('mouseleave', onLeave);
        };
    }, []);

    // --- toggle gpu-active class on document root -------------------------
    useEffect(() => {
        if (gpuOverlayEnabled.value) {
            document.documentElement.classList.add('gpu-active');
        } else {
            document.documentElement.classList.remove('gpu-active');
        }
        return () => document.documentElement.classList.remove('gpu-active');
    }, [gpuOverlayEnabled.value]);

    // --- toggle fx-active class on document root ---------------------------
    useEffect(() => {
        if (gpuOverlayEnabled.value && fxEnabled.value) {
            document.documentElement.classList.add('fx-active');
        } else {
            document.documentElement.classList.remove('fx-active');
        }
        return () => document.documentElement.classList.remove('fx-active');
    }, [gpuOverlayEnabled.value, fxEnabled.value]);


    // --- keep canvas sized to the viewport --------------------------------
    useEffect(() => {
        if (!gpuOverlayEnabled.value) return;
        const canvas = canvasRef.current;
        if (!canvas) return;

        const sync = () => {
            canvas.width  = window.innerWidth;
            canvas.height = window.innerHeight;
            // Explicitly sync CSS dimensions to buffer size.
            // On mobile Safari `100vh` includes the address bar area and
            // can be taller than `window.innerHeight`, causing a canvas
            // stretch that misaligns GPU edges from DOM nodes.
            canvas.style.width  = `${window.innerWidth}px`;
            canvas.style.height = `${window.innerHeight}px`;
        };
        sync();
        window.addEventListener('resize', sync);
        // Also listen to visualViewport resize for mobile browser chrome changes
        window.visualViewport?.addEventListener('resize', sync);
        return () => {
            window.removeEventListener('resize', sync);
            window.visualViewport?.removeEventListener('resize', sync);
        };
    }, [gpuOverlayEnabled.value]);

    // --- WebGPU init & render loop ----------------------------------------
    useEffect(() => {
        if (!gpuOverlayEnabled.value) {
            teardown(sessionRef.current);
            sessionRef.current = null;
            return;
        }

        const canvas = canvasRef.current;
        if (!canvas) return;

        let cancelled = false;

        async function init() {
            const pipelines = await initGpu(canvas!);
            if (!pipelines || cancelled) {
                if (pipelines) pipelines.device.destroy();
                return;
            }

            // Expose device for external renderers (e.g. HypergraphView)
            overlayGpu.value = { device: pipelines.device, format: pipelines.format };

            const buffers = new GpuBufferManager(pipelines.device);
            const selectorMeta = buildSelectorMeta(schema.selectors);
            const priorityIndices = buildPriorityIndices(schema.selectors);
            const scanner = new ElementScanner(selectorMeta, priorityIndices);
            const loop    = new RenderLoop(pipelines, buffers, scanner, canvas!, schema);

            const session: GpuSession = { pipelines, buffers, scanner, loop };
            sessionRef.current = session;

            // Wire up markOverlayScanDirty to delegate to this scanner
            setScanInvalidator(() => scanner.invalidateAll());

            // Wire up resetOverlayParticles to delegate to buffer manager
            setParticleResetter(() => buffers.resetParticles());

            // Start observing DOM and launch render loop
            scanner.start();
            loop.start();
        }

        init().catch(console.error);

        return () => {
            cancelled = true;
            teardown(sessionRef.current);
            sessionRef.current = null;
        };
    }, [gpuOverlayEnabled.value]);

    if (!gpuOverlayEnabled.value) return null;

    return (
        <canvas
            ref={canvasRef}
            aria-hidden="true"
            style={{
                position:      'fixed',
                top:           0,
                left:          0,
                width:         '100vw',
                height:        '100dvh',  /* dvh tracks actual viewport, fallback overridden by resize handler */
                pointerEvents: 'none',
                zIndex:        -1,
            }}
        />
    );
}

function teardown(session: GpuSession | null) {
    if (!session) return;
    session.loop.stop();
    session.scanner.destroy();
    setScanInvalidator(null);
    setParticleResetter(null);
    session.buffers.destroy();
    overlayGpu.value = null;
    session.pipelines.device.destroy();
}
