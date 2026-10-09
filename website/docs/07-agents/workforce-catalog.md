# Catálogo completo de workforce do Prumo — Petunia Design Studio

> **Snapshot de origem:** [poppy-lat/prumo@`e213260c99d2`](https://github.com/poppy-lat/prumo/tree/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/); inventariado diretamente da árvore de arquivos `AGENT.md`, `SKILL.md` e `RECIPE.md` de `src/prumo/resources/workforce/`, em 2026-10-08. Esta relação tem **todos os IDs existentes** nesse snapshot, inclusive complementares e não usuais. Os nomes têm links para os arquivos de origem. **Listagem não implica instalação local nem ativação automática.**

## Resumo verificável

| Tipo | Prumo upstream | Já presente no checkout Petunia | Ainda não sincronizado |
|---|---:|---:|---:|
| Agents | **39** | **19** | **20** |
| Skills | **189** | **44** em `.ai/skills` | **145** |
| Recipes | **20** | **0** sob `.agents`/`.ai` nas rotas de recipes inspecionadas | **20** |

**Source of truth** da lista: arquivos da árvore real do Prumo no commit fixo, não exemplos narrativos das páginas `docs/workforce/*.md` que podem mostrar IDs ilustrativos diferentes do catálogo instalado. No Petunia `.agents/agents` possui 19 agentes e `.agents/skills` e `.ai/skills` têm um subconjunto; evitar escrever "tudo instalado" porque a cópia atual é parcial.

**Política de carga:** base de skills sempre relevante + especialistas **por gatilho**. Não carregar as 189 skills simultaneamente. "Base" significa leitura/enforcement por orquestrador, não ativar todos os especialistas em cada edição trivial.

## 1. Agentes (39) — inventário integral

### Coordenação, exploração e entrega

| Agente | Estado no checkout | Ativação |
|---|---|---|
| [explorer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/explorer/AGENT.md) | Presente | Padrão do fluxo |
| [architect](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/architect/AGENT.md) | Presente | Padrão do fluxo |
| [implementer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/implementer/AGENT.md) | Presente | Padrão do fluxo |
| [prototyper](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/prototyper/AGENT.md) | Ausente | Quando o escopo exigir |
| [reviewer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/reviewer/AGENT.md) | Presente | Padrão do fluxo |
| [quality-reviewer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/quality-reviewer/AGENT.md) | Presente | Padrão do fluxo |
| [tester](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/tester/AGENT.md) | Presente | Padrão do fluxo |
| [debugger](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/debugger/AGENT.md) | Presente | Quando o escopo exigir |
| [issue-author](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/issue-author/AGENT.md) | Ausente | Quando o escopo exigir |
| [issue-triager](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/issue-triager/AGENT.md) | Ausente | Quando o escopo exigir |
| [release-verifier](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/release-verifier/AGENT.md) | Presente | Quando o escopo exigir |
| [documentation-maintainer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/documentation-maintainer/AGENT.md) | Presente | Padrão do fluxo |
| [technology-decision-agent](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/technology-decision-agent/AGENT.md) | Ausente | Quando o escopo exigir |

### Core, Engine, Render e sistemas

| Agente | Estado no checkout | Ativação |
|---|---|---|
| [systems-architect](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/systems-architect/AGENT.md) | Ausente | Quando o escopo exigir |
| [engine-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/engine-engineer/AGENT.md) | Presente | Quando o escopo exigir |
| [renderer-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/renderer-engineer/AGENT.md) | Presente | Quando o escopo exigir |
| [editor-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/editor-engineer/AGENT.md) | Presente | Quando o escopo exigir |
| [performance-agent](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/performance-agent/AGENT.md) | Ausente | Quando o escopo exigir |
| [scientific-computing-agent](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/scientific-computing-agent/AGENT.md) | Ausente | Quando o escopo exigir |
| [compiler-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/compiler-engineer/AGENT.md) | Ausente | Quando o escopo exigir |
| [backend-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/backend-engineer/AGENT.md) | Ausente | Quando o escopo exigir |
| [database-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/database-engineer/AGENT.md) | Ausente | Quando o escopo exigir |
| [networking-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/networking-engineer/AGENT.md) | Ausente | Quando o escopo exigir |
| [devops-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/devops-engineer/AGENT.md) | Ausente | Quando o escopo exigir |

### Acessibilidade, GUI, UX e design

| Agente | Estado no checkout | Ativação |
|---|---|---|
| [accessibility-reviewer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/accessibility-reviewer/AGENT.md) | Presente | Quando o escopo exigir |
| [ux-architect](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/ux-architect/AGENT.md) | Presente | Quando o escopo exigir |
| [design-system-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/design-system-engineer/AGENT.md) | Presente | Quando o escopo exigir |
| [ui-component-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/ui-component-engineer/AGENT.md) | Ausente | Quando o escopo exigir |
| [frontend-engineer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/frontend-engineer/AGENT.md) | Presente | Quando o escopo exigir |
| [design-researcher](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/design-researcher/AGENT.md) | Presente | Quando o escopo exigir |
| [creative-director](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/creative-director/AGENT.md) | Ausente | Quando o escopo exigir |
| [brand-designer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/brand-designer/AGENT.md) | Ausente | Quando o escopo exigir |
| [visual-identity-auditor](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/visual-identity-auditor/AGENT.md) | Ausente | Quando o escopo exigir |
| [svg-artist](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/svg-artist/AGENT.md) | Ausente | Quando o escopo exigir |
| [motion-designer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/motion-designer/AGENT.md) | Ausente | Quando o escopo exigir |
| [advertising-designer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/advertising-designer/AGENT.md) | Ausente | Quando o escopo exigir |

### Segurança e isolamento

| Agente | Estado no checkout | Ativação |
|---|---|---|
| [security-architect](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/security-architect/AGENT.md) | Presente | Quando o escopo exigir |
| [security-reviewer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/security-reviewer/AGENT.md) | Presente | Quando o escopo exigir |
| [isolation-auditor](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/agents/isolation-auditor/AGENT.md) | Ausente | Quando o escopo exigir |


## 2. Skills (189) — inventário integral

Critérios: **base** para procedimentos fundamentais de qualidade, grounding, leitura e segurança de engenharia; **contextual** quando tarefa, risco ou alteração justifica. Status **local** indica `.ai/skills/{id}/SKILL.md` encontrado no commit atual do Petunia; não garante instalação de dependências, scripts de verificação ou propagação entre adapters.

### Leitura, documentação, prompts e colaboração (16)

- [agentic-workflow-design](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/agentic-workflow-design/SKILL.md) — contextual; não sincronizada.
- [cognitive-clarity](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/cognitive-clarity/SKILL.md) — **base**; local.
- [context-optimization](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/context-optimization/SKILL.md) — contextual; local.
- [documentation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/documentation/SKILL.md) — **base**; local.
- [documentation-for-llms](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/documentation-for-llms/SKILL.md) — **base**; não sincronizada.
- [documentation-publishing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/documentation-publishing/SKILL.md) — contextual; não sincronizada.
- [goal-management](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/goal-management/SKILL.md) — contextual; não sincronizada.
- [grounded-implementation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/grounded-implementation/SKILL.md) — **base**; local.
- [implementation-reality-verification](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/implementation-reality-verification/SKILL.md) — **base**; local.
- [lean-progressive-context](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lean-progressive-context/SKILL.md) — **base**; local.
- [orchestration-multi-agent](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/orchestration-multi-agent/SKILL.md) — contextual; não sincronizada.
- [project-documentation-architect](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/project-documentation-architect/SKILL.md) — contextual; local.
- [project-intelligence](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/project-intelligence/SKILL.md) — contextual; não sincronizada.
- [prompt-engineering](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/prompt-engineering/SKILL.md) — contextual; não sincronizada.
- [prumo-navigation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/prumo-navigation/SKILL.md) — **base**; não sincronizada.
- [traycer-orchestration](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/traycer-orchestration/SKILL.md) — contextual; não sincronizada.

### Acessibilidade, cognição e UX (20)

- [accessibility](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/accessibility/SKILL.md) — contextual; local.
- [contrast](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/contrast/SKILL.md) — contextual; local.
- [design-critique](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/design-critique/SKILL.md) — contextual; não sincronizada.
- [design-psychology](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/design-psychology/SKILL.md) — contextual; não sincronizada.
- [focus-management](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/focus-management/SKILL.md) — contextual; local.
- [information-architecture](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/information-architecture/SKILL.md) — contextual; não sincronizada.
- [interaction-design](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/interaction-design/SKILL.md) — contextual; não sincronizada.
- [interaction-research](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/interaction-research/SKILL.md) — contextual; não sincronizada.
- [keyboard-accessibility](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/keyboard-accessibility/SKILL.md) — contextual; local.
- [layout-patterns](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/layout-patterns/SKILL.md) — contextual; não sincronizada.
- [motion-accessibility](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/motion-accessibility/SKILL.md) — contextual; local.
- [prototyping](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/prototyping/SKILL.md) — contextual; não sincronizada.
- [responsive-design](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/responsive-design/SKILL.md) — contextual; não sincronizada.
- [screen-reader](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/screen-reader/SKILL.md) — contextual; local.
- [ui-ux-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ui-ux-review/SKILL.md) — contextual; não sincronizada.
- [user-flows](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/user-flows/SKILL.md) — contextual; não sincronizada.
- [ux-architecture](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ux-architecture/SKILL.md) — contextual; não sincronizada.
- [wireframe-styleguide](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/wireframe-styleguide/SKILL.md) — contextual; não sincronizada.
- [wireframing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/wireframing/SKILL.md) — contextual; não sincronizada.
- [zoom-reflow](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/zoom-reflow/SKILL.md) — contextual; local.

### Design visual, tipografia, vetores e identidade (19)

- [advertising-design](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/advertising-design/SKILL.md) — contextual; não sincronizada.
- [aesthetic-analysis](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/aesthetic-analysis/SKILL.md) — contextual; não sincronizada.
- [asset-reference-research](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/asset-reference-research/SKILL.md) — contextual; não sincronizada.
- [brand-identity](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/brand-identity/SKILL.md) — contextual; não sincronizada.
- [color-science](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/color-science/SKILL.md) — contextual; não sincronizada.
- [component-specification](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/component-specification/SKILL.md) — contextual; local.
- [design-research](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/design-research/SKILL.md) — contextual; não sincronizada.
- [design-system](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/design-system/SKILL.md) — contextual; local.
- [design-tokens](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/design-tokens/SKILL.md) — contextual; local.
- [generative-svg-art](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/generative-svg-art/SKILL.md) — contextual; não sincronizada.
- [icon-system](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/icon-system/SKILL.md) — contextual; não sincronizada.
- [marketing-collateral](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/marketing-collateral/SKILL.md) — contextual; não sincronizada.
- [motion-library](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/motion-library/SKILL.md) — contextual; não sincronizada.
- [svg-engineering](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/svg-engineering/SKILL.md) — contextual; não sincronizada.
- [typography-system](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/typography-system/SKILL.md) — contextual; não sincronizada.
- [visual-communication](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/visual-communication/SKILL.md) — contextual; não sincronizada.
- [visual-qa](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/visual-qa/SKILL.md) — contextual; não sincronizada.
- [visual-reference-research](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/visual-reference-research/SKILL.md) — contextual; não sincronizada.
- [visual-regression](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/visual-regression/SKILL.md) — contextual; não sincronizada.

### Arquitetura, implementação e qualidade (32)

- [api-contract-testing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/api-contract-testing/SKILL.md) — contextual; não sincronizada.
- [architecture-quality](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/architecture-quality/SKILL.md) — **base**; local.
- [benchmarking](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/benchmarking/SKILL.md) — contextual; local.
- [caching](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/caching/SKILL.md) — contextual; não sincronizada.
- [ci-cd](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ci-cd/SKILL.md) — contextual; não sincronizada.
- [clean-code](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/clean-code/SKILL.md) — **base**; local.
- [code-quality](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/code-quality/SKILL.md) — **base**; local.
- [code-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/code-review/SKILL.md) — contextual; local.
- [concurrency-quality](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/concurrency-quality/SKILL.md) — contextual; local.
- [containers](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/containers/SKILL.md) — contextual; não sincronizada.
- [dependency-management](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/dependency-management/SKILL.md) — contextual; não sincronizada.
- [error-handling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/error-handling/SKILL.md) — **base**; local.
- [game-ui-testing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/game-ui-testing/SKILL.md) — contextual; não sincronizada.
- [git-workflow](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/git-workflow/SKILL.md) — contextual; local.
- [github-ci-debug](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-ci-debug/SKILL.md) — contextual; não sincronizada.
- [github-issue-create](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-issue-create/SKILL.md) — contextual; não sincronizada.
- [github-issue-refine](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-issue-refine/SKILL.md) — contextual; não sincronizada.
- [github-issue-triage](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-issue-triage/SKILL.md) — contextual; não sincronizada.
- [github-pr-create](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-pr-create/SKILL.md) — contextual; não sincronizada.
- [github-pr-feedback](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-pr-feedback/SKILL.md) — contextual; não sincronizada.
- [github-pr-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-pr-review/SKILL.md) — contextual; não sincronizada.
- [github-release](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-release/SKILL.md) — contextual; não sincronizada.
- [github-repository](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/github-repository/SKILL.md) — contextual; não sincronizada.
- [memory-management](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/memory-management/SKILL.md) — contextual; local.
- [observability](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/observability/SKILL.md) — contextual; não sincronizada.
- [performance-native](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/performance-native/SKILL.md) — contextual; local.
- [performance-web](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/performance-web/SKILL.md) — contextual; não sincronizada.
- [refactoring](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/refactoring/SKILL.md) — contextual; local.
- [release-engineering](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/release-engineering/SKILL.md) — contextual; não sincronizada.
- [serialization](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/serialization/SKILL.md) — contextual; local.
- [state-management](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/state-management/SKILL.md) — contextual; local.
- [testing-quality](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/testing-quality/SKILL.md) — **base**; local.

### Engine, renderização, gráficos, assets e interfaces (18)

- [applied-mathematics-dsp](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/applied-mathematics-dsp/SKILL.md) — contextual; não sincronizada.
- [asset-pipeline](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/asset-pipeline/SKILL.md) — contextual; não sincronizada.
- [audio-engineering](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/audio-engineering/SKILL.md) — contextual; não sincronizada.
- [backend-api](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/backend-api/SKILL.md) — contextual; não sincronizada.
- [editor-tooling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/editor-tooling/SKILL.md) — contextual; local.
- [frontend-web](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/frontend-web/SKILL.md) — contextual; não sincronizada.
- [game-engine-architecture](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/game-engine-architecture/SKILL.md) — contextual; não sincronizada.
- [game-runtime](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/game-runtime/SKILL.md) — contextual; não sincronizada.
- [input-handling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/input-handling/SKILL.md) — contextual; local.
- [physics-collision](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/physics-collision/SKILL.md) — contextual; não sincronizada.
- [physics-testing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/physics-testing/SKILL.md) — contextual; não sincronizada.
- [playtest-automation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/playtest-automation/SKILL.md) — contextual; não sincronizada.
- [realtime-audio-dsp](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/realtime-audio-dsp/SKILL.md) — contextual; não sincronizada.
- [rendering-2d](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/rendering-2d/SKILL.md) — contextual; local.
- [rendering-3d](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/rendering-3d/SKILL.md) — contextual; não sincronizada.
- [scene-graph](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/scene-graph/SKILL.md) — contextual; local.
- [shaders](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/shaders/SKILL.md) — contextual; não sincronizada.
- [ui-implementation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ui-implementation/SKILL.md) — contextual; não sincronizada.

### Segurança, sandbox e integrações (28)

- [acp-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/acp-security/SKILL.md) — contextual; não sincronizada.
- [api-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/api-security/SKILL.md) — contextual; não sincronizada.
- [auth-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/auth-security/SKILL.md) — contextual; não sincronizada.
- [binary-security-mitigations](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/binary-security-mitigations/SKILL.md) — contextual; não sincronizada.
- [desktop-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/desktop-security/SKILL.md) — contextual; não sincronizada.
- [filesystem-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/filesystem-security/SKILL.md) — contextual; não sincronizada.
- [linux-kernel-isolation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/linux-kernel-isolation/SKILL.md) — contextual; não sincronizada.
- [mcp-integration](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/mcp-integration/SKILL.md) — contextual; local.
- [mcp-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/mcp-security/SKILL.md) — contextual; não sincronizada.
- [mcp-tooling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/mcp-tooling/SKILL.md) — contextual; local.
- [memory-safety-sanitizers](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/memory-safety-sanitizers/SKILL.md) — contextual; não sincronizada.
- [network-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/network-security/SKILL.md) — contextual; não sincronizada.
- [network-testing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/network-testing/SKILL.md) — contextual; não sincronizada.
- [plugin-architecture](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/plugin-architecture/SKILL.md) — contextual; local.
- [plugin-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/plugin-security/SKILL.md) — contextual; não sincronizada.
- [process-execution-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/process-execution-security/SKILL.md) — contextual; não sincronizada.
- [rpc-protocols](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/rpc-protocols/SKILL.md) — contextual; não sincronizada.
- [secrets-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/secrets-security/SKILL.md) — contextual; não sincronizada.
- [secure-coding](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/secure-coding/SKILL.md) — contextual; local.
- [security-agent-mcp](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/security-agent-mcp/SKILL.md) — contextual; não sincronizada.
- [security-authz-matrix](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/security-authz-matrix/SKILL.md) — contextual; não sincronizada.
- [security-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/security-review/SKILL.md) — contextual; local.
- [security-saas-isolation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/security-saas-isolation/SKILL.md) — contextual; não sincronizada.
- [security-threat-model](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/security-threat-model/SKILL.md) — contextual; não sincronizada.
- [supply-chain-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/supply-chain-security/SKILL.md) — contextual; não sincronizada.
- [threat-modeling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/threat-modeling/SKILL.md) — contextual; não sincronizada.
- [untrusted-project-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/untrusted-project-security/SKILL.md) — contextual; não sincronizada.
- [web-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/web-security/SKILL.md) — contextual; não sincronizada.

### Linguagens, compilação, análise de código (15)

- [ast-transformation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ast-transformation/SKILL.md) — contextual; não sincronizada.
- [bytecode-vm-architecture](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/bytecode-vm-architecture/SKILL.md) — contextual; não sincronizada.
- [clang-context-indexing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/clang-context-indexing/SKILL.md) — contextual; não sincronizada.
- [compiler-development](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/compiler-development/SKILL.md) — contextual; não sincronizada.
- [compiler-frontend-engineering](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/compiler-frontend-engineering/SKILL.md) — contextual; não sincronizada.
- [compiler-ir-optimization](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/compiler-ir-optimization/SKILL.md) — contextual; não sincronizada.
- [fuzz-grammar-testing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/fuzz-grammar-testing/SKILL.md) — contextual; não sincronizada.
- [language-tooling](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/language-tooling/SKILL.md) — contextual; não sincronizada.
- [roslyn-context-indexing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/roslyn-context-indexing/SKILL.md) — contextual; não sincronizada.
- [runtime-memory-gc](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/runtime-memory-gc/SKILL.md) — contextual; não sincronizada.
- [rust-analyzer-context-indexing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/rust-analyzer-context-indexing/SKILL.md) — contextual; local.
- [surface-protocol-conformance](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/surface-protocol-conformance/SKILL.md) — contextual; não sincronizada.
- [tree-sitter-context-indexing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/tree-sitter-context-indexing/SKILL.md) — contextual; não sincronizada.
- [ts-morph-context-indexing](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/ts-morph-context-indexing/SKILL.md) — contextual; não sincronizada.
- [type-system-theory](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/type-system-theory/SKILL.md) — contextual; não sincronizada.

### Outros especialistas e atividades condicionais (9)

- [competitor-analysis](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/competitor-analysis/SKILL.md) — contextual; não sincronizada.
- [computational-chemistry-materials](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/computational-chemistry-materials/SKILL.md) — contextual; não sincronizada.
- [computational-physics](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/computational-physics/SKILL.md) — contextual; não sincronizada.
- [database-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/database-review/SKILL.md) — contextual; não sincronizada.
- [marketing-collateral](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/marketing-collateral/SKILL.md) — contextual; não sincronizada.
- [multiplayer-networking](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/multiplayer-networking/SKILL.md) — contextual; não sincronizada.
- [realtime-synchronization](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/realtime-synchronization/SKILL.md) — contextual; não sincronizada.
- [web-security](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/web-security/SKILL.md) — contextual; não sincronizada.
- [website-forensics](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/website-forensics/SKILL.md) — contextual; não sincronizada.

### Linguagens de programação e infraestrutura (seleção por arquivo) (33)

- [lang-asm](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-asm/SKILL.md) — contextual; não sincronizada.
- [lang-bash](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-bash/SKILL.md) — contextual; não sincronizada.
- [lang-c](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-c/SKILL.md) — contextual; local.
- [lang-c3](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-c3/SKILL.md) — contextual; não sincronizada.
- [lang-cpp](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-cpp/SKILL.md) — contextual; local.
- [lang-csharp](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-csharp/SKILL.md) — contextual; não sincronizada.
- [lang-css](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-css/SKILL.md) — contextual; não sincronizada.
- [lang-d](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-d/SKILL.md) — contextual; não sincronizada.
- [lang-dart](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-dart/SKILL.md) — contextual; não sincronizada.
- [lang-dockerfile](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-dockerfile/SKILL.md) — contextual; não sincronizada.
- [lang-elixir](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-elixir/SKILL.md) — contextual; não sincronizada.
- [lang-glsl](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-glsl/SKILL.md) — contextual; não sincronizada.
- [lang-go](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-go/SKILL.md) — contextual; não sincronizada.
- [lang-graphql](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-graphql/SKILL.md) — contextual; não sincronizada.
- [lang-hlsl](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-hlsl/SKILL.md) — contextual; não sincronizada.
- [lang-html](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-html/SKILL.md) — contextual; não sincronizada.
- [lang-java](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-java/SKILL.md) — contextual; não sincronizada.
- [lang-javascript](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-javascript/SKILL.md) — contextual; não sincronizada.
- [lang-json](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-json/SKILL.md) — contextual; não sincronizada.
- [lang-kotlin](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-kotlin/SKILL.md) — contextual; não sincronizada.
- [lang-lua](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-lua/SKILL.md) — contextual; não sincronizada.
- [lang-odin](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-odin/SKILL.md) — contextual; não sincronizada.
- [lang-php](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-php/SKILL.md) — contextual; não sincronizada.
- [lang-python](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-python/SKILL.md) — contextual; não sincronizada.
- [lang-ruby](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-ruby/SKILL.md) — contextual; não sincronizada.
- [lang-rust](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-rust/SKILL.md) — **base**; local.
- [lang-sql](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-sql/SKILL.md) — contextual; não sincronizada.
- [lang-swift](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-swift/SKILL.md) — contextual; não sincronizada.
- [lang-terraform](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-terraform/SKILL.md) — contextual; não sincronizada.
- [lang-typescript](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-typescript/SKILL.md) — contextual; não sincronizada.
- [lang-wgsl](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-wgsl/SKILL.md) — contextual; não sincronizada.
- [lang-yaml](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-yaml/SKILL.md) — contextual; não sincronizada.
- [lang-zig](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/lang-zig/SKILL.md) — contextual; não sincronizada.

### Demais especialidades (inventário residual sem omissões) (1)

- [playwright-ui](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/skills/playwright-ui/SKILL.md) — contextual; não sincronizada.


## 3. Recipes (20) — inventário integral

Recipes não são agentes nem skills: orquestram passos e gates definidos no `recipe.json`; abrir também esse arquivo antes de execução.

| Receita | Pertinência |
|---|---|
| [architecture-change](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/architecture-change/RECIPE.md) | Aplicável sob demanda |
| [brand-creation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/brand-creation/RECIPE.md) | Especializada / complementar |
| [bug-fix](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/bug-fix/RECIPE.md) | Aplicável sob demanda |
| [compiler-change](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/compiler-change/RECIPE.md) | Especializada / complementar |
| [compiler-conformance](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/compiler-conformance/RECIPE.md) | Especializada / complementar |
| [design-system-foundation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/design-system-foundation/RECIPE.md) | Aplicável sob demanda |
| [documentation-refactor](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/documentation-refactor/RECIPE.md) | Aplicável sob demanda |
| [engine-renderer](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/engine-renderer/RECIPE.md) | Aplicável sob demanda |
| [feature-standard](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/feature-standard/RECIPE.md) | Aplicável sob demanda |
| [github-issue](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/github-issue/RECIPE.md) | Especializada / complementar |
| [icon-library-creation](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/icon-library-creation/RECIPE.md) | Especializada / complementar |
| [marketing-campaign](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/marketing-campaign/RECIPE.md) | Especializada / complementar |
| [multiplayer-feature](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/multiplayer-feature/RECIPE.md) | Especializada / complementar |
| [project-bootstrap](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/project-bootstrap/RECIPE.md) | Especializada / complementar |
| [release](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/release/RECIPE.md) | Aplicável sob demanda |
| [security-audit-release](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/security-audit-release/RECIPE.md) | Especializada / complementar |
| [security-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/security-review/RECIPE.md) | Aplicável sob demanda |
| [ui-feature](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/ui-feature/RECIPE.md) | Aplicável sob demanda |
| [ui-review](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/ui-review/RECIPE.md) | Aplicável sob demanda |
| [web-feature](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/recipes/web-feature/RECIPE.md) | Especializada / complementar |

## 4. Matriz de gatilhos importantes para Petunia

| Gatilho real da tarefa | Agents essenciais por tarefa | Skills iniciais adicionais |
|---|---|---|
| Implementar algoritmos de path, Shape Builder, Snap | `engine-engineer`, `editor-engineer`, `tester` | `applied-mathematics-dsp`, `scene-graph`, `lang-rust`, `testing-quality`, `benchmarking` |
| Paint, tiles COW, brushes, import/export | `engine-engineer`, `renderer-engineer`, `security-reviewer` | `rendering-2d`, `memory-management`, `serialization`, `supply-chain-security` |
| QML/Qt, toolbars, painéis, estados de GUI | `ux-architect`, `ui-component-engineer`, `accessibility-reviewer` | `cognitive-clarity`, `interaction-design`, `focus-management`, `keyboard-accessibility`, `ui-implementation`, `screen-reader` |
| Neurodivergência, conforto cognitivo e estrutura | `ux-architect`, `design-researcher`, `accessibility-reviewer` | `cognitive-clarity`, `design-psychology`, `user-flows`, `visual-qa`, `zoom-reflow`, `motion-accessibility` |
| Tokens, ícones, contraste e aparência | `design-system-engineer`, `svg-artist`, `visual-identity-auditor` | `design-system`, `typography-system`, `icon-system`, `contrast`, `aesthetic-analysis` |
| Documentação, handoff e LLM context | `documentation-maintainer`, `architect`, `reviewer` | `documentation-for-llms`, `project-documentation-architect`, `prumo-navigation`, `context-optimization` |
| Plugins, MCP, serialização, arquivos não confiáveis | `security-architect`, `security-reviewer`, `isolation-auditor` | `plugin-security`, `mcp-security`, `serialization`, `fuzz-grammar-testing`, `untrusted-project-security` |
| Release, packaging, CI | `devops-engineer`, `release-verifier`, `quality-reviewer` | `ci-cd`, `release-engineering`, `git-workflow`, `supply-chain-security` |
| Features de backend, rede, DB, áudio e compiladores | especialista correspondente | carregar somente quando escopo realmente envolver subsistema correspondente |

**Não existe agent `neurodivergence-reviewer` no snapshot consultado**: essa revisão é realizada por combinação de `ux-architect`, `accessibility-reviewer`, `design-researcher` e skills `cognitive-clarity`/`design-psychology`, sem inventar um ID. **`frontend-web` não substitui Qt/QML**; podem ser aproveitados princípios de semântica, focus e componentes, adaptados para o toolkit desktop.

## 5. Como obter e atualizar corretamente

1. Abrir [Prumo upstream no commit fixo](https://github.com/poppy-lat/prumo/tree/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce/) e ler `AGENT.md`, `SKILL.md` ou `RECIPE.md` relevantes **com seu manifest**. Para recipes ler `recipe.json` (agentes, preconditions, gates, capabilities, handoffs).
2. Consultar a [documentação de instalação do Prumo](https://github.com/poppy-lat/prumo/blob/e213260c99d22c89bf31890ec595a89725031c71/docs/manual/installation.md) e o `prumo --help` da versão efetivamente instalada; **não inventar flags para sincronizar**.
3. O Petunia já usa `.agents/agents`, `.agents/skills`, `.ai/skills` e `prumo.json`. Antes de sincronizar o catálogo inteiro, executar auditoria de versões/manifests, detectar conflito de arquivos customizados e confirmar quais adapters a CLI gerencia.
4. Importar/sincronizar somente os assets apropriados pela CLI/versionamento verificado, **sem sobrescrever customizações locais ou logs da workforce**. Esta página garante inventário completo, mas a disponibilidade física permanece parcial até sincronização verificada.
5. Registrar `upstream_sha`, quantidade, arquivos implantados, diferenças de manifests e testes de validação; atualizar esta página se o Prumo mudar.
6. Durante execução, selecionar **o mínimo de skills necessário** e chamar especialistas adicionais nas revisões de UI/segurança/desempenho conforme os gates.

**Versões de documentação:** Petunia usa pt-BR como idioma canônico. Uma skill genérica do Prumo que sugira `en-US` como fonte principal deve ser adaptada a esta política **sem** inverter a autoridade do projeto.

[Índice de diretivas](#/docs/07-agents/index.md) · [Orquestração](#/docs/07-agents/orchestration.md) · [Acessibilidade](#/docs/07-agents/accessibility.md).
