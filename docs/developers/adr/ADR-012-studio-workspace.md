# ADR-012: Compact Studio workspace and semantic desktop controls

**Status:** Accepted implementation contract; execution evidence is separate.
**Date:** 2026-10-04
**Scope:** Milestone Required (MVP); shared desktop foundation for V1 Required workflows.

## Context

The desktop mixed locale strings, squeezed panels into one inspector, rendered two-column tools as a single horizontal row, and exposed small icon controls without a keyboard identity. New tabs could inherit an incorrectly sized camera. The Navigator painted object bounding boxes instead of composed artwork. Swatches replaced stroke widths and changed ignored raster fills. Several settings claimed GPU capabilities or simulated jobs without representing the running implementation.

The reference is the current [Affinity Studio](https://www.affinity.studio/graphic-design-software) and the official [workspace anatomy](https://affinity.help/photo2/en-US.lproj/pages/Workspace/interface.html): compact neutral chrome, a Studio selector, contextual controls, tools to the left, artwork in the centre, independent colour and inspection groups to the right, and contextual feedback below. Petunia retains its own branding, assets, capabilities and interaction boundary. A Layout persona is not fabricated.

## Decision

1. The production shell composes a menu strip, Vector/Pixel Studio toolbar, document tabs, a horizontally scrollable context toolbar, a scrollable one/two-column tool rail, canvas, optional auxiliary docks, a bounded right Studio and status feedback. Colour/Swatches remain available above the lower Layers/Properties/Colors/History/Navigator/Tasks tabs. The upper group and whole right Studio can collapse independently. Auxiliary panels reserve both canvas dimensions. Shared splitters capture a drag across the window, accept arrow keys (Shift for a larger step), and cancel an active drag with Escape. They may open/close after mount without changing hook order. Requested widths survive window resizing; the effective width respects a minimum canvas reserve.
2. Shared Studio controls own accessible labels, keyboard activation, selected/disabled states, focus rings, pointer targets and original semantic icons. UI typography and colour/spacing metrics live in the theme. Filled icons use an outline fallback where no equivalent filled glyph exists. EN and pt-BR copy use one catalogue and update with the active locale. Appearance settings belong in Preferences, together with a workspace reset.
3. Layer rows use typed ObjectId plus SessionIdentity for collapse state. Hiding descendants never removes document content. Additive selection and independent rename/visibility/lock controls preserve the selection. Object mutation uses bridge commands; workspace visibility, panel state and camera changes remain view state.
4. The compact colour wheel reads the actual selected fill/stroke or raster brush and supports pointer and arrow-key input. Swatches and full RGB/hex/Lab/spot editors share a target. Vector colour changes apply atomically to unlocked selected vector objects and retain stroke widths. Raster colour changes update the actual brush, preserve alpha and clear a previous literal CMYK ink on RGB selection. Native CMYK entry and ICC-derived previews retain ADR-011 semantics. Favourite swatches store the current colour in application-session UI state; disk persistence and certified spot libraries are not claimed.
5. The Navigator reuses immutable preview sources and the existing composition worker, including text, images, raster and effects. It draws the main camera footprint and pans the real view by pointer or keyboard without changing zoom or document revision. Canvas presentation epochs publish new worker frames, camera changes and selection/text overlays immediately despite callback equality caching. New/opened sessions fit their active artboard once after the canvas is sized; switching back retains the session camera. Embeddable low-level workspaces may opt out of initial fit.
6. New Document accepts exact name, width, height, bleed and margin with explicit point units, decimal commas and validated dimensions. Invalid values retain the draft and create no tab/history. Presets and orientation are shortcuts to the same fields. Modal forms use bounded Popups; search results and inspectors scroll. Export only displays limitations relevant to the selected format. Preferences describe the actual renderer, and Tasks presents real jobs.
7. Canvas shortcuts ignore text-input focus and unmodified keys owned by buttons/tabs/tree items. Modal ownership continues to suppress editing shortcuts. Disabled capability reasons, worker failures and file feedback stay visible; there are no success claims for unimplemented features.

## Atlas mapping

| Contract | Implementation |
| --- | --- |
| 08.1 / 08.2 / 08.3 | Theme tokens, compact shell, independent Studio groups, resize/collapse/reset |
| 08.5 / 08.14 | Shared accessible Studio controls, focus and shortcut ownership |
| 08.6 / 08.17 | Session camera fit, actual Navigator composition, layer collapse, semantic colour |
| 08.18 / 08.19 | Exact numeric New Document, bounded modals, truthful feedback and units |
| 08.12 / 09.16 | Appearance preferences and canonical EN/pt-BR copy |
| 08.16 / 14.7 | Production-shell headless interaction, pixel and screenshot evidence |

## Consequences and verification

No document schema or domain crate imports change. Existing raster, text, ICC, export and undo contracts remain in their established adapters. Focused interaction tests and the applicable gauntlets run **after implementation**, as requested. [Execution ledger](/implementation/uiux-studio.json) records actual commands, results, screenshots and hashes. [Studio implementation](/developers/uiux-studio) records the audit and bounded acceptance.

This increment does not certify a complete MVP/V1 release or Affinity feature parity. Mixed numeric multi-selection, a fully configurable dock manager, favourite persistence, light/system themes, full high-contrast modes, native screen-reader/IME/tablet/HiDPI acceptance, representative usability and remaining professional print/typography capabilities retain their roadmap scope. Headless accessibility assertions are not real assistive-technology certification. Prumo was unavailable; the ledger is the local goal/contract/evidence microcontext.
