# Ferramentas

Uma Tool é um **controller de interação**, não o lugar onde vive a geometria.

## Contrato

```rust
pub trait ToolController {
    fn begin(&mut self, ctx: &mut ToolContext, event: PointerSample) -> ToolResponse;
    fn update(&mut self, ctx: &mut ToolContext, event: PointerSample) -> ToolResponse;
    fn end(&mut self, ctx: &mut ToolContext, event: PointerSample) -> ToolResponse;
    fn cancel(&mut self, ctx: &mut ToolContext) -> ToolResponse;
}
```

`ToolContext` expõe serviços estreitos: queries, snapping, transaction preview, selection. Não entrega `&mut Document`.

## ToolResponse

Pode solicitar:
- cursor
- overlay primitives
- transient preview
- commit transaction
- selection change
- status hint.

Isso mantém QML/Qt fora da lógica de domínio.

## Pen Tool

UI/controller: pointer state e modifiers. Geometry: Bézier/snap. Core: Path data. Render: preview segment, nodes, handles.

Estados explícitos: Idle, PlacingNode, DraggingHandle, ClosingContour, EditingContinuation.

## Node Tool

Sub-selection usa NodeId, não índices. Move múltiplos nodes em transaction preview; smooth/symmetric constraints calculadas pelo Geometry Engine.

## Move/Transform

Transform box é overlay. Engine calcula pivot, constraints, snap e affine delta. Commit altera local transforms ou geometria conforme modo.

## Shape Tool

Cria `ParametricShape`, não Path. Drag define bounds; modifiers controlam ratio/center. Convert to Curves é Command separado.

## Brush Tool

Controller envia samples ao Brush Engine. Não gera dabs em QML/Qt; o algoritmo permanece no Brush Engine.

## Guide Tool

Drag da régua cria preview de guide; release gera AddGuideCommand. Mover guide é transaction; lock é propriedade Core.

## Eraser/Knife

Precisam semantics claras:
- vector eraser altera geometry via Engine.
- raster eraser pinta alpha/coverage.
- knife corta paths/topologia.

Não esconder três comportamentos sob implementação única.
