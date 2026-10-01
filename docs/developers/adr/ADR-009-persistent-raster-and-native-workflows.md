# ADR-009: Persistent raster, binary resources and native workflow publication

**Status:** Accepted contract; implementation awaiting validation  
**Date:** 2026-10-01  
**Scope:** Milestone Required (MVP)

## Context

The MVP lacked canonical editable pixels, binary resource persistence and recovery. Brush stamps were presentation artifacts; masks and group clipboard operations could lose semantics. Cold file/image work and export ran on the UI thread. Text style, fill rules and SVG output did not consistently preserve the evaluated scene. This extends [ADR-005](/developers/adr/ADR-005-immutable-image-assets), [ADR-006](/developers/adr/ADR-006-shaped-text-and-canvas-preview), [ADR-007](/developers/adr/ADR-007-desktop-file-workflows) and [ADR-008](/developers/adr/ADR-008-object-edit-drafts). Earlier pending statements in those wave records are historical where superseded below.

## Decision

### Canonical resources and schema

Native schema **4** adds `ShapeKind::Raster`, finite pixel/mask planes and typed uniform `TextStyle`. Vector fill winding is persisted per object: existing documents default to EvenOdd; declared SVG input defaults to NonZero; prepared text uses NonZero. Schemas 1–3 migrate through existing geometry admission; raster descriptors cannot enter an older schema.

Pixel layers use straight RGBA8/RGBA16; mask layers use Gray8/Gray16 coverage. All 16-bit samples are little-endian. A sorted sparse 128×128 tile map owns `Arc<Tile>` with separately shared `Arc<Vec<u8>>` payloads. Snapshot metadata and touched pixel data detach independently. Zero-background empty tiles are pruned on commit. Opaque masks start with no resident tiles and implicit full finite-plane coverage; edited zero tiles are retained and outside-plane padding stays zero. Original placed images remain encoded editable sources; painting creates a separate layer unless an editable raster layer is selected. Eraser requires a pixel/mask target.

Admission limits are conjunctive: 32,768 pixels per dimension, 16,777,216 pixels per plane, 2,048 resident tiles and 128 MiB tile storage; 1,024 surfaces, 100,000 objects, 1,000,000 source path verbs, 64 KiB text per object and 256 MiB unique canonical resource capacity. These are resource quotas, **not a total process RSS guarantee**. COW container metadata, history, font engines, codecs and transient surfaces have separate budgets. History estimates exclude JSON pixel arrays and account for changed retained tile payloads.

PTND manifests require the binary-resource flag for schema 4. `resources/index.json` binds stable object IDs to descriptors and SHA-256-addressed original-image/tile binaries. Resource metadata is separated from readable document JSON. Identical payloads are written once and shared after opening. Entry names, descriptor dimensions/format/alpha/coordinates, exact entry sets, hashes and byte/count budgets are checked before attachment. Missing image sources fail saving; file paths are informational, never re-read during rendering.

### Gesture and asynchronous publication

Brush/eraser draft planes, cumulative coverage and AA selection stencils are derived immutable previews. Interpolated pressure/spacing/flow work has bounded event/stroke quotas; dabs composite against the captured original, so repeated overlap does not repeatedly exceed stroke master opacity. Pointer Up publishes one expected-source Command transaction. Esc, a changed session/revision/surface or a failed quota discards the draft. Active empty raster selections paint nothing; inverted empty selections cover the finite plane. Four-connected flood fill uses a bounded scanline span algorithm, a visited bitset and cooperative cancellation before one Command commit.

Desktop open/save/place/export and clipboard codecs run in one worker with one queued slot. A captured `SessionIdentity`, revision and surface prevent equal-revision tabs from accepting each other's asynchronous mutations. Saves acknowledge the captured history state on the original tab; intervening edits stay dirty. Saving before Close All completes all dirty snapshots before closing any tab; newer edits abort closure. Native dialogs populate path fields asynchronously; actual publication remains explicit.

Ordinary native saves and exports acquire a stable same-directory sidecar file lock before encoding/publication. Temporary content is flushed and synchronized before atomic replacement, then the parent directory is synchronized on Unix. Lock inodes are retained to prevent unlink/recreate split ownership; target symlinks/non-regular outputs and lock symlinks are rejected. This coordinates cooperating writers; it is not a guarantee against an unrelated process intentionally rewriting the destination. Recovery candidates have private random names and are published under a separate store lease.

