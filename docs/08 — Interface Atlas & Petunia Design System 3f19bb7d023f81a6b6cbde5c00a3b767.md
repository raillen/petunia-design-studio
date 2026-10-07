# 08 — Interface Atlas & Petunia Design System

# Authority

Este atlas é a autoridade visual e interacional do Petunia Design Studio. A referência de ergonomia é o Affinity unificado de 2025/2026, traduzido para uma identidade própria. **Não copiar marca, ícones proprietários, assets, trade dress ou medidas cegamente.**

# Objetivos visuais

- desktop profissional, sóbrio e contemporâneo;
- alta densidade informacional com hierarquia clara;
- canvas como região dominante;
- chrome discreto e previsível;
- feedback instantâneo, sem animação ornamental excessiva;
- dark e light themes desde o design system;
- alto contraste opcional;
- métricas tokenizadas e escala HiDPI correta.

# Anatomy macro

1. window/chrome nativo;
2. menu bar quando aplicável;
3. document tabs;
4. workspace/persona + primary toolbar;
5. context toolbar;
6. workspace: tool rail + dock left + canvas + dock right;
7. optional bottom dock;
8. status bar.

# Personas

Design e Photo são workspace modes. Trocar Persona:

- preserva o documento;
- preserva seleção quando válida;
- troca tool registry, context toolbar e panel preset;
- nunca rasteriza, expande ou converte conteúdo;
- permite workspace personalizado por Persona.

# Interaction grammar

Toda tool possui: ToolId, icon, accessible name, default shortcut, cursor, context controls, modifier map, pointer/pen state machine, preview, commit/cancel, HUD, Properties schema, undo grouping e semantic help.

# Density

Comfortable e Compact são perfis de densidade do Design System. Hit target visual pode ser compacto, mas hit region e keyboard focus permanecem acessíveis.

# Atlas coverage

As subpáginas detalham foundations, shell, docking, menus, controls, canvas, cada ferramenta Design/Photo, cada painel, dialogs, files, export, data merge, preferences, accessibility, implementação Qt e conformance.

[08.1 — Visual Foundations, Tokens, Typography, Spacing & Motion](08%201%20%E2%80%94%20Visual%20Foundations,%20Tokens,%20Typography,%20Spa%203f19bb7d023f812a8789cf2332b63a2d.md)

[08.2 — Application Shell, Persona Strip, Toolbars, Tabs & Status Bar](08%202%20%E2%80%94%20Application%20Shell,%20Persona%20Strip,%20Toolbars,%203f19bb7d023f81f48b7cf309c31ff446.md)

[08.3 — Docking, Studio Panels, Floating Palettes & Workspace Presets](08%203%20%E2%80%94%20Docking,%20Studio%20Panels,%20Floating%20Palettes%20&%203f19bb7d023f81e2bf5aea25da47c0d7.md)

[08.4 — Menus, Context Menus, Command Palette, Shortcuts & Action Search](08%204%20%E2%80%94%20Menus,%20Context%20Menus,%20Command%20Palette,%20Shor%203f19bb7d023f8124869af82dbe850a0f.md)

[08.5 — Control Catalog: Fields, Sliders, Pickers, Lists, Trees, Tables & Popovers](08%205%20%E2%80%94%20Control%20Catalog%20Fields,%20Sliders,%20Pickers,%20L%203f19bb7d023f818eaf88d71ed89475cb.md)

[08.6 — Canvas, Rulers, Guides, Grid, Zoom, Selection, Snapping & HUD](08%206%20%E2%80%94%20Canvas,%20Rulers,%20Guides,%20Grid,%20Zoom,%20Selecti%203f19bb7d023f817ca4fdfa0505dcda01.md)

[08.7 — Design Persona Tool-by-Tool GUI Contract](08%207%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20GUI%20Contract%203f19bb7d023f8139a7b9fbd4692603fc.md)

[08.8 — Photo Persona Tool-by-Tool GUI Contract](08%208%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20GUI%20Contract%203f19bb7d023f8164a3c6c0ec484a17dd.md)

[08.9 — Panel Atlas: Layers, Properties, Appearance, Color, Assets, History & Photo Panels](08%209%20%E2%80%94%20Panel%20Atlas%20Layers,%20Properties,%20Appearance,%203f19bb7d023f81e39197ee6d4a027db9.md)

[08.10 — Window & Dialog Atlas](08%2010%20%E2%80%94%20Window%20&%20Dialog%20Atlas%203f19bb7d023f818cae93ee125cec39bf.md)

[08.11 — Welcome, New/Open, Place, Import, Relink, Autosave & Recovery UX](08%2011%20%E2%80%94%20Welcome,%20New%20Open,%20Place,%20Import,%20Relink,%20%203f19bb7d023f81ec8891d39da46980a5.md)

[08.12 — Export, Quick Export, Slices, Batch Export & Preflight UX](08%2012%20%E2%80%94%20Export,%20Quick%20Export,%20Slices,%20Batch%20Export%203f19bb7d023f816391afe7c8c19a725f.md)

[08.13 — Variable Data / Data Merge Window, Bindings, Preview & Generation](08%2013%20%E2%80%94%20Variable%20Data%20Data%20Merge%20Window,%20Bindings,%203f19bb7d023f8145a0c4e9d8252d50ae.md)

