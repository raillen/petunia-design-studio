# 09.9 — Color Management, ICC, CMYK, Spot & Proofing Architecture

# Canonical semantics

ColorValue variants preserve RGB/CMYK/Gray/Lab/Spot/Registration plus profile/reference semantics. UI display color is derived.

# Color context

Document working profiles, assigned profiles on raster resources, proof profile, display profile, rendering intent, black-point behavior and output policy are separate concepts.

# Transform service

Transforms are cached by source/destination profiles + intent/options + pixel format. **Resolved V1 backend policy:** `moxcms` is the default/primary production CMM because it keeps the normal path safe and Rust-native; LittleCMS 2 remains behind a narrow adapter as both differential oracle and compatibility fallback for profiles/features not yet accepted by the moxcms gauntlet. Callers depend on Aubrieta's `ColorManagementProvider`, never either library directly.

The long-term preference remains to remove the production fallback only after the ICC corpus proves moxcms coverage/accuracy is sufficient. LittleCMS support is therefore a safety net, not a second semantic color model.

# Profile lifecycle

ICC resources have stable IDs/fingerprints and can be embedded, linked or system-resolved according to policy. Missing profile never silently becomes an arbitrary profile; show assigned fallback and preserve metadata.

# CMYK

Numeric CMYK values remain canonical when policy says preserve numbers. UI conversion to RGB happens only for display. Raster CMYK paths avoid unnecessary round trips.

# Spot/Registration

Spot color includes stable swatch identity/name, alternate color and tint. Registration is a semantic special color with restricted use.

# Soft proof

Proof transform is render context; gamut warning/ink coverage are derived analysis. Toggling proof changes display, not document data.

# Color conversion commands

Assign Profile changes interpretation; Convert to Profile changes values. They are distinct Actions with previews/undo.

# Tests

Known ICC fixtures, moxcms/LCMS numerical comparisons, roundtrip policy tests, profile missing/corrupt cases, spot preservation through PDF where supported and 8/16-bit raster conversions.

# Canonical color value representation

`ColorValue` is a tagged semantic union. It must preserve the authored model and associated profile/reference identity rather than normalizing every color to RGB.

Conceptual V1 variants:

```
ColorValue
├── Rgb { channels, profile_ref }
├── Cmyk { channels, profile_ref, preserve_numbers }
├── Gray { channel, profile_ref }
├── Lab { l, a, b, white_point/profile_ref }
├── Spot { swatch_id, tint }
└── Registration
```

Alpha/opacity is an appearance property unless a specific color resource contract explicitly includes alpha. This avoids conflating source color semantics with object compositing opacity.

Channel values use validated finite numeric wrappers with documented ranges. Serialization stores enough precision to round-trip authored values without UI formatting loss.

# Profile identity

A profile is referenced through `ColorProfileId`/resource identity plus content fingerprint. The resource records profile class/type, description metadata, source (`embedded`, `linked`, `system-resolved`) and raw ICC payload when embedded.

System profile path/name is not sufficient persistent identity by itself. When a linked/system profile disappears, Aubrieta preserves its requested identity/fingerprint metadata and reports resolution state.

# Document color context

Separate these fields/contracts explicitly:

- default RGB working profile;
- default CMYK working profile;
- optional Gray/Lab policies;
- rendering intent defaults;
- black-point compensation policy where backend supports it;
- proof profile + proof intent/options;
- display profile, which is session/platform state and is **not** canonical document content;
- export/output profile chosen per export job/preset.

A document may contain resources/pixel surfaces tagged with profiles different from the document defaults.

# Assignment vs conversion semantics

`AssignProfile` preserves channel numbers and changes their interpretation/profile association. `ConvertProfile` computes new channel values from source→destination to preserve appearance according to intent/options.

Both operations must declare their target scope: selected raster/object color, resource, whole document defaults, or document raster population. Whole-document conversion that changes many canonical values is one explicit undoable transaction with preflight and memory/history planning.

# Preserve-numbers policy

CMYK workflows may intentionally preserve source process values. `preserve_numbers` is not a UI checkbox accidentally propagated through every operation; it belongs to a typed conversion/output policy. Conversion code must not transform a CMYK color marked for number preservation merely because the display pipeline needs RGB preview.

# Spot color identity

A spot color is a reusable semantic swatch/resource with:

- stable `SwatchId`/spot identity;
- user-visible name;
- alternate process/display color;
- tint behavior;
- optional vendor/library metadata;
- separation/export metadata.

Objects reference spot identity + tint. Renaming/redefining the swatch changes its resource metadata/alternate representation without losing spot identity.

