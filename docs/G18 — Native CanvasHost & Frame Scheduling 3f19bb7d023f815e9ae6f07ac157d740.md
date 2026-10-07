# G18 — Native CanvasHost & Frame Scheduling

# Goal

Embed renderer-owned canvas surface into Qt with correct lifecycle/input shell.

# Depends

G06, G08, G14.

# Authority

05.3–05.4, 09.8.

# Owner

renderer-engineer + ui-component-engineer.

# Deliverables

CanvasHost QWidget/QWindow abstraction, DPR/resize/exposure forwarding, normalized mouse/pen/key events, dirty/request-frame scheduler and dummy backend clear/frame output.

# Acceptance

No continuous frame loop while idle; resize/mixed-DPI correct; pen event data captured; hidden canvas pauses; renderer resources owned native.

# Evidence

input trace, frame scheduling tests, platform smoke.