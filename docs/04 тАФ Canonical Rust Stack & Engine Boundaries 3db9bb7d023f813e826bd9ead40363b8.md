# 04 — Canonical Rust Stack & Engine Boundaries

# Core/engine stack

| Área | Escolha atual | Função |
| --- | --- | --- |
| Language | Rust | core, engines, adapters principais |
| Vector renderer | Vello | display vector/text |
| Raster/compositor GPU | wgpu | Photo, compute, compositing |
| Paths/curves | Kurbo | Bézier/geometry vocabulary |
| Boolean/offset | iOverlay | Pathfinder, regions, offsets |
| Tessellation aux. | Lyon | special GPU geometry |
| Spatial | rstar | hit test, snapping, culling |
| Text layout | Parley | rich text/text frames |
| Shaping/fonts | HarfRust + Skrifa + Fontique + ICU4X | OpenType, fallback, Unicode |
| Text interaction prior art | COSMIC Text | caret/selection/IME study |
| SVG | usvg/resvg | normalization/import/render reference |
| Color | moxcms | Rust-first ICC/CMYK/Lab transforms |
| Color reference | LittleCMS 2 | differential/reference tests initially |
| PDF | Krilla | high-level professional PDF |
| PDF low-level | pdf-writer | escape hatch |
| Raster I/O | image + image-tiff | image codecs/TIFF |
| Parallel CPU | Rayon | jobs/data parallelism |
| Accessibility | AccessKit | semantic tree and OS adapters |
| Plugin host | Runtime-neutral semantic SDK + capability broker + **Lua 5.5/`mlua` scripting adapter** | common safe extension contract; Lua is accepted V1 scripting runtime |
| High-isolation plugin tier | Wasmtime + WASI Component Model | candidate/preferred strongly isolated component tier |
| Serialization | Serde | native/document adapters |
| Property/fuzz tests | proptest + cargo-fuzz | geometry, formats, commands |
| Snapshots/bench | insta + Criterion | regression/performance |

# Engine boundaries

- `aubrieta_document` conhece tipos próprios, não Kurbo/Vello/wgpu/GUI types em interfaces públicas.
- `aubrieta_scene` extrai uma representação de render reconstruível e incremental.
- `aubrieta_vector_renderer` consome scene fragments vetoriais; `aubrieta_raster_gpu` trata pixel work; `aubrieta_compositor` resolve masks/clips/blend/isolation.
- export adapters leem o documento/evaluated scene, nunca screenshots do viewport.

# Color Engine

`ColorValue` representa RGB, CMYK, Gray, Lab, Spot e Registration. `ColorProfile`, `ColorTransform`, `RenderingIntent`, `ProofingConfig` e políticas de output ficam em `aubrieta_color`.

Display pipeline converte semantic color → working/proof/display profile → display RGB. O documento nunca faz roundtrip CMYK→RGB→CMYK para armazenar cor.

# Raster

Tile-based storage/cache; dirty regions; **8-bit and 16-bit/channel are V1-required storage/processing profiles where each operation supports them**. GPU resource cache é derivado e descartável. Any temporarily unsupported 16-bit operation must fail/convert only through an explicit documented policy; silent precision loss is forbidden.

# Formato nativo

**Aubrieta Design** uses `.aubrieta` as the canonical native extension and `.aubri` as an accepted short alias for the exact same package/schema. `.petunia` remains reserved for Petunia3D; `.pds` and `.abrt` are rejected. The file is an open ZIP-compatible package with readable `manifest.json`, canonical schema-versioned `document/document.json`, separate binary resources, optional disposable caches and derived PDF/SVG interoperability projections. Package identity comes from the internal media-type/manifest/schema rather than the filename suffix. See 09.10/09.10.1 for the canonical package contract.

# Primary UI stack — GPUI

GPUI is now the default and primary desktop UI target for Aubrieta Design. This changes implementation priority, not the domain boundary: no canonical document, geometry, raster, color, typography or export crate may depend on GPUI types.

## Canonical GPUI layers

- `gpui-kit` — preferred facade that pins and re-exports compatible GPUI/Kit layers and provides application/bootstrap infrastructure.
- `gpui-base` — **preferred foundation for Aubrieta-owned UI**: unstyled behavior, state and infrastructure for text editing, selection, docking, virtual lists, dialogs, popovers, controls, motion/history and navigation. Aubrieta should own the final design language on top of this layer.
- `gpui-component` — selectively reused complete styled components when they materially reduce implementation cost. Do not let its default visual identity become Aubrieta's design system by accident.
- `gpui-kit-assets` — default Lucide asset bundle and typed icon paths; use compile-time selection where practical.
- `gpui-fps` — development-only frame/FPS/resource telemetry.
- `gpui-wry` — optional WebView for documentation/help/release surfaces only; never the main creative canvas.
- `gpui-shell` — research/optional scripting prior art only. It does not define Aubrieta's Plugin SDK. **Lua 5.5 via `mlua` is the accepted V1 scripting runtime**; Wasmtime/WASI remains the high-isolation component tier.
- `gpui-plot` / GPUI Kit Chart/Plot — optional histogram, curves, levels, telemetry and Photo analysis surfaces after benchmark.
- `gpui-ui-kit` and `gpui-engram` — prior art/reference implementations, not baseline dependencies unless a component proves uniquely valuable.

