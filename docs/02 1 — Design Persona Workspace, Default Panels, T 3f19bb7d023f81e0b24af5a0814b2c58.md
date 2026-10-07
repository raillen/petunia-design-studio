# 02.1 — Design Persona Workspace, Default Panels, Tool Groups & Context Model

# Default workspace

Left tool rail: Move, Node, Pen group, Pencil/Vector Brush, Shape group, Corner/Knife, Fill/Transparency, Text group, Surface, Measure, Hand/Zoom.

Right docks: Layers + Appearance, Properties, Color/Swatches, Stroke, Transform/Align.

Secondary tabs: Typography/Paragraph, Assets/Symbols/Styles, Pathfinder, Data Merge, Export.

# Context model

Active tool owns ToolContextSchema; selection contributes PropertySchemas. Context toolbar renders high-frequency overlap while Properties remains complete authority.

# Selection

Design supports object, node/path sub-selection, appearance entry selection and Surface selection. Each mode has clear visual state and compatible actions.

# Workspace transitions

Entering text edit or node edit does not replace whole UI; context changes locally. Persona switch retains canvas/selection where valid.

# Professional density

Common operations within one click/shortcut; advanced options no more than one panel/popover deeper when possible.

# Keyboard

V Move, A Node, P Pen, T text family, G gradient/fill, Space temporary Hand and customizable mappings. Exact defaults versioned as shortcut profile.

# Accessibility

Tool groups expose child tools through keyboard/menu, not long-press only. Active mode announced.