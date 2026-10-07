# G007 — Python Type Stubs, Strict Typing & Native API Parity

# Goal

Make petunia_native fully consumable under Pyright strict with no Any leakage.

# Depends

G006 and minimal native exports.

# Primary

implementer + quality-reviewer.

# Skills

lang-python, clean-code, testing-quality, documentation.

# Deliverables

py.typed package marker; .pyi generation/manual authoritative stubs; enums/ID/value type annotations; exception signatures; overload policy; CI stub/runtime symbol parity check; strict sample usage.

# Acceptance

python/petunia_app passes Pyright strict when using every baseline native API. Public native methods have parameter/return types and docstrings/help references.

# Tests

Stub import, reveal_type snapshots where useful, runtime/stub symbol diff, invalid type fixture expected failure.

# Non-goals

Full public plugin SDK typings (later Goal).