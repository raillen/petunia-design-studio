# G07 — Python Type Stubs & Strict Application Contracts

# Goal

Ensure native API and Python application layer are fully statically typed.

# Depends

G06.

# Authority

12.1, 04.3.

# Owner

implementer + quality-reviewer.

# Deliverables

py.typed, .pyi/generated stubs, typed wrapper/facade package, Pyright strict configuration and stub/API drift test.

# Acceptance

No implicit Any in public Petunia app code; representative native API completion works; CI fails on stub mismatch or untyped public surface.

# Evidence

Pyright report and generated-stub diff test.