# 10.7.2 — Margins, Columns, Baseline Grid, Guides & Layout Constraint Semantics

# Margins

Top/right/bottom/left values, linked option; can be negative only if explicitly allowed. Margin guides derived, snap targets, not ordinary Guide objects.

# Columns

count ≥1, gutter, optional unequal explicit columns future. Derived column bounds inside margin/content box.

# Baseline grid

start offset, spacing, visibility/snap/text-align flags; scope document or Surface. Paragraph AlignToBaselineGrid consumes baseline positions.

# Guides

GuideId, axis/position or future angled guide, scope, lock, visibility, color metadata and snap-enabled. Drag ruler creates.

# Layout constraints

These settings guide/snap/layout but do not auto-move arbitrary objects unless explicit layout feature is introduced.

# UI

Manage Guides dialog/table can bulk edit exact positions; canvas drag respects locks.

# Tests

unit conversion, per-Surface guides, hidden+snap policy, columns with resize and baseline text alignment.