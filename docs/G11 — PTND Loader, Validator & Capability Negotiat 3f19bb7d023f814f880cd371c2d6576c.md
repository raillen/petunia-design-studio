# G11 — PTND Loader, Validator & Capability Negotiation

# Goal

Open PTND safely into validated canonical model.

# Depends

G09–G10.

# Authority

09.11, 09.11.1.*, 14.5.

# Owner

editor-engineer + security-reviewer.

# Deliverables

bounded archive reader, manifest/document schema validation, capability negotiation, resource index verification, migration hook and structured IssueReport.

# Acceptance

Valid files open; unknown optional extension preserved; unknown required capability refuses/read-only explicitly; malformed paths/sizes/JSON cannot exhaust/crash; semantic validator catches cycles/dangling refs.

# Evidence

Malformed corpus, ASan/UBSan, fuzz smoke, golden roundtrip.