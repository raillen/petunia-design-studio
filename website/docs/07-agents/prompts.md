# Biblioteca de prompts — execução confiável por code agents

**Resumo:** modelos de prompt reutilizáveis para tarefas reais. Eles não substituem a leitura das fontes canônicas, não autorizam mudanças estruturais fechadas e não garantem que todo agente/skill esteja disponível no runtime. Substitua campos `[...]` por informações verificadas, não invente valores.

**Uso recomendado:** começar com o prompt de bootstrap apenas na **primeira sessão** ou após mudança de escopo. Depois usar somente o prompt específico, mais um ContextPack/Handoff atualizado. Não concatenar todos os prompts em um mega-prompt.

## Prompt 0 — Bootstrap com grounded context

```text
Você é um code agent trabalhando no Petunia Design Studio, projeto desktop open source em Rust, com Qt/QML via CXX-Qt definido como GUI.

Antes de propor ou implementar qualquer mudança:
1. Leia AGENTS.md, prumo.json e website/docs/07-agents/index.md.
2. Use website/docs/manifest.json para encontrar só as páginas canônicas do escopo pedido. Leia ADRs aplicáveis e a matriz de implementação.
3. Verifique os arquivos Rust presentes e o estado real do código/testes. Distinga PROPOSED, APPROVED, SPECIFIED, PARTIAL, IMPLEMENTED, TESTED e VERIFIED.
4. Identifique os owners Core/Engine/Render/UI, os tipos autorais, IDs estáveis, Undo/Redo, color/units, limites de segurança e acessibilidade.
5. Consulte website/docs/07-agents/workforce-catalog.md e selecione o mínimo de agents/skills necessários. Não afirme que executou um especialista não disponível.
6. Consulte referências OSS da área quando isso evitar reinventar código; se copiar, registre a licença, origem, commit e testes.
7. Entregue um ContextPack curto com as fontes lidas, gaps, escopo, critérios de aceite e primeiro slice implementável.
Não reabra decisões arquiteturais fechadas nem use documentação como prova de implementação.
```

## Prompt 1 — Implementar ferramenta vetorial / Smart Path

```text
Implemente [FERRAMENTA/OPERAÇÃO] no Petunia, cobrindo [FUNÇÕES]. Trate a documentação em website/docs/04-ui/[PÁGINA].md e ADR-0011 quando aplicável como fonte canônica de UX; leia Core Path, Geometry, Command/Transaction, Spatial, RenderModel e quality gates pertinentes.
Faça gap analysis do checkout antes de alterar código.
Selecione explorer, engine-engineer, editor-engineer, tester, reviewer e documentation-maintainer, com lang-rust, applied-mathematics-dsp, scene-graph, testing-quality, grounded-implementation e skills adicionais realmente necessárias.
Investigue o VectorCraft no commit pinado e o legado 13fe6b4 quanto a algoritmos/fixtures; escolha reutilização licenciada ou implementação nova por evidência. Não crie outro SceneGraph nem degrade NodeId/ContourId, FillRule ou Cusp/Smooth/Symmetric.
Implemente Engine com contracts tipados, preview/cancel/Undo, stale revision guard, erros e testes; só integre Qt se no escopo.
Cobertura mínima: caminho aberto/fechado, cubics/line, handles opcionais, degenerados, concorrência e propriedades de geometria.
Execute comandos de teste, reporte resultados reais, atualize a matriz de implementação e o handoff. Não anuncie feature completa se apenas alguns modos funcionarem.
```

## Prompt 2 — Criar ou reestruturar GUI Qt/QML acessível

```text
Implemente [COMPONENTE/WORKSPACE/TOOLBAR] em Qt/QML via CXX-Qt (não egui, não PySide6) seguindo a documentação específica de 04-ui, a seção 07-agents/accessibility.md e ADR-0011 quando houver Vector Edit.
Audite primeiro a implementação atual do repo Rust e os componentes QML existentes no legado commit 13fe6b4. Se adaptável, reutilize QML; substitua o bridge Python e revise bindings, desempenho e UX conforme política Petunia.
Orquestre ux-architect, design-system-engineer, ui-component-engineer, accessibility-reviewer, tester e reviewer, selecionando cognitive-clarity, design-psychology, focus-management, keyboard-accessibility, screen-reader, contrast, motion-accessibility, typography-system, design-tokens e ui-implementation conforme o caso.
Produza state matrix: default, hover, focus, pressed, active, selected, disabled, mixed, pending, error. Especifique e implemente teclado, navegação semântica, tooltips, toasts, modais, dialogs, docking, snapping e números editáveis pertinentes.
Garanta keyboard-only operation e screen-reader/accessibility-tree inspection nativo quando tooling disponível; documente cenários não testados.
Não alegue screenshot/QA visual executados se só revisou QML em texto. Gere evidências dos controles implementados, teste e registre gaps.
```

