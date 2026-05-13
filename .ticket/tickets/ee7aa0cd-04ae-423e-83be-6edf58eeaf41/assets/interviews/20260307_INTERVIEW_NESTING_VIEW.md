# Interview: Nesting View Mode Feature

**Date:** 2026-03-07  
**Feature:** Hypergraph Nesting/Hierarchy View with Optional Duplication  
**Status:** In Progress

---

## Batch 1: Core Visual Design

### Q1. Selected Node Expansion Style
When a node is selected and expanded to show children inside, how should it appear?

- [ ] A. **Card expansion** — Node grows into a larger card/box with children arranged inside as smaller nodes
- [ ] B. **Nested circles** — Node becomes a larger circle containing child circles
- [ ] C. **Row layout** — Current decomposition style (horizontal row of children below parent label)
- [ ] D. **Other** — Describe: ___

**Answer:** 

---

### Q2. Parent Context Positioning
How should parent nodes be positioned around the selected (central) node?

- [ ] A. **Arc above** — Parents arranged in semi-circle arc above the selected node
- [ ] B. **Surrounding ring** — Parents distributed in a ring around the selected node
- [ ] C. **Layered shells** — Each parent level forms a concentric larger shell
- [ ] D. **Stacked vertically** — Parents stacked above (Y-axis) as in current force-directed layout
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q3. Parent Context Visual Treatment
How should parent nodes appear when shown as context around the selected node?

- [ ] A. **Dimmed + larger** — Semi-transparent, scaled up to "contain" the view
- [ ] B. **Outline only** — Just a border/outline, no fill
- [ ] C. **Blurred background** — Behind everything, blurred like depth-of-field
- [ ] D. **Collapsed badges** — Small indicators showing parent exists, expandable
- [ ] E. **Normal but dimmed** — Same size, just opacity reduced
- [ ] F. **Other** — Describe: ___

**Answer:** 

---

## Batch 2: Duplication Mode Behavior

### Q4. Duplicate Node Appearance
In duplicate mode, how should the duplicated child nodes (inside the expanded parent) look compared to their originals?

- [ ] A. **Identical** — Same appearance, just positioned inside parent
- [ ] B. **Ghost/translucent** — Semi-transparent to indicate "this is a copy"
- [ ] C. **Dashed border** — Normal fill but dashed/dotted border
- [ ] D. **Different color tint** — Slight color shift to distinguish
- [ ] E. **Badge/icon** — Small duplicate indicator icon
- [ ] F. **Other** — Describe: ___

**Answer:** 

---

### Q5. Clicking Duplicates
When the user clicks on a duplicated child node (inside an expanded parent), what should happen?

- [ ] A. **Select the original** — Navigate to and select the original node at its home position
- [ ] B. **Select in-place** — Keep view centered on current parent, just highlight selection
- [ ] C. **Expand duplicate** — Expand that duplicate as the new center (stay in nested view)
- [ ] D. **Prompt choice** — Show small popup: "Go to original" vs "Expand here"
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q6. Original Node When Duplicated
When a node appears as a duplicate inside an expanded parent, what happens to its original in the graph?

- [ ] A. **Stays visible, normal** — Original remains fully visible at its position
- [ ] B. **Stays visible, dimmed** — Original is dimmed to show it's "also shown elsewhere"
- [ ] C. **Hidden** — Original is hidden when duplicate is shown
- [ ] D. **Connected line** — Faint line drawn between duplicate and original
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

## Batch 3: Edge Rendering

### Q7. Edges to Duplicates
How should edges be drawn when child nodes are duplicated inside a parent?

- [ ] A. **Only to originals** — Edges connect to original positions only
- [ ] B. **Only to duplicates** — Edges connect to where nodes appear inside parent
- [ ] C. **Both** — Draw edges to both original and duplicate positions
- [ ] D. **Smart toggle** — Show duplicate edges only when parent is expanded
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q8. Internal Parent-Child Edges
When a parent is expanded showing children inside, should the edges between that parent and its children be:

- [ ] A. **Hidden** — No edge drawn (containment is implicit)
- [ ] B. **Visible but dimmed** — Faint lines showing the relationship
- [ ] C. **Styled differently** — Different color/style (e.g., dotted)
- [ ] D. **Visible normal** — Same as other edges
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

## Batch 4: Hierarchy Depth & Navigation

### Q9. Parent Context Depth
How many levels of parent hierarchy should be shown by default?

- [ ] A. **1 level** — Direct parents only
- [ ] B. **2 levels** — Parents and grandparents
- [ ] C. **All ancestors** — Full path to root(s)
- [ ] D. **User configurable** — Slider/input to set depth
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q10. Child Expansion Depth  
How many levels of children should be shown inside an expanded node?

