# 12.1 — Python Strict Typing, Data Models, Errors & Package Discipline

# Type policy

Product Python uses **Pyright strict** as blocking CI gate. Ruff handles lint/format/import hygiene. Public APIs cannot rely on implicit Any.

# Types by purpose

- dataclass(frozen=True, slots=True): immutable value objects/DTOs;
- Enum/StrEnum: closed semantic states;
- Protocol: ports/interfaces where structural typing helps;
- TypedDict: JSON-shaped boundary payload only;
- Pydantic or generated validator: external/config schemas when runtime validation matters;
- NewType/typed wrapper: semantic IDs when native binding does not already expose distinct ID type.

# Avoid

dict[str, object] as internal protocol, stringly typed states, dynamic setattr feature systems, giant union of arbitrary payloads, mutable default arguments, bare except, exceptions as normal branch control.

# Native stubs

petunia_native ships .pyi/generated signatures matching nanobind API. CI imports extension and checks stub/API parity where possible. Native enums/DTOs expose stable Python-friendly names.

# Errors

Define PetuniaError hierarchy by boundary: ValidationError, CapabilityError, NotFoundError, StaleRevisionError, PermissionDenied, Cancelled, IOErrorAdapter etc. User-facing localization is separate TextId metadata; exceptions retain technical structured fields.

# Async

Qt event loop remains GUI authority. Asyncio is used only for explicit async integrations/MCP/plugin RPC with controlled bridge; do not nest uncontrolled event loops. CPU tasks belong native JobScheduler.

# Resources

Use context managers/RAII-like wrappers for file/process/network grants. Qt QObject ownership remains presentation concern, not core dependency.

# Dependency injection

Application composition root constructs services/adapters. No global singleton registries accessed from arbitrary modules.

# Testing

pytest, pytest-qt where needed, hypothesis for pure Python property tests. Typecheck test fixtures too unless isolated exceptions documented.

# Packaging

pyproject centralizes metadata/tool config. uv lock pins environment. Runtime imports have one directional architecture; optional dev dependencies separated.