# ADR-0003 — PTND ZIP/ZIP64 + DTO versionado

**Status:** Accepted

## Contexto

O formato nativo precisa preservar intenção autoral, resources e extensões sem ser um dump de structs Rust.

## Decisão

PTND v0.1 usa container **ZIP/ZIP64 compatível**:

~~~text
manifest.json
document.json
resources/
extensions/
previews/
~~~

O domínio é adaptado para DTO persistente versionado.

~~~text
Document
≠ DTO
≠ ZIP entries
~~~

Save escreve container temporário completo, valida e realiza replace seguro.

## Alternativas rejeitadas

**Serde direto sobre Document:** acoplaria refactor interno ao schema persistente.

**Container binário proprietário:** aumenta implementação, tooling e superfície de corrupção sem necessidade comprovada.

**Incremental-save container próprio desde v0.1:** complexidade prematura; reescrita completa basta até profiling provar o contrário.

## Consequências

- SchemaVersion independente de ApplicationVersion;
- migrations DTO→DTO;
- resources grandes fora do JSON;
- limites contra decompression bomb/path traversal;
- JSON canônico;
- BLAKE3-256 como ContentHash inicial;
- recovery separado do PTND principal.

## Referências

[Formato PTND](#/docs/00-architecture/ptnd-format.md)  
[IDs e serialização](#/docs/01-core/ids-serialization.md)
