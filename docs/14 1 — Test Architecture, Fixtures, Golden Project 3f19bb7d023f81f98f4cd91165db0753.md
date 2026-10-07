# 14.1 — Test Architecture, Fixtures, Golden Projects & Regression Policy

# Layers

Unit -> property/fuzz -> subsystem integration -> binding contract -> headless workflow -> UI semantic -> visual -> end-to-end release smoke.

# Fixture corpus

Golden .PTND projects cover vector, raster, mixed, text multilingual, CMYK/ICC, masks/effects, multi-Surface, data merge, plugin extension payload, huge/sparse raster and intentionally corrupted files.

# Determinism

Tests pin random seeds, fonts/resources, color profiles and renderer fixture conditions. Nondeterministic timing never determines correctness.

# Semantic snapshots

Canonical document snapshot normalizes nonsemantic metadata/timestamps and compares IDs/properties/order. Used for UI/MCP/plugin parity.

# Golden update

Golden updates require reason, diff/review and evidence. Never auto-accept visual changes on failed CI.

# Regression

Every fixed bug gets smallest reproducer fixture/test unless impossible; known impossible cases documented.

# Cross-language

Python facade behavior and direct C++ service tests compare same expected semantics. Bindings cannot silently translate defaults differently.

[14.1.1 — Unit, Property & Algebraic Invariant Test Matrix](14%201%201%20%E2%80%94%20Unit,%20Property%20&%20Algebraic%20Invariant%20Test%203f19bb7d023f81158d73c1abf340aa1a.md)

[14.1.2 — Golden Project Corpus, Canonical Snapshot & Visual Reference Taxonomy](14%201%202%20%E2%80%94%20Golden%20Project%20Corpus,%20Canonical%20Snapshot%203f19bb7d023f813eb276c5e06c8e9f7b.md)

[14.1.3 — Binding, Python/C++ Parity & API Contract Testing](14%201%203%20%E2%80%94%20Binding,%20Python%20C++%20Parity%20&%20API%20Contract%203f19bb7d023f815c8a29fb9760c2a7d9.md)

[14.1.4 — End-to-End Workflow Suite, Scenario DSL & Deterministic Replay](14%201%204%20%E2%80%94%20End-to-End%20Workflow%20Suite,%20Scenario%20DSL%20&%203f19bb7d023f8192843feb69b59ff50a.md)

[14.1.5 — Test Data, Font/Profile Licensing, Reproducibility & Fixture Provenance](14%201%205%20%E2%80%94%20Test%20Data,%20Font%20Profile%20Licensing,%20Reprod%203f19bb7d023f816b89a2fb346d22f827.md)