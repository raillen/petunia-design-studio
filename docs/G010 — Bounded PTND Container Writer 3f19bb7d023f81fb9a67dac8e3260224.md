# G010 — Bounded PTND Container Writer

# Goal

Write minimal valid .PTND packages safely and deterministically enough for fixtures.

# Depends

G009, G005.

# Primary

editor-engineer + security-reviewer.

# Skills

serialization, filesystem-security, secure-coding, lang-cpp.

# Deliverables

PackageWriter; safe path normalization; mimetype/manifest/document entries; resource index baseline; ZIP profile enforcement; temp output; deterministic entry ordering; size/hash metadata.

# Acceptance

Given a valid immutable snapshot, writer creates package that reference validator can inspect. It never writes traversal paths, duplicate canonical paths or nonfinite JSON. Failure leaves destination unchanged when atomic wrapper used later.

# Tests

Minimal document package, Unicode names, huge-count guard, duplicate resource path, write failure injection and reproducible structural listing.

# Non-goals

Atomic destination replace/recovery journal (G012), raster PTILE compression, importers/exporters.