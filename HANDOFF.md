# Handoff — Petunia Design Studio

Documento de retomada. Escrito para continuar o trabalho em outra máquina
sem perder contexto. Última atualização: commit `252eead`.

---

## 1. Estado em uma linha

O rebranding Aubrieta → Petunia Design Studio, o formato `.PTND` e a migração
de namespace estão **prontos e testados**. O pipeline de ações
(Action → Command → DocumentMutator → ChangeSet) tem **57 ações resolvíveis**.
A UI Slint ainda **não está ligada a esse pipeline** — esse é o próximo salto.

---

## 2. Onde o trabalho está

| | |
|---|---|
| Repositório | `git@github.com:raillen/petunia-design-studio.git` |
| Branch de trabalho | `refactor/petunia-design-studio` |
| Base | `dad0904` (branch `slint_ui`) |
| HEAD | `252eead` |
| Commits desta sessão | 12 |

---

## 3. Hardware: leia isto antes de compilar

A máquina anterior tinha **5,5 GB de RAM**, o que fazia o *link* do binário de
teste do app Slint morrer com `ld: signal 9 [Killed]` (OOM). Isso bloqueava
toda validação da UI.

**Solução que funcionou** (linkou em 21 min):

```bash
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design --no-run -j 1
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design
```

Resultado: `tests::slint_app_smoke_test_headless ... ok`

Numa máquina com 16 GB+ provavelmente não é necessário, mas manter o hábito de
`-j 1` em builds do app Slint evita picos.

> **Nota:** o alvo de build está em `~/.cargo-targets/aubrieta/`, não em
> `target/`. Confira `CARGO_TARGET_DIR` ou `.cargo/config.toml`.

---

## 4. Como validar (comandos exatos)

```bash
# Testes de todo o workspace, exceto o app (rápido, ~2 min)
cargo test --workspace --exclude petunia-design

# App Slint (ver seção 3 para as variáveis de ambiente)
CARGO_PROFILE_DEV_DEBUG=0 cargo test -p petunia-design

# Lint
cargo clippy --workspace --all-targets

# Guardrail arquitetural (crates de domínio não importam toolkit de UI)
cargo run -p xtask -- arch
```

**Estado na última execução:**

| Gate | Resultado |
|---|---|
| `cargo test --workspace --exclude petunia-design` | **231 passed / 0 failed** |
| `cargo test -p petunia-design` | **1 passed / 0 failed** |
| `cargo clippy --workspace --all-targets` | **0 warnings** |
| `cargo run -p xtask -- arch` | **0 arestas proibidas** |

---

## 5. O que foi feito nesta sessão (12 commits)

### `149ce10` — Identidade, formato, namespace

- 16 crates `aubrieta_*` → `petunia_design_*`; apps `aubrieta-slint` →
  `petunia-design`, `aubrieta-cli` → `petunia-design-cli`.
- Tipos: `AubrietaError` → `PetuniaError`, `AubrietaGuiBridge` →
  `PetuniaDesignGuiBridge`, `AubrietaShell` → `PetuniaShell`,
  `AubrietaPackage` → `PtndPackage`, `AubrietaSlintState` → `PetuniaSlintState`.
- **Formato `.PTND`** em `crates/petunia_design_io/src/package.rs`:
  - `MEDIA_TYPE = "application/vnd.petunia-design-studio.project+zip"`
  - `NATIVE_SUFFIX = "ptnd"`, `NATIVE_EXTENSION = "PTND"`,
    `NATIVE_EXTENSION_DISPLAY = ".PTND"`
  - `LEGACY_SUFFIXES = ["aubrieta", "aubri"]` — **somente leitura**
  - `PackageFormat { Ptnd, Legacy }` com `requires_save_as()`
  - `OpenedPackage { document, format }`
  - `check_writable_suffix()` recusa legado; `check_readable_suffix()` aceita
