# Orquestração de agentes, skills e recipes

**Resumo:** toda tarefa ganha um responsável, um contexto mínimo, specialists por risco e gates verificáveis. **Não carregar 189 skills em cada requisição.** Ver [catálogo completo e estado instalado](#/docs/07-agents/workforce-catalog.md).

## 1. Camadas de responsabilidade

**Orquestrador:** identifica Goal, procura a spec e classifica escopo. Não assume que todos os especialistas estão instalados ou conectados. Se nenhum subagente executável estiver disponível, seguir com mesmo protocolo em sequência lógica, sem fingir delegação.

**Explorer / Architect:** mapeia realidade do código, fronteiras, ADRs e plano; não decide unilateralmente alterar decisões fechadas.

**Implementer + specialist:** escreve código de domínio e testes; especialista variável (engine, render, editor, UI, design, security).

**Tester + Reviewer:** validam comportamento e arquitetura de modo independente quando possível; a revisão não pode simplesmente carimbar o que o implementer afirma.

**Documentation Maintainer:** atualiza páginas canônicas/status e informa evidência; não transforma spec em código imaginário.

## 2. Skills obrigatórias por contexto, não por volume

**Base de leitura e execução:** `prumo-navigation`, `lean-progressive-context`, `cognitive-clarity`, `grounded-implementation`, `architecture-quality`, `clean-code`, `lang-rust`, `testing-quality`, `implementation-reality-verification`. Em tarefas documentais, incluir `documentation-for-llms` + `documentation`. O baseline deve ser aplicado por responsabilidade, não pela inserção de dez textos longos e duplicados num mesmo prompt.

**Habilitar por gatilho:**
- UI → `ux-architect`, `ui-component-engineer`, `accessibility-reviewer`; skills `interaction-design`, `keyboard-accessibility`, `focus-management`, `screen-reader`, `cognitive-clarity`, `design-psychology`.
- Engine/path → `engine-engineer`, `scientific-computing-agent`, `tester`; `applied-mathematics-dsp`, `scene-graph`, `testing-quality`, `benchmarking`.
- Render/Color → `renderer-engineer`, `performance-agent`; `rendering-2d`, `shaders`, `color-science`, `visual-regression`.
- Plugins/MCP/external input → `security-reviewer`, `isolation-auditor`; `plugin-security`, `mcp-security`, `untrusted-project-security`, `fuzz-grammar-testing`.
- Design/brand/ícones → `design-researcher`, `design-system-engineer`, `svg-artist`, `visual-identity-auditor`; `icon-system`, `typography-system`, `visual-communication`, `contrast`.
- Docs/refactor → `documentation-maintainer`, `quality-reviewer`; `project-documentation-architect` (modo DELTA), `documentation-for-llms`, `code-review`.

**Complementares:** todas as demais skills de [catálogo 189](#/docs/07-agents/workforce-catalog.md) são elegíveis sob escopo apropriado. `lang-python`/`lang-cpp` servem **somente** à auditoria do legado ou integração de fontes; não convertem a nova base Rust em Python/C++.

## 3. Recipes Prumo: selecione por fluxo, revise suitability

| Intenção | Receita real Prumo | Ajuste necessário no Petunia |
|---|---|---|
| Feature normal | `feature-standard` | docs canônicas e revisão independente |
| Bug | `bug-fix` | reproduzir, regressão, causa-raiz |
| Arquitetura | `architecture-change` | ADR autorizado; evitar rediscutir decisões fechadas |
| UI | `ui-feature` | **trocar exemplos web/ARIA por equivalents Qt/QML sem perder semântica** |
| Revisão visual | `ui-review` | keyboard, focus, screen reader, multi-panel |
| Render/Engine | `engine-renderer` | adaptar gatilho `project_types: game-engine` ao desktop creative editor |
| Design system | `design-system-foundation` | design tokens, styles, Qt components |
| Documentação | `documentation-refactor` | pt-BR fonte canônica; delta |
| Segurança | `security-review`, `security-audit-release` | I/O/ICC/Font/SVG/PTND/plugins/MCP |
| Release | `release` | Windows/Linux/macOS e packaging real |

**Importante:** Prumo `ui-feature` enumera `frontend-web` e `engine-renderer` pode filtrar por `game-engine`. Não alterar o `recipe.json` original sem versão derivada, nem executar receitas incompatíveis ao pé da letra. Use como checklist/workflow adaptado, sob `PETUNIA` policy.

## 4. Exemplo de DAG de feature vetorial

```text
Explorer → (scope, existing code, risk)
            ↓
         Architect → (type contracts, no Core/UI leakage)
            ↓
       Engine Engineer → (geometry + typed command + preview)
            ↓
        Tester → (analytic + degenerate + property + undo)
            ↓
        Editor Engineer → (ToolController + action + a11y path)
            ↓
        Accessibility Reviewer → (keyboard + semantic feedback)
            ↓
        Reviewer / Security Reviewer (if external input)
            ↓
        Documentation Maintainer → (real status + evidence)
```

Parallelize independentes somente onde inputs e arquivos não colidem; writer único para cada documento canônico. Não marcar aprovação de reviewer sem o papel ter executado o review efetivo.

## 5. Protocolo de handoff entre agentes

Um especialista recebe **TaskPack limitado**:
- Goal, escopo e evidência de aprovação;
- 1–3 páginas canônicas com âncoras + trecho mínimo de código;
- invariantes, tipos e owners por crate;
- explicit non-goals;
- acceptance tests + error budget;
- autorização/ferramentas/capabilities;
- saída e status esperados.

Ele devolve `finding -> code path -> evidence -> risk -> proposed fix`. Nunca passar o output completo de todos os agentes para o próximo sem resumir e referenciar arquivos. Ver [Handoff](#/docs/07-agents/handoff.md).

## 6. Regras de execução segura

Respeitar permissões descritas em manifests: por exemplo `accessibility-reviewer` pode **revisar e propor docs**, mas não assumir permissão de escrever código se manifest não permitir. Instalação não dá permissão de shell/network além do ambiente real. `Review required` não é dispensado porque a recipe terminou.

**Sucesso final:** aceite do Goal, gates efetivamente executados, evidência linkada e docs com status real.

[Workforce integral](#/docs/07-agents/workforce-catalog.md) · [Workflow](#/docs/07-agents/implementation-workflow.md) · [Prompts](#/docs/07-agents/prompts.md).
