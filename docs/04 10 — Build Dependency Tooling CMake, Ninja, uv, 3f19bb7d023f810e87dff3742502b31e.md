# 04.10 — Build/Dependency Tooling: CMake, Ninja, uv, pyproject, vcpkg/Conan Evaluation & Reproducibility

# CMake

Single native source of truth for C++ targets/tests/bindings. CMakePresets.json defines supported profiles and CI/local parity.

# Ninja

Default fast generator where available; IDE generators may wrap same CMake targets.

# Python

pyproject.toml defines Python package/tooling metadata. uv provides lock/sync/dev environment and deterministic tool invocation.

# Native dependencies

Evaluate vcpkg versus Conan versus controlled FetchContent/system packages using:

- cross-platform coverage;
- binary caching;
- lock/reproducibility;
- patching/provenance;
- CI ergonomics;
- package availability;
- licensing/SBOM integration.

No package manager is canonical until ADR.

# Binding build

Python extension produced by CMake and staged into Python package. Build backend may use scikit-build-core if it simplifies wheel/app packaging without hiding CMake.

# Caching

Compiler cache (sccache/ccache) and native dependency binary cache may accelerate CI but are never required for correctness.

# Offline/reproducible

Release build must be possible from pinned sources/artifacts under documented dependency mirror/cache strategy.

# Developer command

One project CLI/task entrypoint orchestrates configure/build/test/typecheck rather than requiring developers to memorize divergent commands.