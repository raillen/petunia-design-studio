# G15 — WorkspaceDockModel & Qt Dock Adapter

# Goal

Implement canonical inspectable docking/workspace system.

# Depends

G13–G14.

# Authority

05.2, 08.3, 22.3.

# Owner

ui-component-engineer + architect.

# Deliverables

DockTree schema, PanelRegistry skeleton, QDockWidget/custom adapter, split/tab/float operations, persistence/migration and missing-provider placeholder.

# Acceptance

Save/restore layout independent from Qt opaque state; plugin panel missing/return roundtrip; monitor geometry clamped; keyboard dock/float actions.

# Evidence

workspace JSON fixtures and mixed-DPI restore tests.