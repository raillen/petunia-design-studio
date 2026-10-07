# 18 — Individual Tool Specifications

# Purpose

Provide one implementation-grade contract per interactive tool. Each page is normative for activation, state machine, pointer/pen/keyboard behavior, modifiers, context toolbar, HUD/overlays, Commands, undo grouping, core services, accessibility, performance and tests.

# Universal tool requirements

Every tool page must define:

ToolId, ActionId(s), shortcut defaults, accepted targets, state machine, pointer capture, temporary modifiers, preview vs commit, cancel/lost-capture behavior, context schema, PropertyIds, cursor, status hints, overlays, Command payload, serialization impact, MCP/plugin exposure, accessibility route, performance budget and fixtures.

# State machine rule

Booleans like is_dragging/is_editing are not sufficient when more than two phases exist. Use explicit states/events.

# Interaction parity

Toolbar activation, shortcut activation and MCP/tool activation resolve the same ToolId. Pointer gestures eventually produce the same semantic Commands available from numeric/property editing when applicable.

# No hidden conversion

If tool requires rasterization/expansion/target switch, it must be explicit and independently undoable.

[18.01 — Move / Select Tool Specification](18%2001%20%E2%80%94%20Move%20Select%20Tool%20Specification%203f19bb7d023f81f1b112daaf96ec908d.md)

[18.02 — Node Tool Specification](18%2002%20%E2%80%94%20Node%20Tool%20Specification%203f19bb7d023f8181b363e9a3d7b0b37e.md)

[18.03 — Pen Tool Specification](18%2003%20%E2%80%94%20Pen%20Tool%20Specification%203f19bb7d023f8167b7aadb5e596f6317.md)

[18.04 — Pencil Tool Specification](18%2004%20%E2%80%94%20Pencil%20Tool%20Specification%203f19bb7d023f814fb704d9cc9649e67e.md)

[18.05 — Vector Brush Tool Specification](18%2005%20%E2%80%94%20Vector%20Brush%20Tool%20Specification%203f19bb7d023f815b8e43e8ada21c9fab.md)

[18.06 — Rectangle / Rounded Rectangle Tool Specification](18%2006%20%E2%80%94%20Rectangle%20Rounded%20Rectangle%20Tool%20Specifica%203f19bb7d023f81f08528c8b9d533d3b5.md)

[18.07 — Ellipse / Arc / Pie / Donut Tool Specification](18%2007%20%E2%80%94%20Ellipse%20Arc%20Pie%20Donut%20Tool%20Specification%203f19bb7d023f816baa1fc7fd9b612d02.md)

[18.08 — Polygon / Star / Cog Tool Specification](18%2008%20%E2%80%94%20Polygon%20Star%20Cog%20Tool%20Specification%203f19bb7d023f8151a75bd1bd839a9eb7.md)

[18.09 — Shape Builder Tool Specification](18%2009%20%E2%80%94%20Shape%20Builder%20Tool%20Specification%203f19bb7d023f81f08fa3c46b17ddb652.md)

[18.10 — Boolean Operations Specification](18%2010%20%E2%80%94%20Boolean%20Operations%20Specification%203f19bb7d023f817a80dfc1f0eab1fa13.md)

[18.11 — Fill / Gradient Tool Specification](18%2011%20%E2%80%94%20Fill%20Gradient%20Tool%20Specification%203f19bb7d023f8193bd11e542d6a52673.md)

[18.12 — Transparency Tool Specification](18%2012%20%E2%80%94%20Transparency%20Tool%20Specification%203f19bb7d023f813591e8fdb855181541.md)

[18.13 — Corner Tool Specification](18%2013%20%E2%80%94%20Corner%20Tool%20Specification%203f19bb7d023f813fbedadd37ff4ce3be.md)

[18.14 — Knife / Scissors Tool Specification](18%2014%20%E2%80%94%20Knife%20Scissors%20Tool%20Specification%203f19bb7d023f8142acc7d00df0994830.md)

[18.15 — Stroke Width / Profile Tool Specification](18%2015%20%E2%80%94%20Stroke%20Width%20Profile%20Tool%20Specification%203f19bb7d023f81e19bd3fcf0b0f991fe.md)

[18.16 — Eyedropper / Color Picker Tool Specification](18%2016%20%E2%80%94%20Eyedropper%20Color%20Picker%20Tool%20Specification%203f19bb7d023f8129be92da9d4c9c8076.md)

