# 09.18 — Security Model, Trust Boundaries, Hostile Files & Safe Extension Rules

# Trust model

Creative documents, imported media, SVG/PDF/images/fonts/ICC profiles, resource packs, clipboard, data-merge sources and plugins can all be attacker-controlled. Parsing is never considered trusted just because the file was opened locally.

# Boundaries

- parsers/importers: untrusted bytes → validated intermediate/domain data;
- plugin runtime: sandboxed component + explicit capability broker;
- resource packs: data-only, no code execution;
- MCP: authenticated/policy-controlled external actor;
- linked assets/network adapters: external mutable resources.

# Resource limits

Bound file size, decompressed size, archive entry count/path depth, image dimensions/pixels, SVG complexity, font tables, ICC profile size, recursion depth, object counts and allocation multiplication. Fail with structured resource-limit diagnostic.

# Path safety

Archive/resource paths cannot escape pack/document root (`..`, absolute paths, symlink tricks). Export filename templates are sanitized per target OS.

# SVG/assets

Disable/strip scripts, external active content and remote resolution by default. Custom icon SVG subset is stricter than imported document SVG.

# Memory safety

Safe Rust default. `unsafe` isolated in reviewed adapter modules with safety contracts/tests. FFI only behind safe wrappers.

# Supply chain

Pin/review high-risk native/parser dependencies, cargo-audit/deny policy, SBOM, license checks and dependency provenance in release pipeline.

# Secrets/privacy

No telemetry/document upload by default. Diagnostics redact paths/content according to policy. Plugins get no credentials/environment secrets implicitly.

# Fuzzing

Prioritize native loader, resource-pack parser, SVG/image/PDF/font/ICC adapters, data-merge CSV/JSON and plugin manifests/component boundaries.

# Incident behavior

Malformed input cannot corrupt currently open document. Import is staged; save never overwrites only good source after validation failure.

# Plugin and automation security extension

The accepted **Lua 5.5 + mlua V1 scripting tier** must not weaken the host security model. Python/JavaScript remain external automation/client-SDK or future research options and would be required to satisfy the same rules before any embedded adoption:

- no ambient filesystem, network, environment or process access;
- Lua `io`/`os`/unsafe debug/native C module loading unavailable to ordinary plugins;
- native/package-extension loading disabled unless explicitly brokered and trusted;
- capability grants are host-owned and revocable;
- host validates every document mutation through canonical Commands;
- execution can be interrupted/terminated under memory/instruction/time quotas;
- plugin exceptions/timeouts roll back active document transactions;
- plugin-owned storage is namespaced and size-bounded;
- permission escalation after plugin update requires explicit renewed consent;
- untrusted marketplace code may require the stronger WASM/Wasmtime or process-isolated tier even though ordinary local Lua scripts run in-process.

# MCP safety extension

MCP safety is semantic, not merely transport authentication. Separate read/write/file/network/clipboard/admin/developer permissions, enforce revision-safe transactions and prefer brokered file grants over arbitrary host paths. `localhost` is not synonymous with trusted omnipotent access.

# Security documentation requirement

Plugin/MCP APIs must document permission requirements, side effects, retry/idempotency behavior, quotas, sensitive-data handling and recovery paths. Code examples must never normalize unsafe shortcuts such as unrestricted filesystem access.