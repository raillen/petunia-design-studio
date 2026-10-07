# 10 — Functional Tool & Engine Atlas

# Rule

Nenhuma ferramenta é apenas um botão. Cada ferramenta deve ter contrato funcional + engine semantics + GUI semantics + accessibility + automation + persistence + tests.

# Tool contract template

Para cada Tool/Function documentar:

1. intent e scope;
2. activation paths;
3. selection/context preconditions;
4. pointer/pen/key states;
5. modifiers;
6. context toolbar;
7. on-canvas preview/HUD;
8. Properties panel;
9. Command payload;
10. undo grouping;
11. core implementation;
12. Python↔C++ boundary;
13. snapping/hit-testing;
14. serialization effects;
15. plugin/MCP exposure;
16. failure/cancel behavior;
17. accessibility;
18. performance target;
19. fixtures/tests;
20. destructive/non-destructive classification.

# Coverage

O atlas cobre seleção, paths, shapes, appearance, layers/resources, text/layout, perspective/warp, Photo tools, adjustments, Data Merge, Design↔Photo e gauntlet.

[10.1 — Selection, Transform, Arrange, Align, Distribution & Snapping](10%201%20%E2%80%94%20Selection,%20Transform,%20Arrange,%20Align,%20Distr%203f19bb7d023f81449196e01b3714d308.md)

[10.2 — Pen, Pencil, Nodes, Path Editing, Corners, Knife & Scissors](10%202%20%E2%80%94%20Pen,%20Pencil,%20Nodes,%20Path%20Editing,%20Corners,%20%203f19bb7d023f811a81f4c0c21ccf79ee.md)

[10.3 — Shapes, Boolean Operations, Shape Builder, Offset, Outline & Compound Paths](10%203%20%E2%80%94%20Shapes,%20Boolean%20Operations,%20Shape%20Builder,%20%203f19bb7d023f81988fc4e36703e288a9.md)

[10.4 — Fill, Stroke, Gradients, Transparency, Appearance, Effects & Blend Modes](10%204%20%E2%80%94%20Fill,%20Stroke,%20Gradients,%20Transparency,%20Appe%203f19bb7d023f818b9ce8cb74893e0a25.md)

[10.5 — Layers, Groups, Clips, Masks, Symbols, Styles, Assets & Resource Libraries](10%205%20%E2%80%94%20Layers,%20Groups,%20Clips,%20Masks,%20Symbols,%20Styl%203f19bb7d023f81e48af5d9327b003d9e.md)

[10.6 — Typography Tools, Artistic Text, Text Frames, Text-on-Path & Layout Editing](10%206%20%E2%80%94%20Typography%20Tools,%20Artistic%20Text,%20Text%20Frame%203f19bb7d023f819ab08ddaefe0a0104b.md)

[10.7 — Surfaces, Artboards, Pages, Guides, Margins, Columns, Bleed & Lightweight Layout](10%207%20%E2%80%94%20Surfaces,%20Artboards,%20Pages,%20Guides,%20Margins%203f19bb7d023f81dbbd71e3b947892c7d.md)

[10.8 — Perspective, Projective Transform, Perspective Grid, Warp & Envelope](10%208%20%E2%80%94%20Perspective,%20Projective%20Transform,%20Perspect%203f19bb7d023f81a1a14fd7e669a2c03e.md)

[10.9 — Photo Selection, Brush, Eraser, Crop, Gradient, Clone, Heal & Retouch Tools](10%209%20%E2%80%94%20Photo%20Selection,%20Brush,%20Eraser,%20Crop,%20Gradi%203f19bb7d023f81b79e9ce619b2fcf674.md)

[10.10 — Photo Masks, Adjustments, Live Filters, Channels, Histogram & Analysis](10%2010%20%E2%80%94%20Photo%20Masks,%20Adjustments,%20Live%20Filters,%20Ch%203f19bb7d023f81bea935d673b783ab71.md)

[10.11 — Variable Data / Data Merge Engine: Sources, Bindings, Expressions, Generation & Preflight](10%2011%20%E2%80%94%20Variable%20Data%20Data%20Merge%20Engine%20Sources,%20B%203f19bb7d023f8119bb41cff7b61112f3.md)

[10.12 — Design ↔ Photo Interoperability, Conversion Commands & Shared Composition Rules](10%2012%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Interoperability,%20Conversio%203f19bb7d023f8173b675c4c5bf6ae860.md)

[10.13 — Functional Gauntlet, Edge-Case Matrix & Tool Definition of Done](10%2013%20%E2%80%94%20Functional%20Gauntlet,%20Edge-Case%20Matrix%20&%20To%203f19bb7d023f814eb8cec613582e54dc.md)