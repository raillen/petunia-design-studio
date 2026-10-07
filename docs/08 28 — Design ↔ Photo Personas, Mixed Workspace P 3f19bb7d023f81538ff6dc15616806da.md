# 08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow

# Persona transition

Switch is instant workspace reconfiguration over same DocumentSession. Selection stays if object still meaningful. Active editing gesture must finish/cancel safely before switch.

# Shared panels

Layers, Properties, Color, Swatches, Assets, History, Navigator, Transform and Export can persist across Personas.

# Persona-specific panels

Design favors Stroke/Typography/Paragraph/Symbols/Pathfinder/Data Merge. Photo favors Histogram/Adjustments/Channels/Brushes/Masks/Info.

# Mixed workflows

Designer can select raster layer in Design and perform common transform/clip; Photo exposes raster-edit tools. Vector/text can be selected/transformed in Photo but unsupported pixel operation explains need for rasterization/mask rather than doing it silently.

# Workspace

User may save hybrid workspace containing preferred shared + specialized panels. Persona preset is default, not hard wall.

# Conversion

Rasterize/Expand/Convert to Curves are actions independent from Persona and preserve explicit fidelity semantics.

# Context

Status/context bar clearly identifies active Persona, tool and pixel target where relevant. Switching does not rewrite layer/object types.

# Testing

Switch repeatedly during mixed document, preserve selection/zoom/pan according view policy, verify zero document revision change for pure Persona switch.