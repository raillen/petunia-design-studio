# ADR-0006 — Single writer + immutable snapshots

**Status:** Accepted

## Contexto

Render, jobs, export, trace e geometry precisam usar múltiplos cores sem espalhar locks no Document ou permitir mutação concorrente imprevisível.

## Decisão

O Document possui **single writer**. Leitores concorrentes recebem snapshots imutáveis associados a uma `DocumentRevision`.

~~~text
Document owner
↓ publish
Immutable Snapshot
├→ Render
├→ Engine query
└→ Jobs
~~~

Workers retornam resultados; mutações voltam pela Transaction API.

## Alternativas rejeitadas

**Arc<Mutex<Document>> global:** aumenta contention, deadlock risk e dificuldade de raciocínio sobre History/Revision.

**Document mutável em Rayon workers:** quebra atomicidade.

**Snapshot via serialize/deserialize:** mistura persistência com runtime e custa demais.

## Consequências

- jobs usam expected revision;
- v0.1 usa revision global para stale-result safety;
- structural sharing/COW entram onde medidos;
- Qt thread affinity fica na UI;
- Rayon é data-parallel mechanism, não Job API;
- Interactive tem prioridade sobre Background/Batch.

## Referências

[Fronteiras e invariantes](#/docs/00-architecture/boundaries.md)
