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

## Ferramentas criativas por sistema

As novas ferramentas foram modeladas como **capacidades do produto**, não como obrigação de colocar um ícone independente para cada operação.

- [Catálogo + contrato transversal](#/docs/04-ui/creative-tools-overview.md)
- [Smart Path](#/docs/04-ui/smart-path.md): Direct Bend, Smart Delete, Clean Vector, Select Similar
- [Smart Region](#/docs/04-ui/smart-region.md): Shape Builder, Region Paint, Close Gap, Intertwine
- [Smart Distribution](#/docs/04-ui/smart-distribution.md): Repeat, Symmetry, Objects on Path, Blend
- [Smart Color](#/docs/04-ui/smart-color.md): Palette Extract, Recolor, harmonies, linked swatches
- [Smart Measure](#/docs/04-ui/smart-measure.md): measurements e annotations associativas
- [Avançadas](#/docs/04-ui/advanced-creative-tools.md): Variable Width, Pattern Editor, Brand Sheet, Vector Feather, Warp, Vector Brushes, Mesh Gradient, Knife/Erasers

O **modelo de navegação e interação híbrido** foi aprovado; atalhos secundários, nomes exibidos, agrupamento final da toolbar, detalhes visuais e acessibilidade seguem **em discussão conjunta**. Nenhuma dessas páginas declara que a funcionalidade já está implementada.

## Modelo híbrido Select + Vector Edit

**Decisão aprovada:** Select para objetos/hierarquia e Vector Edit para nodes, segmentos e handles, com Node Tool disponível explicitamente.

O contexto de edição é acessado por duplo clique em Path elegível, Enter sobre Path selecionado ou ação de Node. Group/Text/Shape/Symbol abrem contextos próprios, sem conversão destrutiva.

Dentro de Vector Edit, Node/Bend/Pen/Cut/Smooth/Width são operações disponíveis sob a mesma seleção, com mudanças de operação previsíveis. Drag de segmento em Node não executa Bend inesperadamente.

**Escape cancela primeiro a interação capturada; quando ocioso, sai um nível de contexto.** Todos os previews são transitórios, e Commit é uma Transaction.

O contrato completo, incluindo multi-selection, Group Isolation, focus, pointer capture e feedback, está em [Vector Edit — interação híbrida](#/docs/04-ui/vector-edit-interaction.md); a decisão registrada está no [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md).

Os atalhos secundários e a apresentação visual final seguem para discussão conjunta.

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
