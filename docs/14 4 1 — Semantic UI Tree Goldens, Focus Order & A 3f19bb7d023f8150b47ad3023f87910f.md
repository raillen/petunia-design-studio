# 14.4.1 — Semantic UI Tree Goldens, Focus Order & Action/Property Binding Verification

# Semantic snapshot

Capture window/workspace hierarchy, ToolId/PanelId/control semantic IDs, roles, labels, enabled/visible/checked/value state, ActionId/PropertyId binding, focus order and critical bounds.

# Stability

Snapshot ignores volatile object addresses, localized labels when testing semantic IDs, timestamps and noncritical pixel geometry. Separate locale tests verify labels.

# Assertions

No unlabeled interactive controls; no duplicate semantic IDs in same scope; disabled action includes reason when discoverability rule requires; focused control exists/visible; control binding points to registered action/property.

# Panels

Tree/table row semantics sampled for representative hierarchy, not serialized millions of rows. Virtualization must expose currently materialized/accessibility-visible rows correctly.

# Dialog

Initial focus/default/cancel and validation relationships included.

# Tests

Dark/light should share semantic tree except theme-specific states; plugin panel contributes valid host-rendered semantics.