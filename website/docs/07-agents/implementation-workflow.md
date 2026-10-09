# Workflow de implementação — do contrato ao código verificado

**Resumo:** faça implementação vertical pequena, código e testes reais, validação de arquitetura e accessibility pertinente, documentação sincronizada. O projeto atual tem quatro crates Rust presentes; contratos futuros de RenderModel, QML/CXX-Qt, plugins e sistemas avançados **não são automaticamente implementações existentes**.

## 1. Etapas de execução obrigatórias

| Fase | Entrada | Saída verificável |
|---|---|---|
| 0. Definir Goal e escopo | pedido + ADR + docs canônicas | acceptance criteria e non-goals |
| 1. Investigar o checkout | implementação-matrix + arquivos reais | gap map, APIs, dependências, testes existentes |
| 2. Reuso/riscos | upstream ref + licença + alternativas | decisão reuse/adapter/reimplementação com procedência |
| 3. Projetar fatia vertical | boundaries + types | interface/Command/DTO, erros, mudanças por crate |
| 4. Implementar e testar | design de contratos | código mínimo, fixtures e testes automatizados relevantes |
| 5. Integrar Editor | ToolController + GUI Qt e ActionIds quando escopo incluir GUI | UI, feedback, teclado e Session State corretos |
| 6. Verificar | Quality gates | comandos e resultados reais, benchmarks por risco, QA |
| 7. Revisar | diff + relatório de testes | revisão independente e correções |
| 8. Documentar | estado realmente obtido | docs/matriz/status + handoff reproduzível |

Tarefa puramente documental pode parar em docs/consistência/links e **não** precisa fingir build de aplicativo. Tarefa de código deve registrar check/testes necessários antes de dizer pronta. Se a ferramenta não permite executar testes, finalizar `IMPLEMENTED / UNVERIFIED` com blocker explícito.

## 2. Ownership por crate

```text
petunia-core    = dados persistentes, IDs tipados, documento/SceneGraph
petunia-engine  = geometria, tools/commands, validação, snapping, jobs
petunia-render  = software reference rendering, paint/composition/caches
petunia-ui      = Qt/QML/CXX-Qt bridge, Input+ToolController, panels

petunia-render-model = contrato arquitetural definido, não crate existente
```

Boundary principal: `UI → Engine → Core`, `UI → Render → Core`; `Engine → render-model ← Render` **quando a crate for criada no milestone correto**. `Core` não importa Qt, engine ou render; `Engine` não importa render/UI; `Render` não importa Engine/UI.

**Um comando autoral = uma transação semântica**, com preview, cancel, undo/redo e error handling. Input de ponteiro e overlays vivem no Session State enquanto a mudança ainda não foi confirmada. Não registrar um `Command` por frame de drag. Invalidation/version guard preserva state se source mudar.

## 3. Regras de código Rust e organização

- Nomes claros, types explícitos, módulos pequenos e coesos, sem "god file" nem mutabilidade global opaca.
- `Result<T, E>` e `thiserror` em boundaries. Não `unwrap/expect` em input não confiável/produção sem justificativa. `unsafe` só sob exceção documentada e auditada, nunca contornar wrapper seguro apenas por comodidade.
- Testes para invariantes geométricos f64, IDs, degenerados, alocação e validadores de formato; property/fuzz para superfícies de parsing, golden para render.
- Evitar `#[allow(...)]`, `dead_code`, `todo!()`, mocks que passam sem comportamento e asserts sobre valores fictícios como substitutos de implementação real.
- Normalizar premultiplied alpha, coord transform/units, FillRule/ICC e índices/IDs ao cruzar fronteiras; não adotar semântica upstream inadvertidamente.

## 4. Gate adaptativo (não teatral)

