# G053 — MCP Server: Discovery, Query, Transactions, Jobs & Auth

# Goal

Expose semantic product API through MCP without bypassing core.

# Depends

Actions/Properties/Jobs, 09.15, 24.4–24.6.

# Primary

architect/implementer + security reviewer.

# Deliverables

local transport; auth/scope model; method schemas; sessions/document/object/selection/actions/properties/transactions/jobs/import/export baseline; revision/idempotency; audit.

# Acceptance

Headless client can inspect/create/edit/export golden document with same semantic result as direct Action API and cannot exceed scopes.

# Tests

Cookbook, stale revision, auth/scope, replay, pagination/backpressure and schema compatibility.