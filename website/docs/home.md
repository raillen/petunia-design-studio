# Petunia Design

Documentação técnica do **Petunia Design Studio**: um editor open source de imagem, vetor e layout pensado para ser modular, não destrutivo e previsível de evoluir.

Esta documentação descreve **as decisões antes da implementação** — tipos de dados, fronteiras entre módulos, invariantes, algoritmos, fluxo de edição e responsabilidades de cada camada.

## Como a arquitetura é dividida

| Domínio | Responsabilidade |
|---|---|
| **Núcleo** | Documento, scene graph, paths, cores, texto, recursos e dados persistentes. |
| **Engine** | Geometria, snapping, brushes, raster, layout, color management, commands e I/O. |
| **Render Model** | Contrato imutável Engine → Render, sem backend. |
| **Render** | Rasterização, composição, efeitos, caches, overlays e saída para tela/export. |
| **Interface** | Workspace, ferramentas, input, painéis, atalhos, acessibilidade e integração Qt/QML via CXX-Qt. |

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
6. [Verificação e quality gates](#/docs/00-architecture/verification.md)
7. Depois avance para o domínio que estiver implementando.

A documentação é iterativa: decisões consolidadas devem acompanhar o código, e mudanças estruturais importantes devem ser registradas como ADR.

## Estado da especificação

A arquitetura técnica de **Core, Engine, Render Model e Render** está fechada em nível suficiente para orientar implementação incremental.

Isso inclui:

- identidade/persistência PTND;
- modelo não destrutivo;
- paths/shapes;
- color/appearance;
- scene/document/resources;
- transactions/history;
- Geometry/Shape Builder;
- Spatial/Snapping;
- Brush/Raster;
- Text/Layout;
- ICC Color Management;
- I/O, scheduler, plugins WASM e MCP;
- RenderSnapshot, software tiled renderer, compositor, cache/output;
- testes, fuzzing, profiling e quality gates.

Pontos que dependem de benchmark permanecem explicitamente abertos na Matriz de Implementação.

**Tools, Workspace, Acessibilidade e GUI/UX/UI não foram congelados**. Essa parte volta para discussão colaborativa antes de avançar.



## Roadmap

O [Roadmap de capacidades](#/docs/00-roadmap/capabilities.md) registra as funcionalidades aprovadas para o horizonte do produto sem congelar APIs antes da hora.

As features são especificadas em detalhe somente quando o domínio técnico que as sustenta estiver maduro. **Placed 3D permanece explicitamente pós-`v0.1.0-stable`.**


## Referências técnicas e estudos de código

A seção [Referências Técnicas](#/docs/06-references/index.md) reúne uma auditoria de arquitetura, código e contratos de três editores open source em Rust: [VectorCraft](#/docs/06-references/vectorcraft.md), [PhotoCraft](#/docs/06-references/photocraft.md) e [LightCraft](#/docs/06-references/lightcraft.md). O estudo inclui [matriz de integração](#/docs/06-references/integration-matrix.md), a [política de cópia e reutilização direta de código open source](#/docs/06-references/code-reuse-policy.md) e [protocolo de leitura para agentes](#/docs/06-references/agent-research-protocol.md).

As referências podem servir **tanto para estudo quanto para cópia literal/adaptação de código licenciado**, com proveniência, notices e testes. A documentação não equivale a dependências aprovadas nem a código portado. Os módulos upstream têm diferenças de modelo autoral, interface, licenças e gerenciamento de cor que exigem adapters e validação antes de qualquer integração.


## Diretivas para Code Agents

A [seção de Diretivas para Code Agents](#/docs/07-agents/index.md) é o ponto de entrada para LLMs e desenvolvedores que implementam o Petunia. Contém [ordem de leitura e hierarquia das fontes](#/docs/07-agents/authority-reading.md), [workflow verificável](#/docs/07-agents/implementation-workflow.md), [biblioteca de prompts](#/docs/07-agents/prompts.md), [orquestração de agentes](#/docs/07-agents/orchestration.md) e [inventário integral do Prumo: 39 agents, 189 skills, 20 recipes](#/docs/07-agents/workforce-catalog.md).

Há também um guia específico de [acessibilidade e neurodivergência](#/docs/07-agents/accessibility.md), um [contrato de handoff](#/docs/07-agents/handoff.md) e uma [auditoria para decidir que código Python/C++/QML do projeto histórico podemos reaproveitar](#/docs/07-agents/legacy-migration.md). O [índice llms.txt](/llms.txt) permite leitura progressiva sem despejar toda a documentação no contexto.
