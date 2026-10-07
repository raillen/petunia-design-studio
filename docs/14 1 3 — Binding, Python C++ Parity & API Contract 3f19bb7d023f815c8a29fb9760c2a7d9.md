# 14.1.3 — Binding, Python/C++ Parity & API Contract Testing

# Goal

Ensure nanobind/Python semantics exactly match native application facade.

# Signature

Stub/API introspection checks exported names, enums, defaults and overload behavior against generated/declared contract.

# Values

IDs, enums, errors, snapshots and buffers roundtrip without lossy conversion or accidental Any/dict semantics.

# Exceptions

Each native error maps to stable Python exception/result code with structured fields. Unknown internal errors map to safe InternalError and preserve diagnostics correlation.

# GIL

Concurrency tests prove expensive calls release GIL where promised and no Python API is touched while released.

# Lifetime

GC ordering, deleted object ID query, session close with wrappers alive, buffer owner destruction and interpreter shutdown under ASan.

# Semantic parity

Run same Action/transaction directly native and via Python facade, compare normalized canonical snapshot and ChangeSet.