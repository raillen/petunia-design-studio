# 10.12 — Design ↔ Photo Interoperability, Conversion Commands & Shared Composition Rules

# Same document

Switching Persona never converts data. Tool availability changes; object model remains mixed.

# Shared operations

Move/transform, opacity, blend, masks, clipping, grouping, color/proof context, history and export operate across vector/text/raster when semantics permit.

# Explicit conversion commands

- Rasterize: vector/text/group → PixelLayer or raster result;
- Convert to Curves: text/shape → path(s);
- Expand/Outline: live geometry/stroke/effect → more explicit geometry;
- Bake/Apply: modifier/effect → committed result;
- Embed/Link/Relink: resource ownership changes.

Each command declares what editability is lost and is fully undoable within history.

# Cross-type masks

Vector mask over raster and vector content supported through compositor. Raster mask may mask vector/group without rasterizing source; only mask evaluation rasterizes coverage internally.

# Adjustments over groups

Adjustment/filter scope on mixed groups defines where raster intermediate is required. Canonical children remain editable even if compositor uses offscreen rasterization.

# Text/raster

Photo adjustments can target placed/rasterized pixels; semantic text itself is not silently changed into pixels just because Photo Persona is active.

# Export

Adapters decide preservation/degradation from canonical mixed scene, not Persona used last.

# Tool handoff

Selecting unsupported object for current tool offers action to switch tool/Persona or perform explicit conversion. Never auto-rasterize on first brush click without confirmation/policy.

# Tests

Mixed group with text/path/photo + mask + blend + adjustment, Persona switch after undo, vector mask edits in Photo, export from either Persona, explicit conversions save/reopen and exact undo.

# Implementation contract — V1

## Product invariant

**Persona switching is presentation/workflow state only.** `Design ↔ Photo` changes available tools, panels, workspace defaults and contextual Actions; it must not convert, reorder, rasterize, relink or otherwise mutate canonical document content. A Persona switch therefore creates no undo entry and never marks a document dirty.

## Shared-object compatibility matrix

Every core operation declares applicability across `Path/Shape/Text/ImageObject/PixelLayer/Group/Mask/Adjustment/...` rather than assuming “Design objects” versus “Photo objects.” Unsupported combinations return a reason/capability and an explicit conversion option when one exists.

Maintain a generated compatibility matrix from Action/Property schemas so UI, MCP, plugins and docs do not duplicate this logic.

## Explicit conversion policy

Each conversion command declares:

- source type/capabilities;
- resulting type(s);
- resolution/scale and color context where raster output is produced;
- whether masks/effects/blends are baked or preserved;
- resource/link behavior;
- lost editability/capabilities;
- bounds/bleed handling;
- undo payload/memory implications;
- export/interchange consequences.

No generic `convert()` should hide these decisions.

## Rasterize

Rasterize captures an evaluated semantic snapshot under explicit rasterization settings (target DPI/pixel dimensions or source pixel mapping, bit depth, color profile/space, alpha/background policy, effect/mask scope). The command creates/replaces with PixelLayer/raster output according to its variant and records enough undo data to restore source semantics.

A brush click on vector/text/image content cannot invoke Rasterize implicitly.

## Convert to Curves / Expand / Bake

These commands remain type-specific and named according to what editability is lost. `Convert to Curves` preserves current text/shape appearance as paths; `Outline Stroke` expands stroke semantics; `Bake Boolean/Offset/Warp` commits evaluated geometry; `Apply Filter` commits raster effect. UI/MCP documentation must not collapse them into one ambiguous destructive action.

## Mixed composition

Compositor intermediates do not imply document conversion. A group containing vector + text + PixelLayer + live adjustment may be rendered through offscreen raster surfaces while every child remains canonical/editable. Debug/inspection must distinguish `render intermediate` from `PixelLayer document object`.

## Cross-type masks and clips

Mask/clip coverage is evaluated uniformly by the compositor. Vector mask over PixelLayer or raster mask over vector/group does not convert the target. Editing mask representation follows its own tool semantics. Conversion of a mask is explicit.

## Adjustment scope

Photo adjustments/live filters may be attached to mixed groups when their semantic contract supports the evaluated group result. Target ObjectId/subtree is stored explicitly; simply switching to Photo does not retarget an adjustment.

## Resources and placed images

`ImageObject` remains a placed/linked/embedded image resource object; `PixelLayer` remains editable raster storage. “Edit pixels” on ImageObject offers explicit choices such as create editable PixelLayer/rasterize/embedded-edit workflow according to capability. Link→Embed/Relink are resource commands, not Persona behavior.

## Selection/tool handoff UX

When a tool cannot operate on the selected type, return one of: switch to compatible tool, change target (content/mask/layer), execute an explicit conversion, create required layer, or cancel. The UI must describe the consequence before destructive conversion. Keyboard/MCP receives the same disabled reason/action alternatives through semantic metadata.

## Undo/history across Personas

History is document-session-wide. An edit made in Photo is undoable after switching to Design and vice versa. History labels derive from semantic Actions, never Persona-specific implementation strings.

## Automation/plugins

MCP/Lua operate on mixed document types independent of Persona unless a method explicitly controls workspace UI. Automation should not need to switch Persona to execute a semantic command. Persona state can be inspected/changed only as UI/workspace automation.

## Required tests

Add no-dirty Persona switch, shared Action applicability matrix, vector/text rasterize at 8/16-bit and CMYK/RGB, explicit alpha/background policy, mixed-group offscreen render without canonical conversion, ImageObject vs PixelLayer behavior, adjustment target persistence across Persona switch, history across Personas, wrong-tool recovery and headless/MCP execution without Persona switching.