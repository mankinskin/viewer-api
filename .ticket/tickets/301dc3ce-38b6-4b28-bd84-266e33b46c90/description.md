---
tags: `#plan` `#rendering` `#3d` `#webgpu` `#dom`
summary: Render DOM element tree as texture in 3D space, integrated with WebGPU particles and 3D scenes
status: 📋
---

# Plan: DOM → 3D Space Integration

## Objective

Render the HTML DOM element tree as a texture that can be displayed and transformed in 3D space, seamlessly integrating with existing WebGPU particles, 3D objects (cubes, hypergraph nodes), and post-processing effects.

---

## Current Architecture Analysis

### How DOM Rendering Works Today

The app uses a **layered compositing** model with two distinct rendering surfaces:

```
┌───────────────────────────────────────┐
│  HTML DOM (z-index: auto, above)      │  ← Preact renders here
│  ┌─ .app                             │
│  │  ├─ Header, Sidebar, TabBar       │
│  │  ├─ .view-container               │
│  │  │  ├─ LogViewer / Stats / etc.   │  ← Standard 2D DOM views
│  │  │  ├─ HypergraphView             │  ← DOM nodes + overlay callback
│  │  │  └─ Scene3D                    │  ← DOM labels + overlay callback
│  │  └─ EffectsDebug, Settings        │
│  └── WgpuOverlay <canvas>            │
│      (z-index: -1, BEHIND HTML)      │  ← WebGPU renders here
│      ├─ Background (smoke/CRT/grain) │
│      ├─ Element rects (scanned)      │
│      ├─ Particles (compute+render)   │
│      ├─ 3D overlays (edges/grid)     │
│      └─ cursor                       │
└───────────────────────────────────────┘
```

#### Key observations:

1. **Canvas is BEHIND HTML** (`z-index: -1, opaque`). HTML backgrounds are set to transparent (`gpu-active` class) so the GPU-rendered stone texture shows through.

2. **Element scanner** (`element-scanner.ts`) uses `MutationObserver` + `IntersectionObserver` + `ResizeObserver` to track DOM element positions. It reads `getBoundingClientRect()` and packs `[x, y, w, h, hue, kind]` into a `Float32Array` uploaded to the GPU.

3. **The GPU never sees actual DOM pixels.** It only sees element *rects* (bounding boxes). The `background.wgsl` shader draws procedural decorations (borders, glows, shadows, noise) at those rect positions. The actual text and DOM content is rendered by the browser's compositor on top.

4. **3D views** (HypergraphView, Scene3D) use the `registerOverlayRenderer` callback system to draw into the shared WgpuOverlay canvas with their own pipelines (edges, grids, cubes). They set viewport/scissor to their container's screen region. DOM elements (`.hg-node` divs) are positioned with CSS `transform: translate()` based on `worldToScreen()` projection.

5. **Particles** simulate in world space and project through `particle_vp` matrix. For 2D views, an orthographic matrix is used (world ≡ screen pixels). For 3D views, the view's camera viewProj is composed with a post-transform.

### What the GPU Hooks Do NOT Have Access To

- **Rasterized HTML content**: The browser's compositor renders DOM elements after CSS/layout. There is no WebGPU mechanism to read the composited HTML pixel output. WebGPU operates on its own canvas independently from the browser's DOM renderer.
- **Text rendering**: All text is rendered by the browser. GPU shaders only draw procedural effects.
- **CSS-applied styles**: The scanner reads computed bounding boxes, but not visual properties like colors, fonts, borders, shadows, etc.

---

## Technical Approaches for DOM-in-3D

### Approach 1: `html2canvas` / `dom-to-image` → Texture (Screen Capture)

**How it works:** Capture the DOM tree as a rasterized image using libraries that re-implement the browser's renderer in JavaScript (traversing the DOM, reading computed styles, drawing to a 2D canvas).

**Flow:**
```
DOM tree → html2canvas → Canvas2D → ImageBitmap → GPUTexture → 3D quad
```

**Pros:**
- Gets actual pixel-perfect (or near) DOM appearance
- Captures text, CSS effects, custom fonts
- Works with standard DOM

**Cons:**
- **Very slow** (50-200ms per capture) — not viable for 60fps
- Incomplete CSS support (no `backdrop-filter`, limited transforms, etc.)
- Doesn't handle `<canvas>`, `<video>`, cross-origin images
- Would need to re-capture on every DOM change

**Verdict:** Too slow for real-time 3D integration. Only useful for static snapshots.

### Approach 2: CSS 3D Transforms on DOM Elements

