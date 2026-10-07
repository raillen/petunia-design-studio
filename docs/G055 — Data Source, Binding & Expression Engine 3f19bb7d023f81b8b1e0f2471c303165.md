# G055 — Data Source, Binding & Expression Engine

# Goal

Build safe deterministic Data Merge core.

# Depends

Property schemas, resources, text/layout, jobs.

# Primary

editor-engineer + security review.

# Deliverables

CSV/JSON providers; typed schema/rows; Binding model; sandbox expression parser/evaluator; preview override projection; image resource resolution; source fingerprint/refresh.

# Acceptance

Template can bind text/image/color/property across records without mutating canonical template during preview; expressions cannot execute Python/files/network.

# Tests

encodings, null/types, 100k rows streaming, expression fuzz/security, missing fields/resources and deterministic preview.