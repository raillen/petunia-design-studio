# 08.30 — UI/UX Evidence Corpus, Interaction Conformance & Human-Factors Gauntlet

<aside>
🧪

**UI documentation is not complete until it produces reproducible evidence.** This page turns the Interface Atlas into fixtures, semantic assertions, usability scenarios and visual baselines.

</aside>

# Evidence layers

1. **component gallery** — every reusable component and state;
2. **tool fixtures** — deterministic document + selection + tool state;
3. **workflow fixtures** — end-to-end user jobs;
4. **semantic snapshots** — focus, accessibility tree, actions, properties and selection;
5. **visual goldens** — stable screenshots;
6. **interaction traces** — pointer/key sequence and resulting Actions/Commands;
7. **performance traces** — latency/frame-time/memory;
8. **human usability notes** — structured observation, not anecdote.

# Canonical UX workflows

- create document → draw primitive → node edit → style → save;
- Pen logo → Boolean/Live Boolean → Corner/Contour → export SVG/PDF;
- import complex SVG → locate object in Layers → edit path/text → export;
- vector composition → add pixel texture/adjustment → return to vector work;
- multi-Surface document → guides/bleed/profile → batch export;
- missing font/link → diagnose → repair → continue without data loss;
- discover unfamiliar command via menu/command palette/help;
- keyboard-only select → inspect Properties → edit → export.

# Tool-specific evidence

For each tool record:

- FixtureId;
- starting document revision;
- selection;
- active tool/mode;
- pointer/keyboard steps;
- expected preview;
- expected Action/Command;
- expected ChangeSet;
- expected undo state;
- semantic UI snapshot;
- optional visual golden;
- performance budget.

# Visual regression policy

Pixels are evidence, not truth. A screenshot pass cannot override wrong focus, wrong ActionId, missing accessible name, incorrect undo or semantic state.

Use fixed theme, density, locale, scale, viewport and seeded fixture.

# Interaction conformance

Prefer semantic selectors and stable IDs. Pixel coordinates are only allowed for canvas geometry interactions where coordinates are themselves the behavior under test.

# Human usability protocol

For each observed task record:

- task intent;
- starting profile;
- prior experience category;
- first-attempt completion;
- wrong-mode/wrong-target errors;
- command discovery path;
- unexpected undo;
- recovery success;
- repeated-task improvement;
- qualitative confusion.

Do not produce demographic personas from small samples.

# Affinity comparison sessions

When evaluating an Affinity-inspired workflow, run the same conceptual task in:

- current Affinity;
- Aubrieta prototype;
- keyboard-only Aubrieta where applicable.

Measure extra state changes, hidden-mode errors, path length and recovery. The purpose is not to beat Affinity by a vanity score; it is to verify that a deliberate Aubrieta divergence actually removes friction.

# Accessibility corpus

Keyboard only, 200% scale, high contrast, reduced motion, screen-reader semantic tree, CJK IME, RTL locale and colour-vision simulation where practical.

# Performance UX corpus

- 10k and 100k Layers rows;
- high-density vector nodes;
- large raster document;
- panel resize during canvas motion;
- command search;
- font search;
- brush/pen continuous input;
- mixed-DPI multi-monitor restore.

# Release evidence bundle

Every UI-affecting release records changed goldens, semantic conformance status, keyboard/a11y status, usability regressions, known UX debt and performance deltas.

# Agent rule

A code agent must cite the canonical fixture/evidence it used when claiming a user-visible feature complete. If no fixture exists for a new interaction, creating the fixture is part of implementation.