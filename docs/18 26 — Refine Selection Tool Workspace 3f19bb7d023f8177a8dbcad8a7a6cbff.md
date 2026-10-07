# 18.26 — Refine Selection Tool / Workspace

# Entry

Available when nonempty PixelSelection exists. Invoking creates a RefineSession over immutable source selection + image/composite revision.

# Workspace

Dedicated temporary view state with overlay modes: selection, mask, black/white, on-layers and edge visualization.

# Parameters

edge radius, feather, smooth, contrast/threshold-like control, decontamination if implemented, edge brush size/mode and output target.

# Edge brush

User paints regions requiring foreground/background/unknown reevaluation; hints remain staged session data.

# Preview

Native refine engine may use reduced resolution while interacting; Final Apply recomputes full quality before atomic commit.

# Output

Replace Selection, New Pixel Mask, Mask on current layer/group, or New Layer with Mask according supported choices.

# Cancel

Leaves original selection unchanged and releases staged caches.

# Accessibility

All numeric controls keyboard accessible; edge painting remains pointer interaction but output/overlay states announced.

# Tests

Hair/fur fixtures, translucent edges, cancel, high-res memory, output modes, stale source and full-quality convergence.