## Aubrieta UI architecture

```
GPUI / gpui-kit
    ↓
gpui-base
    ↓
Aubrieta Design System
    ├── foundation tokens
    ├── typography
    ├── spacing / radius / elevation
    ├── icons
    ├── controls
    ├── docking chrome
    ├── panels
    └── persona-specific surfaces
          ↓
AubrietaGuiBridge
          ↓
Actions / Commands / Application Core
```

The preferred rule is **reuse behavior, own presentation**.

## Icon stack

Aubrieta needs a toolkit-independent semantic icon registry. Runtime IDs are extensible namespaced newtypes such as `aubrieta.tool.node-edit`, `aubrieta.action.boolean.union`, `aubrieta.mask` and `aubrieta.adjustment.curves`; generated Rust constants may provide enum-like ergonomics for built-ins, while adapters resolve the semantic ID to the active icon family.

Primary icon sources:

- **Lucide via `gpui-kit-assets`** — default baseline because GPUI Kit already bundles and maintains it.
- **`gpui-phosphor`** — preferred optional family for users who want Phosphor; it provides typed icon names, embedded SVGs and selectable weights including Regular, Thin, Light, Bold, Fill and Duotone.
- **Aubrieta custom SVG set** — mandatory for domain-specific creative-tool icons that generic sets do not express well: Pen/Node tools, boolean variants, stroke alignment, masks, channels, CMYK/separations, gradients, perspective, data merge, raster adjustments and other product-specific glyphs.
- **`gpui-symbols`** — macOS-only optional use of SF Symbols for platform-native affordances; never canonical for cross-platform feature icons.
- Other icon families such as Tabler/Iconoir may be imported as ordinary SVG assets through an Aubrieta asset adapter if later desired; no framework dependency is required just to use an SVG set.

Rules:

1. document/domain crates never reference icon names;
2. actions reference semantic `IconId`, not SVG paths;
3. the UI adapter maps `IconId` → active family/fallback;
4. an Aubrieta-specific icon always overrides a generic family icon when semantic precision matters;
5. all icon licenses/attributions remain tracked in third-party notices.

## UI testing and observability

Use upstream GPUI `test-support` / `#[gpui::test]` capabilities where compatible, GPUI Kit's accessibility-driven testing APIs, and Aubrieta-specific semantic hooks. Performance builds should enable GPUI profiling/bench features only in dedicated configurations. Visual regression, keyboard/focus traversal, docking persistence, high-DPI and large virtualized trees remain mandatory gauntlet targets.

# Configuration/resource formats

Canonical policy:

- JSON (`serde_json`) for token/resource catalogs and schemas requiring interoperability: design tokens, themes, semantic icon maps and locale catalogs;
- TOML (`toml` + Serde) for human-authored application/module manifests and configuration.

This is intentional rather than accidental format sprawl. DTCG-compatible design tokens are JSON by specification, while TOML is reserved for configuration/manifests where comments and hand editing are first-class.

# New foundational architecture crates/services

The audit adds these canonical logical responsibilities to the stack:

- `serde_json` — JSON resource/token/catalog payloads;
- `toml` + Serde — manifests and human-edited configuration;
- `aubrieta_capabilities` — capability/contribution registries;
- `aubrieta_schema` — property/parameter/setting descriptors;
- `aubrieta_resources` / `aubrieta_resource_packs` — semantic resource IDs, pack loading/validation;
- `aubrieta_i18n` — TextId resolution + Unicode MessageFormat 2 catalog semantics + ICU4X-backed locale services where applicable;
- `aubrieta_config` — typed layered settings/workspace persistence;
- `aubrieta_jobs` — priority/cancellation/stale-result handling;
- `aubrieta_platform` — OS service boundary;
- `aubrieta_diagnostics` — structured diagnostics/tracing contracts.

Semantic IDs such as ActionId/IconId/TextId/PanelId are extensible namespaced newtypes with generated constants for built-ins, not closed enums. The canonical built-in namespace is `aubrieta.*`; third parties own their own reverse-domain/plugin namespaces.