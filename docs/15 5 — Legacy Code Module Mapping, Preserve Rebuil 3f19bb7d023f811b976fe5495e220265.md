# 15.5 — Legacy Code/Module Mapping, Preserve/Rebuild/Drop Decisions & Oracle Use

# Purpose

Map old Rust/Slint implementation artifacts to new architecture without performing a mechanical port.

# Classification

Preserve Semantics; Rebuild Idiomatically; Use as Test Oracle; Migrate Data Only; Archive; Drop as Defect/Dead Code.

# Module map

Legacy document/core -> C++ core/document/application.

Legacy Slint shell/components -> Python/PySide6 UI/design-system equivalents.

Legacy renderer abstractions -> RenderScene/RenderBackend evaluation; backend chosen anew.

Legacy Lua/mlua plugins -> Python process plugin semantic SDK; migrate feature concepts, not runtime assumptions.

Legacy serialization -> PTND schema fixtures/migration source.

# Oracle

Old build/fixtures may serve as behavior/visual/reference oracle only where behavior is intentional and documented. Accidental bugs do not become compatibility requirements.

# Traceability

Each migration task links legacy path/commit, new module/Goal, semantic decision and parity test.

# Removal

Legacy runtime dependency removed only after replacement capability and fixture evidence pass; historical branch/docs retained for audit.