# G16 — PropertySchema Registry & Generic Inspector

# Goal

Create shared property metadata/editing path used by tools, panels, plugins and MCP.

# Depends

G05–G07, G13–G14.

# Authority

09.25, 05.7, 19.02.

# Owner

editor-engineer + ui-component-engineer.

# Deliverables

PropertyId, schema types, value states Same/Mixed/etc., native query/set facade, PropertyEditorFactory, numeric/bool/enum/color placeholder editors and preview transaction lifecycle.

# Acceptance

Editing rectangle transform/property through inspector invokes same Command as headless Action; mixed values correct; invalid/stale edit rejected; one scrub = one undo item.

# Evidence

binding parity tests and UI semantic snapshot.