**How it works:** Use CSS `perspective`, `transform-style: preserve-3d`, and `transform: matrix3d()` to position DOM elements in true 3D space. The browser's compositor handles depth ordering and perspective projection.

**Flow:**
```
DOM elements → CSS matrix3d(viewProj) → Browser compositor → Screen
WebGPU canvas → Particles/3D objects → Screen (composited behind/above)
```

**Pros:**
- **Near-zero performance cost** — browser's GPU compositor does the 3D transform
- Full CSS support (text, borders, shadows, all layout properties)
- Sub-pixel text rendering preserved
- Interactive (pointer events, hover, focus all work natively)
- Already partially implemented (HypergraphView uses `translate()` + `scale()`)

**Cons:**
- Depth interleaving with WebGPU content is hard — DOM and canvas are separate compositing layers
- Can't achieve true depth-tested occlusion between DOM text and 3D objects/particles
- Browser compositing limitations (no per-pixel depth test between DOM and WebGPU)

**Verdict:** Best approach for DOM element positioning. Need a strategy for depth compositing.

### Approach 3: Offscreen Document / `<iframe>` → `texImage2D` Capture

**How it works:** Render DOM content in a hidden `<iframe>` or offscreen document, then capture it using the experimental `element.captureStream()` or `drawImage()`.

**Cons:** Cross-origin restrictions, `captureStream()` availability, iframe overhead.

**Verdict:** Fragile and browser-specific. Not recommended.

### Approach 4: Custom Text/UI Rendering in WebGPU (SDF Text, etc.)

**How it works:** Bypass the DOM entirely. Render all text as SDF (signed distance field) font atlases in WebGPU. Build a mini UI toolkit on the GPU.

**Pros:** Full depth integration with 3D, pixel-perfect control

**Cons:**
- Massive engineering effort (text shaping, layout, scrolling, input)
- Worse text quality than browser native rendering
- Must re-implement all CSS features

**Verdict:** Extreme overkill for this use case. Possibly relevant for specific HUD elements later.

### Approach 5: Hybrid — CSS 3D + Multi-Pass WebGPU (Recommended)

**How it works:** Combine CSS 3D transforms for DOM positioning with a multi-layer WebGPU rendering strategy:

```
    Layer 0 (back):   WebGPU canvas (z-index: -1) — background, element rects, far 3D objects
    Layer 1 (middle): DOM elements   — positioned with CSS matrix3d(), transparent backgrounds
    Layer 2 (front):  WebGPU canvas (z-index: +1) — particles, near 3D objects, post-processing
```

Or more advanced: use TWO WebGPU canvases (one behind DOM, one in front).

**Key insight:** We already have the "canvas behind DOM" pattern. We can extend it with a second canvas or use CSS `mix-blend-mode` / `pointer-events: none` canvas in front.

---

## Recommended Implementation Plan

### Phase 1: CSS 3D Transform Infrastructure (Foundation)

**Goal:** Position DOM elements in 3D space using CSS `matrix3d()` transforms derived from the same viewProj that particles/3D objects use.

**Changes:**
- [ ] Create a `worldToCSS3D(worldPos, viewProj, canvasW, canvasH)` utility that produces a CSS `matrix3d()` string
- [ ] Modify HypergraphView to use `matrix3d()` instead of `translate() + scale()`
- [ ] Ensure CSS 3D transforms use the same projection matrix as the WebGPU particle pipeline
- [ ] Add `transform-style: preserve-3d` and `perspective` to container
- [ ] DOM elements maintain correct depth ordering relative to each other via `z-index`

**Files:** HypergraphView.tsx, math3d.ts, hypergraph.css

### Phase 2: Front-Layer Particle Canvas (Depth Compositing)

**Goal:** Add a second transparent WebGPU canvas (or `<div>` overlay) in front of DOM elements for particles that should appear in front of DOM content.

**Changes:**
- [ ] Create `WgpuFrontOverlay` — a second `<canvas>` with `z-index: +1`, `pointer-events: none`, transparent background
- [ ] Split the render loop: background effects → back canvas, particles → front canvas
- [ ] Or: use a single canvas with `alphaMode: 'premultiplied'` above DOM, rendering only particles/beams
- [ ] Background smoke/CRT stays on the back canvas at `z-index: -1`

**Files:** WgpuOverlay.tsx, gpu-render-loop.ts, App.tsx, layout.css

### Phase 3: Element Depth Uniforms (Per-Element Z)

**Goal:** Allow individual DOM elements and their associated particle effects to exist at different depths in 3D space.

