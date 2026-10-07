# G12 — Atomic Save, Recovery Journal & Fault-Injection Harness

# Goal

Guarantee user document safety under interruption/failure.

# Depends

G10–G11, G08.

# Authority

09.11.3, 14.5, 21.5.

# Owner

systems-architect + editor-engineer.

# Deliverables

snapshot save, sibling temp write, fsync/replace helpers, recovery journal/checkpoint skeleton, startup recovery discovery and fault injection abstraction.

# Acceptance

At each injected failure, old valid file or new valid file survives; recovery opens unsaved copy; disk-full/permission failure never clears dirty state.

# Evidence

Kill/fault matrix and file hashes.