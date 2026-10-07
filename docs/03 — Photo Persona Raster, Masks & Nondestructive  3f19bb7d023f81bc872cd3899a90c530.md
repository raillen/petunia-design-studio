# 03 — Photo Persona: Raster, Masks & Nondestructive Imaging

# Photo Persona

Photo fornece edição raster profissional sem criar um segundo documento. A seleção e o layer tree são compartilhados com Design.

# Ferramentas baseline

- Move;
- Pixel Brush;
- Eraser;
- Flood Fill;
- Gradient;
- Marquee Rectangle/Ellipse/Row/Column;
- Freehand/Lasso/Polygon/Magnetic selection;
- Selection Brush;
- Refine Selection;
- Crop/Straighten;
- Clone;
- Healing;
- Inpainting;
- Blemish removal;
- Dodge/Burn;
- Smudge/Blur/Sharpen brush;
- Color Picker;
- Hand/Zoom.

# Raster storage

RasterLayer é tiled. Tile default é configurável e benchmark-driven; 256×256 é baseline inicial para reduzir overhead sem explodir invalidation. Tiles podem ter backing compressed/mmap e GPU residency separada.

# Brush pipeline

Input samples -> smoothing/stabilizer -> spacing -> dynamics -> dab generation -> texture/scatter -> blend -> affected-tile invalidation. Pressure, tilt, rotation, velocity e randomization são parâmetros tipados. Python recebe begin/update/end sem processar dabs.

# Selection e masks

PixelSelection é recurso derivado/editável com combine modes replace/add/subtract/intersect. Masks podem ser pixel, vector ou compound. Feather e refine são nondestructive enquanto possível.

# Adjustments

Levels, Curves, Exposure, Brightness/Contrast, HSL, Vibrance, White Balance, Black & White, Channel Mixer, Gradient Map, Selective Color e LUT são nodes não destrutivos.

# Live filters

Gaussian blur, motion blur, sharpen, high pass, noise, denoise, distortions e outros são EffectNodes avaliados sob demanda. Preview pode usar qualidade reduzida; export usa quality policy final.

# GUI

Photo troca tool rail e preset de painéis. Brush context bar prioriza size/hardness/opacity/flow/stabilizer; Histogram/Channels/Adjustments aparecem no right dock. On-canvas brush outline deve manter tamanho perceptual correto e indicar hardness/rotation.

# Performance

Stroke preview deve evitar round-trips por dab. C++ brush engine recebe batches de input e devolve damage regions/progress. GPU compositing e CPU SIMD são escolhidos por operação e disponibilidade.

[03.1 — Photo Persona Workspace, Pixel Targets, Panels & Context Model](03%201%20%E2%80%94%20Photo%20Persona%20Workspace,%20Pixel%20Targets,%20Pan%203f19bb7d023f8189a14dc96e5e0d475c.md)

[03.2 — Brush Engine, Presets, Dynamics, Stabilization, Texture & Painting Semantics](03%202%20%E2%80%94%20Brush%20Engine,%20Presets,%20Dynamics,%20Stabilizat%203f19bb7d023f81c6b470dee16db14f1b.md)

[03.3 — Pixel Selections, Lasso, Magnetic Selection, Selection Brush & Refine Workflow](03%203%20%E2%80%94%20Pixel%20Selections,%20Lasso,%20Magnetic%20Selection%203f19bb7d023f81959ae6f061242b01b7.md)

[03.4 — Nondestructive Adjustments, Live Filters, Masks, Blend & Compositing Workflow](03%204%20%E2%80%94%20Nondestructive%20Adjustments,%20Live%20Filters,%20M%203f19bb7d023f81f580cbce749b62e234.md)

[03.5 — Retouch: Clone, Heal, Inpaint, Blemish, Dodge/Burn, Smudge, Blur & Sharpen](03%205%20%E2%80%94%20Retouch%20Clone,%20Heal,%20Inpaint,%20Blemish,%20Dodg%203f19bb7d023f81a58034c0ddbc387529.md)

[03.6 — Crop, Straighten, Resize, Resample, Canvas/Surface Size & Image Transform Semantics](03%206%20%E2%80%94%20Crop,%20Straighten,%20Resize,%20Resample,%20Canvas%20%203f19bb7d023f815986cbd094eed18e3e.md)

[03.7 — Photo Color, Histogram, Channels, Scopes, Soft Proof & Sampling](03%207%20%E2%80%94%20Photo%20Color,%20Histogram,%20Channels,%20Scopes,%20S%203f19bb7d023f811085c7fa20c9b141f2.md)

[03.8 — Photo Persona V1 Feature Gate, Commercial Retouch Workflows & Post-V1 Roadmap](03%208%20%E2%80%94%20Photo%20Persona%20V1%20Feature%20Gate,%20Commercial%20R%203f19bb7d023f814da1a5ee3b520b9992.md)