[08.14 — Preferences, Customization, Themes, Workspaces, Shortcuts & Help](08%2014%20%E2%80%94%20Preferences,%20Customization,%20Themes,%20Worksp%203f19bb7d023f8164b9a0d2d257204ca5.md)

[08.15 — Accessibility, Keyboard, Pointer, Pen, IME, Localization & HiDPI](08%2015%20%E2%80%94%20Accessibility,%20Keyboard,%20Pointer,%20Pen,%20IME%203f19bb7d023f8116a534e058b0ba0acd.md)

[08.16 — PySide6/Qt Implementation Map & Component Ownership](08%2016%20%E2%80%94%20PySide6%20Qt%20Implementation%20Map%20&%20Component%20%203f19bb7d023f8172a65ad323ca357caf.md)

[08.17 — Affinity 2026 Reference Model, Petunia Divergence & Workflow Vocabulary](08%2017%20%E2%80%94%20Affinity%202026%20Reference%20Model,%20Petunia%20Div%203f19bb7d023f8168a9d1cf6009087d43.md)

[08.18 — UI/UX Conformance, Visual Regression & No-Gap Coverage Ledger](08%2018%20%E2%80%94%20UI%20UX%20Conformance,%20Visual%20Regression%20&%20No-%203f19bb7d023f81e491d2c98d7ef8919b.md)

[08.19 — Microcopy, Labels, Naming, States, Units & Interface Language](08%2019%20%E2%80%94%20Microcopy,%20Labels,%20Naming,%20States,%20Units%20&%203f19bb7d023f812cae1bca4e20575926.md)

[08.20 — UX, Usability, Information Architecture, Cognitive Load & User Research](08%2020%20%E2%80%94%20UX,%20Usability,%20Information%20Architecture,%20C%203f19bb7d023f816aa3b9d6791bfb77b7.md)

[08.21 — Design System Governance, Token Enforcement, Customization & UI Portability](08%2021%20%E2%80%94%20Design%20System%20Governance,%20Token%20Enforcemen%203f19bb7d023f8112acbffed53642fd5c.md)

[08.22 — Affinity Reference Model, Petunia Divergence Rules & Workflow Vocabulary](08%2022%20%E2%80%94%20Affinity%20Reference%20Model,%20Petunia%20Divergen%203f19bb7d023f81ae944cc65ee284a9a0.md)

[08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203f19bb7d023f8198844ccf43ba81e91e.md)

[08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping](08%2024%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20%203f19bb7d023f818dbd52fece19e19784.md)

[08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract](08%2025%20%E2%80%94%20Panels,%20Layers,%20Appearance,%20Colour,%20Stroke%203f19bb7d023f81738dcbd371abc71151.md)

[08.26 — Typography, Text Editing, Layout & Precision UX Contract](08%2026%20%E2%80%94%20Typography,%20Text%20Editing,%20Layout%20&%20Precisi%203f19bb7d023f81d1849be0869377698c.md)

[08.27 — Snapping, Guides, Measurement, Numeric Precision & Spatial Feedback](08%2027%20%E2%80%94%20Snapping,%20Guides,%20Measurement,%20Numeric%20Pre%203f19bb7d023f81ac8a0ad97dc8e2fd18.md)

[08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow](08%2028%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Personas,%20Mixed%20Workspace%20P%203f19bb7d023f81538ff6dc15616806da.md)

[08.29 — Import, Place, Export, Preflight & Workflow Completion UX](08%2029%20%E2%80%94%20Import,%20Place,%20Export,%20Preflight%20&%20Workflo%203f19bb7d023f8112b55ccf325732e5c3.md)

[08.30 — UI/UX Evidence Corpus, Interaction Conformance & Human-Factors Gauntlet](08%2030%20%E2%80%94%20UI%20UX%20Evidence%20Corpus,%20Interaction%20Conform%203f19bb7d023f81a89fe7c0d0d02af652.md)

[08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping](08%2031%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20A%203f19bb7d023f818d96bee42d1ecf2e30.md)

[08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX](08%2032%20%E2%80%94%20Photo%20Layers,%20Adjustments,%20Masks,%20Live%20Fil%203f19bb7d023f81f89d68dcd61c5d3a65.md)

[08.33 — Affinity Tool & Panel Coverage Ledger → Petunia Scope/Authority Matrix](08%2033%20%E2%80%94%20Affinity%20Tool%20&%20Panel%20Coverage%20Ledger%20%E2%86%92%20Pe%203f19bb7d023f812ca28be2066d96b149.md)

[08.34 — Affinity 2026 Forensic Interface Reference → Petunia Qt Translation](08%2034%20%E2%80%94%20Affinity%202026%20Forensic%20Interface%20Reference%203f19bb7d023f81a2b94dd7f77f4c115f.md)

[08.35 — Qt Pixel Geometry, Color Tokens, Panels, Icons & Microinteraction Contract](08%2035%20%E2%80%94%20Qt%20Pixel%20Geometry,%20Color%20Tokens,%20Panels,%20I%203f19bb7d023f81919ea5ec731810dc10.md)

[08.36 — Qt Interface Conformance Gauntlet, Interaction Manifest & Release Gates](08%2036%20%E2%80%94%20Qt%20Interface%20Conformance%20Gauntlet,%20Interac%203f19bb7d023f818784b5c26a55135b50.md)