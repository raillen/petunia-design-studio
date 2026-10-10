# Sessão e input

UI mantém estado efêmero e traduz input para intenção.

## StudioSession

O `StudioSession` atual mistura documento, history, active tool e viewport. Evoluir separando:

```rust
pub struct EditorSession {
    pub active_document: DocumentHandle,
    pub selection: SelectionState,
    pub tool: ToolSession,
    pub view: ViewState,
    pub transient: TransientEdits,
}
```

History pertence ao serviço de documento/Engine, não ao widget.

## Contextos de edição aprovados

O modelo híbrido Select/Vector Edit é [decisão aceita](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md). Além da ferramenta ativa, EditorSession precisa de **ContextStack**:

~~~text
Scene
↓ Group Isolation (se necessário)
↓ Vector Edit(targets, operation)
↓ operation transient
~~~

- **Scene / Select**: Object Selection, transforms e navegação na hierarquia.
- **Vector Edit**: NodeId/segment/handle sub-selection de paths elegíveis.
- **Group/Text/Shape/Symbol**: contextos distintos, sem materialização implícita.
- **TransientInteraction**: pointer capture, preview e estado provisório de um único gesto.

Uma operação não deve ficar ativa fora de contexto elegível. Trocar Node→Bend preserva seleção por IDs sempre que ela continuar válida; sair do contexto mantém a seleção de objetos e não muda DocumentRevision.

O `ToolKind` atualmente implementado é uma enumeração simples e **não** constitui o contrato final. Event routing e dispatch devem respeitar prioridade: controle focado (campo/text editor/diálogo) → interação capturada → operação contextual → actions de canvas → shortcuts globais não conflitantes.

### Escape, Enter e double-click

Escape cancela drag/preview primeiro e somente depois, quando ocioso, desempilha um nível de contexto. Enter em Select sobre path elegível abre Vector Edit; Enter em Pen/editor de texto/controle focado mantém a semântica desse contexto. Duplo clique abre o editor correto ao tipo: Path→Vector, Group→Isolation, Shape→paramétrico, Text→texto, Symbol→instância/overrides.

Seleção mista não perde silenciosamente objetos ao entrar em multi-path editing. Entrada explícita pode selecionar apenas alvos elegíveis depois de informar a exclusão.

### Seleção de área, candidatos e teclado aprovados

A [especificação canônica de seleção](#/docs/04-ui/selection-nodes-handles.md) registra Marquee por Contenção padrão/Interseção explícita/Direcional opt-in, Lasso contextual Replace/Add/Subtract, Shift toggle, transform bounds para 2+ nodes, candidatos sobrepostos com alternância por ActionId e navegação semântica por teclado. Essas operações atualizam **SelectionState/foco/TransientEdits** conforme o caso e não alteram `DocumentRevision` apenas por selecionar. A UI pode guardar preferência de política na Session/Workspace sem persistir no documento PTND. Os tipos concretos de evento e remapeamento de shortcuts permanecem matéria de implementação.

[Especificação de interação](#/docs/04-ui/vector-edit-interaction.md)

## ViewState

- zoom
- pan
- canvas rotation
- viewport size
- DPR
- preview mode.

Não serializar em PTND; opcionalmente salvar em workspace session separado.

## Input normalizado

```rust
pub struct PointerSample {
    pub device: PointerDevice,
    pub position_view: Point,
    pub pressure: f32,
    pub tilt: Vec2f,
    pub rotation: f32,
    pub modifiers: Modifiers,
    pub timestamp: InstantLike,
}
```

A camada Qt/CXX-Qt traduz eventos nativos de mouse, teclado e caneta para esse formato normalizado antes de entregá-los às ferramentas.

## Actions

Atalhos, menus e toolbar disparam `ActionId`/Commands. Evitar código duplicado “menu save” e “Ctrl+S”.

## Selection

Seleção é IDs + sub-selection opcional:

```rust
pub struct SelectionState {
    pub objects: OrderedSet<ObjectId>,
    pub sub_selection: Option<SubSelection>,
}
```

Render lê seleção para overlay; Document não.

## Focus

Canvas keyboard handling só recebe shortcuts quando foco/contexto permitem. Text editing deve consumir teclas de texto antes de commands globais conflitantes.

## Pointer capture

Ferramenta em drag mantém capture até up/cancel para não perder transação quando o cursor sai do canvas.

## Verificacao ADR-0012 (headless, 2026-10-09)

Escopo: camada de sessao e interacao de `petunia-ui`, sem Qt/QML. Revisao `56575ea3fb2ee393111755209e758c67d92bbe9d` sobre branch `petunia-design-rust`.

Changed: `context.rs` (ContextStack, ContextRejection, NodeId/SegmentId/HandleId, SubSelection, SelectionState), `tools/mod.rs` (ToolController, ToolResponse, SelectionDelta, HitTarget, SelectTool, NodeTool), `shortcuts.rs` (ActionId, KeyCombo, tabela com 15 acoes e editor de rebind), `numeric.rs` (NumericField, SizeFields), `workspace.rs` (WorkspaceState, ViewState, document_tolerance) e `app.rs` (StudioSession com hit-test na ordem D1 Handle > Node > Segment > Fill, índice de z pelo root_order, aplicacao dos deltas e commit por transacao).

Gap: as tools devolvem resposta declarativa, mas nada aplicava `SelectionDelta`/`Commit`; o primeiro clique em vazio nao limpava a selecao e o teste pressupunha mutacao direta do stub. A sessao passou a ser a unica escritora, alinhado a ADR-0012.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 278 passed / 0 failed (petunia-ui 35 testes) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass: 14 processos, 101 documentos, TODO 0 / IN PROGRESS 1 / DONE 13 |
| `node --test website/tests/progress.test.cjs` | pass: 6 testes |
| `#![forbid(unsafe_code)]` nos 5 crates | pass: nenhum `unsafe` fora do forbid |

Riscos/limites: verificacao headless apenas. Nenhuma GUI real, foco, leitor de tela ou contraste exercitado; overlays, snapping e arraste de node ainda emitem resposta sem geometria final. U01 permanece IN PROGRESS (2/3 checkpoints).

Next checkpoint: paineis visuais e QA de acessibilidade por padrao (U01), que exigem GUI.

## Verificacao Smart Delete, foco e tooltips (headless, 2026-10-09)

Escopo: segunda fatia headless do ADR-0012, ainda sem Qt/QML. Revisao `0ec440d1ce97e039af7617d840d80a0515e3c9a3` sobre branch `petunia-design-rust`.

Changed: `crates/petunia-engine/src/geometry/smart_delete.rs` (D7), `crates/petunia-ui/src/focus.rs` (D9 e Escape de D8), `crates/petunia-ui/src/tooltips.rs` (D5), acoes de nudge em `shortcuts.rs` e integracao em `app.rs`.

Gap encontrado e corrigido durante a verificacao: a camada de foco consumia Escape mesmo com o foco ja no canvas, impedindo o desenrolar da pilha de contexto. Escape no canvas volta a pertencer a cadeia de contexto; Escape em barra ou painel devolve o foco ao canvas.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 305 passed / 0 failed (petunia-ui 54, engine com 8 de smart_delete) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass: 14 processos, 101 documentos, TODO 0 / IN PROGRESS 1 / DONE 13 |
| `#![forbid(unsafe_code)]` nos 5 crates | pass: nenhum `unsafe` fora do forbid |

Riscos/limites: verificacao headless apenas. Nenhuma GUI real exercitada; Smart Delete ainda nao tem preview visual do erro, e o arraste de node continua sem geometria final. U01 permanece IN PROGRESS (2/3 checkpoints).
