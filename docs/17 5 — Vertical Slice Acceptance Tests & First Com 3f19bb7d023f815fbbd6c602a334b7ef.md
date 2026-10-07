# 17.5 — Vertical Slice Acceptance Tests & First Commercial Demo Definition

# Demo A — Vector identity card/poster

Create new 1920×1080 Surface; draw rectangle/circle; Pen path; gradient; text; align; group; rename layers; undo/redo; save PTND; reopen; export PNG and SVG.

Acceptance:

- zero canonical drift after reopen;
- SVG remains editable for supported objects;
- PNG visual golden passes;
- Layers/Properties reflect changes;
- keyboard can complete critical flow;
- no Python hot-loop performance issue.

# Demo B — Mixed photo poster

Place high-res image; crop; brush mask; add Curves/HSL; add vector title/logo; blend/effect; save/reopen; export PNG/PDF.

Acceptance:

- raster tiles lazy/bounded;
- adjustment remains editable;
- vector/text not rasterized by Persona switch;
- output color profile reported;
- memory/frame budgets recorded.

# Demo C — Data-driven social set

Template Surface + text/image bindings; CSV source; preview rows; preflight; generate 20 exports.

Acceptance:

- preview does not mutate template;
- filename sanitization/collision works;
- missing data produces structured issue;
- cancellation leaves completed outputs valid and no partial corrupt file.

# Demo D — Extension/automation

Install sample third-party Python plugin panel/action; grant scoped file permission; invoke equivalent MCP action; inspect semantic UI.

Acceptance:

- plugin crash isolated;
- permissions enforced;
- MCP and UI semantic outcomes equal;
- audit logs identify source/action without private content.

# Commercial pre-alpha gate

All four demos do not need full feature breadth, but must prove the architecture can support real work end-to-end.