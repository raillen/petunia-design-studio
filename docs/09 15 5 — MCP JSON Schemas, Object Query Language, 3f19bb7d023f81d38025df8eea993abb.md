# 09.15.5 — MCP JSON Schemas, Object Query Language, Dry-Run Plans & Idempotency

# Schema source

MCP tool schemas generated/composed from stable Action/Property metadata plus MCP-specific request envelope. Schemas live under schemas/mcp/v1 and are version-controlled.

# Object query

Filter grammar supports type IDs, ObjectIds, parent/Surface scope, name exact/contains, tags, visibility/lock and selected PropertyId comparisons from allowlisted operators. Boolean AND/OR nesting depth bounded. No regex unless bounded implementation chosen; no script expressions.

# Projection

Client requests fields/properties to minimize context. Default query returns compact ID/type/name/bounds summary only.

# Dry-run

For mutation/export/import return Plan {

validated;

affectedIds/count;

requiredPermissions;

expectedNewRevision;

degradations/issues;

estimatedJobClass;

resource/file grants;

conflicts;

}

Dry-run has no canonical mutation and no history.

# Idempotency

Mutating request may include idempotencyKey scoped client/session. Server stores bounded recent result fingerprints; duplicate exact request returns same logical result, conflicting body with same key errors.

# Expected revision

If supplied and mismatched, reject before side effects. File output job also records source revision and conflict policy.

# Tests

Deep query rejection, pagination stability under immutable snapshot, dry-run no revision, duplicate request retry and key collision.