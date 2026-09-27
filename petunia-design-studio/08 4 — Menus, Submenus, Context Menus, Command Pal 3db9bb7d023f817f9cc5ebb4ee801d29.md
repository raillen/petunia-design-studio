# 08.4 — Menus, Submenus, Context Menus, Command Palette, Shortcuts & Search

# Unified action model

Every command has a semantic Aubrieta Action with:

- stable action ID;
- localized title;
- optional description;
- semantic `IconId`;
- default shortcut(s);
- current enabled/checked/mixed state;
- context predicate;
- command execution binding;
- telemetry/test name;
- menu/toolbar eligibility metadata.

GPUI actions are adapters to this registry.

# Main menu bar

Canonical top-level families:

- File
- Edit
- Object
- Layer
- Select
- Text
- View
- Window
- Help

Photo may add Image/Filter semantics through adaptive menu population while keeping menu structure predictable.

# Menu anatomy

Rows support:

- optional leading icon for high-recognition actions;
- label;
- optional state check/radio;
- trailing shortcut;
- trailing submenu chevron.

Use separators for semantic groups, not arbitrary visual rhythm.

# Menu rules

- all essential toolbar commands exist in menus;
- unavailable standard menu items may remain disabled to teach capability and shortcut;
- destructive commands use clear verbs and are separated where useful;
- dynamic recent-file lists are bounded and expose `Clear Menu`;
- submenus are ideally one level deep; two levels only for domain hierarchies that remain understandable;
- menu opening and keyboard navigation must feel immediate.

# Context menus

Context menus are short, selection-specific accelerators.

Examples:

Canvas object: Cut, Copy, Duplicate, Group/Ungroup, Arrange, Lock, Hide, Convert/Expand, Export Selection.

Layers row: Rename, Duplicate, Group, Mask, Clip, Lock, Hide, Locate on Canvas, Export.

Panel tab: Close, Float, Dock, Move.

Swatch: Apply Fill, Apply Stroke, Edit, Duplicate, Delete.

Rules:

- hide irrelevant items rather than presenting dozens of disabled actions;
- keep frequently used commands near pointer reveal point;
- no essential action exists only here;
- submenu depth one preferred;
- labels adapt to current semantic object, e.g. `Rasterize Text` rather than generic `Rasterize` when useful.

# Command palette

**V1 default:** `Ctrl/Cmd+K` opens the command palette. The binding is user-configurable through the semantic keymap system and platform conflicts may be adapted only through a documented platform profile.

Palette structure:

- search/input row;
- optional filters: All, Actions, Tools, Files, Settings, Help;
- ranked results;
- recent commands;
- contextual suggestions;
- shortcut hints;
- optional secondary preview/help area for complex actions.

Search matches action titles, aliases, tool names and settings. Fuzzy matching must tolerate abbreviations but prioritize exact semantic matches.

Examples:

`convert curves` → Convert to Curves

`proof` → Toggle Proof Colors

`workspace` → Switch Workspace…, Save Workspace As…

`data merge` → Open Data Merge, Preview Record, Generate Merge

# Shortcut editor

Preferences exposes searchable command table:

- action name;
- context;
- current bindings;
- conflict indicator;
- restore default.

Shortcut capture modal/popup must distinguish:

- chord sequence;
- modifier-only invalid bindings;
- conflicts in same context;
- conflicts allowed across non-overlapping contexts.

Affinity/Adobe/Corel/Blender-familiar keymap profiles are **Post-V1 Candidate** presets. Aubrieta V1 ships one coherent documented default plus user-editable bindings; presets must never fork Action semantics.

# Key contexts

Examples:

- Global;
- Canvas;
- TextEditing;
- NodeEditing;
- PhotoBrush;
- Dialog;
- LayersPanel;
- DataMergeTable.

Text entry consumes typing shortcuts safely; global destructive commands cannot accidentally fire while editing numeric/text fields unless explicitly designed.

# Search

Search fields follow a standard contract:

- leading search icon;
- clear button when nonempty;
- Esc clears first, then closes transient search if pressed again;
- optional filter button/chips;
- debounced only where underlying query is expensive;
- result count where useful.

# Submenus and disclosure

Prefer submenu for hierarchical variants (`Arrange > Bring Forward`) and popover for parameterized actions (`Export…`, `Stroke Options…`). Ellipsis in command names means additional input/confirmation follows.