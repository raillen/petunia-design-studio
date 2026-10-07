# 18 — Individual Tool Specifications

# Authority

Cada ferramenta do Petunia possui uma especificação individual. As páginas deste capítulo são a autoridade operacional para ToolController, contexto GUI, Command payload, acessibilidade e testes.

# Mandatory template

ToolId; intent; eligible targets; activation; cursor; state machine; pointer/pen events; keyboard/modifiers; context toolbar schema; Properties schema; HUD/overlays; snapping/hit test; preview model; commit/cancel; Commands; undo; Python/C++ split; serialization impact; MCP/plugin exposure; errors/disabled reasons; accessibility; performance budget; fixtures; edge cases.

# Rule

Quando esta página conflitar com uma descrição resumida em 08/10, a individual tool spec deve ser reconciliada e ambas atualizadas; nenhuma implementação deve inferir comportamento ausente.

[18.01 — Move / Selection Tool](18%2001%20%E2%80%94%20Move%20Selection%20Tool%203f19bb7d023f815bb7afd0b742cbe735.md)

[18.02 — Node Tool](18%2002%20%E2%80%94%20Node%20Tool%203f19bb7d023f8104b02cc67b675b57bc.md)

[18.03 — Pen Tool](18%2003%20%E2%80%94%20Pen%20Tool%203f19bb7d023f812b8af4f7d178ae7d7a.md)

[18.04 — Pencil Tool](18%2004%20%E2%80%94%20Pencil%20Tool%203f19bb7d023f81dcb5edc79ab6a5c718.md)

[18.05 — Vector Brush Tool](18%2005%20%E2%80%94%20Vector%20Brush%20Tool%203f19bb7d023f818c8f5fe8a347f70ca0.md)

[18.06 — Parametric Shape Tool Family](18%2006%20%E2%80%94%20Parametric%20Shape%20Tool%20Family%203f19bb7d023f815b9771e3eb96c70284.md)

[18.07 — Corner Tool](18%2007%20%E2%80%94%20Corner%20Tool%203f19bb7d023f81b09fa5ea7a04f8d7bd.md)

[18.08 — Knife & Scissors Tools](18%2008%20%E2%80%94%20Knife%20&%20Scissors%20Tools%203f19bb7d023f819aa86dd3960a5d6345.md)

[18.09 — Shape Builder Tool](18%2009%20%E2%80%94%20Shape%20Builder%20Tool%203f19bb7d023f819589cde764c95cd32f.md)

[18.10 — Fill / Gradient Tool](18%2010%20%E2%80%94%20Fill%20Gradient%20Tool%203f19bb7d023f81199b92eb5b52015b87.md)

[18.11 — Transparency Tool](18%2011%20%E2%80%94%20Transparency%20Tool%203f19bb7d023f816c9bd4f141b62ee6cd.md)

[18.12 — Stroke Width / Profile Tool](18%2012%20%E2%80%94%20Stroke%20Width%20Profile%20Tool%203f19bb7d023f81fe9bc7ff2058849d61.md)

[18.13 — Eyedropper / Color Picker Tool](18%2013%20%E2%80%94%20Eyedropper%20Color%20Picker%20Tool%203f19bb7d023f81bcb8aac1c76870eabf.md)

[18.14 — Artistic Text Tool](18%2014%20%E2%80%94%20Artistic%20Text%20Tool%203f19bb7d023f81de98bcfa869e9c869b.md)

[18.15 — Frame Text Tool](18%2015%20%E2%80%94%20Frame%20Text%20Tool%203f19bb7d023f81cd8d3efc170409ce2a.md)

[18.16 — Text on Path Tool](18%2016%20%E2%80%94%20Text%20on%20Path%20Tool%203f19bb7d023f819da750e9feb740fa44.md)

[18.17 — Surface / Artboard Tool](18%2017%20%E2%80%94%20Surface%20Artboard%20Tool%203f19bb7d023f81fdb711fbfd99ab161d.md)

[18.18 — Measure Tool](18%2018%20%E2%80%94%20Measure%20Tool%203f19bb7d023f81ec8d30dda64c7783eb.md)

[18.19 — Hand / Pan Tool](18%2019%20%E2%80%94%20Hand%20Pan%20Tool%203f19bb7d023f8106bd1cec7f52b1d995.md)

[18.20 — Zoom Tool & View Navigation Actions](18%2020%20%E2%80%94%20Zoom%20Tool%20&%20View%20Navigation%20Actions%203f19bb7d023f81b4852ef520b51e01ba.md)

[18.21 — Pixel Brush Tool](18%2021%20%E2%80%94%20Pixel%20Brush%20Tool%203f19bb7d023f81ddbafbda6f5618224e.md)

[18.22 — Eraser Tool](18%2022%20%E2%80%94%20Eraser%20Tool%203f19bb7d023f810f81affcfbc5afdb0b.md)

[18.23 — Marquee Selection Tool Family](18%2023%20%E2%80%94%20Marquee%20Selection%20Tool%20Family%203f19bb7d023f811c8871f2e207bec45e.md)

[18.24 — Lasso / Polygon / Magnetic Selection Tools](18%2024%20%E2%80%94%20Lasso%20Polygon%20Magnetic%20Selection%20Tools%203f19bb7d023f812fac1fc96dabeafedd.md)

[18.25 — Selection Brush Tool](18%2025%20%E2%80%94%20Selection%20Brush%20Tool%203f19bb7d023f8143a57df5b4f2ca76e3.md)

[18.26 — Refine Selection Tool / Workspace](18%2026%20%E2%80%94%20Refine%20Selection%20Tool%20Workspace%203f19bb7d023f8177a8dbcad8a7a6cbff.md)

[18.27 — Crop / Straighten Tool](18%2027%20%E2%80%94%20Crop%20Straighten%20Tool%203f19bb7d023f8191a629c4fc4d692b8b.md)

[18.28 — Clone Tool](18%2028%20%E2%80%94%20Clone%20Tool%203f19bb7d023f81349bcadd600eff810f.md)

[18.29 — Healing Tool](18%2029%20%E2%80%94%20Healing%20Tool%203f19bb7d023f810fa3edffc9c850b2cf.md)

[18.30 — Inpainting Tool](18%2030%20%E2%80%94%20Inpainting%20Tool%203f19bb7d023f817a9c3ddb70f6ede432.md)

[18.31 — Blemish Removal Tool](18%2031%20%E2%80%94%20Blemish%20Removal%20Tool%203f19bb7d023f814ca628d753c0839353.md)

[18.32 — Dodge / Burn Tools](18%2032%20%E2%80%94%20Dodge%20Burn%20Tools%203f19bb7d023f814ca2d9c62e4e3952f2.md)

[18.33 — Smudge Tool](18%2033%20%E2%80%94%20Smudge%20Tool%203f19bb7d023f81c189fcc9ae73907d26.md)

[18.34 — Blur / Sharpen Brush Tools](18%2034%20%E2%80%94%20Blur%20Sharpen%20Brush%20Tools%203f19bb7d023f811ea9e4fe6e32529ab1.md)

[18.35 — Flood Fill Tool](18%2035%20%E2%80%94%20Flood%20Fill%20Tool%203f19bb7d023f81f281a4e254494e4f8a.md)

[18.36 — Raster Gradient Tool](18%2036%20%E2%80%94%20Raster%20Gradient%20Tool%203f19bb7d023f81f787a8d41b7968dffe.md)