- **Namespace `ptnd.*`** em `crates/petunia_design_foundation/src/namespace.rs`:
  - `normalize_legacy_namespace()` — `aubrieta.*` → `ptnd.*`
  - `normalize_action_id()` — `ptnd.<domínio>.*` → `ptnd.action.<domínio>.*`
  - Ligados em `resolve_color_to_rgb`, `DocumentSession::dispatch_action`,
    `Document::from_json`
- API Lua: global `ptnd` canônico + alias `aubrieta` somente leitura.

### `8dcb0f7` — Tokens Slint + surface registry

- `apps/petunia-design/ui/tokens.slint` — paleta canônica do doc `08.35`
  (surfaces, borders, texto, accent Bloom, studio.design/photo, status,
  geometria, raios, spacing, tipografia, focus).
- 276 literais de cor migrados no `app.slint`. Cores de *artwork* continuam
  literais — exceção explícita do contrato.
- `crates/petunia_design_application/src/surfaces.rs` — registry
  machine-readable do doc `15.G`: 108 superfícies com
  `id / kind / scope / status / label / action / shortcut`.

### `20887c0` → `252eead` — Reconciliação e implementação de ações

O registry **super-declarava** 17 superfícies como funcionais. A auditoria
contra o inventário real (extraído do código, não escrito à mão) revelou três
problemas estruturais:

1. **Constante não é comportamento.** `LIVE_ACTIONS` extraía de
   `actions.rs`, mas uma constante pode existir sem nunca ser despachada.
   Agora só conta *caminho de resolução*: braços de `dispatch_action`, a tabela
   `ToolKind::action_id`, e braços do host em `gui_bridge.rs`.
2. **Entradas `kind = Action` escapavam da verificação** (tinham
   `action: None`). Agora o próprio `id` precisa resolver.
3. **`ptnd.action.*` era só aspiração.** O código usava `ptnd.<domínio>.*`.
   Migrado para a gramática canônica com shim de leitura.

Ações implementadas e despachadas nesta sessão:

| Ação | Arquivo |
|---|---|
| `view.zoom_in` / `zoom_out` / `zoom_100` / `fit_surface` | `session.rs` |
| `view.toggle_rulers` / `toggle_snapping` | `session.rs` |
| `file.save` / `file.save_as` | `session.rs` |
| `file.new` / `file.open` | `gui_bridge.rs` (substituem a sessão) |
| `edit.undo` / `edit.redo` | `session.rs` |
| `edit.duplicate` | `session.rs` |
| `object.hide` / `object.lock` | `session.rs` |
| `object.arrange.front` / `arrange.back` | `session.rs` |
| `object.group` / `object.ungroup` | `session.rs` |

### Mudança arquitetural: `ViewportCamera`

Estava em `petunia_design_shell::canvas::camera` apesar de ser geometria pura.
Isso impedia que `ptnd.action.view.*` existisse — o shell não pode ser chamado
pela camada de aplicação. Movida para
`petunia_design_application::view_camera` com um wrapper `ViewState`
(câmera + réguas + snapping). O shell apenas re-exporta.

---

## 6. Bugs reais encontrados e corrigidos

Vale ler — foram todos encontrados pelos próprios testes, não por inspeção.

1. **`with_native_extension` devolvia caminho legado inalterado.**
   Efeito: `Save As` num projeto `.aubrieta` era recusado pelo
   `save_package`. O usuário ficava **sem conseguir salvar**. Agora o sufixo é
   atualizado no lugar (`Old.aubrieta` → `Old.PTND`) e o arquivo original
   permanece intocado.

2. **Gerador de IDs só conhecia identidades que ele mesmo emitira.**
   Duplicar um objeto criado pela lane de comando retornava um ID já existente.
   Corrigido com `IdGenerator::observe` + reconciliação a cada commit
   (`observe_document_identities`).

3. **`arrange` confiava em `active_surface`.** Uma sessão pode ter seleção sem
   superfície ativa, e a ação silenciosamente não fazia nada. Agora a
   superfície é derivada do objeto, e seleções multi-superfície são rejeitadas
   em vez de meio-arranjadas.

### Suposições minhas derrubadas pelos testes

