# 09.17 — Build, Packaging, CI/CD, Release, Updates, SBOM & Licensing

# Build graph

CMake presets build C++ libraries, tests, native extension and CLI. Python package uses pyproject/uv. Ninja default local/CI generator where supported.

# CI matrix

Linux clang/gcc, Windows MSVC/clang-cl as policy, macOS AppleClang; Python supported minor(s); Debug/Release and sanitizer jobs. UI smoke on each OS.

# Static gates

Pyright strict, Ruff, clang-format check, clang-tidy selected rules, warnings-as-errors, dependency cycles, schema generation diff.

# Dynamic gates

pytest, C++ tests, integration, ASan/UBSan, TSan dedicated, fuzz corpus, headless renderer/export, UI smoke.

# Packaging

Windows installer/MSIX candidate, macOS signed/notarized app bundle, Linux AppImage/Flatpak/package strategy via ADR. Bundle Qt/Python/native libs reproducibly.

# Updates

Signed metadata/artifacts, staged channel stable/beta/nightly, rollback path. Updater must not run arbitrary plugin code.

# SBOM

CycloneDX/SPDX output for Python, C++ and bundled assets/libs. Licenses audited before release.

# Versioning

App semver/calendar policy by ADR; native format versions independent. BuildId identifies immutable artifact.

# Reproducibility

Pinned dependency locks, compiler/toolchain versions recorded, generated schemas checked in or reproducibly generated.