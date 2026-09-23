# 15 — Petunia Design Studio Rebrand, Slint Migration & Total Assurance Program

<aside>
🌺

**Canonical decision — 2026-09-21:** Aubrieta Design is retired as the active product identity. The product is **Petunia Design Studio**; native project suffix is **.PTND**; Slint is the primary UI adapter. This program governs the rename, UI migration and no-gap documentation refactor under the same Total Assurance discipline used by Prumo.

</aside>

# Goals

1. migrate product identity without ambiguous aliases;
2. preserve the UI-agnostic Rust core;
3. replace GPUI as implementation authority with Slint;
4. rebuild the UI against an explicit, pixel-level Petunia Design System;
5. use the 2026 Affinity Vector Studio as ergonomic prior art while keeping Petunia branding and architecture original;
6. integrate Prumo Total Assurance: surface inventory, evidence graph, negative guarantees, failure/recovery, performance/security, proof-of-use and hard release gates;
7. prevent documentation from declaring a feature ready when code, interaction or evidence is missing.

# Canonical identity

- Product: **Petunia Design Studio**
- Product family: **Petunia**
- Native extension written by the app: **.PTND**
- Legacy names: Aubrieta Design, .aubrieta, .aubri — migration/import only
- Executable/CLI recommendation: **petunia-design**
- Rust crate prefix recommendation: **petunia_design_**
- Resource/Action namespace recommendation: **ptnd.**
- Application ID recommendation: [**studio.petunia.design**](http://studio.petunia.design)
- GUI bridge: **PetuniaDesignGuiBridge**
- Native format identity: schema/media identity is authoritative; extension alone is not.

# Non-goals

This rebrand does not authorize unrelated feature creep, a cloud requirement, Canva/Affinity branding reuse, AI dependency, or rewrite of the domain engines. Slint replaces the shell implementation authority, not the canonical Document/Action/Command model.

# Program phases

## P0 — Identity freeze

Freeze new Aubrieta identifiers. Add deprecation aliases only where migration needs them.

## P1 — Documentation migration

Update canonical pages, generated docs, TextId/IconId examples, file-format docs, screenshots/goldens, diagrams, commands, package names and user-facing terminology.

## P2 — Slint foundation

Implement Petunia Design System primitives, app shell, command/action bindings, docking/panel model, dialogs, menus, accessibility and semantic test IDs.

## P3 — Functional binding

Bind Design and Photo personas through PetuniaDesignGuiBridge. Zero UI-only commands.

## P4 — Visual conformance

Match the approved Petunia interface specification at supported DPI/window sizes; compare against reference ergonomics, not copied brand assets.

## P5 — Total Assurance

Run UI interaction, accessibility, localization, performance, security, persistence, failure/recovery and clean-state proof-of-use Gauntlets.

## P6 — Legacy cleanup

Remove active Aubrieta/GPUI authority only after format migration and documentation/code search are clean. Historical ADRs may retain old wording when explicitly labelled historical.

# Required hard gates

- no active canonical page still calls Aubrieta the current product;
- no new public identifier uses aubrieta namespace;
- .PTND save/open/round-trip specified and tested;
- Slint never leaks into core/domain public contracts;
- every visible UI control has semantic action/behavior or intentional disabled reason;
- no screenshot-only acceptance;
- primary workflows are keyboard-accessible where applicable;
- performance budgets exist before RC;
- malformed input/security corpus exists for format/import surfaces;
- release artifact is tested from clean state.

[15.F — Master Slint UI Implementation & Conformance Contract](15%20F%20%E2%80%94%20Master%20Slint%20UI%20Implementation%20&%20Conformanc%203e29bb7d023f8145a710d72d4b5adb2f.md)

[15.G — Canonical Surface Registry Seed: Menus, Tools, Panels, Dialogs & Commands](15%20G%20%E2%80%94%20Canonical%20Surface%20Registry%20Seed%20Menus,%20Tool%203e29bb7d023f81a2a5cdc0658fd8a602.md)

[15.H — Performance, Memory, GPU, Security & Reliability Torture Suite](15%20H%20%E2%80%94%20Performance,%20Memory,%20GPU,%20Security%20&%20Reliab%203e29bb7d023f81409ceaf8cf13c69f80.md)