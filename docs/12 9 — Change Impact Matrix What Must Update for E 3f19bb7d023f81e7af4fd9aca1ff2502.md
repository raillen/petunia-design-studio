# 12.9 — Change Impact Matrix: What Must Update for Every Feature Change

# Tool change

Update Tool manifest, Action/Command schema, 08 interaction page, 10 functional page, shortcut/help, tests, MCP/plugin discovery if exposed.

# Property/schema change

Update PropertySchema, generic inspector expectations, serialization if canonical, plugin/MCP docs, migration if field persisted.

# PTND change

Schema version/migration, manifest/min reader, fixtures, validator, format docs, import/export compatibility, recovery torture.

# UI component/token

Design system contract, component gallery, accessibility, visual goldens, affected screenshots/workspaces.

# Plugin permission/API

Manifest/SDK schemas, permission UI, security threat model/tests, examples and compatibility/deprecation.

# MCP method

Schema/docs/cookbook, auth scope, revision/transaction semantics, conformance tests.

# Renderer/color/text algorithm

Correctness goldens, performance, export/render parity and device/backend evidence.

# Dependency

Lock/SBOM/license/security scan, build matrix and relevant ADR if architectural.

# Rule

Change is incomplete when required impact row is ignored without documented Not Applicable rationale.