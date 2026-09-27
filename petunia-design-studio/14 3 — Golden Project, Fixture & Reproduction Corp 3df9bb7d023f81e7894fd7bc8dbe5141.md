# 14.3 — Golden Project, Fixture & Reproduction Corpus

<aside>
🧬

One reusable corpus feeds correctness, rendering, performance, migrations, UI workflows, fuzz regressions and interoperability.

</aside>

# Corpus taxonomy

Minimal fixtures: empty document, one shape, one path, one text, one image.

Vector: dense nodes, booleans, strokes, gradients, symbols, multi-Surface.

Raster: tiled 16-bit, masks, adjustments, large canvas, brush history.

Typography: variable fonts, missing fonts, path text, complex scripts, frame flow.

Color: RGB/CMYK/Lab/Gray, ICC, spot/global color, proofing.

Interop: SVG/PDF/raster imports and exports with expected fidelity/degradation.

Migration: each historical schema version and important feature transition.

Hostile: malformed archive, bad JSON/schema, invalid references, oversized resources, decompression bombs, malformed SVG/font/ICC.

Stress: huge Layers tree, deep hierarchy, large history, mixed vector/raster/text, many Surfaces/assets/plugins.

UI: canonical Design, Photo and mixed Workspace Profiles.

# Fixture contract

Every fixture has:

- stable FixtureId;
- purpose and canonical spec;
- expected validity;
- generator/source provenance;
- deterministic seed if generated;
- expected diagnostics;
- permitted tolerances;
- security classification;
- size/resource budget.

# Synthetic-first privacy

Development and CI use synthetic/minimized assets unless a licensed reference asset is explicitly required and provenance is recorded.

# Reproduction packs

A bug report that requires project state can be minimized into a reproducible fixture pack containing document, resources, environment manifest and expected failure/evidence. Secrets and unnecessary user artwork are removed.

# Generation

Large/stress fixtures should be generated deterministically where practical rather than storing enormous binaries.

# Lifecycle

Fixtures are version-controlled and reviewed. Do not delete a regression fixture merely because implementation changed; supersede it only when the canonical contract changed.

# Cross-use

The same AUB-FIX-* should be reused by headless tests, UI workflows and benchmarks when measuring the same semantic scenario to reduce drift between quality systems.