# 14.8 — AgentOps, Tooling Commands, Evidence Bundles & Deterministic Handoff

# Unified developer commands

Project should expose stable task entry points such as configure, build, test, typecheck, lint, sanitize, fuzz-smoke, benchmark, ui-test, package and evidence. Exact runner may be Python tooling CLI/CMake presets.

# Evidence bundle

Manifest includes Goal/Task, commit, environment, commands/exit codes, tests, sanitizer/fuzzer, benchmark links, screenshots/semantic snapshots, docs updated and known limitations.

# Deterministic handoff

Never hand next agent “it seems fixed”. Hand immutable revision + reproducible command/fixture + observed result.

# CI artifacts

Retain failed logs, crash dumps, diff images, benchmark data and generated reports with retention policy appropriate to privacy/cost.

# Local parity

Developer can run same core gates locally without proprietary CI-only magic.

# Prumo

Goal dossier points to evidence bundle and verifier role. Agent may not mark evidence it did not execute as passed.