# 08.1 — Visual Foundations, Tokens, Typography, Spacing & Motion

<aside>
🎨

This page defines the **visual grammar**. No panel or control may invent spacing, type size, radius, shadow or state color outside these tokens without an ADR/design-system extension.

</aside>

# Aesthetic direction

Aubrieta uses a **quiet professional dark/light system** inspired by high-end creative tools and Apple-like restraint. It is not glassmorphism-heavy and not gamer-neon. Depth comes from tonal separation, subtle borders, restrained shadow and occasional translucency for temporary overlays.

Primary characteristics:

- dark graphite/slate surfaces with neutral chroma;
- high-contrast document content against quieter chrome;
- accent used for selection, active tools, focus, primary confirmation and semantic highlights — never as large-area decoration;
- flatter persistent panels, slightly elevated temporary surfaces;
- minimal decorative gradients in application chrome;
- visual density closer to professional desktop tools than mobile applications.

# Base spacing scale

Use a 4 px base unit with half-step exceptions only for optical alignment.

| Token | Value | Use |
| --- | --- | --- |
| `space-0` | 0 | flush relationships |
| `space-1` | 4 px | icon-label micro gap, tight control internals |
| `space-2` | 8 px | default sibling gap, compact padding |
| `space-3` | 12 px | form row/panel inner grouping |
| `space-4` | 16 px | default section padding |
| `space-5` | 20 px | larger grouped spacing |
| `space-6` | 24 px | major section separation |
| `space-8` | 32 px | dialog large grouping |
| `space-10` | 40 px | rare hero/onboarding spacing |

# Density modes

Aubrieta exposes **Compact / Comfortable / Spacious**. Density changes vertical control metrics and panel paddings, not typography semantics or hit-target accessibility.

- Compact: rows 28–30 px, tool buttons 30 px, panel tabs 30 px.
- Comfortable: rows 32–34 px, tool buttons 34 px, panel tabs 32–34 px. **Default.**
- Spacious: rows 38–40 px, tool buttons 38–40 px, panel tabs 36–38 px.
- Minimum pointer target: 28 px visual target, but interaction hit box should aim for 32 px+; keyboard focus must not depend on pointer target size.

# Corner radii

- `radius-xs`: 3 px — numeric field inner affordance, small tags.
- `radius-sm`: 5 px — standard fields/buttons.
- `radius-md`: 7 px — popovers, panel-contained cards.
- `radius-lg`: 10 px — floating palettes/dialog containers.
- `radius-xl`: 14 px — onboarding/large preferences cards only.
- Persistent dock boundaries normally remain square or only 2–4 px at exposed edges; avoid cardifying every panel.

# Borders and separators

- Hairline separator: 1 physical pixel where possible; scale-aware.
- Persistent panel boundaries use low-contrast separator tokens.
- Active panel tab indicator may use a 2 px accent underline or subtle filled selection, not both simultaneously.
- Focus ring: 2 px semantic focus color with 1 px offset where geometry allows.
- Selected rows use a tinted background plus icon/text state; selection must not depend on color alone.

# Surface hierarchy

From lowest to highest:

1. canvas/workspace background;
2. application chrome base;
3. dock panel surface;
4. field/control surface;
5. selected/active surface;
6. popover/context menu;
7. floating palette;
8. modal dialog/sheet;
9. alert/critical confirmation.

Temporary surfaces may use slightly stronger shadow and optional blur/translucency. Persistent chrome should remain mostly opaque for readability and GPU predictability.

# Typography

UI type uses a neutral system-oriented sans stack. On macOS prefer system UI typography; on Windows/Linux map to a carefully selected Aubrieta UI fallback with equivalent metrics. Creative document fonts are unrelated to shell typography.

Semantic scale:

- Caption: 11 px / 14 px — metadata, shortcuts, secondary status.
- Small: 12 px / 16 px — dense labels, tree rows.
- Body: 13 px / 18 px — default UI text.
- Emphasized body: 13 px / 18 px, medium/semibold.
- Panel title: 13–14 px / 18 px, semibold.
- Dialog section title: 15 px / 20 px, semibold.
- Dialog title: 18–20 px / 24–26 px, semibold.
- Onboarding display: 28–36 px only where content justifies it.

