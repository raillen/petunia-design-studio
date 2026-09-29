---
layout: home
title: Petunia Design Studio
titleTemplate: Design vetorial + fotográfico, não destrutivo por padrão
hero:
  name: Petunia Design Studio
  text: Desenhe vetor. Retoque foto. Não quebre nada.
  tagline: Um estúdio de design desktop com núcleo não destrutivo, formato aberto .ptnd e APIs voltadas a automação (CLI, MCP, plugins).
  image:
    src: /logo.svg
    alt: Logotipo do Petunia Design Studio
  actions:
    - theme: brand
      text: Comece em 15 min
      link: /pt/getting-started/quickstart
    - theme: alt
      text: Ler o manual
      link: /pt/manual/
    - theme: alt
      text: Catálogo de ferramentas
      link: /pt/tools/
features:
  - icon: 🖋️
    title: Persona Design
    details: Caneta, nós, formas, booleanos, preenchimentos, gradientes, tipografia e pranchetas sobre uma única árvore de documento.
    link: /pt/tools/
    linkText: Ver ferramentas
  - icon: 📷
    title: Persona Photo
    details: Seleções, pincéis, ajustes e máscaras como nós reordenáveis e não destrutivos — os pixels originais nunca são tocados.
    link: /pt/tools/
    linkText: Ver ferramentas
  - icon: 🧬
    title: Núcleo não destrutivo
    details: Toda mutação flui Ação → Comando → DocumentMutator → ChangeSet. Operações destrutivas (Bake, Expandir, Rasterizar) são explícitas.
    link: /pt/developers/architecture
    linkText: Como funciona
  - icon: 🤖
    title: Automação primeiro
    details: CLI headless, servidor MCP JSON-RPC, sandbox Lua e registros de capacidades — sem UI falsa, nunca.
    link: /pt/developers/
    linkText: Docs de desenvolvimento
  - icon: 📦
    title: Formato .ptnd aberto
    details: Pacote ZIP atômico com exportação SVG/PDF/PNG, verificações de pré-voo e prova de cor CMYK para impressão.
    link: /pt/manual/
    linkText: Aprender o fluxo
  - icon: 🌍
    title: Bilíngue por construção
    details: O inglês é canônico; o pt-BR espelha cada página 1:1. Novos idiomas entram pelo mesmo padrão de locale.
    link: /pt/contributing/translations
    linkText: Traduzir
---

## A jornada

Um caminho da primeira instalação ao estúdio automatizado:

```mermaid
flowchart LR
  A[Descobrir] --> B[Instalar]
  B --> C[Criar documento]
  C --> D[Desenhar e compor]
  D --> E[Conferir e exportar]
  E --> F[Automatizar e estender]
```

| Etapa | Você faz | Comece aqui |
| ----- | -------- | ----------- |
| Descobrir | Entende o que o Petunia é — e o que não é | [Constituição](/pt/bible/) |
| Instalar | Compila o workspace e abre o app ou a CLI | [Instalação](/pt/getting-started/installation) |
| Criar documento | Abre sua primeira superfície `.ptnd` | [Início rápido](/pt/getting-started/quickstart) |
| Desenhar e compor | Domina ferramentas, painéis e personas | [Manual](/pt/manual/) · [Ferramentas](/pt/tools/) |
| Conferir e exportar | Pré-voo, prova de cor e exportação SVG/PDF/PNG | [Manual](/pt/manual/) |
| Automatizar e estender | Controla o Petunia via scripts, agentes e plugins | [Desenvolvedores](/pt/developers/) |

## Vocabulário canônico

O Petunia usa nomes precisos: **Superfície** hospeda a composição, **Camada** é um papel na árvore única de documento (não uma hierarquia paralela), **Vínculo** liga uma propriedade a um dado variável. Termos novos nunca são inventados em silêncio — veja o [Glossário](/pt/glossary) (EN ↔ PT-BR) e a [SPEC-001](/pt/bible/SPEC-001).
