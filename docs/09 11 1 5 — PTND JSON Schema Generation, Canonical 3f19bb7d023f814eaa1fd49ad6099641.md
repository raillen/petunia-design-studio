# 09.11.1.5 — PTND JSON Schema Generation, Canonicalization, Migrations & Reference Validator CLI

# Schema source

Repository owns machine-readable JSON Schema files under schemas/ptnd/v1. C++ types and Python DTOs are not the sole schema source; generation direction is explicitly chosen and tested for drift.

# Validator

petunia-cli validate file.ptnd:

1. ZIP envelope safety;
2. manifest schema;
3. resource index/path/hash optional mode;
4. document JSON schema;
5. semantic validator;
6. capability compatibility;
7. optional full resource decode.

Output JSON/text report with stable issue codes, paths/ObjectIds and severity.

# Canonicalization

Test canonicalizer sorts map-like objects deterministically and strips nonsemantic writer timestamps/build info when semantic comparison needed. It does not rewrite user text/order.

# Migration

vN -> vN+1 pure transform over validated old model. Never jump migrations in one opaque function. Migration result validates target schema/semantics before session opens.

# Migration policy

Preserve unknown optional extension payload. Required extension migration delegated only to compatible provider; otherwise read-only/refuse.

# Downgrade

Save-as-older-version only if explicit exporter/migration-down contract proves feature compatibility; not assumed.

# Golden corpus

Each schema version has minimal/full/edge/malformed fixtures and expected issues. Current writer output revalidates with standalone CLI.

# CI

Any schema change requires migration/compatibility note, fixture update and generated code/stub synchronization.