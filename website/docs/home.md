# Petunia Design

Documentação técnica do **Petunia Design Studio**: um editor open source de imagem, vetor e layout pensado para ser modular, não destrutivo e previsível de evoluir.

Esta documentação descreve **as decisões antes da implementação** — tipos de dados, fronteiras entre módulos, invariantes, algoritmos, fluxo de edição e responsabilidades de cada camada.

## Como a arquitetura é dividida

| Domínio | Responsabilidade |
|---|---|
| **Núcleo** | Documento, scene graph, paths, cores, texto, recursos e dados persistentes. |
| **Engine** | Geometria, snapping, brushes, raster, layout, color management, commands e I/O. |
| **Render** | Rasterização, composição, efeitos, caches, overlays e saída para tela/export. |
| **Interface** | Workspace, ferramentas, input, painéis, atalhos, acessibilidade e integração egui. |

> A regra central é simples: **dados autorais ficam no Núcleo; cálculos ficam no Engine; pixels ficam no Render; interação fica na Interface.**

## Como usar esta documentação

A navegação à esquerda segue a estrutura **domínio → arquivo → tópico**. Cada página registra o modelo recomendado, decisões obrigatórias, invariantes e pontos que ainda precisam ser fechados.

Use a busca no topo — ou pressione **/** — para localizar rapidamente tipos, algoritmos e conceitos mesmo com pequenas diferenças de escrita.

## Por onde começar

Se você está entrando no projeto agora, leia nesta ordem:

1. [Manifesto de engenharia](#/docs/00-philosophy/manifesto.md)
2. [Princípios de refatoração](#/docs/00-philosophy/refactor-principles.md)
3. [Qualidade de código](#/docs/00-philosophy/code-quality.md)
4. [Fronteiras e invariantes](#/docs/00-architecture/boundaries.md)
5. [Não destrutibilidade](#/docs/00-architecture/non-destructive.md)
6. Depois avance para o domínio que estiver implementando.

A documentação é iterativa: decisões consolidadas devem acompanhar o código, e mudanças estruturais importantes devem ser registradas como ADR.


## Roadmap

O [Roadmap de capacidades](#/docs/00-roadmap/capabilities.md) registra as funcionalidades aprovadas para o horizonte do produto sem congelar APIs antes da hora.

As features são especificadas em detalhe somente quando o domínio técnico que as sustenta estiver maduro. **Placed 3D permanece explicitamente pós-`v0.1.0-stable`.**
