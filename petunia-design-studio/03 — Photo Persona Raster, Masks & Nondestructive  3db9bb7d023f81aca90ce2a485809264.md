# 03 — Photo Persona: Raster, Masks & Nondestructive Imaging

# Princípio

Photo trabalha sobre o **mesmo Document Core e mesma Layers hierarchy** da Persona Design.

Um grupo pode conter simultaneamente paths, text, placed images, PixelLayers, masks, adjustments e effects.

# Tipos raster

- `ImageObject` — asset colocado, preserva source e transformações;
- `PixelLayer` — superfície raster editável;
- rasterização é explícita quando transformar Image/Vector/Text em pixels destrutivos for necessário.

# Ferramentas MVP

- Move;
- Rect/Ellipse Marquee;
- Lasso/Freehand Selection;
- Brush;
- Eraser;
- Gradient;
- Crop;
- Color Picker;
- raster/vector masks;
- transform;
- basic Clone/Heal — **Post-V1 Candidate** after the raster/tile/selection/undo foundation is production-stable; do not block V1 core architecture on retouch-specific algorithms.

# Adjustments e filters

Baseline: Levels, Curves, HSL, Exposure, White Balance, Gaussian Blur, Sharpen e Noise. Adjustments/filters devem ser não destrutivos sempre que possível, mascaráveis e reordenáveis na EffectChain/appearance hierarchy.

# Raster Engine

Implementação separada de Vello: `wgpu` para textures, compute pipelines, filters, brushes, masks e compositor; CPU fallback/headless para testes e operações específicas.

A **tile-based architecture is required**, not merely preferred, for large-image scalability, dirty regions, bounded GPU residency and undo efficiency. The canonical logical tile default is defined in 09.6/09.26; do not assume one giant texture or make GPU atlas geometry equal to persistent tile geometry.

# Pixel semantics

`PixelSurface` carrega `PixelFormat`, `BitDepth`, `ColorProfile` e `AlphaMode`. **8-bit and 16-bit/channel are V1-required semantic/processing profiles.** 32-bit float remains **Post-V1 Candidate / Open ADR only if a concrete workflow requires it**. Operations that cannot preserve the current bit depth must report/preview an explicit conversion policy; silent precision loss is forbidden.

Não assumir RGBA8 como universo do produto: Photo também deve poder participar de workflows CMYK/ICC.

# Integração Design ↔ Photo

- vector mask sobre raster;
- image clipped inside vector/text;
- raster adjustment/effect sobre groups quando semanticamente válido;
- mesmas opacity/blend/masks/compositor;
- voltar à Persona Design nunca perde editabilidade do Photo.

# Detailed functional contracts

This page remains the Persona-level scope. Tool-by-tool Photo semantics are canonical in [10 — Functional Engine Atlas](10%20%E2%80%94%20Functional%20Engine%20Atlas%203db9bb7d023f81a2b96dc9446aca0a77.md), especially 10.9, 10.10, 10.12 and 10.13, while raster storage/brush/compositor internals live in Architecture Atlas 09.6–09.7. New Photo features must specify both UX and engine behavior.