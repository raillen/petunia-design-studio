# Handoff — Petunia Design Studio

Documento de retomada. Escrito para continuar o trabalho em outra máquina
sem perder contexto. Última atualização: árvore de trabalho sobre `e90a6c9`
(**nada commitado ainda**).

---

## 1. Estado em uma linha

O rebranding Aubrieta → Petunia, o formato `.PTND`, a migração de namespace e
agora **a ligação da UI Slint ao pipeline de ações** estão prontos e testados:
menu bar rail gerado do registry, command palette, `file.export` com pipeline
real e a geometria canônica (08.35) aplicada site-a-site. Faltam
**file.place** (bloqueado por modelo), docking/painéis e i18n das strings
de chrome.

---

## 2. Onde o trabalho está

| | |
|---|---|
| Repositório | `git@github.com:raillen/petunia-design-studio.git` |
| Branch de trabalho | `refactor/petunia-design-studio` |
| Base | `dad0904` (branch `slint_ui`) |
| HEAD | `e90a6c9` |
| Alterações | **75 arquivos, +2665 / −1571, nenhum commit** |

> `cargo fmt --all` foi executado: parte do diff é só formatação. Os arquivos
> novos (sem formatação prévia) são:
> `export_service.rs`, `menus.rs`, `shell_strings.rs`, `shell/menu/`,
> `tests/menu_test.rs`.

---

## 3. Hardware: leia isto antes de compilar

A máquina anterior tinha **5,5 GB de RAM**, o que fazia o *link* do binário de
teste do app Slint morrer com `ld: signal 9 [Killed]` (OOM).

**Solução que funcionou** (linkou em 21 min):

```bash
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design --no-run -j 1
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design
```

Nesta máquina (30 GB) não é necessário: o app compila e testa direto. Manter o
hábito de `-j 1` em builds do app Slint evita picos em máquinas pequenas.

> **Nota:** o alvo de build está em `~/.cargo-targets/aubrieta/`, não em
> `target/`. Confira `CARGO_TARGET_DIR` ou `.cargo/config.toml` (o nome do
> diretório ainda é o antigo — inofensivo, mas vale renomear um dia).

---

## 4. Como validar (comandos exatos)

```bash
# Gate completo: fmt --check + clippy -D warnings + testes + arquitetura
cargo run -p xtask -- verify

# Gauntlet P00 (testes + arch + CLI conformance + fixtures + docs)
cargo run -p xtask -- gauntlet

# Só o app Slint
cargo test -p petunia-design
```

**Estado na última execução (esta sessão):**

| Gate | Resultado |
|---|---|
| `cargo run -p xtask -- verify` | **verde** (fmt, clippy, testes, arch) |
| `cargo test --workspace` | **297 passed / 0 failed** |
| `cargo test -p petunia-design` | **4 passed / 0 failed** |
| `cargo clippy --workspace --all-targets` | **0 warnings** |
| `cargo run -p xtask -- gauntlet` | **P00 slice green**, 0 arestas proibidas |

---

## 5. O que foi feito nesta sessão

### 5.1 Menu bar dirigido pelo registry (15.G)

A UI owns nenhum rótulo e nenhum action id. Ambos chegam de
`PetuniaDesignGuiBridge::query_menu_bar()`:

| Arquivo | Papel |
|---|---|
| `crates/petunia_design_application/src/menus.rs` | Modelo de menu derivado do registry: famílias, itens, `enabled` + `disabled_reason` |
| `crates/petunia_design_shell/src/menu/mod.rs` | Apresentação (labels resolvidos) para a UI |
| `crates/petunia_design_resources/src/shell_strings.rs` | Catálogo `TextId` → en-US + pt-BR de tudo o que o registry referencia |
| `crates/petunia_design_shell/tests/menu_test.rs` | Prova que toda ação *wired* é acionável e toda bloqueada sai desabilitada com motivo |

Regra derivada (15.F §2 "no fake UI"): **ação declarada e não ligada não
desaparece nem mente — aparece desabilitada com a razão.** É o caso de
`file.place`.

### 5.2 Command palette (Ctrl+K)

Overlay Slint + índice de comandos. Os itens oferecidos são **exatamente** os
itens de menu habilitados (`command_palette_offers_exactly_the_enabled_menu_items`),
então a paleta não pode divergir do menu.

### 5.3 `file.export` — pipeline real

`crates/petunia_design_application/src/export_service.rs`: documento →
SVG / PDF / PNG, com bytes de verdade. O diálogo de exportação da UI está
ligado a ele (o smoke test do app verifica que os três formatos produzem
bytes).

### 5.4 Geometria canônica aplicada (08.35)

