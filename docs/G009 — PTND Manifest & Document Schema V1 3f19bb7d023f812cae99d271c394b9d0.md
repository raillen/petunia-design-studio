# G009 — PTND Manifest & Document Schema V1

# Goal

Create actual machine-readable PTND v1 schemas from 09.11.4–09.11.6.

# Depends

G003, G004.

# Primary

editor-engineer + documentation-maintainer; security review.

# Skills

serialization, lang-cpp, lang-python, secure-coding, documentation.

# Deliverables

schemas/ptnd/v1/manifest.schema.json; document.schema.json; shared ID/value definitions; tagged object union for Surface/Group/Rectangle baseline plus reserved extension mechanism; examples/minimal.ptnd-json; schema validation tooling; spec markdown generated/linked.

# Acceptance

Schemas reject nonfinite/invalid enum/reference-shape data structurally; representative valid examples pass. Field semantics documented and stable for implementation.

# Security

Maximum lengths/counts enforced additionally by reader semantic limits even if JSON Schema alone cannot.

# Tests

Positive/negative fixture corpus; schema version field; unknown extension handling.

# Non-goals

ZIP container writer, full future object variants not yet implemented except schema-ready placeholders.