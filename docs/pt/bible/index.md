# Constituição (SSOT)

O núcleo inegociável do Petunia Design Studio. Todo o resto — ferramentas, painéis, shells, docs — deriva desta página. Em conflito, esta página vence; o conflito é registrado, nunca resolvido em silêncio.

## I. Identidade

- **I-1.** O produto é **Petunia Design Studio**. Nomes legados (Aubrieta) são história só-leitura, nunca emitidos.
- **I-2.** O formato nativo é **`.ptnd`** (pacote ZIP aberto). `.aubrieta`/`.aubri` legados abrem; salvamentos novos emitem `.ptnd`.
- **I-3.** O inglês é o idioma canônico da documentação; o [pt-BR](/pt/bible/) espelha cada página 1:1.

## II. Pipeline de mutação (inviolável)

- **II-1.** Toda mutação flui **UI / Atalho / Plugin / MCP → Ação → Comando → DocumentMutator → ChangeSet.**
- **II-2.** Nada toca o armazenamento do documento diretamente. `DocumentMutator` é o escritor único.
- **II-3.** Um gesto = um undo. Preview não escreve histórico; `Up` commita uma vez.

## III. Não destruição

- **III-1.** A edição é não destrutiva por padrão via **EffectChain** ordenada tipada ([ADR-001](/pt/developers/adr/ADR-001-effect-chain)).
- **III-2.** Consolidar (bake), Expandir, Rasterizar e Converter-em-curvas são **operações explícitas do usuário**. Sem bake silencioso, sem deleção silenciosa.
- **III-3.** Base vs. avaliado: ferramentas editam a geometria base; render/hit/seleção/exportação leem o resultado avaliado.

## IV. Fronteiras

- **IV-1.** Crates de domínio nunca importam tipos de toolkit GUI. A UI recebe DTOs/view-models; envia `ActionRequest`/`CommandRequest` pela ponte.
- **IV-2.** Features compõem-se via registros de capacidade. Capacidade ausente é estado normal desabilitado **com motivo** — nunca pânico, nunca botão morto.
- **IV-3.** Identidade é tipada e estável (`ObjectId`, `SurfaceId`, `ResourceId`, …). Índice de Vec, ponteiro ou handle nunca é identidade.

## V. Honestidade

- **V-1.** Sem UI falsa: comportamento não implementado é implementado, desabilitado-com-motivo, oculto ou marcado experimental.
- **V-2.** Compilar ≠ implementar. Teste pulado ≠ passe. Exportação degradada (ex.: opacidade amostrada) é registrada, nunca silenciosa.
- **V-3.** Toda feature carrega status de escopo explícito. Um futuro/planejado/“later” sozinho não autoriza nada.

## VI. Verificação

- **VI-1.** Pronto = testes focados + gauntlets aplicáveis + evidência registrada (headless-first, prova de detach, token/a11y onde há UI).
- **VI-2.** Contratos mudam com seus docs: Atlas, registro de ADRs, referências **e páginas EN+pt-BR** atualizam na mesma mudança.

Primeira formalização da seção VI como spec imposta por build: [SPEC-001](/pt/bible/SPEC-001).
