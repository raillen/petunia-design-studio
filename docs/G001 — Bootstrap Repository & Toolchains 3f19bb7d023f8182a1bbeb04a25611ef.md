# G001 — Bootstrap Repository & Toolchains

# Goal

Create reproducible repository skeleton for Python/PySide6 + C++23 without implementing product features.

# Depends

None.

# Primary

architect + implementer. Review: systems-architect, quality-reviewer.

# Skills

lang-python, lang-cpp, architecture-quality, dependency-management, ci-cd, project-documentation-architect.

# Deliverables

Repository topology from 17; root CMakeLists/CMakePresets; pyproject + uv lock; minimal petunia_native nanobind module; Python app bootstrap; C++ test target; Python tests; Ruff/Pyright strict; clang-format/clang-tidy; CI Linux/Windows/macOS skeleton; developer task CLI; README/contributor bootstrap.

# Acceptance

Fresh checkout can sync dependencies, configure, build native extension, import it from Python, launch empty Qt window in smoke mode and run all empty baseline tests. No machine-local paths.

# Tests

configure/build Debug+Release; Python import; C++ unit smoke; pytest smoke; lint/typecheck; package path validation.

# Evidence

Build logs and environment manifest for each CI OS.

# Non-goals

Document model, renderer, real UI chrome, plugin/MCP functionality.