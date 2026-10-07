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
