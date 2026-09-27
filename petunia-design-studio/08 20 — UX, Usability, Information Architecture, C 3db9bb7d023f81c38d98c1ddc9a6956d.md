# 08.20 — UX, Usability, Information Architecture, Cognitive Load & User Research

<aside>
🧠

**Canonical usability contract.** Visual polish is insufficient. Every Aubrieta workflow must be learnable, predictable, recoverable, efficient for experts and testable with semantic usability criteria.

</aside>

# Product usability goals

Aubrieta serves both new creative-tool users and experienced designers. The interface therefore optimizes for **progressive mastery** rather than choosing between beginner simplicity and expert density.

A feature is not usable merely because it can be reached. It must be discoverable, understandable, reversible when possible, consistent with neighboring workflows and efficient after repetition.

# User profiles as workflow lenses

Do not invent demographic personas without research. Use workflow profiles to validate decisions:

- **New user:** needs recognition, safe defaults, visible next actions, contextual help and forgiving undo.
- **Experienced vector designer:** expects direct manipulation, precise numeric entry, keyboard accelerators, predictable Layers/Properties behavior and low interaction latency.
- **Photo retoucher:** expects pen/brush continuity, nondestructive adjustments, mask visibility, before/after feedback and interruption-free canvas input.
- **Print/production designer:** expects units, CMYK/profile clarity, bleed/preflight, deterministic export and no silent color degradation.
- **Automation/plugin user:** expects semantic identifiers, discoverable commands, deterministic scripts and understandable permissions.

# Jobs-to-be-done requirement

Every major subsystem documents at least one end-to-end job, not just components. Required structure:

1. user intent;
2. entry points;
3. minimum successful path;
4. optional expert accelerators;
5. error/recovery branches;
6. keyboard-only path where applicable;
7. automation equivalent where applicable;
8. persistence/undo consequences;
9. completion signal.

Examples include create logo from primitives, edit a complex imported SVG, retouch a photo nondestructively, produce a CMYK flyer, generate certificates from CSV and export selected Surfaces.

# Information architecture

Navigation and command placement follow stable semantic homes:

- **File** owns document lifecycle/import/export handoff;
- **Edit** owns general editing/history/preferences conventions;
- **Object/Layer/Select/Text/View/Window** retain recognizable creative-tool meaning;
- persona-specific additions must not create parallel names for the same semantic operation;
- Properties is selection-driven inspection, not a dumping ground;
- dedicated panels exist only when a persistent mental model or browsing workflow justifies them.

A command may have multiple entry points, but one semantic `ActionId` and one canonical name.

# Recognition over recall

- common actions must be visible or easily searchable;
- uncommon icon-only actions require tooltip and accessible name;
- low-frequency or ambiguous actions should prefer icon + text in menus/palettes;
- command palette search includes aliases and industry-standard terminology;
- current tool, Persona, selection mode and transient preview mode must be visually and semantically inspectable.

# Progressive disclosure

Three levels are preferred:

1. primary controls visible in context;
2. advanced controls inside section disclosure/popover;
3. subsystem window for high-dimensional configuration.

Do not hide basic success paths behind nested disclosures. Avoid more than two nested collapsible levels in persistent panels.

# Cognitive-load rules

- one concept must keep one canonical name across menu, tooltip, panel, documentation and MCP metadata;
- a visual group should correspond to one user decision cluster;
- avoid simultaneous competing accents;
- defaults should make the common case successful without setup;
- destructive, irreversible or fidelity-losing choices require consequence-first wording;
- never require users to memorize whether a state lives in document, workspace or preference scope;
- mode changes must have persistent indication and an obvious exit path;
- no tool may trap input such that Esc/undo behavior becomes ambiguous.

# Direct manipulation contract

Dragging, painting, node editing and transforms must provide immediate visual feedback independent from final expensive computation. Preview and commit are separate where computation is costly.

Targets:

- pointer/pen feedback is frame-bound and never waits on filesystem/network;
- ordinary state changes should feel perceptually immediate, generally under 100 ms on representative hardware;
- operations that exceed roughly 250 ms should expose busy/progress state when the user would otherwise question whether input registered;
- long tasks around or above 1 s should be cancelable where consistency permits and expose structured progress.

These are design budgets, not permission to hide regressions behind spinners.

# Error prevention before error messaging

Prefer constraints, previews and disabled-invalid actions over accepting impossible input and failing later. Examples:

- prevent invalid circular plugin dependency at registration;
- constrain impossible numeric ranges with explanation;
- preflight export degradation before writing;
- show missing capability before a workflow begins;
- validate Data Merge mappings before batch generation.

# Recovery and reversibility

- editing commands are undoable unless explicitly documented otherwise;
- destructive conversion uses explicit verbs such as Rasterize, Expand or Bake;
- Cancel/Esc restores pre-operation state for in-progress transactions;
- autosave/recovery never masquerades as explicit Save;
- error dialogs preserve user work and explain the safest recovery action first.

# Novice-to-expert progression

Beginner assistance must not slow experts:

- tooltips may include shortcut + concise purpose;
- status bar can show context-sensitive interaction hints;
- command palette provides discoverability without menu hunting;
- shortcuts and workspace presets are customizable;
- advanced controls can be disclosed without duplicating the feature in a separate expert-only subsystem;
- onboarding is optional, short and never blocks opening a file.

# Consistency heuristics

Before creating a new interaction pattern, check whether Button, Toggle, SegmentedControl, PropertyField, TreeRow, Popover, Dialog or existing canvas HUD already solves it. New patterns require a design-system extension rationale.

Consistency applies to:

- selection language;
- mixed values;
- reset/revert affordances;
- destructive confirmation;
- drag/drop previews;
- disabled reasons;
- progress/cancel behavior;
- keyboard activation;
- error placement.

# Usability acceptance scenarios

At milestone gates, test representative end-to-end scenarios with a clean profile and with an expert keymap/workspace:

1. create document → draw/edit shape → style → save;
2. import SVG → inspect hierarchy → edit path/text → export;
3. place photo → mask → adjustment → compare → save/export;
4. create multi-Surface print design → bleed/profile → PDF preflight/export;
5. attach CSV → bind fields → preview records → generate/export;
6. discover and execute an unfamiliar action through menus/help/command palette;
7. recover from missing font/link/export failure without losing document state.

# Metrics

Use metrics as signals, not gamified targets:

- task completion rate;
- first-attempt success;
- wrong-mode/wrong-target errors;
- number of unexpected undo operations;
- time to discover an unfamiliar command;
- repeated-task time after learning;
- keyboard-only completion;
- cancellation/recovery success;
- percentage of actions reachable without hidden context menus;
- usability regression reports per release.

# Usability testing protocol

Prototype tests may be internal initially but must be structured. Record task, starting state, expected semantic outcome, observed confusion, errors, unnecessary navigation and recovery behavior. Convert recurring failures into Atlas changes or design-system rules rather than one-off patches.

# Accessibility as usability

Accessibility is not a separate polish phase. Keyboard, screen-reader semantics, contrast, reduced motion, localization expansion, touch/pen target sizing and IME behavior are part of the same interaction contract. See 08.14.

# Documentation requirement

Every new user-visible feature documents:

- user goal and primary workflow;
- discoverability/entry points;
- empty/loading/error/disabled states;
- direct-manipulation and keyboard behavior;
- undo/cancel semantics;
- accessibility labels/roles;
- localization implications;
- usability risks and acceptance scenario.

# Code-agent rule

A code agent must not invent a new UX pattern solely because it is easier to implement. If existing components cannot express the intended behavior, the agent must propose a Design System/Atlas extension and document why before introducing the pattern.