- Sessão nova **não** tem superfície (`DocumentSession::new`).
- `current_revision` é contador **monotônico**; undo/redo o avançam. Comparar
  revisões não prova round-trip — compare estado do documento.
- `ptnd.action.surface.create` e `ptnd.action.object.create` são constantes
  **sem** braço de dispatch.
- `ungroup` **remove** o container vazio (comportamento correto do mutator).
- O registry tinha ids divergentes (`arrange_front` vs `arrange.front`).

---

## 7. O que falta implementar

### Ações declaradas sem comportamento (3 de 57)

| Ação | Bloqueio real |
|---|---|
| `file.export` | pipeline de export (PDF/PNG/SVG) inexistente |
| `file.place` | importer de assets ausente |
| `view.command_palette` | overlay Slint + índice de comandos |

O gap canônico está em `DECLARED_NOT_LIVE`, em `surfaces.rs`, e é verificado
por teste — não pode voltar a crescer silenciosamente.

### Blocos grandes — nenhuma seção de `15.F` tocada

| Área | Estado | Observação |
|---|---|---|
| Menu bar real vindo do registry | ausente | **maior salto de valor**: sem isso as 57 ações wired não são acionáveis pelo usuário |
| Geometria da shell | tokens criados, **não aplicados** | ambíguo por valor: `28px` serve a `menu-row-height`, `panel-section-header` e `layer-row-height`. Exige mapeamento site-a-site por nome de componente |
| Docking real / splitter / dock inferior | ausente | |
| Painéis Cor / Swatches / Assets / Navigator | ausentes | |
| Acessibilidade (`08.36`) | não iniciada | |
| i18n EN + pt-BR sincronizado | não iniciada | |
| Conformidade visual Slint de `15.F` | **desbloqueada** | ver seção 3 |
| Refatoração dos 58 docs | não iniciada | inventário em `.prumo/artifacts/DOCUMENTATION_INVENTORY.md` |
| Performance / RAM / VRAM | não iniciada | |
| Security corpus / fuzz | não iniciada | |

**Cobertura: ~5,5% das 220 seções de `15.F`.**

---

## 8. Ordem sugerida para retomar

1. **Menu bar dirigido pelo registry.** As 57 ações existem mas o usuário não
   alcança nenhuma. O registry já tem `label` (TextId) e `shortcut` por
   entrada — dá para gerar o menu a partir dele e ganhar um teste que prova
   que toda ação wired é acionável.
2. **Geometria da shell**, identificando cada barra por nome de componente.
3. **`file.export`** — maior bloco restante de ações declaradas.

---

## 9. Mapa dos arquivos-chave

| Arquivo | Papel |
|---|---|
| `crates/petunia_design_application/src/surfaces.rs` | **Registry `15.G`.** 108 superfícies. Onde ver o que falta: 39 `Absent` com razão, 5 `Disabled`, 64 `Wired` |
| `crates/petunia_design_foundation/src/namespace.rs` | Mapas de leitura legada (`aubrieta.*`, `ptnd.<domínio>.*`) |
| `crates/petunia_design_io/src/package.rs` | Política `.PTND` / legado, `OpenedPackage`, `PackageFormat` |
| `crates/petunia_design_application/src/session.rs` | `dispatch_action` — a maioria das ações |
| `crates/petunia_design_shell/src/bridge/gui_bridge.rs` | Ações que substituem a sessão (`file.new`, `file.open`) |
| `crates/petunia_design_application/src/view_camera.rs` | `ViewportCamera` + `ViewState` |
| `apps/petunia-design/ui/tokens.slint` | Paleta e geometria canônicas (`08.35`) |
| `apps/petunia-design/ui/app.slint` | UI Slint (~2100 linhas) |
| `.prumo/artifacts/REPOSITORY_INVENTORY.md` | Inventário do repo |
| `.prumo/artifacts/DOCUMENTATION_INVENTORY.md` | Inventário dos 126 docs |
| `petunia-design-studio/` | 126 docs normativos; série **15.A–15.H** é o contrato |

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
