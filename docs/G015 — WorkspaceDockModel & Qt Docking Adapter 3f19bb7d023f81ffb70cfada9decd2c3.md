# G015 — WorkspaceDockModel & Qt Docking Adapter

# Goal

Implement toolkit-neutral workspace persistence with Qt rendering adapter.

# Depends

G013, G014.

# Primary

ui-component-engineer + architect.

# Skills

lang-python, ux-architecture, state-management, testing-quality.

# Deliverables

Typed DockTree/Split/TabStack/PanelInstance/Floating/MissingProvider models; schema/migration v1; PanelRegistry baseline; QDockWidget/custom adapter; drag legal-target logic; floating geometry; collapse/close/reset; workspace save/load.

# Acceptance

Roundtrip workspace independent of Qt saveState blob. Missing PanelId produces placeholder and restores when provider returns. Monitor disappearance/DPI change clamps geometry.

# Tests

Nested splits, tab reorder, floats, missing plugin panel, corrupt schema fallback, keyboard docking and restart restore.

# Non-goals

All real panels; advanced custom docking visuals beyond usable baseline.