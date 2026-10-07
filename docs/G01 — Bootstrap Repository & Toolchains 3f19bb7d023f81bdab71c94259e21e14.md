# G01 — Bootstrap Repository & Toolchains

# Goal

Create the production repository skeleton and reproducible Python/C++ toolchain without implementing product features.

# Authority

04, 09.17, 09.18, 12, 16, 17.1.

# Owner

architect + implementer. Skills: lang-python, lang-cpp, ci-cd, dependency-management, architecture-quality.

# Deliverables

CMake project/presets; cpp/python/apps/tests/fixtures/tooling structure; pyproject + uv lock; lint/typecheck configs; basic C++ test target; petunia_app import; CI matrix; developer task entrypoint; [BUILDING.md](http://BUILDING.md).

# Acceptance

Fresh clone can configure/build native hello library, create Python env, run C++ test, pytest, Pyright strict and Ruff on Linux/Windows/macOS CI profiles.

# Non-goals

No Qt shell, document model or renderer.

# Evidence

Exact bootstrap commands, CI run, dependency graph and environment pin report.