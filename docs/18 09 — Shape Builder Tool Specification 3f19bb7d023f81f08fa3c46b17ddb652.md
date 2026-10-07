# 18.09 — Shape Builder Tool Specification

# Preconditions

At least two eligible vector shapes/paths selected.

# Session

Entering tool snapshots source revisions and builds atomic face arrangement in native geometry engine. Complex setup may background with progress.

# Hover/gesture

Hover highlights face. Add/Subtract mode shown in cursor/HUD. Drag paints across visited faces without duplicate toggles.

# Commit

Reconstruct boundary geometry from chosen faces. Options: replace sources, Keep Originals, preserve styles by provenance where possible.

# Staleness

Any source geometry revision change invalidates session and requires recompute before commit.

# Commands

ShapeBuilderCommit validated against source IDs/revisions.

# Accessibility/tests

Boolean commands are non-spatial alternatives. Test holes, shared edges, source invalidation, drag across regions and style provenance.