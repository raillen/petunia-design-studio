# Architecture Decision Records

**ADR — Architecture Decision Record** registra uma decisão estrutural que seria cara ou arriscada de redescobrir apenas lendo código.

Estados usados:

~~~text
Accepted
Superseded
Deprecated
~~~

Um ADR não substitui a documentação técnica detalhada. Ele explica **por que** uma direção foi escolhida, quais alternativas foram rejeitadas e quais consequências passam a ser obrigatórias.

## Decisões aceitas

1. [ADR-0001 — Fronteiras Core/Engine/Render/UI](#/docs/00-architecture/adr/0001-domain-boundaries.md)
2. [ADR-0002 — Path canônico Line + Cubic](#/docs/00-architecture/adr/0002-canonical-path.md)
3. [ADR-0003 — PTND ZIP/ZIP64 + DTO versionado](#/docs/00-architecture/adr/0003-ptnd-container.md)
4. [ADR-0004 — SceneGraph autoral ordenado](#/docs/00-architecture/adr/0004-scene-graph.md)
5. [ADR-0005 — Avaliação não destrutiva](#/docs/00-architecture/adr/0005-nondestructive-evaluation.md)
6. [ADR-0006 — Single writer + immutable snapshots](#/docs/00-architecture/adr/0006-concurrency.md)
7. [ADR-0007 — Render Model + software reference renderer](#/docs/00-architecture/adr/0007-render-model-software-renderer.md)
8. [ADR-0008 — Little CMS 2 como CMM](#/docs/00-architecture/adr/0008-color-management.md)
9. [ADR-0009 — Raster autoral tiled + copy-on-write](#/docs/00-architecture/adr/0009-raster-tiles.md)
10. [ADR-0010 — Plugins WASM + Host API](#/docs/00-architecture/adr/0010-plugin-abi.md)
11. [ADR-0011 — Select + Vector Edit híbridos contextuais](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md)
12. [ADR-0012 — Contrato de interação: Tools, Workspace e Acessibilidade](#/docs/00-architecture/adr/0012-interaction-contract.md)

## Regra

Nova decisão estrutural recebe ADR quando mudar representação persistente, fronteira entre crates, semântica de documento, backend de referência, ABI/Host API pública, threading/concurrency, formato nativo ou dependência que atravesse domínios.

O ADR novo pode substituir outro, mas nunca apagar silenciosamente o histórico da decisão.
