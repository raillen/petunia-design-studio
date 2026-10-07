# 09.26 — Open ADR Registry, Deferred Decisions & Revisit Triggers

# Open decisions

- final renderer backend after 15.3 spike;
- canonical raster tile size/codec after benchmarks;
- exact internal working/compositing color precision;
- PDF writer/conformance stack;
- optional OCIO role beyond ICC workflows;
- plugin Tier C/WASM milestone;
- native public plugin ABI: currently deferred;
- package signing/marketplace infrastructure;
- mesh gradient V1 vs Post-V1;
- advanced DTP master/template scope;
- AI-assisted inpainting provider architecture if product chooses it.

# Revisit triggers

Benchmark failure, platform blocker, license change, security incident, fidelity gap, V1 scope promotion or upstream library deprecation.

# Rule

Deferred is explicit architecture state, not permission for feature engineer to choose locally.

# Closure

Technology Decision Agent provides candidate evidence; Architect records final ADR and updates impacted canonical pages/ledger.