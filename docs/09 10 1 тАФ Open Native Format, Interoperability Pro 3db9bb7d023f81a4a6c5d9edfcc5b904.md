# 09.10.1 — Open Native Format, Interoperability Profile & Compatibility Representations

<aside>
🔓

**Decision:** Aubrieta's native document is an open, ZIP-compatible package designed for full Aubrieta fidelity while carrying standard editable compatibility projections for other creative applications.

</aside>

# Core principle

No existing interchange format can preserve every Aubrieta concept — hybrid vector/raster state, live effects, masks, data bindings, semantic CMYK/Spot color, plugin extension data and future Aubrieta-specific operations — while also being natively editable by every external application. Therefore Aubrieta separates **canonical authoring state** from **standard interoperability projections**.

# Native extension and product naming

**Aubrieta Design** is the canonical product name. **`.aubrieta` is the canonical native extension.** **`.aubri` is an accepted short alias for the exact same native format** and may be exposed as an advanced Save As choice or accepted on open/import. Writers, documentation, examples, file associations and interchange tooling default to `.aubrieta` so the ecosystem has one obvious canonical suffix.

`.abrt` is explicitly rejected. Even though the extension scan did not surface a prominent established `.abrt` file format, **ABRT** is already the established name of Fedora/RHEL's Automatic Bug Reporting Tool, creating needless Linux/search/documentation ambiguity.

`.petunia` remains reserved for Petunia3D and `.pds` remains rejected because of unrelated established collisions. The domain model must never hard-code a suffix: platform/file adapters own associations and the package `mimetype` + manifest/schema identify the actual format. Extension aliases do not create separate schemas or compatibility levels.

## Extension alias behavior

- **New document / Save As default:** `.aubrieta`.
- **Open:** accept both `.aubrieta` and `.aubri`.
- **Save:** preserve the suffix of the file that was opened; opening `poster.aubri` and pressing Save keeps `poster.aubri`.
- **Save As:** present `.aubrieta` first; `.aubri` may appear as an advanced/short-name choice.
- **CLI/importer detection:** prefer package `mimetype` + manifest magic/schema over filename suffix.
- **OS associations:** both suffixes may map to the same Aubrieta document type and application icon.
- **MIME:** both aliases resolve to the same Aubrieta media type; use a provisional `application/x-aubrieta-design` until/if a formal media-type registration is pursued.
- **Canonical format ID:** `org.aubrieta.design.document`; this stable internal identity is independent of the filename suffix and is stored/validated through the package manifest/media-type marker.
- **No automatic rename:** the application never silently changes `.aubri` to `.aubrieta` or vice versa.
- **Documentation/spec fixtures:** always use `.aubrieta` unless specifically testing alias behavior.

# Container

The file is a standard ZIP-compatible package with deterministic paths and bounded extraction rules. Follow the useful packaging convention of placing a small uncompressed media-type marker first when practical, similar to standardized ZIP document packages.

Conceptual layout:

```
project.aubrieta
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

Container compression handles JSON verbosity. Aubrieta may later generate a binary acceleration cache, but caches are disposable and never the only source of truth.

# Open specification

Publish the package layout, JSON Schemas, namespace/version rules, sample fixtures, migration notes and conformance tests. Prefer a permissive license for the **format specification/schemas** even if the application remains GPLv3, so external programs can implement readers/writers without adopting Aubrieta's application license.

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

Unsupported Aubrieta semantics use the export/degradation policy: preserve as metadata where harmless, approximate, expand, rasterize or report unsupported. Never silently pretend a rasterized effect is still structurally editable.

# Direct universal-save modes

Aubrieta exposes explicit save/export targets in addition to `.aubrieta`:

- **Editable Vector / SVG** — best cross-editor vector editing. Aubrieta may include namespaced metadata that standards-compliant consumers can ignore.
- **Editable/Print PDF** — best multipage, print/color and broad professional-editor exchange.
- **Raster interchange** — TIFF/PNG baseline; PSD can be added after a sufficiently reliable writer/roundtrip suite exists.

A document that exceeds a target format's capabilities gets a preflight/degradation report before writing.

# Compatibility companions

Optional document preference: **Keep compatibility companions next to the Aubrieta document**. When enabled, explicit save also refreshes `project.svg`/Surface SVGs and/or `project.pdf` beside `project.aubrieta`. Default can remain off globally to avoid folder clutter, while export presets can enable it for collaborative workflows.

# Opening without Aubrieta

An `.aubrieta` or `.aubri` file can always be opened by ZIP tooling to inspect/extract its documented contents. A user without Aubrieta can extract `interchange/document.pdf` or Surface SVGs. Third-party applications can implement a direct Aubrieta native-format importer using the published schemas without reverse engineering.

# Third-party integration strategy

Provide a small reference reader library and CLI:

```
aubrieta inspect project.aubrieta
aubrieta unpack project.aubrieta out/
aubrieta convert project.aubrieta --to svg
aubrieta convert project.aubrieta --to pdf
```

Later provide importer/reference plugins where an external editor exposes a suitable extension API.

# Clipboard interoperability

Cross-application copy/paste should advertise multiple representations simultaneously when platform APIs permit: Aubrieta semantic payload + SVG + PDF/vector clipboard flavor + PNG fallback. The receiving application chooses the richest format it understands.

# Fidelity rule

Roundtrip through an external standard format is **not** equivalent to native `.aubrieta`/`.aubri` roundtrip. Aubrieta must expose a compatibility/fidelity report and never silently discard unsupported live semantics.

# Security

ZIP entry count, decompressed size, nesting/path traversal, JSON depth, SVG active/external content and resource sizes are bounded. Opening the package never executes plugin code; unknown extension namespaces remain opaque data.

# Decision status

**Resolved for V1 architecture:** product name **Aubrieta Design**; canonical native suffix **`.aubrieta`**; accepted short alias **`.aubri`** for the same schema/container; `.abrt` rejected; `.petunia` reserved for Petunia3D; open ZIP package + canonical JSON structural payload + binary resources + derived PDF/SVG compatibility representations.