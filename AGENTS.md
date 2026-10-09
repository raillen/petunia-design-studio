# AGENTS.md — Petunia Design Studio (Rust)

Este documento estabelece as diretrizes canônicas, papéis da força de trabalho (*workforce*), habilidades obrigatórias (*skills*) e invariantes inegociáveis para o desenvolvimento e evolução do projeto **Petunia Design Studio** em Rust.

---

## 1. Princípios e Invariantes Fundamentais (Sempre Ativos)

Toda alteração de código, proposta arquitetural ou refatoração no projeto **deve obrigatoriamente seguir** estes princípios:

### 1.1 Clareza Cognitiva & Neurodivergência (`cognitive-clarity`)
* **Comunicação Direta e Sem Ambiguidade:** Declare fatos, restrições e etapas em linguagem clara, direta e objetiva. Evite jargões corporativos vazios ou suposições não documentadas.
* **Foco em Uma Decisão por Vez (Anti-Overwhelm):** Não bombardeie com múltiplos dilemas simultâneos. Isole a decisão mais crítica, recomende o caminho ideal e justifique os trade-offs sucintamente.
* **O "Porquê" antes do "Como":** Explique a motivação de engenharia e a razão arquitetural antes de aplicar mudanças no código.
* **Ancoragem Visual:** Use listas, tabelas e títulos claros. Evite blocos maciços de texto corrido.
* **Segurança Psicológica e Normalização de Erros:** Trate bugs, avisos do compilador e falhas em testes como dados objetivos para diagnóstico, focando sempre na solução limpa e na causa-raiz.

### 1.2 Qualidade de Arquitetura & Modularidade Estrita (`architecture-quality`)
* **Direção de Dependências:** O domínio e a lógica de negócios nunca dependem da infraestrutura ou da interface gráfica. Todas as dependências apontam para dentro (*Clean / Hexagonal Architecture*).
* **Ausência de Ciclos:** Ciclos de dependência entre módulos e crates são estritamente proibidos.
* **Separação Completa de Camadas:** Modelos de domínio (`SceneObject`, `VectorPath`) não vazam para camadas de renderização ou transporte/serialização, nem vice-versa. Adapters e DTOs explícitos são usados nas fronteiras.
* **Eliminação de God Modules & Estado Compartilhado Oculto:** Módulos têm escopo bem delimitado. Evite Singletons e variáveis globais com mutabilidade oculta; prefira injeção explícita de dependências e sincronização explícita via atores/mensagens.

### 1.3 Clean Code & Excelência de Engenharia (`clean-code`, `code-quality`)
* **Legibilidade e Semântica:** Código deve ser legível como prosa técnica. Nomes de variáveis, structs, traits e funções expressam intenção sem abreviações obscuras.
* **Funções Pequenas e Focadas:** Uma única responsabilidade (SRP).
* **Sem Código Morto ou Comentários Obsoletos:** Remova código não utilizado e preserve apenas documentação relevante e atualizada.

### 1.4 Acessibilidade por Padrão (`accessibility`, `screen-reader`, `contrast`, `keyboard-accessibility`)
* **Teclado Primeiro:** Toda ação acessível por ponteiro deve ser igualmente acionável via atalhos e navegação de foco por teclado.
* **Contraste & Design Visual:** Conformidade com diretrizes WCAG (contraste mínimo para painéis, canvas, réguas e seleções).
* **Gestão de Foco:** Ciclos previsíveis de foco sem perda de orientação durante transições de diálogo ou desacoplamento de janelas.
* **Redução de Movimento:** Respeito à preferência de redução de movimento do sistema operacional.

