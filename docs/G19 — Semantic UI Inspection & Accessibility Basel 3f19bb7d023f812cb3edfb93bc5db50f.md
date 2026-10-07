# G19 — Semantic UI Inspection & Accessibility Baseline

# Goal

Expose machine-readable and platform-accessible semantics early.

# Depends

G14–G18.

# Authority

05.10, 09.15.3, 08.15.

# Owner

accessibility-reviewer + ui-component-engineer.

# Deliverables

stable semantic/test IDs, QAccessible names/roles for shell/components/Layers, UI inspection tree service, focus-zone manager and keyboard panel cycling.

# Acceptance

Screen-reader inspection sees essential shell/actions/panel rows; MCP/test inspector can query active window/tool/panel/focus; no screenshot required for semantic assertion.

# Evidence

accessibility tree snapshots and keyboard-only smoke.