- [ ] A. **1 level** — Direct children only
- [ ] B. **2 levels** — Children and grandchildren (nested expansion)
- [ ] C. **User configurable** — Slider/input to set depth
- [ ] D. **On-demand** — Click child to expand it within the current expanded parent
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q11. Navigation Between Parents
When a node has multiple parents and you want to switch focus to a different parent:

- [ ] A. **Click parent directly** — Clicking a parent context node makes it the new center
- [ ] B. **Keyboard shortcuts** — Arrow keys or hotkeys to cycle through parents
- [ ] C. **Parent list panel** — Side panel listing all parents with click-to-focus
- [ ] D. **All of the above**
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

## Batch 5: Mode Switching & Persistence

### Q12. Default Mode
What should be the default state when loading the view?

- [ ] A. **Nesting disabled** — Traditional force-directed view, user enables nesting
- [ ] B. **Nesting enabled, reparent mode** — Current animation behavior
- [ ] C. **Nesting enabled, duplicate mode** — Show duplicates by default
- [ ] D. **Remember last** — Persist user's last choice
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q13. Mode Toggle Location
Where should the nesting/duplication toggles appear?

- [ ] A. **ControlsHUD** — With existing controls (auto-layout toggle, etc.)
- [ ] B. **Dedicated toolbar** — New floating toolbar for view modes
- [ ] C. **Context menu** — Right-click menu option
- [ ] D. **Keyboard only** — Hotkeys (e.g., N for nesting, D for duplicate)
- [ ] E. **Multiple** — Specify which: ___

**Answer:** 

---

### Q14. Transition Animation
When switching between reparent and duplicate modes, how should nodes transition?

- [ ] A. **Instant** — Immediate switch, no animation
- [ ] B. **Fade** — Cross-fade between states
- [ ] C. **Animate** — Nodes smoothly move to new positions
- [ ] D. **Other** — Describe: ___

**Answer:** 

---

## Batch 6: Edge Cases & Concerns

### Q15. Empty Children
If an expanded node has no children (leaf/atom), how should it appear?

- [ ] A. **Collapsed automatically** — Can't expand atoms
- [ ] B. **Empty expanded state** — Show expanded form with "no children" indicator
- [ ] C. **Different styling** — Atoms have distinct non-expandable appearance
- [ ] D. **Other** — Describe: ___

**Answer:** 

---

### Q16. Circular Relationships
If there are cycles (node A is parent of B, B is parent of A - unlikely but possible in some DAGs), how to handle?

- [ ] A. **Break cycle at selection** — Only show each node once in the hierarchy
- [ ] B. **Mark cycles** — Show indicator that cycle exists
- [ ] C. **Error/warning** — Flag as data issue
- [ ] D. **Not applicable** — Our graphs are guaranteed acyclic
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

### Q17. Performance Threshold
At what point should we limit duplication for performance?

- [ ] A. **Max duplicates** — Hard limit (e.g., 20 duplicates max)
- [ ] B. **Depth limit** — Only duplicate 2 levels deep
- [ ] C. **Smart culling** — Hide off-screen duplicates
- [ ] D. **User warning** — Warn when graph is too large for duplicate mode
- [ ] E. **Other** — Describe: ___

**Answer:** 

---

## Additional Notes

*Space for any other requirements, concerns, or ideas not covered above:*

---

## Summary

| Setting | Choice |
|---------|--------|
| Expansion style | **Row layout** — Current decomposition style (horizontal row) |
| Parent positioning | **Layered shells** — Concentric shells, deeper = larger |
| Parent visual | **Dimmed + larger** — Semi-transparent, scaled up |
| Duplicate appearance | **Identical + badge** — Same look, with badge & special directed edge endpoint |
| Click duplicate | **Select original** — Navigate to original node at its home position |
| Original when duplicated | **Stays visible, dimmed** — Original dimmed when duplicate shown |
| Edges to duplicates | **Only to originals** — Edges connect to original positions |
| Internal edges | **Hidden → node highlights** — Edges hidden, highlights applied to nodes instead |
| Parent depth | **User configurable** — Slider/input |
| Child depth | **User configurable** — Slider/input |
| Navigation | **Click parent label** — Smooth transition, collapse current → expand new parent |
| Default mode | **Remember last** (fallback: nesting + duplicate mode) |
| Toggle location | **ControlsHUD** — With existing controls |
| Transition | **Animate** — Smooth position transitions |
| Empty children | **Different styling** — Atoms have distinct non-expandable appearance |
| Cycles | **Not applicable** — Graphs guaranteed acyclic |
| Performance | **Smart culling** — Hide off-screen duplicates |
