# 08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow

<aside>
🪟

**Aubrieta Personas remain canonical defaults; Workspace Profiles make them permeable.** Users can stay in flow across vector and pixel tasks without turning the product into two isolated applications.

</aside>

# 2026 Affinity lesson

The new Affinity unifies vector, pixel and layout in one app and explicitly encourages custom Studios that mix tools and panels across disciplines.[[1]](https://www.canva.com/pt_br/midia/novidades/all-new-affinity/)[[2]](https://www.canva.com/design-school/lessons/creating-custom-studios-in-affinity/)

The official mixed-workflow lessons emphasize switching between vector, pixel and layout without disrupting the creative flow.[[3]](https://www.canva.com/design-school/lessons/vector-pixel-layout-workflows-affinity/)

# Aubrieta model

**Persona** = curated default workflow and command surface.

**Workspace Profile** = user-customized composition of panels/tools/shortcuts.

**Capability availability** = actual engine/module support.

These concepts are independent.

# Default Personas

## Design

Optimized for vector, typography, layout-lite, Surfaces, reusable resources, Data Merge and export preparation.

## Photo

Optimized for raster selection, brush/paint, masks, adjustments, filters, histogram/analysis and image-oriented navigation.

Switching Persona never converts objects, changes document truth or adds history.

# Mixed profiles

A user may pin Photo tools in a Design profile or Design panels in a Photo profile when the underlying capabilities are available.

Examples:

- vector logo work + pixel texture brush;
- placed photo + nondestructive raster adjustment while staying in Design;
- Photo-heavy workspace with vector Pen/Node for masks and overlays.

# Profile persistence

Persist:

- panel layout;
- visible tools;
- toolbar composition;
- density;
- panel tab/group order;
- optional per-profile shortcut layer if architecture allows.

Do not persist document selection, active object semantics or hidden business state as part of the profile.

# Safe reset

Always provide:

- reset current profile to canonical;
- duplicate profile before risky customization;
- import/export profile with validation;
- fallback when a plugin-provided panel/tool is missing.

# Sharing

Inspired by Affinity's shareable Studios, Aubrieta profiles may become shareable resource packs. They must reference semantic IDs, never raw GPUI widget identifiers.

# Profile compatibility

Unknown tools/panels are skipped with diagnostics. A profile from a newer Aubrieta version must not prevent startup.

# Cognitive-load rule

First-run uses canonical Design or Photo profiles, not an empty customization canvas. Customization is progressive disclosure.

# Cross-Persona Actions

One ActionId remains canonical. A command visible in both Personas does not get duplicated implementations or histories.

# Inspection and automation

Expose active Persona, active Workspace Profile, visible semantic panels/tools and profile overrides. Tests can load canonical profiles deterministically.

# UI tests

Canonical Design, canonical Photo, mixed vector+pixel profile, missing-plugin profile, imported older profile, reset flow and small-screen/minimum-window collapse behavior.