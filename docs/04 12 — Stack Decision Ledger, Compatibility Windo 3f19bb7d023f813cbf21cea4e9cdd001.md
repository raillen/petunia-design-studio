# 04.12 — Stack Decision Ledger, Compatibility Windows & Upgrade Policy

# Locked baseline

Python 3.14.x.

Qt/PySide6 6.12 LTS.

C++23.

nanobind.

CMake/Ninja.

Pyright strict + Ruff.

Actions/Commands canonical mutation.

Out-of-process third-party Python plugins.

# Decisions requiring ADR

Renderer backend.

Native package manager.

Geometry engine.

PDF parser/writer.

Exact color precision/pixel formats.

Optional ICU/OCIO scope.

WASM runtime milestone.

# Upgrade cadence

Python/Qt patch updates: dependency PR + focused suite.

Compiler updates: sanitizers/build/perf comparison.

Major/minor framework changes: architecture compatibility review.

File-format dependencies: malformed/fuzz corpus required.

# Compatibility windows

App binary/runtime components ship together. Plugin semantic SDK and MCP API have explicit version windows independent from Python internal packages.

# Revisit

Only evidence or ecosystem lifecycle should change baseline, not novelty. LTS/security support and platform support are first-order constraints.