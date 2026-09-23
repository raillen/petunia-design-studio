# 10.4 — Fill, Stroke, Gradients, Transparency, Appearance, Effects & Blend Modes

# Appearance model

**Resolved architecture:** the canonical model supports an ordered **Appearance Stack from V1**, including multiple fills, multiple strokes, opacity/blend/isolation entries and EffectChain nodes. This avoids a breaking document-model migration later and matches the needs of professional vector workflows.

The initial UI may deliberately expose only the common one-fill/one-stroke workflow by default. An advanced Appearance panel progressively reveals additional stack entries. Simpler UI does not mean a simpler persisted model.

# Fill

None, Solid semantic `ColorValue` and gradient are **V1_REQUIRED** paint variants. Pattern/image fills are **POST_V1_CANDIDATE**. Fill preserves color model/profile references.

# Stroke

Color/paint, width, alignment, cap, join, miter, dash, variable-width profile, brush/profile reference. Scale-with-object behavior explicit.

# Gradient

Linear/radial baseline; stops have offset, semantic color, opacity, midpoint/interpolation metadata. On-canvas tool edits vector geometry; panel edits exact values. Reversing/reordering stops deterministic.

# Transparency

Object opacity distinct from alpha inside fill/stroke/gradient stops. Transparency tool may create an opacity gradient/mask, not destructively rewrite colors.

# Blend modes

Canonical list and mathematical/reference semantics documented in compositor. Group isolation/pass-through explicit. Unsupported export adapter reports degradation.

# Effects

Drop shadow, inner shadow and Gaussian blur are **V1_REQUIRED** typed effects. Additional effect families are **POST_V1_CANDIDATE** unless promoted. All effects are ordered, enable/disable capable and maskable where their semantic contract allows it. UI generated partly from schema but specialist editors allowed.

# Effect parameters

Each parameter declares type, unit, range, default, localization IDs, serialization key and invalidation class. Parameter animation/keyframing is **POST_V1_CANDIDATE** and is not implied by the V1 schema. This same schema serves Properties, plugins, MCP and potential Node view.

# Copy/paste appearance

Commands for Copy Style/Paste Style, clear effects, detach shared style. Never copy resource IDs across documents without resource import/remap.

# Tests

Blend reference images, opacity-vs-alpha, gradient stop extremes, dash scaling, effect order, mixed multi-selection, resource remapping across documents, PDF/SVG degradation diagnostics.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** ordered Appearance Stack; multiple fills/strokes; solid fills; linear/radial gradients; stroke width/alignment/caps/joins/miter/dashes; object/fill/stroke opacity separation; core blend modes; drop shadow, inner shadow and Gaussian blur baseline; Copy/Paste Appearance; enable/disable/reorder stack entries.

**POST_V1_CANDIDATE:** pattern/image fills, advanced artistic brush/effect families and animation/keyframing of appearance parameters unless separately promoted.

## Appearance Stack semantics

The stack is ordered and versioned. Each entry has stable identity within the object appearance, enabled state, opacity/blend semantics where applicable, typed parameter schema and effect/invalidation metadata. Reordering entries is one semantic command and affects rendering deterministically.

UI simplification must not collapse persisted state. If the simple Fill/Stroke UI edits an object with multiple entries, it must target a clearly defined primary entry or direct the user to the Appearance panel; it must never silently delete secondary entries.

## Paint model

A paint is semantic data independent from UI widgets. Baseline paint variants: None, Solid ColorValue and Gradient. Each paint carries color/profile references as needed. Cross-document paste remaps swatches/resources by stable identity/content fingerprint and never leaves foreign dangling IDs.

## Gradient contract

Stops have stable local IDs for editing, offset in normalized gradient space, ColorValue, opacity and interpolation metadata. The gradient geometry (start/end/focal/radius/orientation) is separate from stop list. Stop insertion/removal/reordering and midpoint edits are transactional and must produce equivalent results from canvas, Properties, MCP and Lua.

Interpolation behavior must be defined in a documented color space/policy instead of implicitly inheriting a renderer library default.

## Stroke contract

Stroke evaluation declares whether width scales with object transform, how inside/outside alignment behaves on open paths, how dash phase is measured and how miter clipping is resolved. Invalid dash arrays/ranges produce schema diagnostics. The document stores semantic stroke parameters, never tessellated outlines.

## Opacity and blend order

Distinguish:

1. alpha inside a color/gradient stop;
2. paint-entry opacity;
3. object/group opacity;
4. effect opacity;
5. compositor/group isolation.

The compositor reference order in 09.7 is normative. UI labels/tooltips must explain the difference rather than presenting multiple controls all called only “Opacity” without context.

## Effects

Effects are non-destructive typed nodes. Each node declares required input bounds expansion, parameter schema, preview quality policy, cache invalidation class, CPU/reference support where feasible and export degradation behavior. Effect failure disables/flags only the affected node where safe; it must not corrupt the source object.

## Shared styles

**V1_REQUIRED baseline:** shared Appearance Style resources use stable StyleIds, versioned property payloads and explicit local overrides, consistent with the shared-style contract in 10.5. `Detach Style` materializes the effective appearance atomically; `Clear Overrides` and `Redefine Style` are explicit commands. Complex multi-parent/inheritance cascades are **POST_V1_CANDIDATE** unless separately specified. Ordinary local Appearance Stack remains fully functional without requiring an object to reference a shared style.

## Selection/mixed state

Multi-selection editors return Same/Mixed/Unavailable per property. Applying a property to a mixed selection changes only that semantic property and does not normalize unrelated appearance fields.

## Performance

Expensive effects may evaluate asynchronously, but interactive edits must retain responsive lower-quality preview where allowed and final display/export must reevaluate at required quality. Cache keys include appearance/effect revisions and relevant color context.

## Accessibility/tokenization

Every appearance entry/action/editor uses TextId/IconId/PropertyId/help metadata and Design System controls. Color-only state is insufficient for enabled/disabled/error distinctions.

## Automation/plugins

Appearance stack enumeration/editing is exposed through Property/Action schemas. Plugins may contribute new typed effect/paint kinds only through versioned capability contracts and cannot inject arbitrary GPU code in the scripting tier.

## Required tests

Add stack reorder/undo, simple-UI behavior on multi-entry appearance, gradient interpolation fixtures, cross-document resource remap, mixed-selection single-property edits, effect bounds expansion, disabled/failing effect recovery, color-profile preservation, export degradation, Lua/MCP parity and save/reopen of every V1 appearance node.