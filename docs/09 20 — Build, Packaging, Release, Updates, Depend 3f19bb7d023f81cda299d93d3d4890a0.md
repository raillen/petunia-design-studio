# 09.20 — Build, Packaging, Release, Updates, Dependency Policy, SBOM & Licensing

# Canonical build

CMake/Ninja for C++; pyproject/uv for Python. Build presets define debug, release, sanitize, coverage and profiling variants.

# Dependencies

Every external native/Python dependency records purpose, version policy, license, security exposure, replacement difficulty and owner. Avoid pulling large frameworks for one helper.

# Vendoring

Critical patched/native dependencies may be vendored with provenance; otherwise package managers/locks used consistently. Generated third-party files clearly separated.

# Packaging

Bundle a supported Python runtime/PySide/Qt to avoid dependence on user's system Python. Native extension compiled against bundled runtime/ABI policy.

# Updates

Signed update metadata/artifacts, channels, rollback and migration compatibility. Offline use remains functional.

# SBOM

Python packages, native libraries, codecs, fonts/assets and workforce/tooling licenses included as applicable.

# License gate

No codec/plugin/library enters release until redistribution terms and patent/licensing implications reviewed.

# Reproducibility

BuildId records source commit, toolchain and lock hashes. Release verifier can reproduce dependency inventory.