# Smart Distribution — Repeat, Symmetry, Blend, Objects on Path

**Estado:** aprovado como família; gestos/inspector serão revisados em conjunto. Uma infraestrutura de distribuição com estratégias distintas, sem duplicação implícita de SceneNodes.

## Referências

- [Figma Draw repeats](https://www.figma.com/blog/introducing-figma-draw/): repeats linear/radial integrados à ilustração.
- [CorelDRAW Symmetry](https://www.coreldraw.com/en/learn/tutorials/symmetry-tool/): editar conteúdo no grupo e alterar eixos.
- [Illustrator Objects on Path](https://helpx.adobe.com/illustrator/using/objects-on-path.html): orientação e rearranjo mantendo binding ao spine.
- [Illustrator Blend](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blend-panel-overview.html): steps, spacing, easing, color interpolation, spine, expand/release.

## Modelo

~~~text
DistributionObject
├── source objects
├── distribution spec
├── optional spine/axis/centers
└── seed where random
    ↓ Engine evaluation
virtual instances (derived)
    ↓ RenderSnapshot
~~~

Source editable in place. Repeat Count limitado por budget e diagnóstico; instâncias virtuais não têm ObjectId persistente. Expand = novos objetos tipados com IDs e Undo.

## Live Repeat

**Linear:** count, step vector/spacing, rotation, orientation, reverse.
**Grid:** rows/columns, gaps, stagger/brick, alternating transform, mirror.
**Radial:** center, start angle, angular span, count, orientation, rotate-with-tangent.
**Mirror:** axis position/direction, reflection policy, optional multiple axes.

Fluxo comum: selecionar object(s) → escolher modo → preview imediato → ajustar on-canvas handles ou números exatos → manter source editável → expand/release opcional. Não congelar atalhos aqui.

## Symmetry Draw

É o modo autoral do **Mirror Repeat** com edição da source. Proposta:

1. Criar grupo de simetria a partir da selection ou de uma source vazia.
2. Exibir eixo(s) e uma área de edição primária.
3. Pen/Node/Brush vetorial edita source, resultado refletido acompanha em preview.
4. Múltiplos eixos podem gerar grupos diédricos/repetições angulares, mas a semântica de transformação deve permanecer determinística.
5. Se stroke cruza o eixo, a política seam/weld deve ser explícita (não criar double-stroke involuntário).

Mirror não exige duplicação autoral de cada stroke. Expand materializa geometry.

## Objects on Path

Seleccionar fontes + path/spine e definir:
- distribuição por count/distance/manual offsets;
- tangent rotation, orientation/reverse;
- start/end offset, spacing, corner policy;
- include endpoints; order along curve.

**Arc length** é distância percorrida na curva. Espaçamento por distância deve usar arc-length approximation, e não t uniforme de Bézier.

Alterar spine recalcula posições; ref dangling exige diagnose/detach policy. Há seleção de instâncias virtuais por locator estável dentro da revision corrente; editar instância individual precisa override tipado ou Convert/Expand, não modificar a source aleatoriamente.

## Blend 2.0

Entre duas ou mais fontes com:
- specified steps / spacing / smooth;
- linear/path spine;
- ease curve e color acceleration independentes;
- rotation, scale, opacity, color space/interpolation;
- reverse direction/order.

### Dois modos distintos

**Attribute Blend:** interpola transforms, colors e opacity; preserva conteúdo dos endpoints com visual crossfade se não houver morfologia compatível.

**Geometry Morph:** requer correspondência válida explícita entre contours e nodes/corresponding regions. Com topologias diferentes, não inventar index matching. Mostrar incompatibilidade, oferecer manual mapping/simplify ou Attribute Blend.

Live Blend é generator; Expand cria shapes; Release devolve sources. Blend composto com Text/Groups precisa policy de flatten/partial evaluation antes de habilitar Geometry Morph.

## Scatter / object brush

Compartilha distribuição seeded: quantia/densidade, orientação, scale jitter, collisions, offset from path. Evitar gerar milhares de SceneNodes durante preview. A classe True Vector Brushes usa esta infraestrutura, mas brush stroke authoring não é sinônimo de Repeat.

## Tests

Verificar determinismo mesmo com parallel evaluation, seed, large count, path cusps, zero length, scale negatives, transformed parents, source edits, nested generators/cycle rejection, Undo/Redo, export equivalence e memory limits.

[Core models](#/docs/01-core/creative-features.md) · [Engine](#/docs/02-engine/creative-operations.md)