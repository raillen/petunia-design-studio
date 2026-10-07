# 05.2 — Custom Docking Model on Qt: QDockWidget Adapter, DockTree, Floating Palettes & Restoration

# Model first

WorkspaceDockModel is toolkit-neutral and serializable. Qt adapter renders it through QDockWidget/QTabBar/split arrangements where Qt behavior is sufficient.

# Why not trust Qt state blob

Qt saveState/restoreState may be used as optimization/compatibility aid but **cannot be the canonical workspace format**. Petunia needs stable PanelId, plugin absence recovery, migrations and inspectability.

# Dock node types

Split, TabStack, PanelInstance, FloatingWindow, CollapsedRail, PlaceholderMissingProvider.

# Operations

dock, float, split, merge tabs, reorder, collapse, close, restore, reset preferred size. Every operation mutates workspace model then adapter reconciles widgets.

# Drag targets

Custom overlay can replace ambiguous native indicators if Qt defaults do not meet UX. Model calculates legal targets based on PanelDescriptor.

# Restoration

Resolve PanelIds after all built-in/plugin providers register. Missing panels become placeholders. Geometry clamped to available screens and corrected for DPI/topology changes.

# Performance

Panel movement must not trigger expensive document re-render beyond required viewport resize. Splitter drag throttles expensive content only, not cursor/feedback.

# Tests

Roundtrip DockTree, plugin disappears/returns, monitor removed, DPI changed, corrupt workspace, multiple instances and keyboard docking.