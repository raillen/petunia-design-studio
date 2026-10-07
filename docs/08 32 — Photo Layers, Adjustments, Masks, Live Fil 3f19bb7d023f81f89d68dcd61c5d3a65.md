# 08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX

# Layer semantics

Adjustment/Live Filter shows distinct type icon, enable state, mask thumbnail and nesting/target. Drag placement preview shows whether effect applies clipped/nested vs globally.

# Adjustment editor

Parameters can live Properties/panel/popover, but selecting adjustment always reveals controls. Reset, enable/disable and mask creation visible.

# Live filter

Effect node shows live badge. Destructive Apply to Pixels is separate command and wording.

# Masks

Thumbnail border/indicator shows whether mask or pixels are current edit target. Clicking thumbnail switches target; status/context repeats it.

# Channels

Visibility toggles are view state; active editable channel is separate selected state. Composite-only operations explain restrictions.

# Histogram

Scope selector + channels; stale/recomputing indicator subtle. No flicker from every minor pointer sample due coalescing.

# Analysis overlays

Clipping/gamut/mask overlays have explicit toolbar/status toggle and known overlay precedence.

# Accessibility

Mask/channel target selection announced; graph-based Curves has numeric point table alternative and keyboard manipulation.