# 01.6 — Error Taxonomy, Validation Boundaries, Fault Containment & User Recovery

# Error categories

Validation, NotFound, StaleRevision, Capability, Permission, Cancelled, IO, CorruptData, ResourceMissing, OutOfMemory/ResourceBudget, BackendDevice, Plugin, InternalInvariant.

# Boundary

Untrusted input returns typed recoverable errors; programmer invariant violations assert/log/crash safely according build policy rather than being misreported as user validation.

# Transaction

Validation before mutation where possible. Failure during mutation rolls back. Post-commit notification failure cannot roll back canonical state; consumer catches/rebuilds.

# Native/Python

C++ errors map stable codes/fields to Python exception classes. UI converts structured error to localized presentation.

# Recovery

Error should identify preserved state and next safe action: retry, relink, save copy, reduce operation, reopen recovery, switch backend.

# OOM

Memory allocations in large operations use budget checks where feasible. On recoverable allocation failure cancel operation and preserve document.

# Plugin/backend containment

Plugin process crash isolated. GPU device failure drops derived resources. Parser failure never mutates target document.

# Tests

Injected exceptions at command stages, consumer failure, disk/OOM simulation and UI error mapping.