Os tokens existiam desde `8dcb0f7` mas **não eram usados**. Nesta sessão o
mapeamento foi feito site-a-site por **nome de componente** (o handoff antigo
avisava: `28px` serve a três rows diferentes, então o valor não identifica o
token):

| Site | Token |
|---|---|
| barra de menus | `persona-row-height` (40, porque hospeda o controle de persona) |
| tabs de documento | `tab-strip-height + space-1` |
| context toolbar | `context-toolbar-height` |
| tool rail | `tool-rail-width`, `tool-rail-padding`, `tool-button-size` |
| dock direito | `right-dock-default-width` |
| rows de painel | `panel-tab-height`, `layer-row-height`, `property-row-height` |
| campos numéricos | `property-field-compact` |
| alvos eye/lock/reorder | `layer-tool-hit-size` (28) |
| status bar | `status-bar-height` |
| ícones | `icon-inline-size` (16) / `icon-toolbar-size` (18) |
| tipografia | `caption-size` / `small-size` / `body-size` / `dialog-section-size` |
| spacing/padding | `space-0..space-10`, `space-half`, `space-optical` |
| raios | `radius-micro` / `radius-control` / `radius-popover` / `radius-dialog` |

Corrigido de passagem: o controle de persona (34 px) **estourava** a barra de
menus (32 px).

**O que continua literal, de propósito** — documentado no fim de
`ui/tokens.slint`: geometria de documento (artboard, objetos, réguas, handles),
extensões de janela/diálogo, e micro-geometria sem métrica canônica (swatch de
24, chip de 20, micro tag de 16, hairline de 18). Inventar token para essas
seria fabricar contrato.

### 5.5 Testes novos que tornam o contrato verificável

Em `apps/petunia-design/src/main.rs` (`mod tests`):

1. `ui_references_only_declared_tokens` — todo `Tokens.x` usado existe em
   `tokens.slint` (nenhum token fantasma).
2. `ui_color_literals_are_confined_to_document_artwork` — hex bruto só dentro
   do bloco de artwork (08.21).
3. `ui_shell_rows_use_their_canonical_geometry_tokens` — os rows que o handoff
   chamava de ambíguos usam o token nomeado.

---

## 6. Bugs e armadilhas encontrados

- **Controle de persona estourava a linha.** A barra tinha 32 px e o controle
  34 px. Resolvido com a linha = `persona-row-height` (40), que é o valor
  canônico da linha de persona.
- **Substituição global quebra em ordem.** Trocar `font-size: 11px;` antes de
  `size: 11px;` importa: a segunda string é substring da primeira. O mesmo vale
  para `border-radius: 3px` vs. os blocos que a citavam como contexto.
- **`cargo fmt --all` era obrigatório.** O `xtask verify` roda
  `fmt --check`; a árvore vinha não formatada da sessão anterior.
- Suposições antigas que continuam válidas: sessão nova não tem superfície;
  `current_revision` é monotônico (undo/redo o avançam); `ungroup` remove o
  container vazio.

---

## 7. O que falta implementar

### Ações declaradas sem comportamento (1 de 65)

| Ação | Bloqueio real |
|---|---|
| `file.place` | **`ShapeKind` não tem variante de imagem.** Não existe onde um asset colocado viver. Desabilitada com motivo, não é bug de wiring. |

Fecharam nesta sessão: `file.export`, `view.command_palette`.
(`object.offset_path` e `object.slice_path` continuam em `DECLARED_NOT_LIVE`.)

### Blocos grandes — nenhuma seção de `15.F` fechada

| Área | Estado | Observação |
|---|---|---|
| Menu bar vindo do registry | **feito** | 65 caminhos de resolução despacháveis |
| Command palette | **feito** | |
| `file.export` | **feito** | SVG/PDF/PNG com bytes reais |
| Geometria da shell | **feito** | mapeamento site-a-site acima |
| i18n EN + pt-BR das strings de chrome | **parcial** | menu e paleta vêm do catálogo; tooltips, botões de painel e textos do diálogo de export continuam literais em `app.slint` |
| Docking real / splitter / dock inferior | ausente | |
| Painéis Cor / Swatches / Assets / Navigator | ausentes | |
| `file.place` (imagem como objeto) | bloqueado | falta variante de imagem no `ShapeKind` + importer |
| Acessibilidade (`08.36`) | não iniciada | |
| Conformidade visual Slint de `15.F` | parcial | precisa de QA visual com a app rodando |
| Refatoração dos docs legados | não iniciada | `docs/` (raiz, GPUI/Aubrieta) vs `petunia-design-studio/` (normativo, Slint/Petunia) coexistem |
| `PROJECT_STATE.md` | **desatualizado** | ainda diz "Aubrieta", aponta `docs/` como normativo e conta 179 testes |
| Performance / RAM / VRAM | não iniciada | |
| Security corpus / fuzz | não iniciada | |

