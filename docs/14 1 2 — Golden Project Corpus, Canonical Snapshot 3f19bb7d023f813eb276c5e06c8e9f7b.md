# 14.1.2 — Golden Project Corpus, Canonical Snapshot & Visual Reference Taxonomy

# Golden project classes

Vector Basics, Complex Paths, Typography Multilingual, Photo 8/16/float, Mixed Poster, CMYK/Spot, Multi-Surface Layout, Data Merge, Plugin Extension, Huge Sparse Raster, Recovery/Corrupt.

# Fixture identity

FixtureId, schema version, purpose, expected capabilities, source/license, deterministic fonts/profiles/resources and expected warnings.

# Canonical snapshot

Normalize timestamps/build metadata and compare IDs, hierarchy, properties, resources and order. Snapshot format is testing artifact, not PTND contract.

# Visual golden

Render target size/DPR/backend/reference mode fixed. Compare exact for CPU/reference when possible or bounded pixel/perceptual thresholds for GPU.

# External-format golden

Fixture includes source, import report, canonical expectation where stable, exported output and structural/external validation.

# Updating

Golden update requires diff review and reason linked to Goal/ADR; no blanket regenerate command accepted as evidence.