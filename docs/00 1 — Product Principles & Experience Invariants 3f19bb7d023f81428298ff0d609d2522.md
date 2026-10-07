# 00.1 — Product Principles & Experience Invariants

# Product principles

1. **One document, multiple disciplines.** Vector, raster and layout coexist without persona-conversion.
2. **Nondestructive by default.** Destructive conversions are explicit named commands.
3. **Precision without ritual.** Pointer workflows remain fast; exact numeric control is always available.
4. **Discoverable expertise.** Context bar supports common task; Properties exposes full contract; command search finds everything.
5. **Recoverability over cleverness.** Undo, atomic save, recovery and transparent degradation outrank magical automation.
6. **Local-first professional work.** Core creation/edit/save/export works without account/cloud.
7. **Open format and semantic automation.** PTND, Actions, plugins and MCP are product architecture, not afterthoughts.
8. **Accessibility is baseline.** Keyboard, screen reader, readable focus, cognitive stability and user-scalable controls are product requirements.
9. **Performance is interaction quality.** Heavy work can be asynchronous; cursor/typing/canvas feedback cannot casually block.
10. **Honest interoperability.** Export/import says what is preserved, expanded, rasterized or unsupported.

# UX invariants

Persona switching never mutates content.

Undo label describes semantic user action.

Every long task can report progress and cancel when technically safe.

Every file mutation either succeeds atomically or leaves original recoverable.

Every important pointer-only gesture has numeric/command alternative where reasonable.

Every disabled action can explain why in discoverable UI.

Every document-visible state has one authority; no duplicate Python/UI shadow truth.

# Engineering consequence

A feature that violates these invariants needs ADR/product decision, not local exception.