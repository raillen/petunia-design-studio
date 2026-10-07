# 03.1 — Photo Persona Workspace, Pixel Targets, Panels & Context Model

# Default workspace

Left tool rail: Move, Brush/Eraser group, Selection tools, Gradient/Fill, Crop/Straighten, Clone/Heal/Inpaint, Dodge/Burn/Smudge, Picker, Hand/Zoom.

Right docks: Layers + Masks, Properties, Histogram, Adjustments, Channels, Brushes/Brush Settings, Color, Info.

# Pixel target

Photo editing always resolves an explicit target:

- RasterLayer pixels;
- PixelMask;
- alpha/channel where supported;
- staged selection.

Target appears in context/status and Layers thumbnail focus. Tool cannot silently redirect to another target.

# Context

Brush-like tools share stable controls: preset, size, hardness, opacity, flow, smoothing/stabilizer plus operator-specific fields.

Selection tools share combine mode, feather/anti-alias and refine.

Retouch tools share sampling/source indicators.

# Mixed content

Vector/text objects stay selectable/transformable. Pixel-only tool disabled with reason and offers explicit Rasterize/Mask action rather than auto-convert.

# Persona transition

Switching to Photo changes workspace/tool set only; document revision does not change.

# Accessibility

Target changes and tool state announced; non-pointer alternatives for key parameters and selection operations.