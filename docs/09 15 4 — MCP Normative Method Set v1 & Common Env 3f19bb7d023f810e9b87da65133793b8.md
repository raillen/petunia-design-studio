# 09.15.4 — MCP Normative Method Set v1 & Common Envelopes

# Common request context

client/session identity from transport auth; optional documentSessionId, expectedRevision, requestId/idempotencyKey and dryRun where method supports.

# Common response

apiVersion, result, warnings[], documentRevision if applicable, jobId/cursor if applicable. Error object has code/category/fieldPath/recoverable/helpId/currentRevision.

# V1 methods

app.version

app.capabilities

app.sessions.list

document.summary

document.validate

document.preflight

[document.save](http://document.save)

object.get

object.query

selection.get

selection.set

actions.list

actions.describe

actions.execute

properties.describe

properties.get

properties.set

transactions.dry_run

transactions.execute

import.formats

import.analyze

import.start

export.formats

export.analyze

export.start

jobs.get

jobs.cancel

plugins.contributions

ui.inspect

diagnostics.summary

# Task helpers

document.create_rectangle, text.create_artistic, text.create_frame, photo.add_adjustment and export.surface may wrap generic methods. Helpers cannot bypass permission/revision/transaction semantics.

# Pagination

object.query/actions.list/registries use cursor + limit with strict max. Filters are declarative, no arbitrary code.

# Mutation

Default one method = one transaction unless transactions.execute batches many operations. Partial success only when explicitly requested and return per-operation status.

# Tests

Schema validation for every example, stale revision, idempotent retry and parity with direct Action API.