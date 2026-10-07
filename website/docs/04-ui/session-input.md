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
