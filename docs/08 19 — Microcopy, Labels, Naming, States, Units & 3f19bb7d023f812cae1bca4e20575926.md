# 08.19 — Microcopy, Labels, Naming, States, Units & Interface Language

# Language

Interface text is concise, task-oriented and consistent. Use verbs for actions, nouns for destinations/panels. Avoid internal implementation terms such as DTO, cache node or nanobind in user-facing UI.

# Labels

Action title is localized metadata for ActionId. Same semantic action keeps same wording across menu, tooltip, command palette and context UI unless space requires approved short label.

# States

Use explicit words for ambiguous states: Live, Destructive, Linked, Embedded, Missing, Read-only, Mixed, Overridden, Disabled. Icons supplement, never replace critical distinction.

# Units

Length fields accept document units with conversion; display can follow document/user preference. Angle degrees default. Percent fields show %. DPI/PPI wording consistent. Decimal formatting locale-aware but internal parsing schema unambiguous.

# Errors

Title says what failed; body explains cause/recovery; technical details collapsible/copyable. Never “Unknown error” when structured code exists.

# Destructive copy

Use outcome language: “Rasterize text and lose text editing” rather than generic “Are you sure?”. Cancel remains safe default.

# Progress

Use phase + object/output context when meaningful: “Exporting Surface 4 of 12”. Avoid fake exact percentages for indeterminate work.

# Empty states

State why empty and primary next action. Example Layers empty is rarely necessary because Surface exists; Assets empty offers Import/Create Library.

# Terminology

Canonical words: Surface, Layer, Mask, Live Filter, Adjustment, Appearance, Persona, Workspace, Studio Panel, Data Merge, Preflight.