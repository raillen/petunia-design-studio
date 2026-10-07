# Petunia Design

Documentação técnica do **Petunia Design Studio**: um editor open source de imagem, vetor e layout pensado para ser modular, não destrutivo e previsível de evoluir.

Esta documentação descreve **as decisões antes da implementação** — tipos de dados, fronteiras entre módulos, invariantes, algoritmos, fluxo de edição e responsabilidades de cada camada.

## Como a arquitetura é dividida

| Domínio | Responsabilidade |
|---|---|
| **Núcleo** | Documento, scene graph, paths, cores, texto, recursos e dados persistentes. |
| **Engine** | Geometria, snapping, brushes, raster, layout, color management, commands e I/O. |
| **Render** | Rasterização, composição, efeitos, caches, overlays e saída para tela/export. |
| **Interface** | Workspace, ferramentas, input, painéis, atalhos, acessibilidade e integração Qt/QML. |

> A regra central é simples: **dados autorais ficam no Núcleo; cálculos ficam no Engine; pixels ficam no Render; interação fica na Interface.**

## Como usar esta documentação

A navegação à esquerda segue a estrutura **domínio → arquivo → tópico**. Cada página registra o modelo recomendado, decisões obrigatórias, invariantes e pontos que ainda precisam ser fechados.

Use a busca no topo — ou pressione **/** — para localizar rapidamente tipos, algoritmos e conceitos mesmo com pequenas diferenças de escrita.

## Por onde começar

Se você está entrando no projeto agora, leia nesta ordem:

1. [Fronteiras e invariantes](#/docs/00-architecture/boundaries.md)
2. [Matriz de implementação](#/docs/00-architecture/implementation-matrix.md)
3. [Não destrutibilidade](#/docs/00-architecture/non-destructive.md)
4. Depois avance para o domínio que estiver implementando.

A documentação é iterativa: decisões consolidadas devem acompanhar o código, e mudanças estruturais importantes devem ser registradas como ADR.