**Changes:**
- [ ] Extend `ElemRect` struct from `[x, y, w, h, hue, kind, _p1, _p2]` to include a world-space depth/position
- [ ] Element scanner: for 3D views, store the world-space position (not just screen rect)
- [ ] Particles spawn at the element's world-space position (already partially done)
- [ ] Background shader: use element depth to modulate glow intensity (farther = dimmer)

**Files:** types.wgsl, element-scanner.ts, element-types.ts, background.wgsl, compute.wgsl

### Phase 4: DOM Texture Capture for Static/Billboard Content

**Goal:** For elements that need true pixel-accurate rendering on 3D surfaces (like texture-mapped billboards), capture specific DOM subtrees as textures using optimized techniques.

**Changes:**
- [ ] Use `OffscreenCanvas` + `createImageBitmap()` for lightweight captures
- [ ] Cache textures, invalidate on content change (MutationObserver)
- [ ] Render as textured quads in WebGPU with depth testing
- [ ] Use for: info panels, labels, HUD elements that need to be truly "in" the 3D world
- [ ] Frame budget: capture at most 1 texture per frame, use stale texture otherwise

**Files:** New `dom-texture-cache.ts`, new `billboard.wgsl`, gpu-init.ts, gpu-render-loop.ts

### Phase 5: Post-Processing Pipeline (Screen-Space Effects on Everything)

**Goal:** Apply post-processing (CRT, grain, color grading) to the *composited* output — DOM + WebGPU combined.

**Changes:**
- [ ] Render all WebGPU content to an offscreen texture (not directly to canvas)
- [ ] Final composite pass: blend the offscreen texture with any DOM content
- [ ] Alternatively: use CSS `filter` / `backdrop-filter` for CRT-like effects on DOM
- [ ] Or: capture full screen via `canvas.captureStream()` at reduced framerate for post-FX

**Files:** gpu-init.ts, gpu-render-loop.ts, WgpuOverlay.tsx, new post-process shaders

---

## Immediate TODO List

### Quick Wins (can do now)
1. **CSS `matrix3d()` for HypergraphView nodes** — Replace `translate() + scale()` with full 3D projection. Low risk, high value.
2. **Add world-space depth to element rects** — Store the NDC depth when scanner measures 3D-positioned elements. Minimal API change.
3. **Front overlay canvas for particles** — Second transparent canvas above DOM for particle-over-DOM rendering.

### Medium-Term
4. **Per-element 3D position in scanner** — 3D views tag DOM elements with `data-world-xyz` attributes; scanner reads them.
5. **Billboard texture cache** — Selective DOM→texture capture for truly embedded 3D UI panels.
6. **Split background vs. foreground render passes** — Clean separation for proper compositing.

### Long-Term / Research
7. **`element()` CSS function** (Chrome 127+) — allows CSS `background: element(#some-div)` to use a DOM element as a texture. Very experimental.
8. **WebCodecs VideoFrame capture** — ultra-fast screen capture API for post-processing the combined output.
9. **SDF text for critical HUD elements** — GPU-native text for labels that must be depth-tested with 3D objects.

---

## Risks & Open Questions

1. **Depth interleaving precision:** CSS `z-index` is an integer — can we get smooth depth ordering between DOM elements and WebGPU content?
2. **Pointer event passthrough:** A front overlay canvas with `pointer-events: none` may block hover detection on DOM elements below. Test with transparent canvas.
3. **Performance of two canvases:** Will two WebGPU canvases on the same device cause double-submit overhead?
4. **`preserve-3d` + GPU overlay interaction:** Will the browser's compositor correctly layer DOM 3D transforms with the canvas layers?
5. **Mobile/browser compatibility:** CSS `matrix3d()` support is universal, but `preserve-3d` stacking contexts interact differently across browsers.

---

## Architecture Decision: Why Not Render-to-Texture of Entire DOM?

The browser's composited DOM output is **not accessible to WebGPU/WebGL** without:
- `html2canvas` (slow JS reimplementation, 50-200ms)
- `getDisplayMedia()` (screen capture API — requires user permission prompt, captures entire screen not just tab)
- `svg foreignObject` trick (limited, no interactive elements)

The fundamental constraint is that **the browser's GPU compositor and WebGPU operate on separate render pipelines**. There is no API to read the compositor's output buffer into a WebGPU texture at interactive framerates.

Therefore the recommended approach is the hybrid CSS 3D + multi-layer WebGPU model: let the browser do what it does best (text, CSS, layout) and use WebGPU for what it does best (particles, procedural effects, 3D geometry).
