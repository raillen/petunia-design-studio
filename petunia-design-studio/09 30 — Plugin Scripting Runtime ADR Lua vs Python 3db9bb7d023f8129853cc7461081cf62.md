# 09.30 — Plugin Scripting Runtime ADR: Lua vs Python vs JavaScript

<aside>
✅

**Status: ACCEPTED for V1.** **Lua 5.5 via `mlua` is Aubrieta's primary approachable scripting runtime.** The semantic Plugin SDK remains runtime-neutral; Wasmtime/WASI remains the preferred high-isolation component tier. Python and JavaScript remain first-class external automation languages through MCP/client SDKs and may only become additional embedded runtimes through a new ADR.

</aside>

# Decision question

Which scripting runtime best satisfies Aubrieta's Plugin SDK requirements: beginner/code-agent usability, Rust integration, cross-platform packaging, small startup/memory cost, deterministic host control, safe capability mediation, interruptibility, hot reload and long-term maintenance?

The semantic API is already runtime-neutral in 09.28. This ADR chooses a binding/runtime, **not a new plugin architecture**.

# Candidates

1. **Lua 5.5 + `mlua`** — **ACCEPTED_V1 primary embedded scripting runtime**.
2. **Python + CPython/PyO3** — strongest ecosystem/familiarity, weakest default fit for secure embedded plugins.
3. **JavaScript + QuickJS/rquickjs** — strong familiarity and host controls, but additional language/runtime and C/platform complexity.
4. **JavaScript + Boa** — Rust-first alternative worth tracking, but currently described by its project as experimental.

# Current ecosystem snapshot — 2026-09-14

## Lua / mlua

- Lua's current official release is **5.5.1** and Lua is explicitly designed as a lightweight embeddable extension language.
- `mlua` 0.12.x supports Lua 5.5/5.4/5.3/5.2/5.1, LuaJIT and Luau, plus Serde and async integration.
- `Lua::new()` loads a safe subset and does not allow unsafe standard libraries or C modules; `new_with` can select allowed libraries.
- `set_memory_limit` provides per-state allocation limits.
- instruction hooks can terminate execution after a controlled instruction budget; Luau also exposes interrupt support.
- vendored builds can compile the chosen Lua runtime with Aubrieta.

