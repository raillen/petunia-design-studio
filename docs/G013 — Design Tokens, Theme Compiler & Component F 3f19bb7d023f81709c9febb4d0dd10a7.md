# G013 — Design Tokens, Theme Compiler & Component Foundation

# Goal

Create enforceable Petunia visual foundation before feature widgets proliferate.

# Depends

G001.

# Primary

design-system-engineer + ui-component-engineer; accessibility-reviewer.

# Skills

design-system, design-tokens, ui-implementation, accessibility, contrast.

# Deliverables

Token JSON schemas/primitives/semantic/component states; Dark/Light baseline; Comfortable/Compact metrics; token compiler to Python/Qt; QPalette mapping; limited QSS foundation; IconRegistry; component gallery shell; Button/IconButton/ToolButton/NumericField/Section basic components.

# Acceptance

Theme switch runtime; no hard-coded product chrome colors in sample components; controls meet focus/contrast/keyboard baseline and render at 100/150/200% DPI.

# Tests

Token validation, screenshot goldens, contrast, density, icon cache and missing token detection.

# Non-goals

Full app shell, all custom controls, artwork renderer.