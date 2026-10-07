# 09.12.4 — Import Mapping Rules: SVG/PDF/Raster Ambiguities, Staging AST & Editable Reconstruction

# Principle

Import optimizes for honest editable reconstruction, not pretending foreign format semantics equal Petunia.

# SVG

Parse XML safely -> SVG AST -> CSS/style resolution -> geometry/text/filter resources -> capability mapping.

Preserve groups/transforms, paths/shapes, gradients, clips/masks, text where supported. Unsupported filter becomes placed/rasterized group only according import policy/report.

# PDF

PDF is page-description, not native design model. Import reconstructs paths, text runs/images/groups best effort. Clipping/transparency groups map to Petunia equivalents where semantics provably compatible. Font subset/glyph mapping may prevent editable original Unicode; report outlined/unrecoverable text honestly.

# Raster

Decode pixels + profile + metadata/orientation. Place vs Open-as-document decide RasterLayer/Surface creation. Original encoded resource may be retained.

# Ambiguity

ImportDecision items represent choices such as missing fonts, unsupported blend/effect, external references, page selection, DPI for rasterized content and text reconstruction uncertainty.

# Staging AST

Parser-owned immutable structures never hold DocumentStore pointers. Converter builds StagingDocument DTO with resource plan, then validator commits atomically.

# IDs

Imported objects receive new Petunia IDs. Source IDs/names can be retained metadata for roundtrip/debug but not trusted identity.

# Security

No SVG scripts/external network loads; PDF embedded actions ignored; linked file resolution uses grants and explicit policy.

# Report

Counts, substitutions, rasterizations, dropped metadata, external references and fidelity grade by affected object/page.