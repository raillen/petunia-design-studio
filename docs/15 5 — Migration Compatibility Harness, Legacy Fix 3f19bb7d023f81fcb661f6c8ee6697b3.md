# 15.5 — Migration Compatibility Harness, Legacy Fixtures & Semantic Diff

# Goal

Prove that replacing implementation language/toolkit does not silently change document semantics.

# Legacy fixture corpus

Representative Rust/Slint-era PTND/serialized fixtures if available, plus screenshots/expected semantic snapshots and user workflows.

# Semantic diff

Normalize IDs/order only where migration legitimately changes representation. Compare hierarchy, transforms, geometry, text, appearance, resources and exported appearance.

# Legacy oracle

Old executable/code may be used as behavioral oracle only when result is known-correct; bugs are not preserved blindly. Discrepancy triaged Intentional Fix / Equivalent / Regression / Unknown.

# Migration reader

If legacy schema differs, implement explicit schema migration independent from old runtime. No dependency on Rust binary at production load.

# UI parity

Workflow parity measured at task level, not widget pixel match.

# Gate

Remove legacy implementation only when required fixtures have new-reader/result evidence and no unresolved P0/P1 semantic gaps.