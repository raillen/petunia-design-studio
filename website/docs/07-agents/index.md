# Diretivas para Code Agents — ponto de entrada

**Propósito:** contratos e instruções operacionais para qualquer agente de código que estude, implemente, revise, teste ou documente o **Petunia Design Studio**. Esta seção não altera arquiteturas já aprovadas nem confunde plano de produto com código executável.

**Comece aqui, não carregue tudo.** O ponto de entrada deve ser curto. Cada assunto abaixo é uma página canônica de aprofundamento que deve ser aberta **somente se for relevante** à tarefa atual.

## Em 60 segundos

1. **Identifique o trabalho:** auditoria, arquitetura já decidida, implementação, bug, UI/UX, revisão, segurança, atualização de docs ou migração.
2. **Resolva a fonte de verdade:** siga [hierarquia de documentos, leitura progressiva e estados](#/docs/07-agents/authority-reading.md), começando por `AGENTS.md`, `prumo.json`, manifest do site e ADRs específicos.
3. **Entenda realidade do código:** consulte [matriz de implementação](#/docs/00-architecture/implementation-matrix.md) e **verifique o checkout** antes de afirmar que uma feature existe. Especificação ≠ implementação.
4. **Selecione equipe mínima:** no [catálogo completo](#/docs/07-agents/workforce-catalog.md), escolha skills sempre ativas e especialistas por gatilho; use [orquestração](#/docs/07-agents/orchestration.md).
5. **Entregue uma unidade vertical:** siga [workflow de implementação e gates](#/docs/07-agents/implementation-workflow.md). Preserve as arquiteturas fechadas e decisões de UI.
6. **Comunique evidências:** código, teste, comando real, resultado, riscos, docs alteradas, pendências e handoff num formato fixo; veja [template de context pack](#/docs/07-agents/handoff.md).

## Páginas desta seção

| Abrir quando... | Documento |
|---|---|
| Há dúvidas de autoridade, contexto, escopo, estado ou decisões | [Leitura, hierarquia e rastreabilidade](#/docs/07-agents/authority-reading.md) |
| Você vai implementar, corrigir ou refatorar código | [Workflow de implementação e Definition of Done](#/docs/07-agents/implementation-workflow.md) |
| Você precisa pedir trabalho a uma LLM/code agent | [Prompts operacionais por tipo de tarefa](#/docs/07-agents/prompts.md) |
| Você precisa escolher especialistas, recursos e recipes | [Orquestração e seleção progressiva](#/docs/07-agents/orchestration.md) |
| Você quer **todos** os agents, skills e recipes do Prumo, inclusive complementares | [Catálogo completo (39/189/20)](#/docs/07-agents/workforce-catalog.md) |
| Você vai alterar GUI, controles, tool gestures, feedback e acessibilidade | [A11y, TDAH/dislexia e qualidade cognitiva](#/docs/07-agents/accessibility.md) |
| Você vai entregar um resultado ou passar contexto a outro agente | [Handoff, evidências e memória de tarefa](#/docs/07-agents/handoff.md) |
| Você vai investigar a base C++/Python anterior ou reconstruir a stack | [Auditoria de legado e política de reimplementação](#/docs/07-agents/legacy-migration.md) |

## Invariantes inegociáveis

- **Stack atual e arquitetura:** `petunia-core`, `petunia-engine`, `petunia-render`, `petunia-ui` em Rust; UI arquiteturalmente **Qt/QML via CXX-Qt**, sem Qt no Core/Engine. `petunia-render-model` está **especificado, não presente no workspace atual**.
- **Core é autoral:** IDs tipados, SceneGraph, caminhos, documento PTND e regras não destrutivas permanecem com Petunia; integrações externas passam por adapters.
- **Interação de Vector Edit:** ADR-0011 híbrido Select→Vector Edit; Bend explícito; nenhuma edição por hover; foco distinto da seleção; alteração via Transaction.
- **Cores, coordenadas e render:** seguir docs canônicas de unidade, FillRule, perfis de cor, alpha, transform e RenderSnapshot sem "correção" baseada em palpites.
- **Acessibilidade por padrão:** teclado completo, leitores de tela/semântica, contraste e foco, mensagens, targets ampliados e conforto cognitivo; sem dependência de Alt-only.
- **Segurança:** arquivos externos, plugins e APIs MCP são superfícies não confiáveis; limites, permissões, erros tipados, cancelamento e rollback.
- **Uso pragmático de OSS:** copiar/adaptar código é permitido e desejável com licença/proveniência/QA; leia [referências upstream](#/docs/06-references/index.md) e [política de reutilização](#/docs/06-references/code-reuse-policy.md).
- **Evidência sobre retórica:** nenhuma claim de `DONE`, `tested`, `production-ready` ou `verified` sem procedimento executado e evidência.

## Frase de inicialização recomendada

> Leia primeiro o índice de Diretivas para Code Agents e só depois a documentação canônica do escopo. Identifique decisões fechadas, código realmente presente, lacunas e testes exigidos. Ative as skills mínimas e os especialistas necessários do Prumo. Preserve a filosofia Petunia. Implemente a menor mudança verificável e registre evidência real sem afirmar o que não foi testado.

**Linguagem canônica:** pt-BR na documentação Petunia, código/API em inglês quando indicado pela convenção de Rust. Priorize terminologia idêntica entre docs, commands e UI.

[Home do site](#/docs/home.md) · [Filosofia](#/docs/00-philosophy/manifesto.md) · [ADRs](#/docs/00-architecture/adr/index.md) · [Catálogo completo](#/docs/07-agents/workforce-catalog.md).