| Mudança | Testes mínimos relevantes |
|---|---|
| Core/model | unit, serialization roundtrip, invalid refs, no cycles, property |
| Geometry/Paths | analytic, degenerate, metamorphic, topology/provenance, benchmark |
| Raster/Brush | COW/dirty tiles/seed, cancel/Undo, CPU oracle, perf |
| Render/Color | golden/snapshot com color context, CPU×backend parity, device loss |
| Input/GUI | keyboard traversal, focus, pointer/stylus, screen reader inspection, contrast, reduced-motion |
| Plugins/MCP/I/O | untrusted parsing, sandbox budgets, authz, fuzz, recovery |
| Docs-only | valid nav, real links, conflicts, freshness of status; test commands not required if no code touched |

Para código Rust do checkout, baseline documentada:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**Não alegar que esses comandos foram executados só por estarem aqui.** Rodar quando houver runtime disponível e registrar resultado. Build de Qt depende do estado concreto de integração CXX-Qt; não inventar screenshot/test de tela sem aplicativo em execução.

## 5. Adaptação de OSS permitida

Projeto autoriza copiar funções, módulos, crates e algoritmos do VectorCraft, PhotoCraft e LightCraft quando útil, com licenças apropriadas, provenance e testes. Consultar [matriz upstream](#/docs/06-references/integration-matrix.md) e [cópia licenciada](#/docs/06-references/code-reuse-policy.md). Preferir biblioteca existente/adapter pequeno à reescrita inútil; nunca substituir documento PTND/Core ou UI Qt por modelos upstream.

## 6. Critérios de aprovação de GUI

Antes de aplicar uma mudança de interface, ler UX da ferramenta correspondente e [a11y](#/docs/07-agents/accessibility.md). Textos e layouts não substituem comportamento: especificar foco, hover, pressed, active, disabled, selected, errors, keyboard traversal, tooltips, toast, undo/redo, escape/cancel. Sempre disponibilizar caminho sem ponteiro; falha de a11y bloqueia considerar UI completa.

## 7. Conclusão (Definition of Done)

A feature só é marcada `VERIFIED` se:
1. comportamento entregue equivale ao contrato aprovado e não cria side effects ocultos;
2. arquitetura, tipos e IDs permanecem corretos;
3. testes relevantes executados e resultados disponíveis;
4. revisão independente adequada ao risco concluiu e issues graves resolvidas;
5. docs, matriz de implementação e prompts de manutenção estão atualizados;
6. nenhuma regressão crítica de UI/acessibilidade, performance e segurança conhecida sem decisão explícita.

Se uma etapa não ocorrer, declarar o **status mais baixo que o fato sustenta**; não "aprovado com testes a realizar" como equivalente de Done.

## 8. Atualização de estado do tracker (obrigatória)

`website/progress/tasks.json` é a fonte versionada exibida em `#/progress`; o protocolo completo está em [progresso das etapas](#/docs/00-roadmap/progress.md).

1. Antes de implementar, localizar os IDs afetados e marcá-los `IN PROGRESS` ao iniciar; atualizar `updated` na tarefa e no arquivo.
2. Ao entregar um slice, marcar somente os checkpoints comprovados, com entrada em `evidence` (revisão, resumo e documento com comandos/resultados reais). ADR aprovado não é prova.
3. Em mudança de escopo, dividir/adicionar tarefa com ID novo e critérios verificáveis. Preservar IDs existentes; não alterar o denominador apenas para melhorar a porcentagem.
4. Registrar no capítulo canônico os gates `not run / pass / fail / blocked`; gate manual necessário ainda aberto impede `DONE`.
5. Atualizar status, checkpoints, evidência e datas **no mesmo conjunto de alterações da implementação**. Se nada puder ser concluído, atualizar baseline/impedimento/data mesmo com o percentual inalterado.
6. Rodar os validadores, revisar o diff e incluir IDs/status no handoff:

```bash
node --check website/app.js
node --check website/progress/progress.js
node website/scripts/verify-progress.cjs
node --test website/tests/progress.test.cjs
```

[Orquestração](#/docs/07-agents/orchestration.md) · [Quality gates detalhados](#/docs/00-architecture/verification.md) · [Handoff](#/docs/07-agents/handoff.md).
