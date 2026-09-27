# 09.10.1 — Open Native Format, Interoperability Profile & Compatibility Representations

<aside>
🔓

**Decision:** Petunia's native document is an open, ZIP-compatible package designed for full Petunia fidelity while carrying standard editable compatibility projections for other creative applications.

</aside>

# Core principle

No existing interchange format can preserve every Petunia concept — hybrid vector/raster state, live effects, masks, data bindings, semantic CMYK/Spot color, plugin extension data and future Petunia-specific operations — while also being natively editable by every external application. Therefore Petunia separates **canonical authoring state** from **standard interoperability projections**.

# Native extension and product naming

**Petunia Design Studio** is the canonical product name. **`.PTND` is the only canonical native extension written by current versions.** The former `.aubrieta` and `.aubri` suffixes are **legacy migration inputs only** and must not appear as normal Save As choices, new project associations or current documentation examples.

`.petunia` remains reserved for Petunia3D. The domain model never relies on suffix alone: platform/file adapters own associations and the package `mimetype` + manifest/schema identify the actual format.

## Current and legacy extension behavior

- **New document / Save As default:** `.PTND`.
- **Open current format:** accept `.PTND`; case variations may be tolerated by platform/file matching without creating a second format identity.
- **Open legacy format:** `.aubrieta` / `.aubri` are routed through the explicit legacy reader/migration path only when compatible historical fixtures/schema exist.
- **Save current document:** preserves `.PTND`.
- **Save migrated legacy document:** writes `.PTND` after explicit conversion/save policy; it does **not** silently overwrite the only legacy source.
- **CLI/importer detection:** prefer package media marker + manifest magic/schema over filename suffix.
- **OS associations:** current associations advertise `.PTND`; legacy associations may be registered only as migration-open handlers when intentionally supported.
- **MIME:** use `application/vnd.petunia-design-studio.project+zip` as the project-defined media identity pending any formal registration.
- **Canonical format ID:** `studio.petunia.design.document`; this identity is independent of filename suffix and is stored/validated through package metadata.
- **Documentation/spec fixtures:** use `.PTND` by default. Legacy suffixes appear only in migration tests, compatibility documentation and historical material.

# Container

The file is a standard ZIP-compatible package with deterministic paths and bounded extraction rules. Follow the useful packaging convention of placing a small uncompressed media-type marker first when practical, similar to standardized ZIP document packages.

Conceptual layout:

```
project.PTND
├── mimetype
├── manifest.json
├── document/
│   ├── document.json
│   └── extensions/
│       └── <namespace>.json
├── resources/
│   ├── images/
│   ├── icc/
│   ├── fonts/
│   ├── data/
│   └── other/
├── raster/
│   └── <layer-id>/...
├── interchange/
│   ├── document.pdf
│   ├── surfaces/
│   │   ├── <surface-id>.svg
│   │   └── ...
│   └── interchange-manifest.json
├── previews/
│   ├── thumbnail.png
│   └── composite.webp-or-png
└── cache/                 # optional, disposable
```

# Canonical payload

`document/document.json` is the canonical V1 structural payload. Reasons: open tooling, language neutrality, easy inspection/recovery, JSON Schema validation, stable migration tooling and straightforward third-party implementations. Large pixel/tile/font/image/ICC payloads do **not** become base64 JSON; they live as separately indexed binary resources.

Container compression handles JSON verbosity. Petunia may later generate a binary acceleration cache, but caches are disposable and never the only source of truth.

# Open specification

Publish the package layout, JSON Schemas, namespace/version rules, sample fixtures, migration notes and conformance tests. Prefer a permissive license for the **format specification/schemas** even if the application remains GPLv3, so external programs can implement readers/writers without adopting Petunia's application license.

# Compatibility representation — PDF

Normal explicit saves should be able to maintain an embedded **compatibility PDF**. It is a derived projection, never canonical authoring state.

Use it for:

- multi-Surface/page representation;
- CMYK/ICC/Spot-oriented exchange where supported;
- vector/text output where fidelity permits;
- applications whose strongest interchange path is PDF.

This follows the useful idea behind PDF-compatible native creative files: preserve the application's own state while carrying a representation broadly consumable by other software.

# Compatibility representation — SVG

Generate one editable SVG per Surface/artboard where useful. SVG is the preferred vector-editability projection for paths, shapes, text where possible, fills, gradients, clipping and common vector constructs.

Unsupported Petunia semantics use the export/degradation policy: preserve as metadata where harmless, approximate, expand, rasterize or report unsupported. Never silently pretend a rasterized effect is still structurally editable.

# Direct universal-save modes

Petunia exposes explicit save/export targets in addition to `.PTND`:

- **Editable Vector / SVG** — best cross-editor vector editing. Petunia may include namespaced metadata that standards-compliant consumers can ignore.
- **Editable/Print PDF** — best multipage, print/color and broad professional-editor exchange.
- **Raster interchange** — TIFF/PNG baseline; PSD can be added after a sufficiently reliable writer/roundtrip suite exists.

A document that exceeds a target format's capabilities gets a preflight/degradation report before writing.

# Compatibility companions

Optional document preference: **Keep compatibility companions next to the Petunia document**. When enabled, explicit save also refreshes `project.svg`/Surface SVGs and/or `project.pdf` beside `project.PTND`. Default can remain off globally to avoid folder clutter, while export presets can enable it for collaborative workflows.

# Opening without Petunia

An `.PTND` or `.aubri` file can always be opened by ZIP tooling to inspect/extract its documented contents. A user without Petunia can extract `interchange/document.pdf` or Surface SVGs. Third-party applications can implement a direct Petunia native-format importer using the published schemas without reverse engineering.

# Third-party integration strategy

Provide a small reference reader library and CLI:

```
petunia-design inspect project.PTND
petunia-design unpack project.PTND out/
petunia-design convert project.PTND --to svg
petunia-design convert project.PTND --to pdf
```

Later provide importer/reference plugins where an external editor exposes a suitable extension API.

# Clipboard interoperability

Cross-application copy/paste should advertise multiple representations simultaneously when platform APIs permit: Petunia semantic payload + SVG + PDF/vector clipboard flavor + PNG fallback. The receiving application chooses the richest format it understands.

# Fidelity rule

Roundtrip through an external standard format is **not** equivalent to native `.PTND`/`.aubri` roundtrip. Petunia must expose a compatibility/fidelity report and never silently discard unsupported live semantics.

# Security

ZIP entry count, decompressed size, nesting/path traversal, JSON depth, SVG active/external content and resource sizes are bounded. Opening the package never executes plugin code; unknown extension namespaces remain opaque data.

# Decision status

**Resolved for V1 architecture:** product name **Petunia Design Studio**; canonical native suffix **`.PTND`**; accepted short alias **`.aubri`** for the same schema/container; `.abrt` rejected; `.petunia` reserved for Petunia3D; open ZIP package + canonical JSON structural payload + binary resources + derived PDF/SVG compatibility representations.