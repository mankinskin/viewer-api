---
tags: `#plan` `#viewer-api` `#graph` `#ux` `#rendering` `#webgpu` `#lighting` `#particles`
summary: Improve Graph3D edge legibility and atmosphere with depth-aware shading, clear focus colors, and restrained directional particles
status: [ ] planned
---

# Plan: Graph edge visual polish for graph viewer

## Objective

Define a shared edge-visual treatment for the Graph3D renderer so graph edges remain accurate but gain better depth cues, clearer focus feedback, and controlled motion. The plan should apply first to the current spec graph view and remain reusable for other viewer-api graph surfaces.

## Current problem

The current graph edges are geometrically correct, but they still read as visually flat:

- default edges lack lighting or depth-based shading, so the graph feels visually thin
- hovered or selected relationships do not stand out strongly enough from surrounding context
- there is no motion vocabulary to communicate direction, activity, or emphasis
- the edge layer does not yet feel integrated with the rest of the GPU atmosphere

## UX goals

- Keep the default edge layer readable but quiet; node cards and labels must remain dominant.
- Make incident-edge focus obvious within a single glance when a node or path is hovered or selected.
- Add depth perception without requiring heavy physically based lighting.
- Use motion only as emphasis, not as constant background noise.
- Preserve clarity in dense graphs, not only sparse graphs.
- Respect reduced-motion and low-performance environments.

## Recommended visual treatment

### 1. Base edge shading

Use a two-layer edge material instead of a single flat line:

- a narrow brighter core line for crisp structural readability
- a wider soft halo or falloff layer for atmosphere and separation from the background
- depth attenuation that reduces alpha and halo strength with distance
- slight width gain for near edges, but avoid exaggerated thickness jumps

Recommended default palette direction:

- base: muted slate or steel-blue neutral
- halo: low-alpha cool tint derived from the active theme
- nonessential edges should stay subdued enough that the graph still reads when many edges overlap

### 2. Focus colors and emphasis states

Use a simple state vocabulary so users can distinguish transient focus from committed focus:

- default state: low-saturation neutral edge
- hover or transient incident focus: cool cyan or ice-blue accent, slightly wider core, stronger halo
- selected or pinned path: warm amber or gold accent so it is clearly distinct from hover
- de-emphasized context: aggressively fade non-incident edges when focus is active

Do not rely on hue alone. Pair focus with:

- width increase
- halo strength increase
- opacity drop for unrelated edges
- optional dash or pulse only if needed after testing

Reserve additional semantic colors for future overlays instead of spending the whole palette on base graph state.

### 3. Particle and motion recommendations

Particles should be sparse and purposeful:

- no always-on particles for every edge in the default state
- highlighted edges may show slow directional particles flowing from source to target
- selected primary paths may show occasional pulse packets rather than a constant stream
- junctions near the focused node can receive a brief glow pulse when focus changes

Motion guardrails:

- particle counts must be capped per visible highlighted edge
- particles should be culled on distant, occluded, or heavily faded edges
- all motion should degrade gracefully under reduced-motion preferences or low frame rate
- when performance drops, disable particles before dropping the core highlight treatment

### 4. Lighting model recommendation

Prefer a cheap stylized lighting model over full PBR:

- additive edge halo with theme-tinted bloom
- depth-based attenuation for alpha and glow radius
- slightly brighter edge segments near focused nodes or path endpoints
- optional camera-facing shimmer or rim response only on highlighted edges

The target feel should be technical and atmospheric, not neon overload. Dense graphs should still feel calm.

### 5. Dense-graph readability rules

In dense graphs, clarity matters more than spectacle:

- clamp halo radius and bloom strength as visible edge count rises
- reduce background-edge opacity harder during focus states
- keep highlighted paths continuous and readable across overlaps
- prevent particles from creating visual fog in graph centers
- ensure focus colors remain legible against both the GPU background and the node-card layer

### 6. Accessibility and interaction guidance

- provide a reduced-motion mode that disables particles and pulsing while preserving color and width-based focus
- ensure focus remains distinguishable without color alone
- preserve keyboard-driven focus and selection states with the same visual hierarchy as pointer hover
- expose theme-derived accents rather than hardcoded colors so edge states remain compatible with custom themes

## Recommended implementation phases

### Phase 1: Base edge material

- add core-line plus halo rendering
- add depth-aware alpha and glow attenuation
- tune the neutral default palette against existing graph backgrounds

### Phase 2: Focus-state system

- highlight incident edges on node hover
- add selected-path styling distinct from hover
- fade non-incident edges during focus

### Phase 3: Particle accents

- directional particles for highlighted edges only
- one-shot path pulses for selection changes
- reduced-motion and frame-budget fallback logic

### Phase 4: Tuning and validation

- test sparse, medium, and dense graphs
- verify theme compatibility and contrast
- tune blend strengths so edges support the graph instead of dominating it

## Manual validation scenarios

- sparse graph with no focus state
- dense graph with hover on a central node
- selected path through overlapping edges
- reduced-motion mode enabled
- theme changes applied through existing viewer theme settings

## Open questions

- whether edge kind should influence hue subtly in the default state, or only during explicit semantic overlays
- whether endpoint glow should live on the edge pass or on node-adjacent highlight sprites
- whether particles should encode direction for all selected paths or only for primary paths

## Deliverable

A concrete implementation plan for Graph3D edge visuals that documents:

- default, hover, and selected edge treatments
- palette and motion rules
- accessibility constraints
- density and performance fallbacks
- a phased rollout order for implementation
