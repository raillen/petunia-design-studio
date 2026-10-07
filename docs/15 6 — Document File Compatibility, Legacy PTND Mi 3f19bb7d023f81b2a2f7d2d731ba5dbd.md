# 15.6 — Document/File Compatibility, Legacy PTND Migration & Golden Corpus

# Principle

File compatibility is schema-based, not language-based. A document written by legacy Rust build should migrate if its PTND schema/capabilities are supported.

# Fixture set

Minimal, representative, maximal, old-version documents, raster-heavy, text/color, plugin extension, malformed/corrupt and recovery cases from each released/meaningful legacy schema.

# Migration

Container/schema/resource codecs migrate independently. Stepwise transformations are pure, deterministic and preserve opaque optional extensions where possible.

# Semantic comparison

Open legacy fixture with legacy/reference reader when possible -> normalized semantic snapshot/interchange render -> open/migrate new reader -> compare documented semantics.

# Unknown capability

Required unsupported semantics cause read-only/compatibility refusal with diagnostic; optional opaque payload round-trips.

# Save forward

Migrated document saves current schema by default; Save Copy/backup preserves original file until user chooses replacement.

# Tests

Every supported historical version on CI, idempotent current open/save, corrupt input, resource missing and new-reader/old-reader compatibility statement.