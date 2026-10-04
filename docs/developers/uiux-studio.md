# Studio — UI/UX audit and implementation

Date: 2026-10-04. Scope: **Milestone Required (MVP)**, shared foundation for V1. [ADR-012](/developers/adr/ADR-012-studio-workspace) defines the contract; [evidence](/implementation/uiux-studio.json) records actual execution.

The redesign uses Affinity as a reference for workspace structure and hierarchy while keeping Petunia controls and identity. The audit and corrections cover the shell, tools, documents, panels, colour, navigation, forms, preferences and operation feedback.

| Observed problem | Implemented behaviour |
| --- | --- |
| Commands, appearance and context were mixed | Menu, Studio, document, context and status regions have distinct roles |
| Colour and layers competed for one tab | Colour/Swatches remain independent of the lower inspector |
| Two-column tools became one horizontal row | Actual scrollable grid and Pixel painting tools |
| Auxiliary panels overflowed the canvas and reopening splitters failed | Bounded width/height, stable hooks, window drag capture and keyboard resizing |
| Preview callbacks retained the first frame | New worker frames, camera and overlays republish through a view-local epoch |
| Long tabs/controls exceeded the window | Scrolling, ellipsis, bounded widths and collapse |
| Layers lacked collapse and clear identities | Session/object tree, semantic icons and independent actions |
| Swatches overwrote widths and ignored raster | Atomic vector multiselection; raster brush colour; retained alpha/width |
| Favourites added a fixed pink | Add and deduplicate the current colour; retain app-session favourites |
| Navigator showed boxes instead of composition | Actual worker, camera footprint and click/drag/arrow navigation |
| New Document only offered incremental changes | Typed dimensions, presets, orientation and errors before creation |
| Preferences/tasks described fictitious states | Actual renderer descriptions and real jobs |
| Filled icons changed meanings | Equivalent outline fallback for missing semantic filled glyphs |
| Icons lacked identity and inputs stole shortcuts | Accessible labels/states/focus and keyboard ownership |
| Hard-coded Portuguese copy | Live EN/pt-BR catalogue |

## Final verification

Checks ran after this increment was implemented; failures found during final verification were corrected before the recorded successful runs. `cargo xtask gauntlet` passed 951 workspace tests across 104 targets, with zero failures or ignored tests, strict Clippy, formatting, architecture checks, CLI conformance, four MVP projects and the documentation build. `cargo xtask ui-gauntlet` passed all 84 desktop tests, already included in the workspace total. The documentation has 35 canonical pages and 35 pt-BR mirrors. Independent pypdf/Poppler inspection preserved the original CMYK ink, alpha and ICC in all four PDF fixtures.

Captures are actual production Rust/Freya renders with original fixture content, not HTML recreations. Eighteen Studio captures cover both locales at four sizes and focused interactions; nine additional captures exercise existing desktop modules. The execution ledger records commands, source/artifact hashes and boundaries. ADR-011 baseline counts are historical evidence.

Binary captures and PDF/TIFF fixtures are local execution artifacts. Their hashes and dimensions are recorded in the ledger; the repository contains implementation sources and textual verification reports. Reproduce the artifacts after installing the documented native dependencies:

```sh
PETUNIA_UI_EVIDENCE_DIR="$PWD/docs/public/implementation/studio/artifacts" \
PETUNIA_CMYK_EVIDENCE_DIR="$PWD/docs/public/implementation/studio/cmyk-artifacts" \
cargo xtask gauntlet
python3 scripts/verify-native-cmyk-pdf.py docs/public/implementation/studio/cmyk-artifacts
```

## Acceptance limits

Initial fit occurs once per session. Favourites belong to the app session. Exact numeric transformation still requires one selected object. Several legacy properties still edit only the first object. Free docking, preference persistence, light/system themes, certified spot libraries and a Layout Studio are outside this implementation contract. Professional print/typography and external hardware, accessibility and usability acceptance remain in the [roadmap](/developers/implementation-roadmap-2026-09-30).