## Prompt 3 — Revisão de acessibilidade e neurodivergência

```text
Faça auditoria de acessibilidade e clareza cognitiva de [COMPONENTE/FLUXO] do Petunia.
Consulte 07-agents/accessibility.md e 04-ui/accessibility.md; use accessibility-reviewer e ux-architect (ou procedimentos equivalentes se agente não estiver executável).
Ative accessibility, cognitive-clarity, design-psychology, keyboard-accessibility, screen-reader, focus-management, contrast, motion-accessibility, zoom-reflow e visual-qa sob demanda.
Para cada interação, descreva tarefa do usuário, estado semântico, foco, atalhos remapeáveis, controle sem mouse, densidade, labels, legendas, contraste, scroll, feedback, errors/cancel/undo e custo cognitivo.
Avalie TDAH/dislexia sem generalizações ou promessa clínica. Proponha melhorias priorizadas P0/P1/P2 e critérios reproduzíveis. Execute keyboard/screen-reader/Qt inspection quando disponível; diferencie achado estático de verificação realizada.
Não mude diretamente decisões de UX fechadas: sinalize proposta e mantenha source canonical.
```

## Prompt 4 — Implementar Paint/raster/compositor

```text
Implemente [RECURSO] para a Paint persona com Source PixelLayer/Brush/COW e pipeline Render do Petunia.
Leia 02-engine/brush-raster.md, 03-render, 01-core e contratos de Undo/History, Color Management e IO. Inspecione o checkout: Surface, tools, buffer, masks e render-model podem não existir ainda.
Compare PhotoCraft para tiles 256² COW, brush dynamics, masks/compositor CPU e LightCraft para preview/cache quando pertinente. Copie/adapte código elegível se vantajoso, registre licença e proveniência.
Use renderer-engineer, engine-engineer, performance-agent e tester conforme risco; skills rendering-2d, color-science, memory-management, concurrency-quality, error-handling, benchmarking, testing-quality.
Implemente incrementalmente com dirty regions, budget, interrupt/cancel, release memory, image parity, seeded determinism e recovery; não converta buffers ou color-space silenciosamente.
Execute testes de COW, Undo/Redo, ROI vs full, alpha/compositing e performance reproduzível. Atualize docs/status e handoff.
```

## Prompt 5 — Revisar e aproveitar código open source

```text
Antes de implementar [FEATURE], examine as referências VectorCraft/PhotoCraft/LightCraft em 06-references/ para localizar implementações existentes no source pinado.
Leia arquivos reais, testes, manifests, LICENSE e NOTICE dos candidatos.
Escolha explicitamente: dependency direta, cópia de função, vendor/fork, port/adaptação, referência de testes ou implementação nova. Preserve copyright e histórico de alterações no código copiado; não confunda licença do código com licença de assets/model weights.
Proponha adapter Petunia-owned mínimo e mapeamento de divergências em NodeId/ContourId, SceneGraph, RGBA/ICC, FillRule, precision, units, errors, preview, Undo e Core serialization.
Implemente o caminho escolhido somente após demonstrar custo/risco/testes. Não portar egui como GUI e não inserir crate extra de boolean/ICC sem comparação.
Entregue arquivos usados (path+SHA), mode de reuso, documentação de origem, testes rodados, benchmark comparativo quando necessário, e recomendação de manutenção upstream.
```

## Prompt 6 — Migrar legado Python/C++ com teste primeiro

```text
Investigue [FEATURE] no commit histórico 13fe6b408752b244ee56d4765ea13435eb3ef921 do petunia-ds. O novo checkout Rust é autoridade de implementação atual; docs 07-agents/legacy-migration.md e ADRs atuais são autoridade de comportamento alvo.
1. Localize Python/C++/QML/testes históricos.
2. Extraia casos de teste, invariantes, fixtures, erros e UX observáveis.
3. Compare contratos atuais; registre diferenças de algoritmos, IDs, formatos e fluxo de dados.
4. Classifique cada parte como QML reuse, PORT_ALGORITHM, TESTS_EXTRACTED, REFERENCE_ONLY ou DISCARD.
5. Implemente a fatia Rust mínima ou adapte QML, sem criar monólito bridge/model nem código duplo.
6. Execute regressões equivalentes do legado e testes novos para specs vigentes.
Entregue migração rastreável por path, commit, teste e incompatibilidade conhecida. Não conclua que o legado é inútil apenas por estar em outra linguagem.
```

## Prompt 7 — Encontrar e corrigir bug sem regressões

