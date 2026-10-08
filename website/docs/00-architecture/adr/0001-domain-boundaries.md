# ADR-0001 — Fronteiras Core/Engine/Render/UI

**Status:** Accepted

## Contexto

O Petunia precisa crescer como editor vetorial, raster e editorial sem permitir que UI, renderer ou bibliotecas externas se tornem a autoridade do documento.

## Decisão

Usar quatro domínios autoritativos:

~~~text
Core   → o que o documento é
Engine → como calcular/avaliar
Render → como transformar estado avaliado em pixels
UI     → interação e apresentação
~~~

Qt/QML via CXX-Qt existe somente em `petunia-ui`.

`petunia-render-model` é uma crate interna de contrato imutável entre Engine e Render.

## Alternativas rejeitadas

**UI-centric document:** rejeitada porque widgets/QObject passariam a controlar semântica de documento e impediriam headless/testing.

**Engine ↔ Render dependency direta:** rejeitada porque cria ciclo conceitual e acoplamento de backend.

**Tipos de crates externas como domínio:** rejeitada porque PTND/API ficariam presos ao versionamento de dependências.

## Consequências

- Core não depende de Qt/Render/Engine.
- Engine não depende de UI/Render.
- Render não depende de UI/Engine.
- tipos externos passam por adapters;
- features headless funcionam sem Qt;
- UI envia intenção, não muta structs do Core diretamente.

## Referências

[Fronteiras e invariantes](#/docs/00-architecture/boundaries.md)
