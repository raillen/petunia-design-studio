# Cache, overlays e output

Cache acelera; overlay explica interação; output adapta a imagem ao destino.

## Cache layers

Separar caches:
- evaluated geometry
- bounds
- tessellation
- glyph atlas
- decoded image
- raster effect tiles
- composited group surface.

Cada cache tem key e budget próprios.

## Invalidation

Preferir revision/dependency graph a “clear all”. Uma mudança de nome do layer não invalida tessellation; mudança de path sim.

## Memory budget

Renderer precisa budget configurável e eviction LRU/clock-like. Recursos visíveis e próximos ao viewport têm prioridade.

## Overlays

Editor overlays não pertencem à Scene:
- selection outline
- bounding box
- nodes/handles
- guides
- smart guides
- snap anchors
- marquee
- brush cursor
- measurement labels.

Engine fornece geometry abstrata; Render aplica theme/style.

## Hit feedback

Hover overlay pode atualizar a 120 Hz sem gerar Command ou nova revisão documental.

## Output transform

Pipeline final:

```text
working linear/composite space
→ display transform / soft proof
→ tone/gamut mapping quando necessário
→ device surface
```

Screenshot/export não deve acidentalmente incluir guides/selection overlays.

## Thumbnail

Thumbnail usa o mesmo renderer com `RenderOptions { overlays: false }` e target próprio. Evitar implementar “mini renderer” divergente.

## Headless

Export, tests e batch processing precisam RenderTarget offscreen sem QApplication.
