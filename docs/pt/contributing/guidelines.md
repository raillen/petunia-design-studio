# Diretrizes de contribuição

Como abrir PRs e atender aos padrões de código. Versão curta: diffs pequenos, portões verdes, docs na mesma mudança.

## 1. Antes de codar

- Monte o microcontexto: objetivo, escopo, IDs exatos de página/ADR, crates, capacidades, invariantes. Nunca jogue credenciais ou artwork em prompts ou pacotes.
- Confira o status de escopo. Um "futuro/planejado" sozinho não autoriza nada — confirme `V1 Required` / `Milestone Required` / `Post-V1 Candidate` / `Research` / `Open ADR`.
- Para trabalho de UI: sem UI falsa. Botão sem ação, `todo!()`, callback vazio ou placeholder é proibido — implemente, desabilite com motivo, oculte ou marque experimental.

## 2. Padrões de código

- `cargo fmt --all` antes de cada push; `cargo clippy --workspace --all-targets -- -D warnings` deve ficar silencioso.
- Crates de domínio nunca importam tipos de toolkit GUI (`cargo run -p xtask -- architecture` impõe isso).
- Toda mutação flui UI/Atalho/Plugin/MCP → Ação → Comando → DocumentMutator → ChangeSet.
- Só IDs estáveis tipados (`ObjectId`, `SurfaceId`, …); nunca índices de Vec, ponteiros ou handles como identidade.
- Não destrutivo por padrão: Consolidar / Expandir / Rasterizar / Converter-em-curvas são operações explícitas do usuário.

## 3. Como abrir um PR

```bash
# Rode os portões localmente antes do push
cargo run -p xtask -- verify
```

1. Uma preocupação por PR; mantenha diffs revisáveis (de preferência < ~400 linhas).
2. Inclua: testes focados + evidência de gauntlet, prova headless-first, prova de detach onde aplicável.
3. Atualize Atlas, registro de ADRs e referências geradas **na mesma mudança** que alterar um contrato.
4. Atualize os docs **EN e pt-BR** na mesma mudança (veja [Traduções](/pt/contributing/translations)). O CI falha em drift.

## 4. Expectativas de revisão

- Revisores checam conformidade arquitetural, impacto de segurança e feedback construtivo — não estilo (isso é do fmt/clippy).
- Conflitos entre orientação de agentes e atlas/ADRs canônicos resolvem-se a favor dos atlas/ADRs; registre o conflito no PR.