Recovery writes complete binary PTND snapshots, with title, original path, revision and capture time in one manifest. The bounded store never writes the original. Dirty open tabs are debounced; pending writes finish before cleanup. Startup offers available copies; restore opens a new dirty tab requiring Save As. Corrupt copies fail visibly. Original paths are informational. Dismissing the offer preserves backups.

### Clipboard, typography, SVG and RGB

Whole-subtree copy/duplicate/delete retains descendants and remaps parent, child, mask and text-path references together. Detaching a fragment derives world TRS on copied descriptors without temporarily invalidating the original clip group. Paste offsets account for source/target artboard origins; resource payloads stay shared. Cut removes originals only after native clipboard ownership succeeds and its source revision still matches.

The Linux worker adapter uses `wl-copy`/`wl-paste` on Wayland or `xclip` on X11, without shell evaluation. Distribution must supply `wl-clipboard`/`xclip`. Transfers have a 16 MiB limit and cancellation/timeout. Native editable fragments reuse the PTND codec under `application/vnd.petunia-design-studio.fragment+zip`. Paste prefers the advertised native fragment, then declared SVG, PNG and UTF-8 text; malformed or unsupported advertised vectors are diagnosed instead of silently flattened. Other platform backends remain explicit missing capabilities.

Text family, size, weight, italic, line height, tracking, alignment and artistic/frame flow persist and undo together. Artistic text preserves explicit breaks without wrapping; frames wrap and clip at their finite box before effects. Worker-prepared glyph/cluster data is shared with artistic overflow bounds and text selection. Missing requested families and overset frames produce diagnostics while the editable source stays intact. Native multiline input handles editing/IME in the object draft dialog. Exact in-canvas caret/selection integration with the prepared artwork glyph runs remains **unfinished**; this ADR does not claim full M2.2 acceptance.

SVG output reads the same immutable render scene: precise local paths/world transforms, scoped artboard origins, ordered paints, supported strokes/gradients, groups, masks, glyph outlines, lossless 8/16-bit embedded PNG and supported blur/shadow. Unsupported adjustment representations, stroke alignment, live resampling and text-on-path fail with explicit reasons. Mask instances use unique SVG identifiers. The importer declares a strict bounded subset: basic shapes, groups, affine transforms, solid paints and M/L/H/V/C/S/Q/T/Z paths, with inherited winding/strokes. Arcs, stylesheets, gradients/defs, filters, images and SVG text input are currently unavailable; no external resource is fetched.

ICC RGB image derivatives convert to sRGB with the existing moxcms CMM, relative colorimetric intent, bounded profile/CLUT/TRC parsing and scanline buffers. Coverage is copied unchanged; originals retain source precision/profile bytes. Gray/CMYK profiles and professional proof/monitor acquisition remain unavailable in this path. This extends earlier blanket ICC-image rejection, **not** the V1 true CMYK/proof/PDF scope. Export preview shows the active artboard through the same transparent CPU compositor, fitted to screen; requested DPI still controls the actual PNG output. Canceling export drops its worker handle and atomic publication checks cancellation.

## Consequences and evidence

**All current changes are unvalidated.** The user explicitly deferred tests, builds and gates until every MVP feature is implemented. Regression sources cover sparse opaque masks, binary integrity/sharing, selection/stroke transactions, recovery, native fragment topology, atomic output/cancellation, ICC RGB coverage/precision and strict SVG admission/precision. Source generation and write-mode formatting do not establish correctness. Historical successful gates do not validate this head.

Remaining MVP work includes coherent in-canvas text caret/IME, real Linux tablet pressure beyond the toolkit's touch-force adapter, measured aggregate/cache/history behavior, real histogram composition, the four task fixtures and Linux accessibility/installation/backend acceptance. The cloud has no physical tablet or calibrated display; hardware and representative user evidence cannot be fabricated. Basic RGB input conversion is implemented; display-profile acquisition/configuration is still open. MVP and V1 gates remain open, and PDF is omitted from the MVP export chooser with a V1 reason.

References actually inspected: Paul Heckbert, *A Seed Fill Algorithm*, Graphics Gems (1990), [published source](https://github.com/erich666/GraphicsGems/blob/master/gems/SeedFill.c); W3C SVG 2 [paths](https://www.w3.org/TR/SVG2/paths.html) and [coordinates](https://www.w3.org/TR/SVG2/coords.html); local moxcms 0.9.1 bounded profile parsing/transform implementations; Rust file-lock semantics; Freya native multiline input and touch-force mapping. Existing Porter–Duff/linear-premultiplied filtering rationale remains in ADR-005. This records source/API research, not full-paper review, competitor-equivalence or completed acceptance.
