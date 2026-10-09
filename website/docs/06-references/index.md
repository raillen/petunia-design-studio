# Referências técnicas — ArtCraft e Petunia Design Studio

**Natureza:** estudo de código-fonte de terceiros e **autorização explícita de reutilização direta de código aberto**, inclusive cópia, adaptação, vendor, fork e dependência quando for a melhor solução e a licença permitir. A documentação não significa implementação, testes ou nova dependência já incorporada. **Inspeção em 2026-10-08** por leitura de estrutura Git, manifests, READMEs, documentação, fontes centrais e testes referenciados. **Não foram compilados nem executados** os três projetos; métricas divulgadas nos seus próprios READMEs são alegações upstream não aferidas independentemente.

## Objetivo

Três editores abertos da família ArtCraft oferecem referências valiosas para Petunia Design Studio: um editor vetorial, um editor raster multicamadas e um revelador/organizador fotográfico. O objetivo é **compreender e adaptar boas soluções isoladas**, sem transplantar o design, substituir o GUI Qt/QML, criar um núcleo paralelo ou violar a filosofia de dados autorais tipados e não destrutivos.

| Projeto | Domínio | Fonte fixada (main, 2026-10-08) | Crates internos | Prioridade no Petunia |
|---|---|---|---:|---|
| [VectorCraft](https://github.com/storytold/vectorcraft) | Ilustração vetorial | [5f92f5eb7fd194826aab2ad0844e474f13990938](https://github.com/storytold/vectorcraft/tree/5f92f5eb7fd194826aab2ad0844e474f13990938) | 21 | Muito alta — Vector, Geometry, Shape Builder |
| [PhotoCraft](https://github.com/storytold/photocraft) | Editor multicamadas/raster | [5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7) | 23 | Muito alta — Raster, Brush, Masks, Compositor |
| [LightCraft](https://github.com/storytold/lightcraft) | RAW / catalog / non-destructive photo | [832b8ada51af1e18ddc2eba87fbfe0485d150fc7](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7) | 20 | Alta e seletiva — Photo, Preview, Color, Masks |

Contagens referem-se a diretórios de primeiro nível em `crates/` da revisão inspecionada, **não** a número de features prontas. Principais versões de workspace: VectorCraft 0.7.0, PhotoCraft 0.5.0, LightCraft 0.4.0. Os três adotam Cargo workspace Rust edition 2024, interface `egui/eframe 0.36` e engines independentes do front-end. VectorCraft renderiza via `vello_cpu`; PhotoCraft mantém compositor CPU de referência e GPU `wgpu`; LightCraft mantém pipeline CPU de referência e GPU compute `wgpu`, com fallback.

## Índice do estudo

1. [VectorCraft — auditoria de geometria, UI, comandos e interoperabilidade](#/docs/06-references/vectorcraft.md).
2. [PhotoCraft — auditoria de pintura, documentos, efeitos, composição e formatos](#/docs/06-references/photocraft.md).
3. [LightCraft — auditoria de revelação, RAW, cache, máscara e automação](#/docs/06-references/lightcraft.md).
4. [Matriz de reaproveitamento e plano de integração Petunia](#/docs/06-references/integration-matrix.md).
5. [Política aprovada: copiar e adaptar código open source](#/docs/06-references/code-reuse-policy.md).
6. [Protocolo para agentes: como pesquisar e integrar essas referências](#/docs/06-references/agent-research-protocol.md).

## Conclusão arquitetural

**O que aproveitar:** código de algoritmos, funções, módulos, crates ou forks completos **quando licenciados e adequados**, além de casos degenerados, testes, organização de render/composição, cache COW, pipeline preview/export, segurança de import/export, serialização resiliente, registry de comandos com IDs, automação headless e sandbox de plugins. Evitar reimplementação gratuita. Ver [política de reutilização direta](#/docs/06-references/code-reuse-policy.md).

**O que NÃO importar como modelo persistente:** `vectorcraft_geom::Anchor` com `AnchorKind` apenas Corner/Smooth, índices de nó da PhotoCraft como identidade estável, `NodeKind`/layer tree de terceiro, formato JSON/ZIP externo no lugar de PTND, layout egui e convenções de atalho que conflitam com acessibilidade/Petunia.

**Respeitar** [fronteiras](#/docs/00-architecture/boundaries.md) e [política de dependências](#/docs/00-philosophy/dependency-policy.md): `petunia-core` é autoridade autoral; `petunia-engine` contém adapters, geometria e comandos; `petunia-render` compõe `RenderSnapshot`; `petunia-ui` Qt/QML via CXX-Qt expõe interação. Manter Core/Engine/Render independentes da GUI e da forma de armazenamento de terceiros.

## Semelhanças e diferenças importantes

| Aspecto | ArtCraft | Petunia |
|---|---|---|
| Linguagem | Rust | Rust |
| GUI | egui/eframe | Qt/QML via CXX-Qt (decisão fechada) |
| Operações | Engine Session e IDs de comandos, CLI e MCP | `ActionId`, `ToolController`, Commands/Transactions e MCP alvo |
| Geometria vetorial | `kurbo` 0.13 + `linesweeper` via VectorCraft | `kurbo` 0.11 no workspace atual + `i_overlay` definido |
| Anchors | VectorCraft `Corner/Smooth`, handles absolutos, caminhos `PathData` | `Cusp/Smooth/Symmetric`, `Option<Point>`, `NodeId/ContourId`, Line+Cubic |
| Raster | PhotoCraft surface COW de tiles 256², múltiplos formatos | COW e dirty tiles documentados, modelo PTND próprio |
| Render | Vector CPU `vello_cpu`; Photo CPU/GPU `wgpu`; Light CPU/GPU compute | `RenderModel/RenderSnapshot` definido; backend conforme plano Petunia |
| Arquivo | `.vectorcraft` JSON, `.pcraft` bundle, catálogo LightCraft com snapshot+log | PTND versionado, migrations e recovery próprios |
| Automação | command-first, JSON control e MCP | Action/Command/Tool + Host API segura e MCP planejado |

## Licenças e propriedade intelectual

Raízes dos três workspaces declaram **MIT OR Apache-2.0**. Isso **não autoriza importar tudo indiscriminadamente**:
- Verificar licença e `NOTICE` do arquivo/crate exato, não só do repositório. Em LightCraft, `crates/segment` declara **Apache-2.0 only**, pois porta código do Hugging Face Transformers. Pesos `facebook/sam3` são externos sob **SAM License**, não incluídos e não OSI; requerem avaliação jurídica e opt-in separado, nunca download automático.
- Logos/wordmarks ArtCraft são marcas e seus assets de marca não entram na licença aberta do código. Fontes, ícones e amostras têm avisos específicos (SIL OFL, ISC, CC/public domain e outros). Não copiar assets/screenshots para produto sem licença e attribution compatíveis.
- Ao adaptar código MIT/Apache, conservar os notices e identificações exigidos e registrar arquivos/commits, alterações e dependências transitivas. Não afirmar clean-room se copiar código; distinguir **inspiração comportamental** de **adaptação/derivação de código**.
- `GPL/LGPL/AGPL` ou código/documentos de origem incerta nunca devem ser trazidos sem revisão de compatibilidade e proveniência.

## Evidência e grau de certeza

**Confirmado em código:** workspace manifests, diretórios, structs e APIs citados com paths, cabeçalhos de arquivos de produção e contratos de Rust. **Descrito pelo upstream:** status de features e metas de paridade, promessas de desempenho e de suporte a formatos. **Inferido/recomendado para Petunia:** localização de adapter, vantagem, esforço, prioridades e testes necessários. **Não verificado nesta análise:** execução, perf benchmark reprodutível, qualidade de imagem sob corpus Petunia, licença de cada artefato transitive, aderência pixel-perfect e compatibilidade binária.

A referência é **fonte de pesquisa e também de implementações potencialmente copiáveis**, não uma dependência atual por si só. O mantenedor autorizou reuso literal/adaptado; cada transferência exige licença/proveniência, adapter quando necessário, matriz de viabilidade e quality gates. ADR somente quando mudar decisão estrutural.

## Ordem recomendada

1. VectorCraft `pathops`, `geom`, `tools`, `engine`: protótipos de adaptação para Smart Path e Smart Region.
2. PhotoCraft `raster`, `paint`, `compose`, `gpu`, `doc`: referência para Paint persona, masks, layer effects e COW/undo.
3. LightCraft `pipeline`, `preview`, `develop`, `color`: referência para persona Photo, adjustment de imagem, previews e proxies.
4. Contratos compartilhados: command IDs, headless/mcp, jobs canceláveis e testes cross-engine, **sem absorver** registries ou formatos incompatíveis.

Este documento registra decisões de **estudo e análise**, não aprova novas dependências nem muda o roadmap.
