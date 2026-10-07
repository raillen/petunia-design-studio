# 18.09 — Shape Builder Tool

# Preconditions

Two or more selected eligible vector fills. Strokes participate only under explicit expanded-stroke option.

# Preparation

Native RegionBuilder computes planar atomic regions plus provenance. Preparation is cancellable and cached by geometry revision.

# Interaction

Hover highlights one atomic region. Click selects; drag paints region set. Add mode default; modifier/context switches subtract/delete. Esc cancels staged region selection.

# Output

Single compound result or separate shapes; keep originals optional; deterministic appearance source rule.

# Commit

Reconstruct region boundaries, simplify conservatively, validate topology and mutate in one transaction.

# Performance

After preparation, hover hit-test remains interactive. Excessive region count triggers complexity diagnostic rather than locking UI.

# Accessibility

Region list/commands can expose deterministic region identifiers in advanced/semantic inspection; common workflows also available through Boolean actions.

# Tests

Nested holes, coincident edges, 100+ shapes, sweep, subtract, provenance, cancel, undo.