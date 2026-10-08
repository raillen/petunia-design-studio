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
