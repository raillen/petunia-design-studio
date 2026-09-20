# 14.7 — UI/UX Evidence Integration, Semantic Goldens & Affinity Workflow Comparisons

<aside>
🎨

Section 08 specifies interaction. Section 14.7 defines the evidence required to prove that implemented interaction is usable, accessible, semantically correct and faithful to deliberate Affinity-inspired decisions.

</aside>

# Evidence sources

Canonical UI evidence is defined jointly by 08.16 and 08.30:

- component-state gallery;
- tool/workflow fixtures;
- focus/accessibility/action/property semantic snapshots;
- interaction traces;
- visual goldens;
- UI performance traces;
- structured usability sessions.

# Tool proof packet

Each implemented canvas tool includes:

- Tool/FeatureId;
- canonical 08.x + 10.x reference;
- FixtureId;
- starting selection/revision;
- tool mode/context controls;
- pointer/pen/keyboard script;
- expected preview;
- expected Action/Command and ChangeSet;
- undo/cancel expectation;
- focus/a11y snapshot;
- optional pixel golden;
- performance budget.

# Panel proof packet

Panels test empty, no-selection, supported/mixed selection, loading, error, read-only, disabled capability, huge data, search/filter, keyboard traversal, responsive minimum size and workspace restore.

# Affinity-inspired comparisons

When Aubrieta deliberately changes an Affinity workflow, capture:

- reference task and observed Affinity interaction;
- Aubrieta hypothesis;
- expected friction reduction;
- prototype task trace;
- usability findings;
- decision: keep, revise or revert the divergence.

The objective is evidence-backed improvement, not a vanity “fewer clicks wins” score.

# Semantic before pixel

A screenshot cannot pass a feature with wrong ActionId, focus, accessible name, undo semantics, property target or disabled reason.

# Accessibility matrix

Keyboard only, screen-reader tree, 200% scale, high contrast, reduced motion, pseudo-locale, RTL, CJK IME and non-colour state cues where relevant.

# Human testing

Record task completion, first-attempt success, wrong-mode errors, unexpected undo, discovery path, recovery and repeated-task efficiency. Treat small studies as usability evidence, not demographic truth.

# Release gate

Primary Design and Photo workflows must have current semantic/UI evidence before release. Known UX debt is recorded explicitly rather than hidden by baseline updates.