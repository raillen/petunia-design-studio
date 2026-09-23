# 10.10 — Photo Masks, Adjustments, Live Filters, Channels, Histogram & Analysis

# Adjustment model

Adjustment is typed non-destructive operation with parameter schema and target scope. May exist as layer/object/effect according to canonical appearance model; UI terminology must map consistently.

# Baseline adjustments

Levels, Curves, HSL, Exposure, White Balance. Each defines channel/color-space semantics, parameter ranges/defaults and CPU/reference output for tests.

# Filters

Gaussian Blur, Sharpen, Noise baseline. “Live Filter” means stored/evaluated non-destructively; destructive Apply/Bake is separate command.

# Masks

Every adjustment/filter that supports masking consumes attached mask in defined coordinate space. Mask thumbnails are derived.

# Channels

Channels panel exposes **V1_REQUIRED** document/raster channels semantically: RGB/CMYK/Gray components appropriate to the target, alpha when present, and canonical mask/saved-selection resources that actually exist. Viewing a channel does not mutate document. A full arbitrary channel-compositing workflow is **POST_V1_CANDIDATE**.

# Histogram

Derived async analysis with sample scope (layer/selection/composite), channel/color-space selection and configurable quality. It must not block painting. Cached by relevant revision/profile.

# Curves editor

Graph supports **V1_REQUIRED** add/move/delete points, channel selection and numeric input/output. Black/white eyedropper point tools are **POST_V1_CANDIDATE** unless promoted by an accepted scope decision. Drag transaction coalesces. Curve is monotonic only if operation demands it; otherwise it is an arbitrary transfer curve with stable interpolation.

# Levels

Black/white input, gamma/midpoint, output range. Units/display reflect bit depth but canonical normalized parameter may be used.

# Proof/analysis

Soft proof/gamut warning are color-engine render modes, not adjustments. Pixel inspector reports sampled document/display values with model/profile labels.

# Tests

Known image fixtures at 8/16-bit, CMYK/RGB, mask composition, adjustment order, channel views, histogram race/stale results, extreme curve values, export/rasterization fidelity.

# Implementation contract — V1

## Scope status

**V1_REQUIRED adjustments:** Levels, Curves, HSL, Exposure and White Balance.

**V1_REQUIRED live filters:** Gaussian Blur, Sharpen and Noise.

**V1_REQUIRED analysis:** Histogram, semantic channel viewing, pixel inspector, adjustment/filter masking, 8/16-bit RGB/CMYK/Gray processing and soft-proof integration through the Color Engine.

**POST_V1_CANDIDATE:** saved arbitrary channel workflows beyond mask/alpha needs, advanced retouching filters, liquify, RAW development and AI/content-aware adjustments.

## Adjustment representation

Every adjustment is a versioned typed node with `AdjustmentTypeId`, stable node identity, enabled state, target/scope, parameter schema, optional mask relationship, blend/opacity where supported and evaluation/invalidation metadata. The persisted model stores semantic parameters, never a baked preview image.

Adjustment node ordering is deterministic and visible. Reordering, enable/disable and parameter edits are ordinary Commands with undo. UI, MCP and Lua all edit the same node/property contracts.

## Scope and attachment

An adjustment may target a PixelLayer, compatible object/group subtree or adjustment-layer/container according to the canonical composition model. The target scope must be explicit; “current layer” UI convenience resolves to an ObjectId before the command executes. Changing selection later must not retarget an existing adjustment.

## Color-space semantics

Each adjustment declares the color domain in which its math is defined. It may not accidentally inherit whichever display RGB format a renderer currently uses.

- Display/profile conversion is outside canonical adjustment parameters.
- CMYK/Gray paths preserve semantic channels where the operation defines them.
- If an adjustment requires conversion to a working/intermediate space, the conversion policy is documented and tested through 09.9.

## Levels baseline

Canonical parameters: input black, input white, gamma/midpoint and output black/white per supported channel/master mode. UI may display bit-depth-specific integer scales, but the persisted normalized/semantic representation and conversion are deterministic.

## Curves baseline

Curve stores channel/master selector, ordered control points and interpolation type/version. Duplicate-x points or invalid ordering are resolved/rejected deterministically. Dragging points is a single preview transaction. Numeric entry and on-canvas/graph edits generate the same parameters.

## HSL, Exposure and White Balance

Each receives a formal parameter table in the Property Schema Registry including units/ranges/defaults and a reference CPU implementation or oracle fixtures. White Balance must define temperature/tint meaning rather than treating UI slider values as renderer-specific magic constants.

## Live filters

A live filter node stores parameters and evaluates non-destructively. `Apply/Bake` is a separate explicit command that writes pixels/new raster result and removes/replaces the live operation according to the command variant. No filter silently bakes merely because GPU support is unavailable.

Blur/sharpen/noise define edge handling, alpha handling, selection/mask behavior, color-space policy and preview/final quality. GPU acceleration must remain semantically comparable to the reference path.

## Masks

Adjustment/filter masks use the shared vector/raster mask composition rules. Editing a mask does not edit adjustment parameters. Mask coordinate space/transform attachment is explicit and survives object transforms.

## Channels V1

Channel viewing is a non-mutating inspection mode. Expose RGB, CMYK or Gray components appropriate to the target, plus alpha/mask channels that actually exist. UI must label model/profile and distinguish “view channel” from “edit mask/channel target.” Saving an arbitrary selection as a persistent mask/channel is an explicit command; a full Photoshop-like channel-compositing subsystem is not implied by V1.

## Histogram

Histogram is derived `Job` output keyed by source revision, scope, channel/model, profile/proof context where relevant, sample quality and selection. Stale results are discarded. Preview-quality histograms may subsample, but UI indicates approximate state if it materially changes interpretation. Histogram computation must never hold the document mutation lane or block brush input.

## Pixel inspector

Sampling declares source (`layer`, `composite`, optionally proof/display), coordinate space, sample radius/method and reported color model/profile. It is inspection only and does not mutate artwork.

## Failure behavior

Unknown/missing plugin adjustment kinds remain opaque/preserved and render a diagnostic fallback according to extension policy. A failed built-in filter evaluation disables/flags that node for the frame/job where safe; source pixels remain intact.

## Automation/plugins

Adjustment/filter discovery is schema-driven. MCP/Lua can add/reorder/remove nodes, query/edit parameters and masks through typed IDs. Plugins may register new adjustment/filter kinds only through versioned capability contracts, quotas and declared evaluation backend; scripting plugins cannot inject unrestricted GPU shaders.

## Required tests

Add per-adjustment golden/reference fixtures, 8↔16-bit consistency, CMYK/Gray cases, curve serialization/interpolation, mask-transform behavior, node reorder/undo, live-vs-baked equivalence within tolerance, GPU/reference comparison, stale histogram cancellation, approximate histogram state, pixel inspector profiles and UI↔MCP/Lua parity.