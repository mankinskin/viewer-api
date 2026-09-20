# Goal

Replace the current rich-card-first graph node presentation with property-based level-of-detail rendering that can collapse to points, spheres, icons, labels, and tool-free compact summaries before using full HTML detail.

# Scope

- define render properties and thresholds that choose node representations from minimal through rich tiers
- lock the tier order to point-or-sphere, icon, label, compact, and full
- switch the minimal glyph by camera mode: flat point in 2D and small shaded sphere in 3D
- reserve full HTML rendering for high-detail states only
- raise hover by exactly one detail tier instead of jumping directly to rich preview
- verify the tier model with deterministic browser and Playwright assertions

# Acceptance

- node rendering is selected from explicit property-based rules instead of assuming a rich ticket card by default
- the ordered tier ladder is point-or-sphere → icon → label → compact → full
- minimal tiers switch by camera mode between flat points and shaded spheres
- hover raises a node by exactly one tier
- lower tiers rely on glyphs, icons, labels, and compact summaries without an extra tooltip fallback when promotion is enough
- full HTML rendering is used only for high-detail tiers where projected size, focus, and budget justify it
- release Playwright coverage proves tier ordering and hover promotion behavior