Duplicate spot names with different identities require a conflict/preflight policy on import/export; name alone is not identity.

# Registration color

Registration is a dedicated semantic color intended for marks that appear on all separations. It cannot be accidentally converted to a normal process swatch. UI/exporters may restrict where it can be applied and preflight misuse.

# Transform cache

`ColorTransformKey` includes at least source profile fingerprint, destination profile fingerprint, source/destination pixel representation, rendering intent, black-point/flags and CMM implementation/evaluator version. Device/display generation is included where relevant.

Transform objects are derived/rebuildable. A transform failure is a structured diagnostic; callers never receive an uninitialized transform or silently substitute sRGB unless that fallback is explicitly named by policy.

# CMM fallback/oracle rule

`moxcms` is V1 primary. LittleCMS is invoked only through the same `ColorManagementProvider` compatibility adapter when:

- the profile/feature is outside the currently accepted moxcms capability matrix; or
- differential-validation/dev mode requests an oracle comparison.

Fallback selection is observable in diagnostics/telemetry. No feature code branches directly on library names.

The accepted capability matrix is versioned and tested; reducing fallback use is allowed without changing semantic APIs.

# Differential validation

For a curated ICC corpus, compare moxcms vs LittleCMS on:

- RGB↔RGB;
- RGB↔CMYK;
- CMYK↔CMYK where supported;
- Gray/Lab paths;
- 8-bit and 16-bit integer samples;
- representative intents/black point options.

Record numerical/perceptual tolerances per transform class, not one universal threshold. Large unexpected differences become regression fixtures/diagnostics.

# Display/proof pipeline

Conceptual display path:

```
Canonical/Evaluated working color
   → compositor working/blend context
   → optional proof transform
   → optional gamut-warning/analysis overlay
   → display-profile transform
   → display encoding/swapchain
```

Proofing never writes converted values back to `ColorValue` or pixels. Changing display monitor/profile invalidates display transforms/GPU resources only.

# Soft-proof V1 status

Soft proofing and proof-profile selection are `V1_REQUIRED` architecture/product capabilities. Gamut warning and richer ink-coverage/separation analysis can be staged according to Functional Atlas, but the proof transform itself must not remain an unspecified future feature.

# Total ink / separation analysis

When implemented, TAC/ink coverage is a derived analysis in the selected output/proof CMYK context. It reports values/over-limit regions without mutating document colors. Spot separations are represented distinctly from process CMYK plates.

# Raster color conversions

A canonical PixelSurface conversion is tile-aware, cancelable and transactionally undoable. It processes immutable source tiles into destination representation and commits only when all required tiles/resources are valid. Partial converted pixels do not become canonical on failure/cancel.

16-bit paths remain 16-bit unless the requested destination/output contract explicitly selects another depth. Conversion uses typed alpha/premultiplication boundary rules.

# Vector/semantic color conversion

Converting vector object colors/swatches operates on semantic `ColorValue`s and preserves spot/Registration according to explicit output/conversion policy. A generic profile-convert command must not flatten spot into process without an explicit policy/user action.

# Import policy

When external content has an embedded/assigned profile:

- preserve the source profile association by default where format fidelity allows;
- never assume untagged equals sRGB/working CMYK without applying the configured untagged-content policy;
- present ambiguous/missing-profile cases through Import Interpretation when appearance/editability materially changes.

# Export/preflight

Export adapter receives explicit output profile/conversion policy. Preflight reports:

- missing/corrupt profiles;
- colors outside target capability;
- spot/Registration degradation;
- preserved-number conflicts;
- profile embedding restrictions/unsupported target behavior;
- proof/display settings that are preview-only and will not be exported.

# Hostile profile safety

ICC parsing/transform creation is a hostile-input boundary. Enforce profile-size/tag-count/offset limits and validate tag ranges before use. A corrupt profile cannot cause unchecked reads/allocation or crash the document load path. Preserve original resource when safe even if transform use is disabled.

# Observability

Developer diagnostics expose profile resolution, transform cache hits/misses, active CMM provider, source/destination fingerprints, transform duration, fallback reason and differential-test delta statistics. Ordinary logs must not dump raw ICC payloads or user paths.

# Additional gauntlets

- assigned-vs-converted profile commands with undo/save/reopen;
- CMYK preserve-numbers through display proof and PDF export;
- spot tint + swatch rename/redefine + export;
- Registration misuse/preflight;
- 16-bit raster profile conversion with oracle comparison;
- monitor/display-profile change without canonical document dirtiness;
- missing linked profile recovery;
- malformed ICC tag bounds/size fuzzing;
- fallback from moxcms to LCMS without semantic API difference.