# ADR-0010 — Plugins WASM + Host API

**Status:** Accepted

## Contexto

Plugins precisam estender Engine sem receber memória interna, ABI Rust instável, Qt ou authority bypass.

## Decisão

Formato público inicial:

~~~text
WebAssembly
+
versioned Host API
+
typed DTOs / opaque handles
+
deny-by-default capabilities
~~~

Plugins leem via Query/Snapshot e mutam via Command.

Plugin nativo arbitrário não roda in-process na v0.1. Integração nativa futura, se necessária, usa processo separado + IPC.

## Alternativas rejeitadas

**Rust dynamic-library ABI:** layouts/trait objects Rust não são ABI pública estável.

**C ABI in-process como caminho principal:** aumenta risco de crash/memory corruption e permission bypass.

**Plugin com &mut Document:** quebra Transactions, History e invariantes.

## Consequências

- sandbox e resource limits;
- filesystem/network/process são capabilities;
- effect plugin declara ROI/color/determinism/cancellation;
- runtime WASM concreto é escolhido por benchmark sem mudar Host API;
- UI extension fica para discussão de GUI/UX;
- MCP usa a mesma Query/Command boundary.

## Referências

[I/O, Jobs e Plugins](#/docs/02-engine/io-jobs-plugins.md)  
[Modelo de segurança](#/docs/00-architecture/security-model.md)
