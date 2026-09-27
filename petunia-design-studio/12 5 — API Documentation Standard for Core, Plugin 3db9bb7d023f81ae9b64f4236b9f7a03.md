# 12.5 — API Documentation Standard for Core, Plugin SDK, MCP, Schemas & Diagnostics

# Purpose

API documentation is written for three audiences simultaneously: implementers, plugin/automation users, and code agents. Schema alone is insufficient; prose alone is insufficient.

# Standard API entry

Every public method/action/property/capability documents:

1. stable identifier/name;
2. one-sentence purpose;
3. when to use / when not to use;
4. required capability/permission;
5. parameters with type/range/unit/default;
6. return/result model;
7. side effects;
8. transaction/undo semantics;
9. concurrency/job behavior;
10. persistence impact;
11. errors + recovery;
12. compatibility/since/deprecation;
13. minimal example;
14. realistic example;
15. related API/help topics.

# Beginner-first explanations

Start with intent and a minimal example. Put advanced schemas and edge conditions after the successful path. Avoid requiring readers to understand Rust internals, WIT, GPUI or document storage just to create a basic plugin/automation.

# Core architecture docs

Core internal/public crate docs should state invariants and dependency direction. Public traits explain ownership/threading, not merely method signatures.

# Plugin documentation

Provide:

- 10-minute first plugin;
- permission model explained with real examples;
- manifest reference;
- simple/advanced SDK layers;
- UI schema cookbook;
- tool lifecycle;
- background jobs;
- file/network brokers;
- testing/debugging;
- packaging/distribution;
- migration/deprecation guide.

# MCP documentation

Provide task recipes before exhaustive method listing. Each method includes copyable request/response examples, revision/transaction behavior, permission scope and error recovery.

# Errors/diagnostics catalog

Stable diagnostic codes link to documentation explaining:

- meaning;
- common causes;
- affected subsystem;
- safe recovery;
- whether retry is safe;
- data-loss risk;
- relevant logs/diagnostic bundle fields.

# Schema docs

Machine-generated schema reference should cross-link to conceptual docs. Enumerations explain semantics, not just values. Units/defaults/constraints must be explicit.

# Examples policy

Examples must not use undocumented privileged shortcuts. Prefer deterministic fixture documents and fake paths/hosts. Security-sensitive examples show permission denial/recovery as well as success.

# Compatibility markers

Document `since`, deprecated/replacement and removal policy. Persisted schemas include migration links. Plugin/MCP changes include compatibility table when versions coexist.

# Agent discoverability

Documentation headings and metadata use the same canonical API identifiers so search/retrieval can reliably map `aubrieta.action.boolean.union` or an MCP method to one reference page.

# Generated-vs-handwritten boundary

Generate signatures, parameter tables and registry catalogs when possible. Handwritten text owns rationale, workflows, pitfalls and examples. Never hand-copy hundreds of IDs that can drift from source.

# Documentation tests

At minimum validate examples, internal links, referenced IDs, schema generation and version markers. Cookbook workflows should be runnable in a test harness where feasible.

# Lua Plugin SDK documentation specialization

Because **Lua 5.5 + mlua is accepted for V1**, Plugin SDK documentation must include Lua-first copyable examples while keeping the semantic API runtime-neutral. Document:

- supported Lua version/profile and excluded unsafe standard libraries;
- canonical `aubrieta` root module/submodules;
- manifest → permission → registration lifecycle;
- value/type mapping between Lua and Aubrieta schemas;
- transaction/error idioms;
- jobs/cancellation;
- declarative UI;
- module loading policy and approved pure-Lua dependency packaging;
- memory/CPU quota failures;
- unload/hot-reload persistence rules.

Never teach authors to use `io`, `os`, native module loading, raw sockets or hidden host globals as workarounds for capability brokers.

# MCP documentation specialization

MCP reference must optimize for agent context: compact discovery examples first, schema detail on demand, stable method IDs in headings/metadata, deterministic error/recovery examples, revision/idempotency notes and complete permission/file-grant behavior. Recipes should prove that common workflows do not require GUI pixel automation.