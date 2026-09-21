# 08.21 — Design System Governance, Token Enforcement, Customization & UI Portability Contract

<aside>
🎛️

**Portability rule:** GPUI is the primary shell, but Aubrieta's visual language and semantic UI contracts must survive a future shell replacement. No product feature owns raw GPUI presentation as its canonical definition.

</aside>

# Design-system ownership

The Aubrieta Design System owns presentation vocabulary. Feature modules consume semantic components and tokens; they do not define ad-hoc visual constants.

Canonical layers:

```
Semantic product state / Actions / Property Schemas
                ↓
        AubrietaGuiBridge
                ↓
Presentation Models + Semantic Component Contracts
                ↓
       Aubrieta Design System
                ↓
          GPUI Adapter Today
                ↓
      Alternative Shell Tomorrow
```

# What must never be hard-coded in feature UI

Outside canonical resource/token definitions, production feature code must not contain:

- user-visible English/Portuguese strings;
- literal UI colors or palette hex/RGB values;
- direct SVG/icon filenames;
- hard-coded font family/size/weight for product UI;
- arbitrary spacing, radius, separator, shadow or elevation values;
- motion durations/easing literals;
- cursor asset paths;
- help/documentation URLs;
- tooltip text;
- shortcut display strings;
- accessibility labels/descriptions;
- component-specific duplicated state colors;
- fixed locale-sensitive widths chosen from English copy.

Document colors, export parameters and geometry values are domain data and are not UI tokens.

# Semantic resource identifiers

Use extensible IDs such as:

```
TextId      aubrieta.action.export.title
IconId      aubrieta.icon.boolean.union
TokenId     aubrieta.token.surface.panel
CursorId    aubrieta.cursor.node_edit
HelpTopicId aubrieta.help.boolean_operations
SettingId   aubrieta.setting.ui.density
```

Built-ins may have generated Rust constants, but runtime registries remain open to validated plugin/resource namespaces.

# Token hierarchy

Prefer **primitive → semantic → component** indirection.

Example:

```
palette.neutral.900
        ↓
surface.panel
        ↓
panel.background.default
```

Feature code should normally consume semantic/component roles, never primitive palette values.

# Token domains

The token model must cover at least:

- color;
- typography;
- spacing;
- size/min/max metrics;
- radius;
- border/separator;
- elevation/shadow;
- opacity;
- motion duration/easing;
- density metrics;
- focus treatment;
- interaction hit-box metrics;
- canvas overlay presentation;
- icon sizing/alignment;
- cursor identity;
- z-order/elevation semantics where toolkit-independent.

# Semantic component layer

Prefer wrappers/contracts such as `AubrietaButton`, `AubrietaIconButton`, `PropertyField`, `PropertyRow`, `PanelSection`, `AubrietaTreeRow`, `ToolButton`, `ContextToolbarGroup`, `AubrietaPopover`, `AubrietaDialog`, `EmptyState`, `InlineDiagnostic` and `ProgressSurface`.

Wrappers exist to centralize tokens, semantics and behavior — not to hide the toolkit behind meaningless pass-through classes.

# Component contract

Every reusable component specifies:

- semantic purpose;
- required/optional properties;
- states;
- keyboard behavior;
- pointer behavior;
- focus semantics;
- accessibility role/name/value/actions;
- localization expansion behavior;
- density behavior;
- token dependencies;
- error/mixed/loading state;
- automation/test identifier policy.

# No raw toolkit leakage

GPUI entities/elements/actions/assets may exist inside `aubrieta_shell`. They may not appear in domain/application public interfaces, canonical PropertyDescriptor contracts, document serialization, plugin APIs or MCP schemas.

If a feature needs GPUI-only functionality, expose the smallest toolkit-neutral semantic capability at the bridge and implement it in the adapter.

# Presentation models

Panels consume immutable snapshots/deltas containing semantic state, stable IDs and typed values. They must not receive mutable document references. UI edits become Action/Command/PropertyEdit requests.

A presentation model is derived state and may be rebuilt at any time.

# Customization policy

Users may change presentation without changing semantics:

- Dark/Light/System theme;
- semantic accent family;
- density;
- icon family;
- workspace layout;
- selected display preferences;
- keymap;
- safe resource packs.

Customization may not replace command meaning, bypass permissions, mutate document color, or redefine accessibility semantics.

# Resource-pack validation

Before activation validate:

- manifest/schema version;
- namespace ownership;
- missing required tokens;
- alias cycles;
- invalid token types;
- unsafe SVG/external content;
- contrast-critical combinations;
- missing locale fallback;
- license/provenance metadata;
- resource/path limits.

Invalid keys fall back independently where safe; an invalid pack cannot make the application unbootable.

# Fallback order

Presentation resources resolve deterministically:

1. built-in canonical defaults;
2. selected compatible pack/theme/icon/locale;
3. user override layer;
4. built-in safe fallback for any invalid/missing key.

Diagnostics record missing/invalid IDs without exposing raw internal errors to ordinary users.

# Development token enforcement

CI/code review should detect likely violations. Recommended checks include:

- literal hex/RGB colors outside token/resource definitions;
- direct icon/SVG paths in feature modules;
- direct UI label strings outside localization catalogs/tests;
- raw spacing/radius/motion constants outside Design System modules;
- GPUI types referenced by forbidden crates;
- resource IDs registered but undocumented;
- TextId/IconId references missing from canonical catalogs.

Allow explicit test/prototype escape annotations only with issue/ADR reference.

# Component gallery

Maintain an executable `aubrieta-ui-story`/gallery containing every reusable component in:

- all states;
- all densities;
- Dark/Light/System representative themes;
- Lucide/Phosphor/Aubrieta icon families;
- long pseudo-localized labels;
- RTL sample;
- keyboard focus;
- accessibility semantics;
- 100/150/200% scale;
- error/loading/mixed states.

Gallery screenshots and semantic snapshots are regression inputs.

# Shell portability gauntlet

A future GUI change should not require changing document/application semantics. The bridge is considered portable when a mock/headless adapter can:

- enumerate actions and state;
- read presentation models;
- submit edits/commands;
- host canvas viewport lifecycle;
- resolve resources/tokens externally;
- expose semantic accessibility/inspection tree;
- persist workspace state independently.

# Migration rule

When adopting a new toolkit, first implement the semantic bridge and component primitives. Do not rewrite feature business logic inside the new shell. Any core change required solely because a UI toolkit changed is an architecture regression unless an ADR proves the old boundary was wrong.

# Code-agent rule

Agents must search the Design System and Atlas before creating a control, token or interaction. Duplicate semantic components are defects. New component/token proposals must include states, accessibility, localization, fallback and portability behavior.