### 1.5 Padrão de Engenharia em Rust (`lang-rust`, `concurrency-quality`, `memory-management`)
* **Seguro por Padrão:** `#![forbid(unsafe_code)]` como política padrão do workspace. Qualquer bloco `unsafe` exige justificativa documentada com `// SAFETY:`.
* **Tratamento Tipado de Erros:** Proibido uso indiscriminado de `.unwrap()` e `.expect()` fora do escopo de testes. Propagação via operador `?` com erros fortemente tipados via `thiserror` (em bibliotecas) e `anyhow` (em binários/aplicações).
* **Concorrência Segura:** Aproveitamento das garantias de `Send` e `Sync` do Rust, eliminando data races em tarefas concorrentes de tesselação, renderização e processamento em background.
* **Compilação Limpa:** Todo código deve compilar sem avisos no `cargo clippy --all-targets -- -D warnings`.

### 1.6 Verificação Real e Qualidade de Testes (`testing-quality`, `implementation-reality-verification`)
* **TDD & Testabilidade:** Todo comportamento novo é acompanhado por testes automatizados (unitários, integração e invariantes de geometria).
* **Verificação Concreta:** Nenhuma tarefa é dada como concluída sem que o código compile e a suíte de testes seja executada com sucesso.

### 1.7 Tracker Vivo de Implementação (`implementation-reality-verification`)
* **Fonte única de estado:** `website/progress/tasks.json` é a fonte versionada de tarefas e estados, exibida em `#/progress`. Protocolo em `website/docs/00-roadmap/progress.md`.
* **Atualização obrigatória:** ao iniciar uma implementação, marcar os IDs afetados como `IN PROGRESS`; a cada entrega, atualizar checkpoints, evidências e datas **no mesmo conjunto de alterações do código**.
* **Três estados, percentual derivado:** somente `TODO` / `IN PROGRESS` / `DONE`; o percentual vem dos checkpoints concluídos. `DONE` exige todos os gates aplicáveis ao escopo delimitado da tarefa. Mencionar IDs e estados no handoff.
* **Especificação não fecha tarefa:** contrato, ADR ou código existente sem evidência de execução nunca conclui um checkpoint.

---

## 2. Workforce — Papéis e Especialistas Disponíveis

O projeto conta com agentes especialistas mapeados em `.agents/agents/` para orquestração de tarefas:

| Agente | Escopo e Responsabilidade Principal |
| :--- | :--- |
| **`architect`** | Fronteiras entre crates, contratos de interfaces, decisões de design e elaboração de ADRs. |
| **`engine-engineer`** | Motor geométrico, matemática vetorial (Bézier, curvas cúbicas), álgebra de cena e operações topológicas. |
| **`renderer-engineer`** | Pipeline de renderização 2D/GPU, shaders, tesselação, rasterização e adaptadores de backend gráfico. |
| **`editor-engineer`** | Ferramentas interativas de estúdio (Pen, Node, Selection, Transform, Snapping) e eventos de canvas. |
| **`design-system-engineer`**| Design tokens, paleta de cores, componentes de UI e consistência visual da interface. |
| **`ux-architect`** | Fluxos de usuário, ergonomia das ferramentas e clareza de interação. |
| **`accessibility-reviewer`** | Auditoria e revisão de acessibilidade (a11y), contrastes e navegabilidade por teclado. |
| **`implementer`** | Implementação de funcionalidades e refatorações orientadas a testes com código limpo. |
| **`quality-reviewer`** | Revisão de código, detecção de code smells e garantia de conformidade com os invariantes. |
| **`debugger`** | Diagnóstico, isolamento de causas-raiz e correção de bugs com testes de regressão. |
| **`tester`** | Criação de testes unitários, testes de propriedades, benchmarks e suites de regressão. |
| **`security-reviewer`** | Análise de segurança de arquivos de entrada (SVG, PTND), quarentena de dependências e memory safety. |
| **`documentation-maintainer`**| Manutenção da documentação canônica, especificações de schemas e histórico de arquitetura. |

---

## 3. Catálogo de Skills Instaladas

As habilidades estão disponíveis em `.agents/skills/` e sincronizadas em `.ai/skills/`:

