# 08.21 — Design System Governance, Token Enforcement, Customization & UI Portability

# Governance

Design System Engineer owns token schema/component contracts. Feature engineers consume primitives; they do not fork local Button/Field variants for convenience.

# Token enforcement

Static scan/review rejects literal production UI colors/spacing where token exists. Exceptions: artwork/document content and truly computed values.

# Components

Button, IconButton, ToolButton, SegmentedControl, NumericField, SliderField, ColorWell, Combo, TreeRow, PanelTab, SectionHeader, Popover, Notification, Progress and others have state matrices.

# Customization

Theme/density/workspace are user overrides within semantic token ranges. Plugins use host-rendered controls and inherit tokens automatically.

# Portability

Core action/property/panel schema is toolkit-neutral. Qt component layer interprets it. A future shell can implement same semantic contracts without changing document/plugin/MCP.

# Versioning

Token/component breaking changes version resource schema; plugin declarative UI references semantic control types, not Python class names.

# Visual QA

Component gallery/storybook-like internal app renders all states/themes/densities/DPI for regression and accessibility audit.