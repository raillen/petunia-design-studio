# 13.7 — Documentation Completeness Checklist per Subsystem

# Complete subsystem checklist

A subsystem is documentation-complete when it has:

- purpose/non-goals;
- canonical data model;
- IDs/types/enums;
- invariants/validation;
- ownership/lifetime;
- threading/concurrency;
- operations/Commands;
- errors/recovery;
- UI interaction if exposed;
- plugin/MCP surface or explicit exclusion;
- serialization/migration;
- performance budget/measurement;
- accessibility if user-facing;
- security/trust boundary;
- unit/property/fuzz/golden tests;
- implementation Goal/dependencies;
- open ADRs explicitly listed.

# Application

The checklist is applied to Geometry, Raster, Renderer, Text, Color, PTND, Import/Export, UI shell, Tools, Panels, Plugins, MCP, Data Merge, Prepress, History and Presets.

# No false closure

A page with many paragraphs but missing invariants/tests is not complete. A short page can be complete if it points to exact normative schemas/contracts elsewhere.

# Review

Documentation-maintainer and architect perform this checklist at milestone boundaries; implementation reality verification updates status separately.