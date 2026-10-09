# Auditoria de reuso — execução e evidência

**Status:** P0 concluído (auditoria técnica estática) em 2026-10-09, sobre o protocolo da [matriz de aproveitamento](#/docs/06-references/integration-matrix.md). **Nenhum código foi copiado, portado ou adicionado como dependência neste slice.** O resultado aqui é mapa com evidência e correções de pista; ele não autoriza importar nada sozinho.

## Como a auditoria foi executada

Os três repositórios foram clonados em `--filter=blob:none` e fixados nos SHAs já registrados na política de reuso:

| Projeto | SHA auditado | Licença na raiz | LOC em `crates/` |
|---|---|---|---|
| VectorCraft | `5f92f5eb7fd194826aab2ad0844e474f13990938` | MIT **ou** Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`) | 302.058 |
| PhotoCraft | `5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7` | MIT **ou** Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md`) | 297.627 |
| LightCraft | `832b8ada51af1e18ddc2eba87fbfe0485d150fc7` | MIT **ou** Apache-2.0, **com exceções** (`NOTICE`) | 128.404 |

Confirmação de licença: os três trazem `LICENSE-MIT` e `LICENSE-APACHE` na raiz. LightCraft declara em `NOTICE` que `crates/segment` é **Apache-2.0 somente**, sem a opção MIT.

## Verificação dos caminhos citados na matriz

Todos os caminhos nomeados na matriz foram abertos e medidos. **31 de 32 existem**; uma pista estava errada:

| Caminho citado | Estado | Linhas |
|---|---|---|
| VC `pathops/src/boolean.rs` | existe | 594 |
| VC `pathops/src/planar.rs` | existe | 1.136 |
| VC `tools/src/builder.rs` | existe | 1.024 |
| VC `tools/src/pen.rs` | existe | 745 |
| VC `tools/src/direct.rs` | existe | 1.057 |
| VC `pathops/src/edit.rs` | existe | 453 |
| VC `pathops/src/fit.rs` | existe | 232 |
| VC `effects/src/stroke/width.rs` | existe | 367 |
| VC `doc/src/blend.rs` | existe | 1.158 |
| VC `effects/src/live.rs` | existe | 57 |
| VC `geom/src/path.rs` | existe | 632 |
| PC `raster/src/lib.rs` | existe | 929 |
| PC `paint/src/brush.rs` | existe | 557 |
| PC `paint/src/dynamics.rs` | existe | 666 |
| PC `compose/src/lib.rs` | existe | 1.517 |
| PC `compose/src/adjust.rs` | existe | 912 |
| PC `doc/src/effects.rs` | existe | 341 |
| PC `psd/src/lib.rs` | existe (fachada, 83) | 83 |
| PC `engine/src/selection_cmds.rs` | existe | 844 |
| PC `paint/src/retouch.rs` | existe | 547 |
| PC `vector/src/edit.rs` | existe | 341 |
| PC `plugins/src/lib.rs` | existe (fachada, 55) | 55 |
| LC `develop/src/settings.rs` | existe | 920 |
| LC `pipeline/src/lib.rs` | existe | 512 |
| LC `pipeline/src/masks.rs` | existe | 654 |
| LC `preview/src/lib.rs` | existe | 181 |
| LC `preview/src/pool.rs` | existe | 180 |
| LC `preview/src/disk.rs` | existe | 230 |
| LC `preview/src/lru.rs` | existe | 117 |
| LC `catalog/src/journal.rs` | existe | 695 |
| LC `gpu/src/lib.rs` | existe | 367 |
| LC `segment/src/lib.rs` | existe | 263 |
| **VC `engine/mcp/src/lib.rs`** | **não existe** | — |

**Correção de pista:** o MCP da VectorCraft não está em `crates/engine/mcp/`, e sim em `crates/mcp/` (4.114 linhas em `src/`, com `tools.rs` de 833 linhas e dezenas de arquivos `tests_*.rs`). O MCP da LightCraft está em `crates/mcp/` (`backend.rs`, `headless.rs`, `server.rs`); a automação da PhotoCraft está em `crates/automation/` (`bridge.rs`, `budgets.rs`, `files.rs`, `headless.rs`). A conclusão da matriz continua válida; apenas o caminho precisa de correção.

## Uso de `unsafe`

Contagem de arquivos com ocorrência de `unsafe` em `crates/`: VectorCraft 20, PhotoCraft 26, LightCraft 25. Cobertura de `#![forbid(unsafe_code)]` em `lib.rs`: VectorCraft 18/22, PhotoCraft 22/24, LightCraft 19/20.

Isso importa para o Petunia porque o workspace proíbe `unsafe` sem justificativa documentada. **Consequência prática:** nenhum arquivo com `unsafe` entra por cópia direta; só entra por adaptação, com o `unsafe` removido ou isolado atrás de wrapper seguro com `// SAFETY:`.

Um exemplo favorável: `PC raster/src/lib.rs` (929 linhas, tiles COW esparsos) abre com `#![forbid(unsafe_code)]`. Todo o `LC preview` (`lib.rs`, `pool.rs`, `disk.rs`, `lru.rs`) também declara `#![forbid(unsafe_code)]`. Esses são os candidatos mais limpos para eventual adaptação.

## Licença de pesos e modelos de terceiros

`LC crates/segment` porta código do SAM 3 do HuggingFace Transformers (Copyright The HuggingFace Team e Meta Platforms, Inc.), **Apache-2.0 somente**. O `NOTICE` da LightCraft registra que **os pesos do SAM 3 não são distribuídos com o projeto**: quem usa baixa o checkpoint `facebook/sam3` por opção, sob a SAM License da Meta — **licença separada e não OSI**.