Numeric typography should use tabular figures where alignment matters: transform values, dimensions, percentages, channel values, status metrics.

# Color roles

Do not name semantic tokens after literal colors. Required roles:

- `surface.workspace`, `surface.chrome`, `surface.panel`, `surface.control`, `surface.elevated`;
- `text.primary`, `text.secondary`, `text.tertiary`, `text.disabled`, `text.onAccent`;
- `border.subtle`, `border.strong`, `border.focus`;
- `accent.primary`, `accent.hover`, `accent.pressed`, `accent.selection`;
- `status.info`, `status.success`, `status.warning`, `status.error`;
- `canvas.selection`, `canvas.guide`, `canvas.snap`, `canvas.measurement`.

# Motion

Motion supports spatial continuity; it must never delay expert workflows.

- hover/focus color transitions: 80–120 ms;
- button/segment state: 80–120 ms;
- popover/menu appearance: 120–160 ms, slight fade + 2–4 px translation;
- panel collapse/expand: 140–180 ms;
- docking preview: immediate geometry response with 100–140 ms highlight fade;
- dialog/sheet: 160–220 ms;
- reduce-motion preference: remove translations/scales; retain minimal opacity transitions.

Avoid spring/bounce motion in production editing chrome except direct manipulation where physical continuity helps.

# Icon sizing and optical alignment

- micro: 12 px;
- standard dense: 16 px;
- primary toolbar: 18 px;
- tool rail: 18–20 px;
- empty state/illustrative icon: 24–32 px.

Icons align to optical center, not mathematical bounding-box center. Use semantic `IconId`; Aubrieta custom SVG overrides general families where domain precision matters.

# Shadows

Persistent docked surfaces: usually none. Floating surfaces:

- popover: subtle 0 6 20 equivalent with low alpha;
- floating palette: slightly stronger 0 8 28;
- modal: 0 14 40 plus backdrop dimming.

Never use glow as generic focus/selection treatment.

# Theme requirements

Dark, Light and System themes are mandatory. User accent selection may change accent family while preserving WCAG/APCA-style contrast targets. Theme tokens must account for OLED-dark extremes without using pure black for all surfaces; layered distinction must remain visible.

# Tokenized resource architecture

All visual values in production UI must resolve through semantic tokens. This includes **colors, typography, spacing, radii, borders, shadows, motion, opacity, density metrics, icon identities, cursor identities and user-visible text identifiers**. Feature code may reference semantic IDs; it may not hard-code literal styling or English copy.

## File-format decision

Aubrieta uses a deliberate two-format policy:

- **JSON** is canonical for resource data exchanged/overridden at runtime: design tokens, themes, icon mappings and localized string catalogs. Design-token JSON should remain compatible where practical with the stable DTCG 2025.10 token model (`$type`, `$value`, aliases) so Aubrieta themes can interoperate with external design-system tooling.
- **TOML** is canonical for human-authored manifests and configuration: pack metadata, module/plugin manifests, workspace/config files, dependency declarations and user preference files. TOML is concise, comment-friendly, Serde-compatible and idiomatic in the Rust ecosystem.

Do not duplicate the same datum in both formats. TOML describes **what a pack/module is and how it is composed**; JSON carries **token/resource payloads**.

## Semantic indirection

UI code should ask for `TextId`, `IconId` and token paths rather than literal resources. Example conceptual references: `TextId::FileExport`, `IconId::BooleanUnion`, `Token::SurfacePanel`. Runtime resolution order is documented in the Resource Pack architecture and must support built-in defaults, active theme/icon/locale packs, user overrides and safe fallback.

## Theme safety

A theme is allowed to change presentation, not product semantics. Theme packs cannot rebind commands, grant plugin capabilities or alter document data. Invalid/missing token values fall back to the built-in canonical theme and emit diagnostics rather than breaking the UI.

# Enforcement and portability

The token rules on this page are enforced by **08.21 — Design System Governance, Token Enforcement, Customization & UI Portability Contract**. Feature code must not treat these values as documentation-only recommendations: direct literal copy/icons/colors/spacing/radii/motion/help/a11y resources outside approved Design System/catalog definitions are review/CI violations.

Tokens and semantic component contracts must remain toolkit-independent. GPUI resolves them today, but a future shell must be able to consume the same semantic roles without changing feature/domain behavior.