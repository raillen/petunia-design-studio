# 20.4 — Overprint, Knockout & Transparency Print Semantics

# Properties

FillOverprint and StrokeOverprint flags on eligible Appearance entries/object print properties. Default knockout.

# Semantics

Overprinting retains underlying plate contributions rather than replacing them. Spot/process combinations follow PDF print compositing model selected as normative reference.

# Preview

Overprint Preview view mode required to visualize effect. Normal screen view may not perfectly model inks and must not imply it does.

# Black overprint

Automatic black overprint is not silently enabled globally. Export preset may offer printer-oriented policy with explicit preflight/report.

# Transparency

Transparency groups/effects interact with overprint under PDF rules. If exporter must flatten transparency for older target, overprint preservation is part of flattening fidelity validation.

# UI

Print section in Properties: overprint fill/stroke; warning when property has no effect in current output mode. Separations panel integrates preview.

# Import/export

PDF importer may capture overprint flags when reconstructable. PDF writer preserves in targets supporting it. Raster output bakes appearance and loses plate semantics.

# Tests

Spot over process, black text, transparency group, flattening and external PDF preflight comparison.