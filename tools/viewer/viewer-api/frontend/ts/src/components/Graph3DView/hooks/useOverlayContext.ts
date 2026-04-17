/**
 * Returns the shared GPU overlay context (device + format), or null when
 * WebGPU is unavailable or the overlay is disabled.
 *
 * Prefer calling this hook over reading `overlayGpu.value` directly so that
 * Preact's signal subscription is properly tracked in the component tree.
 */
import { overlayGpu } from '../../WgpuOverlay/WgpuOverlay';

export function useOverlayContext() {
    return overlayGpu.value;
}
