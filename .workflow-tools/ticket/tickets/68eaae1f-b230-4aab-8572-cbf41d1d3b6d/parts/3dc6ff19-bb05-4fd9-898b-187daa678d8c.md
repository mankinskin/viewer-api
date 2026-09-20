# Goal

Add an optional fixed 2D graph presentation mode with a planar camera, 2D grid styling, and presentation keyframing for temporary selection-driven layouts.

# Scope

- add a fixed 2D canvas camera mode alongside the existing 3D-oriented presentation
- render matching 2D grid patterns and camera controls for the planar mode
- use flat points as the minimal glyph in 2D mode while preserving small spheres in 3D mode
- support temporary keyframed layout adjustments when a node is selected
- support both explicit invocation and configurable automatic activation from selection
- restore the prior layout cleanly after the temporary presentation state ends
- add deterministic browser and Playwright validation for mode switching and keyframe transitions

# Acceptance

- users can switch into an explicit 2D graph presentation mode without removing the current 3D mode
- the 2D mode uses a fixed planar camera and matching 2D grid presentation
- the lowest tier in 2D mode renders as flat points while 3D mode continues to use small spheres
- selecting a node can trigger a temporary keyframed layout adjustment through either explicit invocation or settings-controlled automatic activation
- ending the temporary presentation restores the prior layout predictably
- browser and Playwright validation prove 2D mode, configurable auto-keyframing, explicit keyframing, and restore behavior
