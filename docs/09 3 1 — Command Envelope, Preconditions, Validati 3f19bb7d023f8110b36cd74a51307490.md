# 09.3.1 — Command Envelope, Preconditions, Validation & Result Schema

# Command envelope

Command includes CommandKind stable ID/version, typed payload, preconditions, optional expectedRevision, attribution/source metadata and optional coalescing key.

# Preconditions

Object exists/type/capability, not locked/read-only, parent valid, resource available and revision/property generation where needed. Preconditions are checked before mutation and may be rechecked at commit for staged jobs.

# Validation

Payload schema validation → semantic validation → permission/capability validation → transaction application. UI-side validation is advisory; core is authoritative.

# Result

CommandResult includes ChangeSet, created/deleted IDs, optional selectionSuggestion, topologyMap, warnings and diagnostics.

# Errors

invalid_parameter, incompatible_target, locked, stale_revision, missing_resource, unsupported_capability, permission_denied, conflict and internal_failure.

# Determinism

Given same valid snapshot/payload, canonical result is deterministic except explicitly randomized operations carrying seed.

# Tests

Schema fuzz, precondition failure no mutation, partial failure rollback and deterministic repeat.