Números do registry hoje: **113 superfícies** (79 `Wired`, 8 `Disabled`,
26 `Absent`) e **65 caminhos de ação despacháveis**.

---

## 8. Ordem sugerida para retomar

1. **i18n das strings de chrome.** É o maior gap *visível* que sobrou e o
   encanamento já existe (`shell_strings.rs` + `LocalizationService::with_shell_catalog`).
   Cada string literal de `app.slint` vira um `TextId` + `in property <string>`
   preenchido no `sync_ui_from_shell`.
2. **`file.place`**: adicionar variante de imagem ao `ShapeKind`, importer de
   asset e um nó de imagem no render. Sai daí também o painel Assets.
3. **Docking real + painéis restantes** (Cor, Swatches, Assets, Navigator).

---

## 9. Mapa dos arquivos-chave

| Arquivo | Papel |
|---|---|
| `crates/petunia_design_application/src/surfaces.rs` | **Registry `15.G`.** 113 superfícies, `LIVE_ACTIONS`, `DECLARED_NOT_LIVE`. Onde ver o que falta |
| `crates/petunia_design_application/src/menus.rs` | Modelo de menu derivado do registry (`enabled` + `disabled_reason`) |
| `crates/petunia_design_application/src/export_service.rs` | Pipeline SVG/PDF/PNG do `file.export` |
| `crates/petunia_design_application/src/session.rs` | `dispatch_action` — a maioria das ações |
| `crates/petunia_design_application/src/view_camera.rs` | `ViewportCamera` + `ViewState` |
| `crates/petunia_design_application/src/view_models.rs` | View-models (DTOs) para o shell |
| `crates/petunia_design_shell/src/bridge/gui_bridge.rs` | `query_menu_bar()`, ações que substituem a sessão |
| `crates/petunia_design_shell/src/menu/mod.rs` | Apresentação do menu |
| `crates/petunia_design_resources/src/shell_strings.rs` | Catálogo `TextId` en-US + pt-BR |
| `crates/petunia_design_resources/src/i18n.rs` | `LocalizationService`, `with_shell_catalog()` |
| `apps/petunia-design/ui/tokens.slint` | Paleta + geometria canônicas (08.35) e a política do que fica literal |
| `apps/petunia-design/ui/app.slint` | UI Slint (~2300 linhas) |
| `apps/petunia-design/src/main.rs` | Wiring + testes de contrato de token |
| `petunia-design-studio/` | **Docs normativos**; série **15.A–15.H** é o contrato |
| `.prumo/artifacts/` | Inventários de repositório e de documentação |

---

## 10. Regras que não podem ser quebradas

Extraídas do contrato (`15.A`–`15.H`) e do `AGENTS.md`:

- **Crates de domínio nunca importam toolkit de UI.** Verificado por
  `xtask arch`.
- **Toda mutação flui** UI/Shortcut/Plugin/MCP → Action → Command →
  DocumentMutator → ChangeSet. Nada toca o armazenamento do documento direto.
- **Nenhuma UI falsa.** Botão sem ação, `todo!()`, callback vazio ou
  placeholder são proibidos. Feature ausente = implementar, desabilitar com
  razão, esconder, ou marcar experimental.
- **Compilação ≠ implementação. Teste pulado ≠ PASS.** Usar
  `BLOCKED_EXTERNAL` quando o ambiente impedir, nunca fingir sucesso.
- **Rebrand não é search/replace cego.** Classificar cada ocorrência
  (produto atual / formato legado / ADR histórico / fixture de teste).
  `aubrieta.*` só é lido, nunca emitido.
- **Slint é a única UI.** egui, gpui, iced e desktop estão aposentados.
- **Não fazer force-push.** Preservar trabalho não commitado.

---

## 11. Restauração do sistema (ações tomadas na máquina antiga)

Para liberar memória antes do link OOM, foram parados processos que
**reiniciam sozinhos no boot** — nada foi desinstalado ou desabilitado
permanentemente:

```bash
# Se precisar religar sem reiniciar:
systemctl start docker docker.socket containerd   # docker (0 containers rodando)
localsend &        # app de compartilhamento
nm-applet &        # bandeja de rede
blueman-applet &   # bandeja de bluetooth
```

Nada disso é necessário para o projeto. Só foram liberados ~72 MB — o ganho
real veio de `CARGO_PROFILE_DEV_DEBUG=0`.