```text
Corrija [BUG COM PASSOS REPRODUZÍVEIS] respeitando a spec canônica e o comportamento atual.
Use recipe bug-fix e agentes explorer/debugger/implementer/tester/reviewer; skills debugging/review conforme manifesto (não invente ID 'debugging' se não existir).
Comece por reproduzir e escrever/ajustar teste que demonstra a falha. Investigue causa-raiz, ownership, side effects e quais contratos foram violados. Corrija a menor causa estrutural, evitando workarounds e reescrita abrangente.
Rode teste alvo, regressões e cargo checks pertinentes. Reporte antes/depois, comando, saída real, arquivos tocados, riscos e se o problema foi reproduzido no ambiente.
```

## Prompt 8 — Refatoração/clean code com evidência de equivalência

```text
Refatore [MÓDULO] mantendo exatamente comportamento aprovado e performance dentro dos limites.
Leia as fronteiras/ADRs, arquitetura, matriz e testes reais. Antes da mudança, identifique code smells concretos (duplicação, dead code, acoplamento, state leaks, alocações, panics, classes/arquivos monolíticos).
Use architect, implementer, tester e quality-reviewer; skills clean-code, refactoring, architecture-quality, code-review, implementation-reality-verification, lang-rust.
Separe mudanças semânticas de mudanças mecânicas; preservar IDs/serialization/Undo/color. Execute fixtures/golden/perf base antes e depois quando possível, e cite diffs/resultados reais. Não alegar equivalência só porque compila.
```

## Prompt 9 — Documentação e ADR sem repetir fontes

```text
Atualize a documentação canônica do Petunia para [DECISÃO/RECURSO] em modo DELTA.
Leia manifest.json, seções relacionadas, ADRs e code implementation matrix. Detecte contradições, versões e duplicações; não substitua decisão aprovada sem aprovação explícita.
Use documentation-maintainer, architect e reviewer; skills documentation-for-llms, project-documentation-architect (DELTA), prumo-navigation, cognitive-clarity, documentation-publishing conforme publicação.
Escreva em pt-BR como idioma canônico, com anchors estáveis, propósito, contratos, erros, estados, exemplos, critérios de aceitação e referências cross-linkadas. Separe CODE_CONFIRMED/TESTED de SPECIFIED/APPROVED.
Atualize manifest.json e website/llms.txt se houver novas páginas. Verifique links e navegação; informe quais verificações realizou, sem afirmar build/test do produto.
```

## Prompt 10 — Auditoria geral de implementação (sem implementar no impulso)

```text
Faça auditoria de [SUBSISTEMA] verificando os arquivos reais da branch e o conjunto de specs/ADRs relevantes.
Monte uma tabela Feature→Source Path→Evidence→Status→Gap→Severity→Tests→Proposta.
Inspecione segurança, erros tipados, APIs públicas, dead code, concorrência, perf, acessibilidade e docs. Diferencie ausência de código de comportamento não testado e não invente resultados.
Compare criticamente com os repositórios de referência quando aplicável e com o legado preservado, inclusive testes históricos. Proponha ordem de correção por benefício/risco.
Não implementar mudanças fora do escopo de auditoria, salvo pedido explícito. Entregue findings priorizados e verificáveis.
```

## Prompt 11 — Release/QA de aceitação

```text
Avalie release readiness do Petunia para [PLATAFORMA/ALVO] com a matriz de implementação e contratos de quality gates.
Use release-verifier, tester, security-reviewer, accessibility-reviewer quando pertinente; skills release-engineering, ci-cd, supply-chain-security, security-review, visual-regression e testing-quality.
Confirme apenas capabilities presentes e testes executados; não promova planned features a release notes.
Verifique cargo fmt/check/test/clippy, packaging Qt plugins, assets/licenses, codecs, PTND migration/backup/recovery, crash handling, permissions, keyboard/assistive-technology workflows e performance baseline documentada.
Entregue PASS/FAIL/NOT_RUN por gate, responsável, evidência e blockers. Não publique ou crie release sem autorização específica.
```

## Contrato de saída comum a todos os prompts

```text
STATUS: [SPECIFIED | PARTIAL | IMPLEMENTED | TESTED | VERIFIED | BLOCKED]
OBJECTIVE / DECISIONS:
CANONICAL SOURCES (paths and sections):
REAL CODE INSPECTED:
FILES CHANGED:
PROVENANCE (if copied):
TESTS RUN + OUTPUT:
NOT RUN + REASON:
KNOWN RISKS / ACCESSIBILITY:
NEXT ACTION:
```

**Nota:** os prompts são comandos para agentes **que disponham de acesso real ao checkout e tools**. Eles não conferem acesso, não delegam fisicamente sozinhos e não executam testes pela mera inclusão no site.

[Índice](#/docs/07-agents/index.md) · [Handoff](#/docs/07-agents/handoff.md) · [Catálogo Prumo](#/docs/07-agents/workforce-catalog.md).