[18.17 — Artistic Text Tool Specification](18%2017%20%E2%80%94%20Artistic%20Text%20Tool%20Specification%203f19bb7d023f814a89abd8c7908677bc.md)

[18.18 — Frame Text Tool Specification](18%2018%20%E2%80%94%20Frame%20Text%20Tool%20Specification%203f19bb7d023f816f9d16ceae98805d25.md)

[18.19 — Text-on-Path Tool Specification](18%2019%20%E2%80%94%20Text-on-Path%20Tool%20Specification%203f19bb7d023f811784bbf864811b2c27.md)

[18.20 — Surface / Artboard / Page Tool Specification](18%2020%20%E2%80%94%20Surface%20Artboard%20Page%20Tool%20Specification%203f19bb7d023f8110982bf3cb433f2196.md)

[18.21 — Measure Tool Specification](18%2021%20%E2%80%94%20Measure%20Tool%20Specification%203f19bb7d023f81b7afe4f9ecc12a3fee.md)

[18.22 — Hand / Pan & Zoom Tools Specification](18%2022%20%E2%80%94%20Hand%20Pan%20&%20Zoom%20Tools%20Specification%203f19bb7d023f819d9ebacd08f07ffa9d.md)

[18.23 — Pixel Brush Tool Specification](18%2023%20%E2%80%94%20Pixel%20Brush%20Tool%20Specification%203f19bb7d023f81f69289ca114a8a3077.md)

[18.24 — Eraser Tool Specification](18%2024%20%E2%80%94%20Eraser%20Tool%20Specification%203f19bb7d023f81db9e0ed782883a0faf.md)

[18.25 — Marquee Selection Tools Specification](18%2025%20%E2%80%94%20Marquee%20Selection%20Tools%20Specification%203f19bb7d023f815db56dc4adc3a1e4d1.md)

[18.26 — Lasso / Polygonal / Magnetic Selection Tools Specification](18%2026%20%E2%80%94%20Lasso%20Polygonal%20Magnetic%20Selection%20Tools%20S%203f19bb7d023f81968256d969fab27376.md)

[18.27 — Selection Brush Tool Specification](18%2027%20%E2%80%94%20Selection%20Brush%20Tool%20Specification%203f19bb7d023f8137b0b6f3080f589677.md)

[18.28 — Refine Selection Workspace Specification](18%2028%20%E2%80%94%20Refine%20Selection%20Workspace%20Specification%203f19bb7d023f81dbba18cf832688f142.md)

[18.29 — Crop / Straighten Tool Specification](18%2029%20%E2%80%94%20Crop%20Straighten%20Tool%20Specification%203f19bb7d023f81308d80ffe8632fc962.md)

[18.30 — Flood Fill Tool Specification](18%2030%20%E2%80%94%20Flood%20Fill%20Tool%20Specification%203f19bb7d023f8111ab85ec33128b1288.md)

[18.31 — Raster Gradient Tool Specification](18%2031%20%E2%80%94%20Raster%20Gradient%20Tool%20Specification%203f19bb7d023f81c99386cf4506c69bab.md)

[18.32 — Clone Stamp Tool Specification](18%2032%20%E2%80%94%20Clone%20Stamp%20Tool%20Specification%203f19bb7d023f8160a62ede08928f66c4.md)

[18.33 — Healing Tool Specification](18%2033%20%E2%80%94%20Healing%20Tool%20Specification%203f19bb7d023f8160b100da2a3f8472cf.md)

[18.34 — Inpainting Tool Specification](18%2034%20%E2%80%94%20Inpainting%20Tool%20Specification%203f19bb7d023f810a9863edae3cf029f3.md)

[18.35 — Blemish Removal Tool Specification](18%2035%20%E2%80%94%20Blemish%20Removal%20Tool%20Specification%203f19bb7d023f81c78d59ef5815b9e7ca.md)

[18.36 — Dodge / Burn Tool Specification](18%2036%20%E2%80%94%20Dodge%20Burn%20Tool%20Specification%203f19bb7d023f8126ad33cfc2fb143336.md)

[18.37 — Smudge / Blur / Sharpen Brush Specification](18%2037%20%E2%80%94%20Smudge%20Blur%20Sharpen%20Brush%20Specification%203f19bb7d023f8174a804ef1a96239ea7.md)