# Referências técnicas — protocolo de pesquisa e integração para code agents

**Status:** diretriz de leitura e verificação para estudos de VectorCraft, PhotoCraft e LightCraft. Não aprova novos dependencies ou merges de código. Esta página deve ser lida junto ao [catálogo de referências](#/docs/06-references/index.md), [matriz](#/docs/06-references/integration-matrix.md), [política de dependências](#/docs/00-philosophy/dependency-policy.md) e [quality gates](#/docs/00-architecture/verification.md).

## 1. Antes de trabalhar

1. Identificar tarefa e domínio Petunia (Core/Engine/Render/UI/IO/Plugin).
2. Ler documentação canônica correspondente à ferramenta, ADRs e matriz de implementação.
3. Ler a página de estudo do projeto upstream e localizar paths de source. Conferir `README`, `Cargo.toml`, `AGENTS.md`, `NOTICE` e módulo/testes relevantes do commit fixado.
4. Ler **código real e testes**, não concluir funcionamento completo apenas do nome do arquivo ou claim do README; distinguir documentação atual de draft antigo.
5. Registrar `commit SHA` e licença exata antes de copiar/derivar conteúdo; se main mudou, usar snapshot ou fazer re-auditoria explícita, sem trocar de versão silenciosamente.
6. Comparar semântica de modelo Petunia e upstream, preparar adapter mínimo e test plan. Não começar por QML/Qt enquanto API do Engine estiver indefinida.
7. Entregar menor integração possível, testes, bench/regressão, docs, ADR/dependency review quando necessário.

## 2. Seleção da fonte certa

| Caso | Primeira leitura upstream | Segunda leitura |
|---|---|---|
| Nodes/Bézier/Symmetry | VC `geom/src/path.rs`, `tools/src/direct.rs` | VC `pathops/src/edit.rs`, Petunia `01-core/path.md` |
| Shape Builder/Region Paint | VC `pathops/src/planar.rs` | VC `tools/src/builder.rs`, Petunia Smart Region |
| Boolean/Simplify | VC `pathops/src/boolean.rs`, `edit.rs` | VC `engine/src/cmd/pathops.rs`, Petunia Geometry |
| Pen UX | VC `tools/src/pen.rs` | Petunia Pen/Vector Edit |
| Strokes/Brush/Tablet | PC `paint/src/brush.rs`, `raster/src/lib.rs` | PC `paint/src/dynamics.rs`, Petunia Brush/Raster |
| Compositor e layer effect | PC `compose/src/lib.rs`, `gpu/src/lib.rs` | PC `doc/src/effects.rs`, Petunia Render |
| PSD e persistência | PC `psd/src/`, `format/src/lib.rs` | Petunia PTND/IO |
| RAW e develop | LC `raw/src/lib.rs`, `pipeline/src/lib.rs` | LC `develop/src/settings.rs`, Petunia Color |
| Máscaras e selection | LC `pipeline/src/masks.rs`, PC `compose/src/masks.rs` | Petunia Mask/Appearance |
| Preview/cache/jobs | LC `preview/src/lib.rs`, `pool.rs` | Petunia Render Cache/Jobs |
| Automação/MCP | VC/LC `mcp/src/lib.rs`, PC `automation/src/lib.rs` | Petunia ActionIds/Plugin Host API |
| Sandbox WASM | VC/PC `plugins/src/lib.rs`, `docs/plugins.md` | Petunia dependency/security model |

Links:
- [VectorCraft @ commit](https://github.com/storytold/vectorcraft/tree/5f92f5eb7fd194826aab2ad0844e474f13990938)
- [PhotoCraft @ commit](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7)
- [LightCraft @ commit](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7)

## 3. Restrições de integração irrenunciáveis

- **Não transplantar egui/eframe para o Petunia**; GUI pertence exclusivamente à camada Qt/QML via CXX-Qt.
- **Não substituir types persistentes do Core** por `Anchor`, `PathData`, `Surface`, `DevelopSettings`, `Catalog`, `egui::Id` ou `NodeKind` estrangeiros.
- **Não duplicar Engines** de boolean, ICC ou Compose quando stack já foi definida sem ADR + benchmark comparativo.
- **Não usar índices como identidade estável de node**, nem grupos identificados pelo próprio nome como tipo autoral. Manter NodeId/ContourId e Region bindings explícitos.
- **Não degradar Symmetric** ao enum Corner/Smooth upstream; `Line` não vira `Cubic` implicitamente por edição de handle que deveria ser inerte.
- **Não alterar a ordem de blending** (linear/document-space vs display RGB) sem modelo e testes de compatibilidade explícitos.
- **Não sobrescrever PTND** com formato `.pcraft`, JSON `.vectorcraft` ou journal de catálogo.
- **Não copiar assets/brand** de terceiros como se licenças de código cobrissem logo, imagens, fontes e screenshots.
- **Não tratar `facebook/sam3` weights como MIT/Apache**. Separar import de algoritmo Apache-only, licença dos pesos e direito de distribuição.
- **Não prometer paridade** com Illustrator, Photoshop ou Lightroom com base no README upstream, e não declarar experimento implementado se apenas foi documentado.

## 4. Tipo de evidência

Usar exatamente um destes níveis em cada conclusão:

- `CODE_CONFIRMED`: fonte de produção verificada em commit fixo, com caminho/classe/função.
- `TEST_PRESENT`: teste encontrado, mas não executado.
- `TEST_RUN_PASSED`: teste executado com comando e resultados capturados.
- `UPSTREAM_CLAIM`: README/roadmap descreve feature/benchmark; ainda não aferido.
- `INFERRED`: consequência técnica razoável, explicitamente delimitada.
- `PETUNIA_PROPOSAL`: decisão recomendada, não implementada.
- `ADOPTED`: código integrado ao Petunia, testado, commit revisado e documentação alinhada.
- `UNVERIFIED`: não há evidência suficiente; não preencher lacuna inventando comportamento.

**Não inflar conclusões:** uma crate compilável não significa que todos os recursos de seu README estejam completos; uma doc é especificação, não prova. Uma pasta de testes não significa testes passando.

## 5. Fluxo de execução do agente

~~~text
Identify exact Petunia feature and scope
→ Read Petunia ADR + canonical feature specs
→ Read source repo pinned snapshot + tests + licenses
→ Compare data/coordinate/color/identity semantics
→ Write minimal adapter design + rollback plan
→ Record alternatives and dependency decision
→ Implement in Petunia-owned crate (no Qt in Core/Engine)
→ Unit/property/fuzz/visual/benchmark
→ Test headless ↔ UI parity, Undo/Redo, cancel/stale
→ Review security, licence, notices, accessibility
→ Update docs and implementation matrix
→ Commit with linked evidence
~~~

### Obrigação de escrever um pequeno relatório de integração

~~~md
# Upstream adaptation: [Feature]
- Petunia spec/ADR:
- Upstream: project / commit / exact path / function:
- Evidence level:
- Behaviour confirmed:
- Known limitations / upstream issue:
- Licence + NOTICE + provenance:
- Decision: inspiration / adapter / vendored code / dependency / reject
- Architectural owner:
- Typed contract + inverse/Undo:
- Source data/coordinate/color/ID mismatches:
- Error/cancellation/resource budgets:
- Test matrix: unit, property, fuzz, render oracle, roundtrip, bench, a11y:
- Result and measured perf (machine + fixture):
- Docs, PR, commit:
~~~

## 6. Prompt operacional pronto para code agents

> Analise a feature atual do Petunia Design Studio tomando a documentação canônica e ADRs como autoridade. Consulte a página de referências técnicas e o commit fixo do VectorCraft, PhotoCraft ou LightCraft indicado para a responsabilidade. Leia o módulo de produção, testes, Cargo.toml e licença/NOTICE antes de concluir qualquer aproveitamento. Registre diferenças de semântica (IDs, knots/anchors, FillRule, unidades, coordinate spaces, color/alpha, COW, persistence, Undo). Proponha o menor adapter em crate Petunia, sem introduzir egui/eframe, outro SceneGraph ou novo formato persistente. Compare bibliotecas já definidas antes de adicionar dependência. Faça a implementação com preview e commit pela mesma Engine, transaction atômica, cancel, stale-revision protection, erros tipados e performance budgets. Teste unit/property/fuzz/roundtrip/golden/benchmark conforme risco e garanta keyboard/screen reader/inputs alternativos. Cite os paths/SHAs realmente usados, registre licença/provenance e atualize a matriz de implementação sem confundir especificado com implementado.

## 7. Checklist de segurança/licenciamento

- [ ] A origem é código que pode ser utilizado na licença Petunia, revisando termos de artefatos/crates específicos?
- [ ] Os notices relevantes estão preservados (inclusive modificação de código Apache-2.0 e terceiros)?
- [ ] Não há licenças/asssets com termos diferentes introduzidos inadvertidamente?
- [ ] Input externo é validado (tamanho, profundidade, coordenadas finitas, memory/time, payload MCP)?
- [ ] Backend/plugin/I/O é cancelável e não promove FS/network arbitrários?
- [ ] Command/preview/Undo/Redo não deixam mutação parcial nem perdem source/provenance?
- [ ] O modo headless executa a operação sem Qt e o frontend usa ActionIds equivalentes?
- [ ] A ferramenta é operável sem mouse, e não depende só de Alt ou cores?
- [ ] O novo código não duplica silenciosamente geometria/ICC/render/persistence existente?
- [ ] Cada benchmark é reproduzível e documenta hardware/fixture/config?
- [ ] Erros de terceiros retornam Result seguro; não expõem panic via biblioteca host?
- [ ] CI/regressões/testes passaram **de verdade** antes de anunciar adoção?

## 8. Reavaliação futura

Toda atualização upstream deve passar por `old_sha...new_sha` para os módulos relevantes. Se o upstream alterar modelos, licenças, APIs ou trade-offs de performance, atualizar o estudo com:
- delta funcional/arquitetural;
- migração de adapter;
- regressões e testes executados;
- riscos de licença e manutenção;
- novas oportunidades que justifiquem replanejar, **sem reabrir a filosofia Petunia automaticamente**.

**Prumo/agents:** usar agentes de investigação, arquitetura, implementação, qualidade, segurança, UX/acessibilidade e documentação já incorporados ao projeto; esta página é instrução específica adicional, não substitui suas skills.

[Índice](#/docs/06-references/index.md) · [Matriz](#/docs/06-references/integration-matrix.md) · [Quality gates](#/docs/00-architecture/verification.md).
