# 10.7.1 — Surface/Page Model, Spatial Artboards vs Sequential Pages & Coordinate Systems

# Surface model

SurfaceId, role Artboard/Page/ExportRegion, rect/transform, background, layout settings, export flags and child ownership scope.

# Spatial artboards

Can be positioned freely on pasteboard; order for UI/export stored separately from position if needed.

# Sequential pages

Page sequence has logical page index/order. Spatial preview may arrange spreads but page identity/order remains semantic.

# Coordinate systems

Document global; Surface local origin; ruler origin can be user view setting. Numeric transform fields disclose coordinate reference.

# Pasteboard

Objects may be Surface-owned or global/pasteboard according product model. Export includes content by intersection/ownership policy defined, not ambiguous “what is visible”.

# Surface move

Move with contents option computes child transform delta only for owned/scoped objects. Guides/margins local remain attached.

# Tests

Overlapping artboards, page reorder, negative positions, moving contents, global object and export target bounds.