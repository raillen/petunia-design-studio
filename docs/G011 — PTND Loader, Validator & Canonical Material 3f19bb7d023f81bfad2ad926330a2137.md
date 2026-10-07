# G011 — PTND Loader, Validator & Canonical Materialization

# Goal

Safely open minimal PTND v1 into validated canonical DocumentStore.

# Depends

G009, G010, G004.

# Primary

editor-engineer + security-reviewer.

# Skills

serialization, fuzz-grammar-testing, filesystem-security, lang-cpp.

# Deliverables

Bounded ZIP reader; manifest parse; schema validation; capability check; resource index validation; document JSON parse; semantic reference/cycle validation; canonical materializer; structured LoadReport; read-only unsupported-capability result.

# Acceptance

Valid package opens to normalized semantic snapshot equal to source. Corrupt/hostile package returns typed error/report without excessive allocation or partial DocumentStore mutation.

# Tests

Traversal, zip bomb ratio, duplicate entries, truncation, bad hashes, invalid refs, cycles, unknown optional/required capabilities and fuzz smoke.

# Non-goals

Historical migrations beyond v1, salvage, recovery journal.