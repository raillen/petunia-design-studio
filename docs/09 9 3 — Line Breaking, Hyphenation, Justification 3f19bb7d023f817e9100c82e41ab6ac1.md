# 09.9.3 — Line Breaking, Hyphenation, Justification, Tabs, Columns & Frame Flow

# Paragraph layout

Available line shapes derived from frame column/wrap geometry. Layout consumes shaped runs and breaking opportunities.

# Line breaking

Use Unicode Line Breaking Algorithm plus language-aware hyphenation opportunities. Hard breaks are canonical text; soft wraps derived.

# Hyphenation

Provider returns language dictionary candidates with min word/left/right constraints. Inserted hyphen glyph is layout artifact unless user types explicit hyphen.

# Justification

Define expansion/shrink priorities for word spaces, letter spacing and glyph justification if supported. Last-line rules explicit. Avoid ad-hoc evenly distributing leftover space.

# Tabs

Tab stops: left/right/center/decimal + leader. Default tab interval from paragraph/document settings. Decimal alignment locale-aware.

# Leading

Line height can be explicit, auto percentage or baseline-grid constrained. Ascender/descender/extents account for mixed fonts and inline objects.

# Columns

Frame interior -> N columns with gutter/insets. Story flows column-to-column then linked frame. Column balancing optional/deferred unless specified.

# Text wrap

External objects contribute wrap exclusion paths expanded by offset. Reflow invalidation depends on wrap object bounds/path revision.

# Linked frames

Flow graph is ordered acyclic chain per story. Editing content or frame geometry incrementally relayouts from earliest affected frame forward until stable.

# Overset

Derived state marks first unplaced logical text offset and affected final frame. Preflight links to frame/story.

# Tests

Hyphenation locales, narrow frames, tabs, justified Arabic/Latin, multiple columns, wrap objects moving, linked frames and overset recovery.