# Smart Measure — medir sem interromper criação

**Estado:** sistema aceito; aparência de overlays e de annotation sujeita à revisão de GUI/UX.

## Referência

[Illustrator Dimension](https://helpx.adobe.com/br/illustrator/using/tool-techniques/dimension-tool.html) mede linear/angular/radial; [Dimension Objects](https://helpx.adobe.com/br/illustrator/desktop/measure-and-align/plot-and-measure/about-dimension-objects.html) criam linhas, extensões e texto.

Petunia oferece medição **transitória** e **persistente** como experiências diferentes.

## Hover Measure

Query sem mudar Scene nem Selection. Exibir quando geometricamente apropriado:
- width/height de geometric bounds versus visual bounds, identificados;
- distâncias entre pontos, nodes e edges;
- comprimento de segmento/curva e perímetro;
- área preenchida conforme FillRule (incluindo holes);
- ângulo entre duas direções;
- raio/diâmetro quando há arco/círculo identificável dentro de tolerância.

Valores aproximados precisam indicador de tolerância. Medição transitória é Session State/Overlay, não cria HistoryEntry.

## Pin Dimension

Proposta: hover → escolher anchors → preview de measure line/label → definir offset → Confirmar cria `DimensionObject` em uma Transaction. Edit source recalcula annotation; source alterada não deve fazer o label ficar congelado sem diagnóstico.

Modalidades: Linear horizontal/vertical/aligned; Angular; Radius/Diameter; Area; Perimeter.

## Vínculo e identidade

Âncoras usam ObjectId + NodeId ou feature geométrica estável; FixedDocumentPoint é diferente de referência associativa. Se o alvo some ou a feature deixa de existir, mostrar unresolved, não religar ao objeto vizinho.

Styles, precision e unidades são autorais; medida calculada é Derived State. Annotations persistentes entram em render/export conforme policy; measure hover nunca.

## Precisão e testes

Usar f64; formatação não muta Document; DPI de export e pixel de tela não são a mesma unidade.

Casos: transforms não uniformes, negativos, rotação, groups, self-intersections, zero-length, delete source, locked/hidden objects, mixed units, save/load, Undo/Redo, headless export.

[Core Model](#/docs/01-core/creative-features.md) · [Geometry](#/docs/02-engine/geometry.md)