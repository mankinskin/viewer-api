/**
 * One-shot JPEG capture of the GPU canvas composited with DOM elements.
 * Used for theme thumbnails.
 */

/**
 * Capture a low-res thumbnail by drawing the GPU canvas and then
 * compositing visible DOM elements on top via a TreeWalker.
 *
 * @param gpuCanvas  The WebGPU canvas element
 * @param thumbWidth Target width in pixels (default 192)
 * @param quality    JPEG quality 0..1 (default 0.55)
 * @returns          Data URL string, or empty string on failure
 */
export function captureFrame(
    gpuCanvas: HTMLCanvasElement,
    thumbWidth = 192,
    quality = 0.55,
): string {
    try {
        const fullW = gpuCanvas.width;
        const fullH = gpuCanvas.height;
        const thumbH = Math.round(thumbWidth * fullH / fullW);
        const scaleX = thumbWidth / window.innerWidth;
        const scaleY = thumbH / window.innerHeight;
        const offscreen = document.createElement('canvas');
        offscreen.width = thumbWidth;
        offscreen.height = thumbH;
        const c = offscreen.getContext('2d')!;

        // Layer 1: GPU canvas background
        c.drawImage(gpuCanvas, 0, 0, thumbWidth, thumbH);

        // Layer 2: composite visible DOM elements on top
        const walker = document.createTreeWalker(
            document.body,
            NodeFilter.SHOW_ELEMENT,
            {
                acceptNode(node) {
                    const el = node as HTMLElement;
                    if (el === gpuCanvas || el.tagName === 'CANVAS') return NodeFilter.FILTER_REJECT;
                    if (el.offsetWidth === 0 && el.offsetHeight === 0) return NodeFilter.FILTER_REJECT;
                    return NodeFilter.FILTER_ACCEPT;
                },
            },
        );

        let node: Node | null;
        while ((node = walker.nextNode())) {
            const el = node as HTMLElement;
            const rect = el.getBoundingClientRect();
            if (rect.bottom < 0 || rect.top > window.innerHeight) continue;
            if (rect.right < 0 || rect.left > window.innerWidth) continue;
            if (rect.width < 2 || rect.height < 2) continue;

            const style = getComputedStyle(el);
            const tx = rect.left * scaleX;
            const ty = rect.top * scaleY;
            const tw = rect.width * scaleX;
            const th = rect.height * scaleY;

            // Draw background if non-transparent
            const bg = style.backgroundColor;
            if (bg && bg !== 'transparent' && bg !== 'rgba(0, 0, 0, 0)') {
                c.fillStyle = bg;
                c.fillRect(tx, ty, tw, th);
            }

            // Draw visible borders
            const bw = parseFloat(style.borderTopWidth);
            if (bw >= 1) {
                const bc = style.borderTopColor;
                if (bc && bc !== 'transparent' && bc !== 'rgba(0, 0, 0, 0)') {
                    c.strokeStyle = bc;
                    c.lineWidth = Math.max(0.5, bw * scaleX);
                    c.strokeRect(tx, ty, tw, th);
                }
            }

            // Draw a simplified text line if element has direct text content
            if (th >= 3 && el.childNodes.length > 0 && el.childNodes[0]?.nodeType === Node.TEXT_NODE) {
                const text = el.childNodes[0].textContent?.trim();
                if (text && text.length > 0) {
                    const color = style.color;
                    if (color && color !== 'transparent') {
                        c.fillStyle = color;
                        c.globalAlpha = 0.6;
                        const lineH = Math.max(1, th * 0.35);
                        const lineW = Math.min(tw * 0.85, text.length * tw * 0.04);
                        c.fillRect(tx + tw * 0.05, ty + (th - lineH) / 2, lineW, lineH);
                        c.globalAlpha = 1.0;
                    }
                }
            }
        }

        return offscreen.toDataURL('image/jpeg', quality);
    } catch {
        return '';
    }
}
