# 07.1 — Quality Gates by Change Type & Required Review Roles

# Change classes

Core/domain, geometry/raster/render, UI, PTND/IO, plugin/MCP/security, dependency/build, docs-only.

# Gate matrix

Core: unit/property + quality review.

Geometry/raster/render: correctness goldens + perf + sanitizer/fuzz as applicable.

UI: semantic interaction + accessibility + visual regression.

PTND/IO: roundtrip + corruption/fuzz + migration.

Plugin/MCP: contract + security review + permission tests.

Dependency: build matrix + SBOM/license/security scan.

Release-sensitive: independent release-verifier.

# Independence

Implementer cannot self-certify security/performance/release evidence where risk is material.

# Waiver

Any skipped gate records Not Applicable rationale or time-bounded waiver with owner/expiry.

# CI

Fast gates per PR; expensive fuzz/perf/full-platform nightly/release. Merge protection maps required checks to changed subsystem.