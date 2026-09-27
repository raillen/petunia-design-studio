# 09.27 — UI-Agnostic Core, Ports/Adapters, Presentation Models & Shell Conformance

# Amendment 2026-09-21 — Slint shell

All GPUI-specific wording in this contract is superseded by Slint. The architectural invariant is unchanged: replacing Slint later must not rewrite document semantics, commands, persistence, engines, plugin APIs or MCP schemas. The canonical bridge is **PetuniaDesignGuiBridge** and headless conformance remains mandatory.

<aside>
🔌

**Architectural invariant:** changing Slint to another shell must not require rewriting document semantics, command logic, persistence, geometry, raster, color, typography, plugin contracts or MCP schemas.

</aside>

# Boundary objective

Petunia follows a ports-and-adapters model around application/domain state. The UI is one adapter among several external actors, alongside MCP, plugins, CLI/headless tools and tests.

```
Slint Shell      MCP      Plugins      CLI/Tests
    \            |          |            /
     \           |          |           /
        Semantic Application Ports
                 ↓
        Actions / Commands / Jobs
                 ↓
           Domain + Engines
                 ↓
       Persistence / Render Scene
```

# Forbidden dependency direction

Domain/application crates must not import:

- Slint toolkit types or any other shell-specific types;
- window/entity/component handles;
- toolkit event types;
- toolkit colors/fonts/icons;
- widget references;
- OS dialog types;
- platform clipboard payload objects;
- direct screen coordinates when document/viewport semantics are sufficient.

UI adapters may depend inward. Core never depends outward.

# Canonical application ports

Define narrow semantic interfaces for:

- `ActionQueryPort` — discover actions and enabled/checked/context state;
- `CommandPort` — validated mutations and transactions;
- `PropertyQueryPort` / `PropertyEditPort` — typed semantic property access;
- `DocumentQueryPort` — immutable summaries/snapshots;
- `SelectionPort` — selection summary/requests without widget ownership;
- `JobPort` — long-task progress/cancel/result;
- `ResourcePort` — semantic TextId/IconId/token/help lookup contracts;
- `InspectionPort` — semantic UI/application state for testing/agents;
- `ViewportPort` — toolkit-neutral viewport/session lifecycle where required.

Exact Rust trait/crate placement may evolve, but semantic responsibilities stay separated.

# PetuniaDesignGuiBridge

`PetuniaDesignGuiBridge` is a coarse-grained application facade, not a mirror of every internal object.

It may expose:

- application/session snapshot;
- document tab/session metadata;
- selection summary;
- action-state map;
- panel presentation models/deltas;
- property schemas/values;
- task/progress summaries;
- semantic notifications/dialog requests;
- viewport session handles;
- resource IDs/help topics.

It must not expose mutable document pointers or Slint-specific values.

# Presentation models

A presentation model is immutable derived state optimized for consumption by UI/automation. Examples:

- `LayersPresentationModel`;
- `PropertiesPresentationModel`;
- `HistoryPresentationModel`;
- `ColorPresentationModel`;
- `DataMergePresentationModel`.

Rules:

- identified by stable semantic IDs;
- cheap to diff or replace;
- no hidden mutation methods;
- no toolkit assets;
- no localized string as identity;
- supports empty/loading/error/stale metadata explicitly;
- may include TextId/IconId/help metadata rather than resolved literals.

# Input normalization

Window-system pointer/key/pen events are normalized at the shell/platform boundary before tool logic consumes them. Tool logic should receive semantic/document-oriented input such as pointer position in viewport/document coordinates, button/modifier intent, pressure/tilt and gesture phase — not raw Slint/window-system event objects.

# Canvas boundary

The canvas is an application renderer host, not ordinary widget business logic. Separate:

- window/surface creation;
- viewport transforms/session input;
- scene extraction;
- renderer/compositor;
- editor overlays;
- tool interaction state.

A shell replacement may change window/surface plumbing without redefining scene/document semantics.

# Dialog and platform requests

Core/application emits semantic requests such as `ChooseOpenFile`, `ChooseSaveDestination`, `ConfirmDestructiveAction`, `OpenHelpTopic` rather than invoking Slint/platform dialogs directly. Platform/UI adapters resolve them.

# Diagnostics and text

Core diagnostics carry stable diagnostic code + `TextId` + typed parameters + related IDs. They do not compose English sentences. UI/MCP/CLI choose presentation/localization independently.

# Workspace vs document ownership

Workspace layout, panel visibility, floating-window geometry, density/theme/icon selection and shell geometry stay outside canonical document. Document sessions expose enough state for any shell to reconstruct presentation.

# Headless proof

Maintain an `petunia-design` or dedicated headless harness that can:

- create/open/save documents;
- execute Actions/Commands;
- query properties;
- import/export;
- run plugin/MCP contract tests;
- evaluate scenes/preflight where feasible;

without initializing Slint.

If a new domain feature cannot be exercised headlessly because UI types are required, the boundary is considered broken.

# Mock-shell conformance suite

Provide a toolkit-free `MockGuiAdapter` used in CI. It validates that the application can:

1. enumerate semantic actions;
2. resolve state snapshots;
3. submit property edits/commands;
4. observe jobs and diagnostics;
5. handle dialog requests symbolically;
6. receive presentation deltas;
7. open/close document sessions;
8. run without visual assets resolved.

# Shell adapter conformance

Any real GUI adapter must pass the same semantic conformance suite plus toolkit-specific tests. Required categories:

- action parity;
- command parity;
- focus/key context mapping;
- localization/resource lookup;
- accessibility semantics;
- dialog lifecycle;
- workspace persistence;
- canvas input transform correctness;
- lossless property roundtrip;
- no forbidden core dependency edges.

# Threading boundary

UI thread ownership is an adapter concern. Application ports declare synchronization/ownership contracts; domain logic must not assume Slint executor/UI-thread identity. Background results return through Jobs/ChangeSets and are committed through the authoritative mutation lane.

# Migration test

A shell-replacement spike is successful when representative flows work using only the public semantic bridge:

New/Open → create/edit vector object → property edit → undo → save → export; Photo adjustment workflow; Data Merge preview; accessibility/action discovery.

If a prototype needs to reach into Slint-only internals from domain code, capture the missing semantic port rather than punching a permanent hole.

# Code-agent guardrail

Agents implementing a visible feature must implement/modify core behavior first or in a toolkit-neutral application layer, then bind it to Slint. A change whose only test is a Slint screenshot is incomplete.