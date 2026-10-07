# 23.3 — Document Scale SLOs: Objects, Paths, Text, Raster, Surfaces & Libraries

# Vector fixtures

10k/100k/1M simple objects for index/render stress.

100k path nodes active document for editing/navigation benchmark.

Pathological boolean fixtures measured separately; no fixed real-time claim for adversarial geometry, but cancellation/progress required.

# Layers

10k hierarchy normal interaction target; 100k stress should remain open/searchable without widget explosion.

# Text

100k characters in a linked story should support interactive typing/caret in visible paragraph. 1M-character stress measured for layout/open/search with background incremental behavior.

# Raster

8k×8k and 16k×16k editable image fixtures; very large sparse canvas (e.g. 100k×100k with sparse tiles) must not allocate full extent.

# Surfaces

100 Surfaces normal; 1k stress for navigator/export target model, with offscreen render culling.

# Libraries

10k brushes/assets/styles searchable/scrollable with lazy thumbnails.

# Rule

Scale number alone is not correctness requirement; memory/responsiveness behavior and graceful background progress are measured.