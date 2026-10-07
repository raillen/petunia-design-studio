# 18.22 — Refine Selection Workspace Specification

# Entry

Action [ptnd.photo](http://ptnd.photo).refine_selection enters a scoped workspace using immutable source selection + composited image snapshot.

# State machine

SourceReady -> AnalyzeEdges -> InteractiveRefine -> OutputPreview -> Apply/Cancel.

# Controls

Edge radius, smooth, feather, contrast, edge-aware brush, foreground/background/unknown hints. Decontaminate only if algorithm/output semantics are explicitly implemented.

# Preview modes

Overlay, black/white matte, on white, on black, original composite. Preview modes are view-only.

# Brush hints

Refine brush edits classification hints, not canonical pixels. Stroke undo inside workspace may use local history independent from document history.

# Output

Replace Selection, Pixel Mask, New Layer With Mask and other explicitly supported destinations. Apply creates one document transaction.

# Cancellation

Cancel discards all workspace-derived state and restores original selection exactly.

# Jobs

Analysis/refinement is cancellable. If document source changes, either stay pinned to source snapshot with warning or require refresh before apply.

# Accessibility

All numeric controls keyboard accessible; brush-only refinements can be supplemented by region/classification controls when feasible.

# Tests

Hair/fur, soft transparency, high-res, cancellation, source changes, output target correctness and repeatability.