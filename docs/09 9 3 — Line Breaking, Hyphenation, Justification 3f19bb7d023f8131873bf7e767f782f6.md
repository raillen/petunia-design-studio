# 09.9.3 — Line Breaking, Hyphenation, Justification, Tabs, Leading, Columns & Frame Flow

# Layout units

All text layout uses document-space units with double precision; font design units converted through scale.

# Line breaking

Use Unicode line break opportunities plus language/hyphenation provider. Measure shaped advances against available line intervals from frame geometry/wrap exclusions.

# Hyphenation

Language-specific dictionary/provider returns candidates with penalties/min word/left/right constraints. User soft hyphen and nonbreaking characters override automatic behavior.

# Justification

Distribute residual width through prioritized opportunities: word spaces, letter spacing and optional glyph scaling only according paragraph settings. Last-line policy explicit.

# Leading

Auto leading based on font metrics × configurable factor or exact leading. Baseline-to-baseline semantics documented independent from font ascender.

# Tabs

Tab stops left/right/center/decimal with leaders. Default tab interval when none specified. Decimal separator locale-aware.

# Frame

Rectangular baseline; shape frame/wrap may provide line interval geometry. Insets reduce available region. Columns split region with gutter and balance policy if supported.

# Flow

Story can flow through linked TextFrames in ordered acyclic graph/list. Layout consumes text range, frame produces consumed range + overset status. Editing upstream invalidates downstream frames incrementally.

# Keep controls

Keep with next, keep lines together, widow/orphan and paragraph spacing become layout constraints when scope enabled.

# Tests

Narrow frames, long unbreakable strings, hyphenation languages, tabs, justified lines, columns, overset, linked frames and wrap obstacles.