Conclusão para o Petunia: código de `segment` pode ser estudado/adaptado sob Apache-2.0 com atribuição; **pesos não entram** no repositório nem são baixados por build. Qualquer feature de máscara por IA permanece opt-in e fora deste slice.

A `ATTRIBUTION.md` da PhotoCraft documenta asset por asset com licença (Inter e JetBrains Mono em SIL OFL 1.1, ícones Lucide em ISC, Feather em MIT, um ícone do Noun Project em CC BY 3.0). Assets de marca **não** são open source pelo `NOTICE`. Nada disso entra sem decidir asset por asset.

## Estado atual do Petunia versus os candidatos

A auditoria encontrou que vários alvos já possuem implementação própria e testada no checkout. Isso muda o valor do port: onde já existe equivalente, ganho potencial é **qualidade**, não capacidade.

| Já implementado no Petunia | Candidato upstream | Leitura da auditoria |
|---|---|---|
| `geometry/bezier.rs` (De Casteljau, split, bounds, flatten) | VC `pathops/fit.rs` (least-squares com reparametrização Newton, Graphics Gems) | Port pode melhorar *refit* após booleans; hoje fazemos flatten sem refitar |
| `geometry/offset.rs` (miter, bevel em spike) | VC `pathops/offset.rs`, `effects/stroke/width.rs` | Equivalente cobre o contrato v0.1; upstream tem joins/caps mais completos |
| `geometry/simplify.rs` (RDP) | VC `pathops/edit.rs` | Equivalente atende; upstream adiciona `average`/`join` e preservação topológica |
| `tiles.rs` (COW + dirty set) | PC `raster/src/lib.rs` | Equivalente atende; upstream tem sparse + interrupt |
| `effects.rs` + `geometry/shape_builder.rs` | VC `pathops/planar.rs`, `tools/builder.rs` | Shape Builder já cobre proveniência; upstream tem face-walking completo |
| `jobs.rs` (prioridades, cancelamento, child tokens) | LC `preview`, `catalog/journal.rs` | Equivalente atende; upstream tem disco + LRU com budgets |
| `mcp.rs` (catálogo, deny-by-default) | VC `crates/mcp`, LC `crates/mcp` | Equivalente cobre o contrato; upstream tem tools abundantes para comparar |
| `compositor.rs` (15 blend modes) | PC `compose/src/lib.rs` | Equivalente cobre; upstream tem multichannel/pattern/proxy |

## Decisão registrada

**Nada foi copiado, adaptado, vendorizado ou adicionado como dependência.** A P0 cumpriu o que promete: mapa com evidência, um caminho corrigido, e a observação de que a maior parte dos alvos de alta prioridade já tem equivalente próprio e testado.

Recomendação para as próximas etapas, mantendo a ordem da matriz:

1. **P1 — primeiro port real, candidato único:** VC `pathops/fit.rs` (232 linhas, depende só de `kurbo` + `thiserror`, sem `unsafe`). Ele melhora o resultado dos booleans, que já usamos via `i_overlay`, e tem superfície pequena o bastante para auditar arquivo por arquivo.
2. **P1 alternativo:** LC `preview/{lru,pool}.rs` (297 linhas somadas, `forbid(unsafe_code)`) para o cache com budgets — mas só se a medição mostrar necessidade.
3. **Permanecer fora:** `LC crates/segment` (Apache-only) e qualquer peso SAM 3; assets de marca da PhotoCraft; UI egui dos três.

Cada item exige, antes de qualquer commit: licença por arquivo, notice de copyright, testes de paridade com o equivalente atual e benchmark no mesmo fixture — conforme [política de reutilização](#/docs/06-references/code-reuse-policy.md) e as etapas da [matriz](#/docs/06-references/integration-matrix.md).

## Registro de proveniência

Nenhum código é importado no Petunia sem registro. Um único port foi executado até agora:

| Arquivo Petunia | Origem | Licença | Alterações |
|---|---|---|---|
| `crates/petunia-engine/src/geometry/fit.rs` | VectorCraft `crates/pathops/src/fit.rs` @ `5f92f5eb7fd194826aab2ad0844e474f13990938` | MIT **ou** Apache-2.0 | reescrito sobre os tipos Petunia (`CubicBez`, `Point`, `Vec2`), sem Kurbo; adicionados `refit_contour`, `g1_continuous`, `sample_uniform`; `newton` e `eval_deriv2` corrigidos durante o port; testes estendidos com caso de paridade contra `simplify` |

Atribuição no código: o cabeçalho de `fit.rs` credita Philip J. Schneider, *Graphics Gems* (1990), e a origem VectorCraft com o SHA. O workspace Petunia segue **MIT OU Apache-2.0**, compatível com a origem.

Nenhum asset, peso de modelo, UI egui ou dependência transitiva foi incorporado.

## Invariantes desta auditoria

1. Auditoria estática não altera código do Petunia.
2. Caminho inexistente é pista de estudo, nunca garantia de funcionamento.
3. Licença vale por arquivo, não pela raiz do repositório.
4. `unsafe` em origem bloqueia cópia direta.
5. Pesos de modelo não-OSI não entram no repositório.
6. Ausência de port não é ausência de capacidade quando há equivalente testado.

## Referências

[Matriz de aproveitamento](#/docs/06-references/integration-matrix.md) · [Política de reutilização](#/docs/06-references/code-reuse-policy.md) · [Protocolo de pesquisa](#/docs/06-references/agent-research-protocol.md) · [Migração do legado](#/docs/07-agents/legacy-migration.md) · [Verificação](#/docs/00-architecture/verification.md).
