# 18 — Individual Tool Specifications: State Machines, Commands, GUI & Tests

# Authority

Este capítulo transforma cada ferramenta em um contrato implementável isoladamente. Nenhuma ferramenta pode depender de comportamento implícito em widget.

# Mandatory template

Cada ToolSpec deve conter:

ToolId; Persona; default shortcut; accepted targets; state machine; pointer/pen events; modifiers; cursor; context schema; HUD/overlays; Properties integration; snapping/hit-testing; Commands; staged preview; commit/cancel; undo grouping; serialization impact; plugin/MCP exposure; accessibility; performance budget; disabled reasons; edge cases; deterministic tests.

# Shared states

Inactive, Hover, Armed, ActiveGesture, EditingSubstate, CommitPending, Cancelled. Tools can refine but must define transitions for pointer capture loss, Esc, Persona switch, document close and selection invalidation.

# Shared invariants

A gesture produces zero or one logical document transaction; preview does not pollute history; cursor/HUD never become semantic truth; numeric edit and pointer edit reach same Command; unsupported targets are disabled with reason rather than silently converted.

[18.01 — Move Tool Specification](18%2001%20%E2%80%94%20Move%20Tool%20Specification%203f19bb7d023f81e5b61ffd3784a41c7f.md)

[18.02 — Node Tool Specification](18%2002%20%E2%80%94%20Node%20Tool%20Specification%203f19bb7d023f813eba51d1ca9b9cbc12.md)

[18.03 — Pen Tool Specification](18%2003%20%E2%80%94%20Pen%20Tool%20Specification%203f19bb7d023f81548982d30f85e0d5d8.md)

[18.04 — Pencil Tool Specification](18%2004%20%E2%80%94%20Pencil%20Tool%20Specification%203f19bb7d023f814cbecad7dad5c6705c.md)

[18.05 — Vector Brush Tool Specification](18%2005%20%E2%80%94%20Vector%20Brush%20Tool%20Specification%203f19bb7d023f812f8089f99077ee8526.md)

[18.06 — Parametric Shape Tool Family Specification](18%2006%20%E2%80%94%20Parametric%20Shape%20Tool%20Family%20Specification%203f19bb7d023f81d082a3e2c73267cbc2.md)

[18.07 — Corner Tool Specification](18%2007%20%E2%80%94%20Corner%20Tool%20Specification%203f19bb7d023f81e1931bfee7e53c8a70.md)

[18.08 — Knife & Scissors Tool Specification](18%2008%20%E2%80%94%20Knife%20&%20Scissors%20Tool%20Specification%203f19bb7d023f815b9f12ead0e657d506.md)

[18.09 — Shape Builder Tool Specification](18%2009%20%E2%80%94%20Shape%20Builder%20Tool%20Specification%203f19bb7d023f8144b389cae9ecca61f2.md)

[18.10 — Fill & Gradient Tool Specification](18%2010%20%E2%80%94%20Fill%20&%20Gradient%20Tool%20Specification%203f19bb7d023f8197bbd0e67569b03f7c.md)

[18.11 — Transparency Tool Specification](18%2011%20%E2%80%94%20Transparency%20Tool%20Specification%203f19bb7d023f813ba8a7ef5e6dbbb55a.md)

[18.12 — Eyedropper / Color Picker Tool Specification](18%2012%20%E2%80%94%20Eyedropper%20Color%20Picker%20Tool%20Specification%203f19bb7d023f815e8848de2af28f39fd.md)

[18.13 — Artistic Text Tool Specification](18%2013%20%E2%80%94%20Artistic%20Text%20Tool%20Specification%203f19bb7d023f81e28bbfca8fe635410b.md)

[18.14 — Frame Text Tool Specification](18%2014%20%E2%80%94%20Frame%20Text%20Tool%20Specification%203f19bb7d023f81c89d86c85a979525fa.md)

[18.15 — Text-on-Path Tool Specification](18%2015%20%E2%80%94%20Text-on-Path%20Tool%20Specification%203f19bb7d023f81189e08c4e3319d70b5.md)

[18.16 — Surface / Artboard Tool Specification](18%2016%20%E2%80%94%20Surface%20Artboard%20Tool%20Specification%203f19bb7d023f81dd9013f05b71449a11.md)

[18.17 — Measure Tool Specification](18%2017%20%E2%80%94%20Measure%20Tool%20Specification%203f19bb7d023f8178b29fd4e7a13ec677.md)

[18.18 — Hand & Zoom Tool Specifications](18%2018%20%E2%80%94%20Hand%20&%20Zoom%20Tool%20Specifications%203f19bb7d023f81a1937dd3d149be7d16.md)

[18.19 — Pixel Brush Tool Specification](18%2019%20%E2%80%94%20Pixel%20Brush%20Tool%20Specification%203f19bb7d023f814382e9c34743305a12.md)

[18.20 — Eraser Tool Specification](18%2020%20%E2%80%94%20Eraser%20Tool%20Specification%203f19bb7d023f81e3ae56c7a2585c2bae.md)

[18.21 — Raster Selection Tool Family Specification](18%2021%20%E2%80%94%20Raster%20Selection%20Tool%20Family%20Specification%203f19bb7d023f819fbbcdff490a3ab770.md)

[18.22 — Refine Selection Workspace Specification](18%2022%20%E2%80%94%20Refine%20Selection%20Workspace%20Specification%203f19bb7d023f8124a565dbcb46bca691.md)

[18.23 — Crop & Straighten Tool Specification](18%2023%20%E2%80%94%20Crop%20&%20Straighten%20Tool%20Specification%203f19bb7d023f81e49ef0ea78e0c11321.md)

[18.24 — Clone Tool Specification](18%2024%20%E2%80%94%20Clone%20Tool%20Specification%203f19bb7d023f815fbe6ee127674b4414.md)

[18.25 — Heal, Blemish & Inpaint Tool Specifications](18%2025%20%E2%80%94%20Heal,%20Blemish%20&%20Inpaint%20Tool%20Specification%203f19bb7d023f810fbd36f83dd3f6fb60.md)

[18.26 — Dodge, Burn, Smudge, Blur & Sharpen Brush Specifications](18%2026%20%E2%80%94%20Dodge,%20Burn,%20Smudge,%20Blur%20&%20Sharpen%20Brush%20%203f19bb7d023f81f0b484f7a687be7b7a.md)

[18.27 — Flood Fill & Raster Gradient Specifications](18%2027%20%E2%80%94%20Flood%20Fill%20&%20Raster%20Gradient%20Specification%203f19bb7d023f81a0848dc2ed3d96ea13.md)

[18.28 — Photo Color Picker Specification](18%2028%20%E2%80%94%20Photo%20Color%20Picker%20Specification%203f19bb7d023f81088a41e1a31391b37e.md)