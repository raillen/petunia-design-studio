# 09.4 — Python/C++ Boundary, nanobind, Ownership, Lifetime, GIL & ABI

# Boundary design

Bindings expose **application/domain facades**, not internal STL containers or every node object.

# Exposed concepts

ApplicationCore, DocumentSession, DocumentSnapshot, ActionRegistryView, QueryService, TransactionBuilder, JobHandle, RenderSession, ImportService, ExportService.

# Ownership

Document/session/native resources owned C++. Python wrapper keeps strong session handle only where lifetime is valid. Child IDs are value handles; querying a deleted ID returns typed NotFound, never dangling pointer.

# Data transfer

Small DTOs convert to immutable Python dataclasses/tuples/enums. Large pixel/vector buffers use zero-copy buffer protocol/span only with explicit lifetime guard and read/write policy. Avoid repeated dict construction in frame loops.

# GIL

Expensive C++ methods release GIL. Native worker threads never invoke arbitrary Python callbacks. Completion/events enqueue typed messages; GUI drains them on main thread.

# Exceptions

C++ internal errors map to stable native error codes and specific Python exceptions/result objects. Public API never exposes implementation exception text as contract.

# ABI

Public extension ABI is **not C++ ABI**. nanobind extension and app are built/released together. Third-party plugins use semantic RPC/WASM APIs. This avoids compiler/STL ABI lock-in.

# Shutdown

Python app requests session/job shutdown; C++ stop tokens cancel; event bridge stops; wrappers invalidate gracefully. Interpreter finalization must not call live Qt/native callbacks.

# Tests

Lifetime stress, delete/query, Python GC ordering, GIL concurrency, exception translation, large-buffer lifetime, interpreter shutdown and sanitizer tests.