# G012 — Atomic Save, Fault Injection & Recovery Harness

# Goal

Guarantee explicit save never destroys last valid file and establish recovery test infrastructure.

# Depends

G010, G011, G005, G008.

# Primary

systems-architect + editor-engineer; tester independent.

# Skills

filesystem-security, serialization, testing-quality, concurrency-quality.

# Deliverables

DocumentSnapshot-for-save; sibling temp writer; flush/fsync abstraction; reopen/validate; atomic replace adapter; destination fingerprint race check; SaveResult; recovery storage skeleton; fault-injection filesystem adapter.

# Acceptance

At every injected failure point either old valid destination remains or new validated destination is committed. Dirty/savepoint state correct if edits happen after snapshot.

# Tests

Disk full, permission loss, crash before/after close/fsync/rename, external destination change, concurrent save coalescing, Unicode path.

# Non-goals

Full incremental recovery journal UI; cloud/network file semantics beyond adapters.