# Workforce Prumo — catálogo integral de 39 agents, 189 skills e 20 recipes

**Snapshot de origem:** [poppy-lat/prumo@e213260c99d2](https://github.com/poppy-lat/prumo/tree/e213260c99d22c89bf31890ec595a89725031c71/src/prumo/resources/workforce). **Inventário integral extraído dos arquivos reais** `AGENT.md`, `SKILL.md` e `RECIPE.md`, em 2026-10-08. Esta página é índice para leitura progressiva: **não carregar todas as listas na mesma sessão**.

## Resumo verificável

| Tipo | Prumo upstream | Já presente no checkout Petunia | Ainda não sincronizado |
|---|---:|---:|---:|
| Agents | **39** | **19** | **20** |
| Skills | **189** | **44** em `.ai/skills` | **145** |
| Recipes | **20** | **0** sob `.agents`/`.ai` nas rotas de recipes inspecionadas | **20** |

**Source of truth** da lista: arquivos da árvore real do Prumo no commit fixo, não exemplos narrativos das páginas `docs/workforce/*.md` que podem mostrar IDs ilustrativos diferentes do catálogo instalado. No Petunia `.agents/agents` possui 19 agentes e `.agents/skills` e `.ai/skills` têm um subconjunto; evitar escrever "tudo instalado" porque a cópia atual é parcial.

**Política de carga:** base de skills sempre relevante + especialistas **por gatilho**. Não carregar as 189 skills simultaneamente. "Base" significa leitura/enforcement por orquestrador, não ativar todos os especialistas em cada edição trivial.


## Acesso ao catálogo completo por tipo e especialidade

- [Todos os 39 agentes — responsabilidades, estado local, perfis e fonte](#/docs/07-agents/workforce-agents.md).
- [Todas as 20 recipes — fluxos Prumo para cada tipo de tarefa](#/docs/07-agents/workforce-recipes.md).
- [Skills — Leitura, documentação, prompts e colaboração](#/docs/07-agents/workforce-skills-navigation.md) (16 skills).
- [Skills — Acessibilidade, cognição e UX](#/docs/07-agents/workforce-skills-accessibility-ux.md) (20 skills).
- [Skills — Design visual, tipografia, vetores e identidade](#/docs/07-agents/workforce-skills-visual-design.md) (19 skills).
- [Skills — Arquitetura, implementação e qualidade](#/docs/07-agents/workforce-skills-engineering-quality.md) (32 skills).
- [Skills — Engine, renderização, gráficos, assets e interfaces](#/docs/07-agents/workforce-skills-graphics-rendering.md) (18 skills).
- [Skills — Segurança, sandbox e integrações](#/docs/07-agents/workforce-skills-security-integration.md) (28 skills).
- [Skills — Linguagens, compilação, análise de código](#/docs/07-agents/workforce-skills-language-tooling.md) (15 skills).
- [Skills — Outros especialistas e atividades condicionais](#/docs/07-agents/workforce-skills-other-specialists.md) (7 skills).
- [Skills — Linguagens de programação e infraestrutura (seleção por arquivo)](#/docs/07-agents/workforce-skills-programming-languages.md) (33 skills).
- [Skills — Demais especialidades (inventário residual sem omissões)](#/docs/07-agents/workforce-skills-remaining.md) (1 skills).

**Cobertura:** estes dez grupos contêm **189 IDs únicos**, sem omissões; tags de ativação, disponibilidade e links individuais para `SKILL.md` preservados em cada grupo. Não confundir a lista oficial com nomes ilustrativos de docs narrativas no upstream.

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


[Diretivas — ponto de entrada](#/docs/07-agents/index.md) · [Orquestração](#/docs/07-agents/orchestration.md) · [Prumo oficial](https://github.com/poppy-lat/prumo).