References: [Lua 5.5 manual](https://www.lua.org/manual/5.5/manual.html), [Lua releases](https://www.lua.org/download.html), [mlua docs](https://docs.rs/mlua/latest/mlua/).

## Python / PyO3

- PyO3 0.29.x supports running Python from Rust and embedding CPython.
- Dynamic embedding is easier but desktop distribution must include/find the Python shared library; PyO3 documents static embedding as substantially more complicated and not generally first-class on Windows.
- CPython isolated configuration can ignore environment variables, command-line configuration and user-site packages, useful for deterministic embedding.
- **Isolated configuration is not a security sandbox.** CPython explicitly states audit hooks are not suitable for implementing a sandbox, and modules such as `ctypes` require removal/strict control for security-sensitive hosts.
- Native extension modules and multiple interpreters introduce additional state/isolation concerns.

References: [PyO3 embedding/distribution](https://pyo3.rs/main/building-and-distribution), [PyO3 docs](https://docs.rs/pyo3/latest/pyo3/), [CPython initialization config](https://docs.python.org/3.14/c-api/init_config.html), [CPython audit hooks](https://docs.python.org/3.14/library/sys.html).

## JavaScript / QuickJS / rquickjs

- QuickJS is deliberately small and embeddable with very low startup overhead.
- QuickJS exposes memory limit, maximum stack size and execution interrupt-handler controls.
- `rquickjs` 0.13.x provides high-level Rust bindings, Rust Future/Promise integration, custom module loaders/resolvers and allocators.
- `rquickjs` does not automatically provide Node/Web APIs, which is positive for capability-based hosting because Aubrieta can expose only approved modules.
- QuickJS/rquickjs compiles a C engine and has platform/binding caveats; current rquickjs documentation marks some Windows MSVC/architecture combinations experimental or less-tested.

References: [QuickJS](https://bellard.org/quickjs/quickjs.html), [rquickjs](https://docs.rs/rquickjs/latest/rquickjs/).

## JavaScript / Boa

Boa is an embeddable JavaScript engine written in Rust and useful prior art for a future Rust-first JS option. Its own project currently calls the engine experimental while reporting more than 90% support of the latest ECMAScript specification. Reference: [Boa](https://github.com/boa-dev/boa).

# Evaluation criteria

Priority order for Aubrieta:

1. **Host security/control** — deny ambient capabilities, bound resources, terminate runaway code.
2. **Rust embedding/maintenance** — clean typed bridge and manageable dependency/toolchain.
3. **Desktop distribution** — Windows/Linux/macOS without external runtime setup.
4. **Plugin-author simplicity** — approachable syntax, small conceptual API, good errors.
5. **Code-agent reliability** — easy to generate, validate and document without framework hallucination.
6. **Startup/memory footprint** — many installed plugins should not make the suite heavy.
7. **Async/background-job fit** — integrate without blocking UI/document mutation lane.
8. **Hot reload/unload** — predictable lifecycle and state separation.
9. **Ecosystem** — useful libraries where they do not defeat sandbox/security.
10. **Future compatibility** — no lock-in of domain semantics to the runtime.

# Comparative matrix

| Criterion | Lua + mlua | Python + PyO3 | JavaScript + rquickjs |
| --- | --- | --- | --- |
| Embedding footprint | **Excellent** — Lua is intentionally small/embedded; vendored build straightforward | Weak–medium — CPython runtime/distribution is much larger and more involved | **Very good** — QuickJS is small, but brings C/bindings build layer |
| Safe-by-default host surface | **Excellent for in-process scripting** — safe stdlib subset, selective libraries, no C modules from safe constructor | Weak — powerful stdlib/import ecosystem requires aggressive removal/brokering; isolated config is not sandbox | Very good — no Node/Web APIs by default; custom module loader allows narrow host surface |
| Memory limit | **Direct per-state limit** through mlua | No equivalently simple complete per-plugin CPython heap quota; process/runtime strategies are more complex | **Direct runtime memory limit** |
| Execution interruption | **Instruction hooks / interrupt mechanisms** | Possible, but reliable hostile-code termination inside a shared process is substantially harder | **Runtime interrupt handler** |
| Rust data bridge | **Excellent** — UserData + Serde + async | Excellent API quality through PyO3, but Python object/runtime lifetime is heavier | Very good — Rust↔JS conversion/classes + async |
| Beginner familiarity | Good — very small language, but less universally taught | **Excellent** | **Excellent** |
| Code-agent familiarity | Very good; smaller API can reduce ambiguity | **Excellent** | **Excellent** |
| Language complexity | **Low** | Medium | High relative to Lua: modules, promises and wider language/tooling expectations |
| General ecosystem | Medium | **Excellent** | **Excellent** |
| Useful ecosystem under Aubrieta sandbox | Good if pure-Lua bundled modules only | Low–medium if native wheels, ctypes, subprocess/network/filesystem are restricted | Medium–good if Node/npm ambient APIs are not provided and dependencies are bundled/audited |
| Cross-platform product packaging | **Excellent** | Medium–weak due embedded CPython/runtime/native-extension distribution | Good, with QuickJS C/binding platform validation required |
| Best Aubrieta role | **Primary in-process V1 scripting** | External/MCP automation or isolated worker later | Strong alternative/possible future secondary runtime |

# Why Lua/mlua was selected for V1

The decisive point is **not raw language speed**. Aubrieta wants a deliberately small extension language whose host controls are stronger than its ambient standard library.

Lua/mlua gives an unusually direct combination:

- safe interpreter constructor;
- explicit safe/unsafe stdlib split;
- no need to ship a large separate runtime;
- per-VM memory accounting/limit;
- instruction interruption;
- coroutines/async bridge;
- Serde conversion;
- one lightweight VM per plugin is practical;
- small language surface is easier to document exhaustively for humans and LLMs.

That combination reduces the amount of custom sandbox infrastructure Aubrieta must invent.

# Lua-specific disadvantages

- smaller general-purpose package ecosystem than Python/npm;
- fewer users already know Lua professionally;
- tables-as-primary-data-structure, `nil`, metatables and 1-based array conventions can surprise authors;
- many third-party Lua modules expect native C loading or unrestricted `io`/`os`; those cannot be automatically trusted/loaded;
- in-process Lua is still **not equivalent to process/WASM isolation**: a bug in an exposed Rust host function can still compromise the host.

# Accepted Lua V1 profile

Canonical V1 runtime contract:

- **Language/runtime:** Lua 5.5 through a pinned reviewed `mlua` 0.12.x-or-later compatible release.
- **Packaging:** vendored Lua runtime; no external Lua installation required.
- **Isolation unit:** one Lua state per plugin by default.
- **Standard libraries:** safe subset only; explicitly exclude unrestricted `io`, `os`, native package loading, `debug`, FFI/JIT-native escape surfaces where unsafe.
- **Modules:** host-controlled `require`; bundled pure-Lua modules and Aubrieta SDK modules only unless explicitly trusted/approved.
- **Host root module:** `aubrieta` with simple/advanced submodules defined by 09.28.
- **Memory:** per-plugin memory limit + telemetry.
- **CPU:** instruction/time budget via hook/watchdog; background work uses Aubrieta Job service.
- **Network/files:** only brokered capabilities; never ambient Lua APIs.
- **Storage:** namespaced Aubrieta plugin storage API.
- **UI:** declarative schema only.
- **Mutation:** canonical transaction/Command API only.
- **Hot reload:** destroy/recreate plugin VM; durable state lives in host-owned plugin storage, not VM globals.
- **Native Lua modules:** disabled for ordinary plugins.

# Python — recommended role instead of primary embedded plugins

Python is exceptionally valuable around Aubrieta, just not necessarily inside the same process as the default untrusted plugin runtime.

Recommended future uses:

- external automation through MCP;
- official Python client SDK for MCP;
- data-science/batch/image workflows in a separate process;
- trusted developer tooling;
- optional isolated worker architecture if strong demand emerges.

This preserves the Python ecosystem without pretending CPython's isolated embedding mode is a sandbox.

# JavaScript — strongest alternative

If user familiarity and web/JS ecosystem become more important than minimal runtime surface, prototype QuickJS/rquickjs next.

Safe profile would require:

- one runtime per plugin;
- memory + stack limits;
- interrupt handler/time budget;
- custom built-in-only module resolver;
- no arbitrary-disk `FileResolver`;
- no dynamic/native module loading;
- no Node APIs/npm install at runtime;
- filesystem/network only through Aubrieta brokers;
- Promises integrated through Aubrieta jobs rather than hidden background authority.

TypeScript could be a future authoring layer compiled to JS, but adding a TypeScript toolchain is a separate ADR and must not become required for simple plugins.

# Why ecosystem size alone does not decide

A plugin sandbox deliberately blocks much of what makes Python/npm ecosystems powerful: arbitrary files, processes, sockets and native libraries. Headline package counts therefore overstate the useful safe advantage. Aubrieta should expose high-level domain APIs so most plugins need little third-party code.

# Mandatory runtime conformance gauntlet

Implement the **same semantic plugin** in all three candidates:

1. register one Action;
2. query selection and edit properties transactionally;
3. create a declarative panel;
4. implement a simple canvas Tool lifecycle;
5. run a cancelable background job;
6. bundle one pure-language dependency;
7. deny unauthorized file/network access;
8. terminate an infinite loop;
9. terminate/contain a memory-growth script;
10. trigger exception during transaction and verify rollback;
11. hot reload/unload and verify no contributions/jobs remain;
12. load 50–100 trivial plugins and measure startup/RAM;
13. package on Windows x64, Linux x64 and macOS ARM64/x64 targets as applicable;
14. have code agents implement the plugin using only published English/pt-BR SDK docs, then score correction rate and hallucinated APIs.

# Measurements

Record cold runtime creation time, incremental memory per idle plugin, 10k API calls individually vs batched, transaction throughput, panel update overhead, cancellation latency, infinite-loop termination latency, memory-quota accuracy, unload/reload leaks, binary/package size impact, CI/cross-compilation friction, documentation complexity and novice/code-agent completion rate.

# Conformance threshold and revisit triggers

The accepted Lua/mlua architecture must still pass Windows/Linux/macOS packaging, reliable CPU/memory interruption, async/jobs integration, deterministic unload and API ergonomics before the plugin subsystem can be considered release-ready. Failure is an implementation blocker, not permission to silently switch runtimes.

A new runtime ADR may revisit JavaScript only if measured plugin-author experience/ecosystem value materially exceeds Lua while preserving equivalent capability isolation and packaging simplicity. Python may only be reconsidered as an in-process default if a credible isolation/distribution design solves the documented security/runtime constraints without turning Aubrieta into a Python-distribution manager.

# Accepted decision

**Aubrieta V1 uses Lua 5.5 + `mlua` as the primary scripting plugin runtime.** Preserve Wasmtime/WASI as the high-isolation component tier and expose Python/JavaScript through MCP client SDKs even though they are not V1 embedded plugin languages. The comparative prototype remains valuable as regression/evidence and future ADR input, but no longer keeps the architecture undecided.