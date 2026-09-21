# 08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract

# Amendment 2026-09-21 — Slint Interaction Grammar

Tool interaction is rendered by Slint but remains application/tool-state semantics. Slint callbacks may request transitions; they do not own tool state. Every visible tool must expose current state, cancel/commit semantics, numeric takeover, modifiers, cursor, status hint and accessibility metadata through PetuniaDesignGuiBridge or toolkit-neutral presentation contracts.

<aside>
🖱️

**Canonical interaction grammar:** tools may differ in geometry semantics, but they must not invent unrelated interaction languages for selection, preview, commit, cancellation, modifiers, snapping or numeric precision.

</aside>

# Normalized tool state

Inactive → Ready → Hover Target → Armed → Dragging/Drawing/Editing → Preview → Commit → Ready.

Optional branches are Candidate Selection, Numeric Entry, Suspended by Pan, Recoverable Error and Cancel/Rollback.

# Context toolbar ordering

Use stable groups from left to right:

1. target or editing scope;
2. primary tool mode;
3. geometry or brush parameters;
4. behavior toggles;
5. local snapping and constraints;
6. explicit Bake/Expand/Commit only where an irreversible boundary exists;
7. advanced overflow.

Equivalent semantic properties keep the same label, unit rules, control family and reset behavior across tools.

# Modifier-key contract

Affinity relies heavily on modifiers and exposes available modifiers in the Status bar.[[1]](https://affinity.help/designer2/English.lproj/pages/Workspace/interface.html) Aubrieta keeps expert modifiers but requires:

- status-bar discoverability;
- tooltip documentation;
- keyboard-help search entry;
- an accessible alternative when a modifier-only action would otherwise be essential;
- inspection metadata listing active modifiers;
- no required V1 workflow available only through an undiscoverable modifier.

# Temporary tool overrides

Holding a tool shortcut may temporarily activate a tool when safe and return to the previous tool on release. Temporary Pan/View overrides are presentation state and never document mutations.

# Preview and commit classes

- **continuous commit:** Move, node drag, paint stroke;
- **candidate then commit:** Shape Builder-style region selection;
- **live parameter object:** Contour, Corner, Live Boolean;
- **explicit destructive commit:** Bake, Expand, Rasterize.

The current class is inspectable. Esc restores the transaction baseline for uncommitted preview.

# Canvas handles

Handles encode role by shape + cursor + optional colour, never colour alone. Roles include transform, rotation, node, Bézier handle, corner radius, contour offset, gradient stop, transparency stop, crop, width point, text-flow link and guide.

Pointer hit area scales separately from visible glyph size for HiDPI and pen usability.

# Numeric takeover

When direct manipulation has an exact numeric equivalent, users may switch from drag to typed precision without restarting the operation. Examples: transform delta, contour radius, corner radius, width, angle and guide position.

# Tool switching during active work

Each tool fixes one policy:

- commit safely then switch;
- cancel then switch;
- prompt only when data would otherwise be lost.

This behavior is part of its specification and tests.

# Status and hint model

The status area exposes:

- current tool and short intent;
- current modifier alternatives;
- transient operation readout such as delta, angle, radius, node count or snap target.

Critical diagnostics never live only in the status area.

# Keyboard and accessibility

Each tool declares activation shortcut, focus entry points for context controls, canvas keyboard commands, Escape semantics, accessible role/name/value, live-region policy for meaningful changes and a non-pointer alternative for primary workflow where practical.

# Automation

Tool UI is not the automation API. MCP, plugins and tests invoke the same semantic Actions/Commands and may query active tool, tool mode, selection summary, snap candidates/result, preview state and semantic hit targets.

# Affinity references

Move and Node demonstrate the selection/edit split and context-specific controls.[[2]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_move.html)[[3]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_node.html)

Pen demonstrates explicit modes plus local snapping; Pencil demonstrates Sculpt, Auto Close, Smoothness, pressure/velocity controller and Rope/Window stabilizers.[[4]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_pen.html)[[5]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_pencil.html)

# Improvement criterion

Aubrieta should feel immediately legible to an Affinity/Illustrator-class user, while a new user should not need hidden modifier knowledge to discover the safe primary path.