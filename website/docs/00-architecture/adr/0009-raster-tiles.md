# ADR-0009 — Raster autoral tiled + copy-on-write

**Status:** Accepted

## Contexto

Pixel layers grandes não podem exigir cópia da superfície completa a cada stroke, snapshot ou Undo.

## Decisão

`PixelSurface` é logicamente tiled:

~~~text
TileCoord → immutable TileVersion/TileBlob
~~~

Edição cria novas versões apenas dos tiles afetados; snapshots/history podem compartilhar os demais por copy-on-write.

Tile size concreto permanece parâmetro definido por benchmark.

## Alternativas rejeitadas

**Vec<Rgba> gigante por layer:** cópia/Undo/ROI ruins para documentos grandes.

**Tile size congelado por intuição:** afeta cache locality, blur, brush e metadata.

**GPU texture como storage autoral:** acopla documento a backend/dispositivo.

## Consequências

- brush/filter operam por dirty tiles/ROI;
- raster history guarda tile versions/diffs;
- authorial alpha é straight;
- compositor pode usar premultiplied temporariamente;
- compression física é lossless e separada;
- sparse tiles podem representar transparent black.

## Referências

[Texto, raster e recursos](#/docs/01-core/text-raster-resources.md)  
[Brush + Raster](#/docs/02-engine/brush-raster.md)
