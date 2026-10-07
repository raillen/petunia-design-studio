# 06.1 — Blender Prior Art: Operators, Undo, UI/Data Separation & Lessons for Petunia

# Why study Blender

Blender is a mature native creative application with a semantic operator layer, heterogeneous undo types and many editors/workspaces.

# Adopt

**Operator-like semantic Actions:** Blender documentation describes operators as the abstraction invoked by buttons, shortcuts, tools and scripts; Petunia's Action/Command architecture uses the same useful separation but keeps business logic out of UI controllers.

**Undo by operation/data type:** Blender's undo system supports different step types and distinguishes stateful/differential behavior. Petunia similarly allows property deltas, vector topology patches and raster COW tiles under one HistoryManager.

**UI state not canonical undo:** Petunia keeps selection/viewport/workspace separate from document history by default.

**Adjust-after-operation idea:** Petunia can expose recently committed command parameters through HUD/Properties only when the command was designed for safe re-execution/staged update.

# Adapt carefully

Blender's broad context object pattern is specifically something Petunia should avoid. ActionContext must be narrow/typed and business services should not accept omniscient GUI context.

# Avoid

Operators containing low-level business logic; uncontrolled contextual dependencies; mode-specific data stores that fragment one document semantic model.

# Sources

- [Blender Operators developer documentation](https://developer.blender.org/docs/features/interface/operators/)
- [Blender Undo System developer documentation](https://developer.blender.org/docs/features/core/undo/)
- [Blender Undo/Redo manual](https://docs.blender.org/manual/en/latest/interface/undo_redo.html)

# Petunia consequence

Actions are discoverable intent descriptors; Commands/Application services own mutation; different history payload strategies remain hidden behind one semantic undo timeline.