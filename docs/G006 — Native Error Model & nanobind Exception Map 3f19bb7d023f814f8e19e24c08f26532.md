# G006 — Native Error Model & nanobind Exception Mapping

# Goal

Define stable native error taxonomy and Python translation without leaking implementation exceptions across boundaries.

# Depends

G001, G003.

# Primary

systems-architect + implementer.

# Skills

lang-cpp, lang-python, error-handling, architecture-quality.

# Deliverables

Native ErrorCode enum/categories; structured Error with code/path/IDs/recoverability; expected/result helpers policy; nanobind translators; Python PetuniaError subclasses; cancellation mapping; diagnostic correlation IDs.

# Acceptance

Every exposed native failure becomes a typed Python exception/result with stable code. No raw std::exception text is contract. Python traceback shows call site while preserving native structured detail.

# Tests

Validation, NotFound, StaleRevision, Cancelled, IO, nested native cause, unknown internal mapping and no exception crossing C ABI unsafely.

# Non-goals

User-facing localization/dialog UX beyond metadata hooks.