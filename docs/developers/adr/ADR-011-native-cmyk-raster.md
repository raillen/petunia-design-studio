# ADR-011: Native CMYK raster, ICC resources and ink-preserving interchange

**Status:** Accepted implementation contract; execution evidence is separate.
**Date:** 2026-10-03
**Scope:** V1 Required; existing MVP behavior remains covered by final workspace gates.

## Context

ADR-010 established real ICC resources and an RGB-composite proof. RGBA storage cannot retain four process inks plus independent transparency. TIFF admission through the generic image facade converts CMYK to RGB, so accepting that result as authored ink would discard separations before editing begins.

## Decision

1. `Cmyka8` and `Cmyka16` store straight C/M/Y/K/alpha, with five integer lanes and little-endian 16-bit tile bytes. Ink is never premultiplied. RGB access returns an error on native ink. Transparent hidden ink survives commit; white and pure cyan with K=0 remain visible. Sparse bounds, zero padding, tile/dense output limits and copy-on-write sharing apply. Native samples require a validated CMYK output ICC profile; an empty resource descriptor is resolved before publication from a binary package.
2. Assignment changes profile metadata and shares exact pixel bytes. Conversion is an explicit immutable CMYK-to-CMYK LittleCMS operation, never an RGB round trip; matching profile identity copies exact samples. Alpha is excluded from CMM transforms. A transform belongs to one operation/worker and processes row batches. Display uses a disposable ICC-derived sRGB tile cache of at most eight tiles (2 MiB), without changing authored resources. Cancellation rejects incomplete outputs. DeviceLink and press-specific black-generation preservation are not claimed.
3. Schema 6/resource index 3 stores layer ICC bytes as SHA-256 binary resources, deduplicated with surface profiles and included in admission/history budgets. The descriptor contains no inline ICC or tile arrays. Schemas 1–5 and indexes 1–2 remain readable for their supported layouts; old contracts cannot admit CMYKA. Invalid digests, unexpected resources, missing profiles and contradictory descriptors fail admission.
4. Painting and flood fill operate on literal process channels with independent alpha. Brush RGB converts through the layer profile once at gesture start; explicit CMYK ink bypasses that conversion. Erasing changes coverage only. Pressure, transformed placement, selections, cancellation, captured session/revision and one-gesture undo remain in the existing Action/Command boundary. Creating a native 16-bit layer requires a loaded layer/surface press profile. TIFF placement produces an editable native layer on a worker; profile pickers capture the original object identity.
5. Native TIFF reads/writes process CMYK 8/16-bit, orientation 1–8, embedded ICC and explicit unassociated alpha (or fully opaque four-channel input). The codec is called directly. Output uses bounded lossless Deflate strips, DPI and atomic publication. Missing/nonpress ICC, associated alpha, planar layouts, multipage/nonprocess inputs and unsupported precision are rejected. Export is **one authored layer**, before masks, effects, opacity and page composition; it is not whole-page separation export.
6. Regular PDF embeds native four-channel samples with the original ICC and an independent soft mask, preserving 8/16-bit ink precision. The encoded native-image path preserves the same semantics. The current backend requires a common CMYK profile across included surfaces, native layers and encoded native images; export rejects differing profiles with a reason to convert explicitly. The original profile is embedded as the document's CMYK color space, including ICC versions the backend would otherwise silently omit from individual images. SVG/RGB-only interchange obtains an explicit display derivative. Strict PDF rejects unsupported page effects; explicitly allowed RGB page rasterization reports `CMYK_PAGE_CONVERTED_TO_RGB`. PDF/X-4, OutputIntent conformance, spot/overprint parity and direct native-ink page proof remain unavailable.
7. `ink_coverage` reports literal visible-pixel ink totals and per-channel maxima. A TAC ceiling is an optional caller-supplied press policy, never a universal constant. Per-layer separation derivatives contain ink times alpha. These are inspection tools, not overprint-aware page plates. The existing RGB-composite proof is blocked for authored CMYK raster/vector/image inputs rather than displaying it as faithful native proof.
8. CLI subcommands perform real file operations: `cmyk-import`, `cmyk-export-layer`, `cmyk-inspect`, `cmyk-assign`, `cmyk-convert` and `export-pdf`. Ambiguous documents require `--object ObjectId:N`. No arguments retain conformance behavior. File operations use the domain command boundary and atomic outputs. EN/pt-BR UI strings and capability reasons describe the supported scope.

## Atlas mapping

| Contract | Implementation and limits |
| --- | --- |
| 09.6 / 10.9 raster | Five-lane 8/16-bit COW tiles, native brush/eraser/fill, immutable drafts |
| 09.3 commands | Typed creation/profile assignment, guarded worker admission and undo/redo |
| 09.9 color / V1-B | ICC assignment/conversion/display; layer TAC and separation inspection |
| 09.13 persistence / 15.A migration | Schema 6/index 3 binary profile resources; legacy RGB/gray reads |
| 08.17 panels / 08.18 workflows | Native layer creation, brush ink, captured-object profile dialog and editable TIFF placement |
| 08.29 / V1-C interchange | Layer TIFF and regular ICC CMYK PDF; explicit RGB degradation |
| 14.1 / 08.16 validation | Native byte/alpha/ICC regressions, CLI flows, actual Freya interactions and independent Poppler reading |

## Consequences and evidence

Focused tests cover exact tile/TIFF/package round trips, 16-bit/hidden samples, orientation, bad resources, budget admission, source/alpha immutability, cancellation/atomic output, gesture history, CLI file flows, native proof blocking and color-panel interactions. CI also runs an independent pypdf/Poppler check of the decompressed ink samples, soft-mask samples and original ICC bytes for both editable layers and encoded images. Validation runs only after implementation, as requested. [Execution ledger](/implementation/native-cmyk-v1.json) records actual results; source/test counts are not execution evidence.

**The complete V1 and MVP release acceptance remain open.** Direct native page composition/proof, overprint/spot parity, DeviceLink black policy, PDF/X-4, advanced typography and representative print/hardware/user acceptance remain V1 Required. The project-owned synthetic profile is an engineering fixture, not a production printer characterization.

Primary API references: [TIFF 6.0](https://download.osgeo.org/libtiff/doc/TIFF6.pdf), [libtiff tag definitions](https://gitlab.com/libtiff/libtiff/-/blob/master/libtiff/tiff.h), [TIFF codec](https://docs.rs/tiff/0.11.3/tiff/), [LittleCMS API](https://www.littlecms.com/LittleCMS2.16%20API.pdf), [ISO-approved PDF ICCBased errata, 8.6.5.5](https://pdf-issues.pdfa.org/32000-2-2020/clause08.html).
