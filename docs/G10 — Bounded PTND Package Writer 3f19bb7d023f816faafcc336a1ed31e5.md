# G10 — Bounded PTND Package Writer

# Goal

Write safe deterministic minimal .PTND packages.

# Depends

G05, G09.

# Authority

09.11, 09.11.1.1–.5.

# Owner

editor-engineer. Security review required.

# Deliverables

ZIP/profile writer, manifest/document serialization, resource index/hash, deterministic entry/path policy, temp output API.

# Acceptance

Generated package revalidates; no path traversal/duplicate paths; deterministic semantic contents; large declared sizes bounded; writer never overwrites destination progressively.

# Evidence

Golden PTND, archive inspection, security tests.