* **Acessibilidade & Neurodivergência:** `cognitive-clarity`, `accessibility`, `screen-reader`, `keyboard-accessibility`, `focus-management`, `contrast`, `motion-accessibility`, `zoom-reflow`.
* **Arquitetura & Clean Code:** `architecture-quality`, `clean-code`, `code-quality`, `code-review`, `refactoring`, `error-handling`, `memory-management`, `concurrency-quality`, `secure-coding`, `security-review`.
* **Rust & Toolchain:** `lang-rust`, `rust-analyzer-context-indexing`, `lang-c`, `lang-cpp`, `git-workflow`.
* **Domínio Gráfico & Editor:** `rendering-2d`, `scene-graph`, `input-handling`, `state-management`, `serialization`, `editor-tooling`, `design-system`, `design-tokens`, `component-specification`, `plugin-architecture`, `performance-native`, `benchmarking`, `testing-quality`, `mcp-integration`, `mcp-tooling`.
* **Processo & Engenharia:** `grounded-implementation`, `implementation-reality-verification`, `project-documentation-architect`, `documentation`, `context-optimization`, `lean-progressive-context`.


---

## Diretivas do site — roteamento canônico para agentes (2026-10-08)

**Ponto de entrada obrigatório para novas sessões:** [Diretivas para Code Agents](website/docs/07-agents/index.md). Índice compacto de leitura automatizada: [website/llms.txt](website/llms.txt). **Navegação:** [website/docs/manifest.json](website/docs/manifest.json). Estas páginas complementam este AGENTS.md sem sobrescrever ADRs ou o contrato do Core/Engine/Render/UI.

**Ordem de execução:** ler [autoridade e estados](website/docs/07-agents/authority-reading.md) → identificar estado real no checkout e [matriz de implementação](website/docs/00-architecture/implementation-matrix.md) → selecionar workforce em [orquestração](website/docs/07-agents/orchestration.md) e [catálogo Prumo](website/docs/07-agents/workforce-catalog.md) → seguir [workflow](website/docs/07-agents/implementation-workflow.md), [prompts](website/docs/07-agents/prompts.md) e [handoff](website/docs/07-agents/handoff.md).

**Workforce:** o Prumo de referência [poppy-lat/prumo@e213260c99d2](https://github.com/poppy-lat/prumo/tree/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce) contém **39 agents, 189 skills e 20 recipes**. O checkout Petunia em 2026-10-08 inclui **19 AGENT.md** em `.agents/agents` e **44 SKILL.md** em `.ai/skills`; **não** assumir os 39/189/20 instalados. A listagem integral com disponibilidade, links e categorias está nas páginas do site. Nunca carregar todas as 189 skills simultaneamente; selecionar por contexto e criticidade. Exigir revisão de acessibilidade/UX para modificações de GUI, em especial [cognitive-clarity e neurodivergência](website/docs/07-agents/accessibility.md).

**Legado:** o commit [13fe6b4](https://github.com/raillen/petunia-ds/tree/13fe6b408752b244ee56d4765ea13435eb3ef921) conserva a base antiga **Python/C++/Qt QML**. Sua arquitetura não governa o novo Rust, mas algoritmos, casos de teste, comportamento e QML podem ser reaproveitados conforme [auditoria de migração](website/docs/07-agents/legacy-migration.md). Não reimplementar tudo por princípio nem portar bridge/DocumentStore antigos diretamente.

**OSS:** copiar/adaptar código licenciado do VectorCraft, PhotoCraft e LightCraft é autorizado sob as condições da [política de reutilização](website/docs/06-references/code-reuse-policy.md), mantendo origem, copyright, notices, adapters e validação real.

**Estado vs. intenção:** `APPROVED`/`SPECIFIED` não são `IMPLEMENTED`/`TESTED`/`VERIFIED`. Não apresentar teste não executado, trabalho histórico ou docs como feature pronta. Para tasks docs-only, qualidade de links/manifest é o gate adequado; para código, compilar/testar e registrar as saídas realmente observadas.
