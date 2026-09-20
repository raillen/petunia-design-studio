# 08.22 — Affinity Reference Model, Aubrieta Divergence Rules & Workflow Vocabulary

<aside>
🧭

**Purpose:** define how Affinity is used as interaction prior art without making Affinity behavior an undocumented dependency. Aubrieta preserves familiar creative-tool workflows where they are strong, improves friction points deliberately, and keeps its own semantic contracts authoritative.

</aside>

# Research baseline

The detailed semantic baseline comes from the official Affinity Designer 2 help because it documents individual tools, context controls, panels, snapping, object operations and text behavior at fine granularity. The 2026 all-new Affinity is the primary reference for modern workspace philosophy: vector, pixel and layout capabilities coexist in one application and customizable Studios can mix tools and panels across disciplines.[[1]](https://affinity.help/designer2/English.lproj/index.html)[[2]](https://www.canva.com/pt_br/midia/novidades/all-new-affinity/)

Official 2026 learning material also teaches Vector Studio, Pixel Studio, Layout Studio, mixed vector/pixel/layout workflows and shareable custom Studios.[[3]](https://www.canva.com/design-school/courses/affinity-essentials/)[[4]](https://www.canva.com/design-school/resources/customize-your-affinity-studios/)

# What Aubrieta adopts

- direct manipulation on canvas with immediate preview;
- a stable tool rail plus a context-sensitive toolbar;
- persistent Studio/panel areas for high-dimensional state;
- one Layers hierarchy as the primary structural navigator;
- familiar vector grammar: Move, Node, Pen, Pencil, Corner, Contour, Knife, Gradient, Transparency, Shape Builder-like region composition and Boolean operations;
- non-destructive alternatives alongside explicit Bake/Expand/Rasterize commands;
- strong artboard/Surface workflows;
- local tool snapping plus global snapping policy;
- numeric precision through Transform/Properties fields;
- professional colour, stroke, typography and appearance panels;
- keyboard shortcuts as first-class workflow accelerators;
- visual workspace customization.

# What Aubrieta changes deliberately

## Personas are defaults, not walls

Aubrieta keeps **Design** and **Photo** as canonical workflow Personas because they communicate user intent and simplify first-run cognitive load. Inspired by the 2026 Affinity Studio model, users may create **Workspace Profiles** that mix approved tools and panels from both Personas without converting document content or creating duplicate business logic.

A Workspace Profile changes presentation composition only. It must not change document semantics, Action IDs, permissions, serialization or undo behavior.

## Context without surprise

Affinity's context toolbar is an important pattern, but Aubrieta adds a stricter rule: the same semantic property keeps the same label, control type and ordering family across tools whenever possible. Controls may appear or disappear by applicability, but equivalent state must not migrate unpredictably between unrelated locations.

## Visible mode state

Any tool mode that changes click or drag meaning must have:

- persistent selected state in the context toolbar or canvas HUD;
- accessible name and current value;
- status-bar explanation;
- Esc exit or rollback semantics;
- inspection API exposure for agents and tests.

## Preview before destructive commit

Where Affinity offers destructive and non-destructive alternatives, Aubrieta makes the distinction explicit in the verb and preview. Examples include Boolean Union versus Create Live Boolean, Bake Contour, Expand Stroke, Rasterize and Detach Symbol.

## Disabled reason

Unavailable operations are not silent grey icons. The semantic Action predicate provides a short disabled reason consumable by tooltip, command palette, accessibility description and automation inspection.

# Workspace grammar

Menu / Command Palette → Persona / Workspace Profile → Primary Toolbar → Context Toolbar → Tool Rail + Canvas + Studio Panels → Status / Modifier Hints / Jobs / Diagnostics.

The official Affinity interface uses Persona Toolbar, Toolbar, Context Toolbar, Tools Panel, Studio panels, Status bar and Document View; Aubrieta retains the recognisable mental model while exposing stronger semantic inspection.[[5]](https://affinity.help/designer2/English.lproj/pages/Workspace/interface.html)

# Reference-to-contract rule

For every Aubrieta tool inspired by Affinity, documentation records:

1. Affinity reference behavior;
2. Aubrieta user goal;
3. Aubrieta semantic Action/Command/Property IDs;
4. input and selection prerequisites;
5. pointer, pen and keyboard grammar;
6. context-toolbar controls;
7. canvas overlays and handles;
8. panel interactions;
9. preview, commit, cancel and undo lifecycle;
10. non-destructive versus destructive distinction;
11. disabled, error and empty states;
12. accessibility and localization behavior;
13. persistence scope;
14. automation parity;
15. performance budget;
16. test and fixture coverage;
17. deliberate divergence from Affinity.

# Scope discipline

Affinity reference coverage does **not** promote a Post-V1 feature. Scope remains owned by Product Charter + Functional Atlas + 12.8. A Post-V1 tool may have complete UX documentation so implementation does not need to reinvent its interaction later.

# Competitive-drift rule

Do not chase Affinity changes automatically. A new Affinity workflow is evaluated only when it reduces cognitive load, improves direct manipulation, improves discoverability or precision, solves an Aubrieta usability finding, strengthens accessibility, removes duplicated UI or offers compelling evidence for a previously open Aubrieta decision.

The result must be recorded as Adopt / Adapt / Reject with rationale.