<!-- prumo:begin agents-core -->
<!-- prumo:generated adapter=agents-md scope=root fingerprint=55d9c3d9d60a4f24 -->
- Domain crates (document, geometry, color, text, raster, commands) never import GUI toolkit types (GPUI, Floem, Slint, Qt, WebView). UI receives DTOs/view-models and sends ActionRequest/CommandRequest across AubrietaGuiBridge.
- Author canonical docs in en-US with a synchronized pt-BR mirror before release. Update the Atlas, ADR registry and generated references in the same change that alters a contract.
- A feature is done only with focused tests plus the applicable gauntlets and recorded evidence (headless-first, detach proof, token/a11y checks, EN+pt-BR docs).
- Build a small authoritative microcontext (goal, scope, exact page/ADR IDs, crates, capabilities, invariants) instead of dumping large context. Never put credentials or artwork in packets.
- Compose tools, panels, effects, importers and data sources through capability registries. A missing capability is a normal state with a disabled reason, never a panic; no lateral feature-to-feature storage access.
- Every mutation flows UI/Shortcut/Plugin/MCP -> Action -> Command -> DocumentMutator -> ChangeSet. Nothing touches document storage directly.
- Default to non-destructive editing via typed ordered EffectChain and live modifiers. Bake, Expand, Rasterize and Convert-to-Curves are explicit user operations only.
- Use Prumo CLI to shrink context and track Goals/Waves/evidence, never to outsource decisions. On any Prumo-vs-Aubrieta conflict, Aubrieta ADRs and atlases win; record the conflict.
- Every feature carries an explicit scope status (V1 Required, Milestone Required, Post-V1 Candidate, Research, Open ADR, Historical, Out of Scope). A bare future/later/planned authorizes nothing.
- Use typed stable IDs (ObjectId, SurfaceId, ResourceId, StyleId, EffectId). A Vec index, pointer or handle is never identity.
<!-- prumo:end agents-core -->
