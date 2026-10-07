# 09.11.1.3 — PTND v1 Vector, Shape, Appearance, Text & Effect Object Schemas

# VectorPath

Fields: id/type, transform, contours with ContourId/closed/nodes, FillRule, AppearanceStack.

Node: NodeId, anchor[x,y], handles optional, NodeKind.

# ParametricShape

type = ptnd.shape.<kind>; shapeVersion; parameters object typed per kind; transform; appearance. Rectangle stores width/height + four radii with corner mode; ellipse stores width/height and arc/pie params only if semantically needed.

# Group

children order + isolation/passThrough semantic + group appearance/effects/masks.

# AppearanceStack

ordered entries:

FillEntry, StrokeEntry, EffectEntry or typed nested references. EntryId stable.

Fill: solid/globalSwatch/gradient/imagePattern.

Stroke: paint + width + alignment + cap/join/miter + dash + profile + arrowheads.

Gradient: kind, transform/geometry, spread, ordered GradientStop {StopId, position, midpoint, ColorSource, opacity}.

# Effect

EffectKind + effectVersion + typed params + enabled + blend/opacity/mask optional. Unknown built-in effect version handled by migration/capability.

# Text

TextObject references StoryId and frame/artistic/path layout descriptor. Story resources live in text section to allow multiple linked frames.

Style runs reference CharacterStyleId/ParagraphStyleId and local overrides.

# Symbol

SymbolDefinition owns subtree/definition root; SymbolInstance refs SymbolId + override map of stable PropertyPath→value.

# Style

StyleId, kind object/character/paragraph, parent style optional acyclic, property map using stable PropertyIds.

# Serialization philosophy

Never serialize renderer tessellation, glyph IDs tied to current font cache, GPU resources, bounds or UI selection.

# Tests

Every object minimal/full JSON fixture, unknown enum, type-version migration, roundtrip stable IDs and appearance order.