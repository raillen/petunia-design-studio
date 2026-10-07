# 06 — Reference Projects: Adopt / Adapt / Avoid

# Regra

Referências fornecem prior art, nunca arquitetura por autoridade.

# Adopt

- Affinity 2026: unified-document workflow, customizable Studios, context toolbar, dense panels, non-destructive editing.
- Qt/PySide: desktop semantics, accessibility, model/view, docking/window lifecycle.
- Skia/Dawn/Vulkan ecosystems: rendering/backends.
- HarfBuzz/FreeType: shaping/font fundamentals.
- LittleCMS/OpenColorIO: color pipelines.
- libvips: large image IO patterns.
- Blender: command/operator discipline, workspace separation, large-app modularity.
- Krita: brush/raster/editor workflows and open-source lessons.
- Inkscape: SVG/vector interoperability and tool semantics.
- Scribus: print/layout/preflight prior art.

# Adapt

- Affinity Studios -> Petunia Personas/workspaces;
- Qt docking -> Petunia WorkspaceDockModel;
- plugin manifests -> capability broker;
- GPU render graph -> document evaluation/render projection;
- Blender operators -> Actions/Commands, without global context coupling.

# Avoid

- document model tied to QWidget/QObject;
- renderer state as document truth;
- in-process untrusted Python plugins;
- “one giant AppState”;
- raw file paths as durable resource identity;
- UI labels as ActionId;
- ad-hoc JSON dict protocols without schemas;
- hidden destructive conversion;
- format support claims without fidelity grade.

[06.1 — Blender Prior Art: Operators, Undo, UI/Data Separation & Lessons for Petunia](06%201%20%E2%80%94%20Blender%20Prior%20Art%20Operators,%20Undo,%20UI%20Data%20%203f19bb7d023f814a98b6e97e82d4285a.md)

[06.2 — Krita Prior Art: Brush Engines, Preset Diversity & Raster Workflow Lessons](06%202%20%E2%80%94%20Krita%20Prior%20Art%20Brush%20Engines,%20Preset%20Diver%203f19bb7d023f81be87b9ecbe31df5479.md)

[06.3 — Skia Prior Art: 2D Rendering, Paths, Images, Text, Filters & Backend Isolation](06%203%20%E2%80%94%20Skia%20Prior%20Art%202D%20Rendering,%20Paths,%20Images,%203f19bb7d023f8192914dd8d78c364d9c.md)

[06.4 — Dawn/WebGPU vs Qt QRhi Prior Art & Renderer Risk Notes](06%204%20%E2%80%94%20Dawn%20WebGPU%20vs%20Qt%20QRhi%20Prior%20Art%20&%20Renderer%203f19bb7d023f81dcba86ca8504d6383e.md)

[06.5 — Reference Adoption Ledger: What Is Inspiration vs Contract](06%205%20%E2%80%94%20Reference%20Adoption%20Ledger%20What%20Is%20Inspirati%203f19bb7d023f81efbfacee8c94a7eb81.md)