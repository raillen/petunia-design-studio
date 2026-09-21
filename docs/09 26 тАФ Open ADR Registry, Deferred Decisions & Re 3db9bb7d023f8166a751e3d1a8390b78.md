# 09.26 — Open ADR Registry, Deferred Decisions & Revisit Triggers

<aside>
✅

**Current state:** the September 2026 architecture audit has **no blocking V1 scripting-runtime ADR left open**. The primary scripting runtime decision is accepted: **Lua 5.5 + `mlua`**. Wasmtime/WASI remains the optional high-isolation component tier; Python and JavaScript remain external MCP/client-SDK languages and future ADR candidates only. Any genuinely unresolved item must be added explicitly below with safe behavior and revisit trigger before implementation may depend on it.

</aside>

# Resolved decision — plugin scripting runtime

**ADR:** [09.30 — Plugin Scripting Runtime ADR: Lua vs Python vs JavaScript](09%2030%20%E2%80%94%20Plugin%20Scripting%20Runtime%20ADR%20Lua%20vs%20Python%203db9bb7d023f8129853cc7461081cf62.md)

**Status:** `ACCEPTED_V1`.

**Decision:** Lua 5.5 + `mlua` is the primary approachable embedded scripting runtime. The semantic Plugin SDK remains runtime-neutral. Wasmtime/WASI is the optional high-isolation component tier. Python/JavaScript are external automation/client-SDK languages unless a new ADR explicitly adds another embedded runtime.

**Release-readiness evidence still required:** runtime packaging, CPU/memory quota enforcement, deterministic unload, permission denial, transaction rollback, 50–100 plugin stress and code-agent documentation conformance remain implementation gates. Failing a gate blocks release of the plugin subsystem; it does not silently reopen the runtime choice.

**Revisit trigger:** only a new measured structural/security/packaging blocker in Lua/mlua, or a future product requirement that clearly justifies another embedded runtime, may open a superseding ADR.

# Resolved decision history

| Decision | Resolved direction | Future revisit only if… |
| --- | --- | --- |
| Native canonical document payload encoding | Canonical V1 structure is schema-versioned **JSON** inside the canonical `.aubrieta` ZIP-compatible package; `.aubri` is a filename alias for the same package/schema. Large binary resources remain separate. Optional binary acceleration caches are noncanonical. | Measured JSON parsing/size becomes a demonstrated product bottleneck; canonical openness remains mandatory. |
| Design/Photo product name + native extension | **RESOLVED:** **Aubrieta Design**. Canonical suffix `.aubrieta`; accepted short alias `.aubri` for the exact same package/schema. `.abrt` rejected because ABRT is already an established Fedora/RHEL software name; `.petunia` remains reserved for Petunia3D. | Revisit only if pre-release legal/trademark clearance or a newly discovered major file-association collision requires a change. |
| moxcms as sole production CMM | `moxcms` is the primary/default Rust-native CMM. LittleCMS stays behind `ColorManagementProvider` as differential oracle and compatibility fallback. | The ICC corpus proves moxcms can safely become the sole production backend. |
| Raster tile size/storage compression | V1 logical CPU tile default **128×128 px**; GPU atlas geometry is independent. Hot RAM uncompressed; cold/swap low-latency compression; persisted tile chunks use versioned Zstd-style compression. | Representative Photo benchmarks show a materially better tile/codec policy. |
| Rich plugin UI | Native plugin UI is **declarative/semantic** and rendered by Aubrieta controls. Arbitrary external GUI injection is forbidden. A future sandboxed embedded/WebView tier may serve genuinely complex plugin UI. | Real plugin use cases cannot be expressed ergonomically by declarative schemas. |
| Desktop GUI Primary & Auxiliary Stacks | **RESOLVED:** **Slint** é a shell primária e única de produção (`apps/aubrieta-slint`). **Egui foi aposentado** (`apps/aubrieta-egui` removido do workspace). **Iced foi aposentado** (`apps/aubrieta-iced` removido do workspace). **GPUI, Floem e Xilem foram descontinuados**. O legado `aubrieta_ui_gpui` foi renomeado para `aubrieta_shell` (neutra, sem toolkit); sessão, portas e view-models vivem em `aubrieta_application`. O core permanece 100% GUI-agnóstico via `AubrietaGuiBridge`. | Impasse estrutural de licenciamento com Slint reabre avaliação de shells alternativas. |
| Locale message expression syntax | Adopt **Unicode MessageFormat 2 (MF2)** as the canonical dynamic-message syntax/semantics, behind an Aubrieta localization facade; ICU4X provides locale services where applicable. | Rust runtime implementation changes; catalog syntax should remain stable. |
| Advanced appearance with multiple fills/strokes | Canonical document model supports an ordered **Appearance Stack from V1**, including multiple fills/strokes/effects. UI may progressively disclose advanced entries. | No architectural revisit expected; only UX/implementation refinement. |
| Master-like reusable layout elements | No Publisher-style Master Pages. Use **Symbols + lightweight SurfaceTemplate/reference composition** for repeating page/surface content and constrained overrides. | Real multi-page workflows prove the lightweight model insufficient. |
| PDF/X native export | Explicitly outside V1 native scope. Produce professional color-managed PDF + preflight; allow external PDF/X conversion/validation. | Ordinary PDF/CMYK/ICC export is production-stable and native PDF/X provides clear user value. |

# Native-format interoperability decision

The **Aubrieta Design native format** is an open ZIP-compatible package with public schemas, canonical JSON authoring state, binary resources and derived PDF/SVG compatibility projections. `.aubrieta` is canonical and `.aubri` is a compatible filename alias; both identify the same internal format. `.petunia` is reserved for Petunia3D. See **09.10.1 — Open Native Format, Interoperability Profile & Compatibility Representations**.

# Rule for future open ADRs

An unresolved item must state: current safe behavior, alternatives/constraint space, what can proceed without the decision, evidence required, and a measurable revisit trigger. `TODO choose later` is not an architectural state.

# Closing future items

Once accepted, update the relevant canonical architecture/functionality pages and add the decision to this history. Never silently delete context.