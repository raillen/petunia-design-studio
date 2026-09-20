# 14.5 — Security Verification, Fuzzing & Hostile-Input Evidence

<aside>
🛡️

Files, clipboard data, fonts, ICC profiles, SVG/PDF/import media, resource packs, plugins and external automation are untrusted boundaries.

</aside>

# Fuzz target classes

Native package/archive/parser, schemas/migrations, SVG/path geometry, import adapters, fonts/text metadata where owned, ICC/profile handling, clipboard fragments, resource packs, Action/Property decoders, plugin manifests/messages and document mutation sequences.

# Resource exhaustion

Tests bound compressed/decompressed bytes, object counts, nesting depth, path complexity, image dimensions, text/glyph counts and parser work where inputs can amplify cost.

# Corpus

Security regressions become permanent minimized AUB-FUZZ/AUB-SEC fixtures.

# Crash requirement

Fuzzing proves more than “does not crash” when possible: parse errors are bounded and typed, partial staging does not mutate the live document, recovery leaves state valid and resources are released.

# Plugin permissions

Test deny-by-default filesystem/network/document write, capability prompts/decisions, revision-safe transactions, CPU/time/memory quota enforcement, runaway script interruption and deterministic unload.

# Archive/package safety

Path traversal, symlink/reparse ambiguity, duplicate entries, zip bombs, filename encoding, oversized entry counts and malformed manifests are explicit tests.

# Supply chain

Release evidence includes dependency vulnerability audit, license policy and SBOM. Native/FFI dependencies receive highlighted review.

# Secrets/privacy

Logs/crash bundles/microcontexts are tested for redaction classes. User artwork and tokens never enter generic Debug output.

# Security gauntlet

Relevant Waves include threat boundary, hostile fixture, expected safe failure, recovery behavior and diagnostic code in the implementation dossier.

# Scheduled campaigns

PR runs build/smoke fuzz targets; scheduled workflows perform deeper campaigns and retain newly discovered minimized cases.