# G016 — Generic Property Schema & Inspector Framework

# Goal

Make object/tool/effect properties renderable generically from typed schemas.

# Depends

G005, G007, G013, G014.

# Primary

editor-engineer + ui-component-engineer.

# Skills

editor-tooling, lang-python, component-specification, state-management.

# Deliverables

PropertyId/PropertySchema metadata registry; Same/Mixed/Unavailable/Error value states; native query/set facade; PropertyBindingController; editors for number/bool/enum/color/string/resource; section renderer; begin/preview/commit/cancel lifecycle; validation display; reset/inheritance hooks.

# Acceptance

Rectangle transform/size/fill properties can be edited through generated inspector with one undo per continuous edit and no direct widget mutation of DocumentStore.

# Tests

Mixed multi-select, validation errors, selection change mid-edit, scrub coalescing, keyboard, schema search and plugin-safe schema sample.

# Non-goals

Specialized curve/profile graph editors.