# 04.3 — nanobind Contract, Native Module Surface, Type Stubs & Call-Cost Budget

# Why nanobind

nanobind is used as a thin, modern C++↔Python binding layer. It is not the architecture itself.

# Native module

Expose a small module, conceptually:

```
petunia_native
  ApplicationCore
  DocumentSession
  DocumentSnapshot
  RenderSession
  QueryService
  JobHandle
  value enums/IDs/errors
```

# Coarse-grained API

Do not expose one Python call per node, glyph, pixel or draw command. Favor batch operations:

- query_objects(filter)
- hit_test_batch(...)
- update_tool_preview(...)
- apply_transaction(...)
- fetch_layer_rows(range)
- render_frame(request)
- brush_add_samples(batch)

# Call-cost budget

Interactive operations should cross the binding at semantic event/batch frequency, not inner-loop frequency. Profiling tracks Python↔C++ calls per frame/gesture to detect accidental chatty APIs.

# Type stubs

Generate/maintain .pyi with exact enums, dataclasses/value wrappers, overloads and errors. Pyright strict treats native module as typed product code, never as Any.

# Conversion

Prefer native ID/value wrapper types and lightweight immutable structs. Avoid arbitrary nested Python dicts for high-frequency calls.

# Buffers

Use Python buffer protocol only where large contiguous data legitimately crosses boundary. Lifetime ownership must be explicit; zero-copy is not automatically safer/faster.

# GIL

Bindings annotate native work that can release GIL. Reacquire only before constructing/calling Python objects.

# ABI

nanobind module is rebuilt with the application; it is not a public stable third-party ABI.