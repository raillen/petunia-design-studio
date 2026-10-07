# G054 — Semantic UI Inspection, Developer Input & Automation Conformance

# Goal

Complete agent/test semantic inspection surface.

# Depends

G019, G053, Qt accessibility.

# Primary

ui-component-engineer + tester + security review.

# Deliverables

ui.get_state/inspect/list_panels/focus; canvas semantic state; developer screenshot/synthetic input scopes; semantic tree goldens; direct-action vs UI-gesture conformance.

# Acceptance

Agent/test can locate controls/tools/panels by stable semantic IDs without coordinate guessing; production scopes cannot control OS UI.

# Tests

localization, plugin panel, dialogs, mixed DPI, synthetic Pen/Move flow and authorization.