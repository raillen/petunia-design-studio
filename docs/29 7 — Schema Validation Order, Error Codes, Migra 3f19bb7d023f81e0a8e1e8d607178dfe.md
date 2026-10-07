# 29.7 — Schema Validation Order, Error Codes, Migrations & Generated Type Bindings

# Validation layers

1 JSON/container structural;

2 tagged-union/type/schema;

3 finite/range/value;

4 referential integrity;

5 graph invariants;

6 capability/version;

7 resource integrity/availability;

8 semantic subsystem validation.

# Error

Return list/tree of stable ErrorCode + JSON/semantic path + object/resource ID + severity/recoverability. Reader can aggregate rather than fail first only where safe.

# Migration

Each schema version step transforms DTO representation before domain materialization. Migrations cannot depend on GUI/network/current workspace.

# Generated types

JSON Schema may generate validation DTOs/enums/stubs, but canonical domain C++ types retain strong semantics/optimized representation. Generated code never replaces invariant enforcement.

# Unknown fields

Policy versioned: optional unknown fields can preserve in extension/roundtrip bucket only when safe; unknown core field must not accidentally be interpreted by older writer.

# Tests

Minimal/maximal each record, invalid refs/cycles, version migrations, unknown optional/required and error-path stability.