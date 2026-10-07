# 09.14.7 — Plugin SDK Python API Surface, Type Contracts & Reference Examples

# Package

petunia_plugin_sdk is separate versioned typed package, py.typed, with dataclasses/enums/Protocols and no access to internal petunia_app modules.

# Context

PluginContext exposes:

app metadata; actions; documents query facade; transactions; ui registration; jobs; files; network; storage; resources; diagnostics; capability inspection.

# Document

Read returns immutable semantic DTOs/paged queries. Write uses TransactionBuilder or registered Action request. No mutable proxy objects whose setters make hidden RPCs.

# Transactions

```python
async with ctx.transactions.begin(document_id, expected_revision=rev) as tx:
    tx.set_property(...)
    tx.create_object(...)
    result = await tx.commit()
```

Illustrative API; generated contract must pin exact signatures.

# Actions

register_action(ActionDescriptor, handler). Handler receives validated args and invocation context; may return semantic result/transaction request.

# Jobs

[ctx.jobs.run](http://ctx.jobs.run)(...) for plugin-process work with cancellation/progress; host-native heavy work requested through supported services instead of Python CPU loops where possible.

# Errors

Typed SDK exceptions mirror stable protocol codes; cancellation is explicit exception/result.

# Examples

Hello Action; property edit; declarative panel; selection-aware tool; data source; exporter; scoped file access; network denied; background job.

# Compatibility

Plugin specifies sdk range. Deprecated methods emit dev warnings before removal window.

# Tests

mypy/Pyright example check, runtime protocol mock, permission simulator and reference plugin CI.