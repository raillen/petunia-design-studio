# ContextPack e handoff — contrato de comunicação entre agentes

**Resumo:** relatórios devem ser curtos, reproduzíveis e linkados à fonte. Não reproduzir dezenas de documentos em cada transferência. Tudo o que outro agente precisa deve caber num TaskPack de poucas referências verificáveis.

## 1. TaskPack de entrada

```yaml
task:
  goal_id: TBD  # não invente um ID
  title: "Implementar operação delimitada"
  mode: implementation  # audit | bugfix | ui | architecture | docs | security | migration
  status: PLANNED
scope:
  owner_crate: petunia-engine
  allowed_files: []
  non_goals: []
canonical_context:
  docs: # usar caminhos reais e seções exatas
    - website/docs/00-architecture/boundaries.md
    - website/docs/00-architecture/implementation-matrix.md
  adrs: []
  code_verified: []
  upstream_refs: []
requirements:
  invariants: []
  acceptance_criteria: []
  test_plan: []
workforce:
  orchestrator: explorer
  specialist_agents: []
  skills: [prumo-navigation, lean-progressive-context, grounded-implementation]
  recipe: feature-standard
constraints:
  security: []
  accessibility: []
  error_budget: null
  performance_budget: null
  forbidden_changes: ["replace PTND as canonical", "introduce Qt into Core"]
evidence_required: [code, test, review, docs]
```

**Os paths e IDs acima são modelos**, não prova de arquivos de implementação. No handoff de uma tarefa real, incluir commit/SHA, link da spec e path dos testes reais. Nunca transmitir credenciais ou tokens.

## 2. ContextPack de saída (reutilizável entre sessões)

```md
# PETUNIA HANDOFF — [goal]
- Snapshot de código / commit:
- Tarefa e status: PROPOSED | APPROVED | SPECIFIED | PARTIAL | IMPLEMENTED | TESTED | VERIFIED
- Fonte canônica: [path#section, ADR]
- Arquivos realmente lidos: [...]
- Decisões fechadas relevantes: [...]
- Findings com evidência: [...]
- Mudanças feitas (paths + commits): [...]
- Licença/proveniência para imports OSS (se houver): [...]
- Comandos realmente executados + saída (não estimar): [...]
- Testes NÃO executados + motivo: [...]
- UX/A11y/Color/Render/Undo verificados: [...]
- Riscos e diffs arquiteturais:
- Pronto para próxima etapa:
- Menor ação seguinte:
```

## 3. Sinalização de evidência

`OBSERVED_IN_CODE`: li o source; `TEST_PRESENT`: teste existe, não rodou; `TEST_EXECUTED`: comando de teste rodou; `UPSTREAM_CLAIM`: alegação externa; `DECISION_APPROVED`: ADR/UX aceito; `SPECIFICATION_ONLY`: contrato sem código; `BLOCKED`: falta ambiente, permissão ou informação indispensável.

**Sem benchmarks inventados, promessas de execução futura, avaliações "parece funcionar" ou testes silenciosamente omitidos.** Exibir teste falhado ou não executado com clareza.

## 4. Registro de decisões e documentação DELTA

Ao alterar comportamento aprovado, editar a página canônica **e** atualizar dependentes com links, sem duplicar novo "padrão" em README paralelo. Para arquitetura fechada, abrir ADR/supersession antes de codificar comportamento incompatível.

Toda tarefa que muda runtime deve atualizar Matriz de Implementação se o status mudou; toda tarefa docs-only deve reportar verificação de links/manifest. Material externo é citado como suporte e não como instrução.

## 5. Resultado de revisão independente

```text
Reviewer:
Reviewed commit:
Contract references:
Severity 1 blockers:
Severity 2 issues:
Tests observed:
Evidence gaps:
Disposition: REJECT / REQUEST_CHANGES / APPROVE_WITH_EVIDENCE
```

O implementer não deve emitir um Review approval usando rótulo de reviewer não invocado. Se nenhum agente revisor estiver disponível, marcar `REVIEW_PENDING`.

[Autoridade](#/docs/07-agents/authority-reading.md) · [Workflow](#/docs/07-agents/implementation-workflow.md) · [Prompts](#/docs/07-agents/prompts.md).
