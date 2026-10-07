# 08.8 — Photo Persona Tool-by-Tool GUI Contract

# Pixel Brush

B. Cursor mostra diameter/hardness. Pointer down inicia BrushStrokeSession C++; move envia batches; pointer up commita uma history entry. Context: preset, size, hardness, opacity, flow, spacing, stabilizer, blend e dynamics.

# Eraser

E. Mesmo brush engine com erase/composite mode. Target indicator deixa claro se edita pixels ou mask.

# Flood Fill

Tolerance, contiguous, anti-alias, sample merged. Click calcula native flood region; preview/progress em imagem grande.

# Raster Gradient

Pode operar direto em pixels ou como live layer/effect conforme mode. UI diferencia destructive/live.

# Marquee

Rectangle/Ellipse/Row/Column. Drag + combine mode New/Add/Subtract/Intersect. Space ajusta origin; Shift/Alt geometry modifiers.

# Lasso

Freehand/Polygon/Magnetic. Magnetic exibe anchors/edge confidence; Backspace remove último point; Enter fecha.

# Selection Brush

Paint selection com edge-aware optional. Overlay/marching ants; context size/hardness/mode.

# Refine Selection

Dedicated refine workspace: overlay modes, radius brush, feather/smooth/contrast, output target. Apply commita; Cancel restaura selection original.

# Crop/Straighten

Handles, aspect presets, grid, horizon straighten. Non-destructive crop preferido; destructive trim explícito.

# Clone

Alt/Option source; source crosshair + destination outline. Current/below/all layers sampling.

# Healing

Source-aware texture/color blend, mesma source grammar do Clone.

# Inpainting

Brush marca region; native/GPU job com progress/cancel; stage result e commit atomically.

# Blemish

Click/short drag spot healing; source auto/manual optional.

# Dodge/Burn

Range Shadows/Midtones/Highlights, exposure, protect tones. Live workflow preferido; direct pixels explicit.

# Smudge/Blur/Sharpen

Brush operator-specific strength; native hot loop.

# Color Picker

Point/average sample, source layer/composite, display/document values; Info panel atualiza channels.

# Target safety

Context toolbar sempre mostra raster target: layer pixels, pixel mask ou channel. Sem target implícito invisível.