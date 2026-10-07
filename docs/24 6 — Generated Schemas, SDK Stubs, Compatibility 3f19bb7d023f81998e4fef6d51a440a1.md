# 24.6 — Generated Schemas, SDK Stubs, Compatibility Tests & Protocol Fuzzing

# Schema source

Versioned JSON Schema/IDL files under schemas/plugins and schemas/mcp are authority. Documentation/examples are generated/validated against them.

# Generated outputs

Python TypedDict/dataclasses/Pydantic-like validators as appropriate, TypeScript or other SDK later, C++ validation DTOs where useful, Markdown method tables and JSON examples.

# Compatibility

Golden schema corpus verifies previous supported client/plugin versions against current host. Additive optional changes pass; breaking changes require major/version migration.

# Contract tests

Reference plugin and MCP client run handshake, discovery, query, mutation, job, error and shutdown workflows.

# Fuzzing

Frame decoder, schema validators, deeply nested/large arrays, invalid UTF-8 at transport boundary where applicable, unknown enum/ID, duplicate keys and state-machine message order.

# Security

Generated validators enforce size constraints before expensive materialization when possible; semantic validation remains server-side.

# CI

Schema diff report classifies compatible/additive/breaking and blocks accidental breaking change without ADR/version bump.