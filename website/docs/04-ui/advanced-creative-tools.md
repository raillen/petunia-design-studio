# Ferramentas criativas avançadas

**Estado:** todas constam como capacidades do projeto; implementation stage não é promessa de release. GUI/UX exata será discutida conjuntamente.

## Variable Width Stroke

**Referências:** [Illustrator Width](https://helpx.adobe.com/br/illustrator/using/stroke-object.html) e [CorelDRAW Variable Outline](https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Mac/CorelDRAW-en/CorelDRAW-Variable-outlines.html).

Hover spine → criar WidthPoint → drag perpendicular ajusta lados simetricamente/assimétricamente → deslocar ponto ao longo da curva → valores exatos → salvar WidthProfile. Path source continua Line/Cubic. Expand Stroke é Command explícito. Largura ≥ 0, pontos ordenados por arc length e cusp discontinuity modelada.

## Pattern Editor

**Referências:** [Inkscape 1.3](https://wiki.inkscape.org/wiki/Release_notes/1.3) e [Figma Draw](https://www.figma.com/blog/introducing-figma-draw/).

Editar PatternDefinition/source; controles on-canvas para tile size, origin, rotation, scale, gap, brick, mirror; preview repetido virtualmente. Mudar Pattern vinculado afeta seus consumidores; Detach cria cópia editável. Recusar self-reference/ciclos.

## Brand Sheet Generator

Após Color/Styles/Text/DocumentFragment. Input swatches/fonts/logo e page preset → preview → Generate cria Group com shapes/textos editáveis, swatch/style refs e variantes de exemplo. Não gerar apenas bitmap. Valores CMYK só são fiéis com profile; não fingir exatidão de impressão. Auto-update de brand sheet exige provenance/refresh contract futuro.

## Vector Feather / Variable Edge Softness

**Após Render/Mask.** Pontos ao longo do contorno definem softness, falloff e unidade. Máscara/coverage derivada e ROI; path source intacto. É diferente de blur uniforme e de variable stroke width.

Método inicial candidato: signed-distance field (SDF = distância assinada até a borda) ou equivalent distance-based coverage, escolhido por profiling. Testar holes, corners, self-overlaps e zoom.

## Perspective / Envelope Warp

**Pós-v0.1-stable.** Perspective = mapping projective por 4 pontos quando válido; Envelope = deformation field/control mesh. Controles no canvas e preview; source + envelope persistem, Expand/Rasterize explícito. Curvas requerem adaptive subdivision; raster requer resampling. Rejeitar transforms degeneradas sem corromper source.

## True Vector Brushes

**Após Brush + Distribution.** Stretch brush usa artwork deformado pela spine; Scatter/Art Brush distribui motivos vetoriais. Input/stabilizer/spacing/dynamics shared, BrushAssetId + seed autorais. Geometry virtual e cache derivado; Expand materializa. Não chamar bitmap brush com textura de true vector. [Figma Draw scatter](https://www.figma.com/blog/figma-draw-scatter-brushes/).

## Mesh Gradient

**Exploração futura pós-v0.1-stable.** Modelo próprio de patches, continuity e color interpolation; não reservar enum incompleto em Paint.

## Knife / Scissors / Vector Eraser / Raster Eraser

| Ferramenta | Semântica |
|---|---|
| Scissors | split path em ponto/posição preservando IDs apropriados |
| Knife | split topo por cutter trajectory/shape, respeitando FillRule |
| Vector Eraser | subtração topológica sobre shapes com preview |
| Raster Eraser | aplica alpha via Brush Engine em PixelLayer tiles/COW |

Não criar implementação única para todos. Todos usam Commands e Undo; input degenerado informa erro, não destrói áreas inesperadas.

## Integrações já documentadas

Live Corners, Offset, Image Trace, Crop/Clipping, Masks, Text on Path, QR/Barcode, adjustments e pattern fills já têm especificação de motor; integram Tools/Actions quando implementados.

[Catálogo](#/docs/04-ui/creative-tools-overview.md) · [Engine](#/docs/02-engine/creative-operations.md)