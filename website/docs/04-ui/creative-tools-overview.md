# Ferramentas criativas — catálogo e contrato

**Estado (2026-10-08):** capacidades já aceitas como escopo e contratos de UX aprovados por delegação nas respectivas páginas de interação. **Especificação não equivale a implementação, teste ou compromisso de entrega na v0.1.** O design visual final é refinável por testes de usabilidade e acessibilidade.

## Princípio

Não transformar 30 capacidades em 30 motores e 30 ícones visíveis. Organizar por **famílias de interação** e compartilhar infraestrutura Core/Engine/Render. A UI Qt/QML via CXX-Qt expõe Commands, ToolControllers, inspectors contextuais e feedback acessível; não calcula geometria, topologia, cor ou layout.

| Sistema | Capacidades | Dependência principal |
|---|---|---|
| [Smart Path](#/docs/04-ui/smart-path.md) | Direct Bend; Smart Delete; Clean Vector; Simplify; Smooth; Select Same/Similar; Pen/Node refinados | Curve fitting, path, scene queries |
| [Smart Region](#/docs/04-ui/smart-region.md) | Shape Builder; Region Paint; Close Gap; Region Select; Intertwine/Weave | Planar subdivision, provenance, compositor |
| [Smart Distribution](#/docs/04-ui/smart-distribution.md) | Linear/Grid/Radial/Mirror Repeat; Symmetry; Blend; Objects on Path; Scatter | Live generator + transforms + arc length |
| [Smart Color](#/docs/04-ui/smart-color.md) | Palette Extract; Recolor Lab; Recolor from Image; Replace Color; harmonies; contrast/luminance locks; Global Swatches | Color Management + swatches + Appearance |
| [Smart Measure](#/docs/04-ui/smart-measure.md) | Hover Measure; linear/angular/radial dimensions; area/perimeter; pinned associative dimensions | Geometry/units + typed references |
| [Ferramentas avançadas](#/docs/04-ui/advanced-creative-tools.md) | Variable Width; Pattern Editor; Brand Sheet; Vector Feather; Perspective/Envelope Warp; True Vector Brushes; Mesh Gradient; Knife/Scissors/Eraser | Stroke/paint, Render effects, geometry/raster |

## Referências e adaptação

Referências funcionais, não cópias visuais:

- [Inkscape Shape Builder + Pattern Editor](https://wiki.inkscape.org/wiki/Release_notes/1.3): manipulação direta de regiões e controle on-canvas.
- [Adobe Illustrator Live Paint](https://helpx.adobe.com/br/illustrator/desktop/paint-and-fill/learn-painting-basics/about-live-paint.html): região pintável independente dos caminhos que a delimitam.
- [Adobe Illustrator Intertwine](https://helpx.adobe.com/pt/illustrator/desktop/manage-objects/reshape-transform-objects/create-intertwined-objects.html): troca local de precedência não destrutiva.
- [Figma Draw](https://www.figma.com/blog/introducing-figma-draw/): edição de múltiplos nodes, Shape Builder, repeats, stroke, patterns e text-on-path.
- [CorelDRAW Symmetry](https://www.coreldraw.com/en/learn/tutorials/symmetry-tool/): edição dentro do grupo e múltiplos eixos.
- [Illustrator Width](https://helpx.adobe.com/br/illustrator/using/stroke-object.html): pontos de largura simétrica/assimétrica e profiles.
- [Illustrator Blend](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blend-panel-overview.html): steps, spacing, ease, spine e expansão.
- [Illustrator Dimensions](https://helpx.adobe.com/br/illustrator/using/tool-techniques/dimension-tool.html): linear/angular/radial e annotations.

## Contrato comum

Toda ferramenta descreve: problema e tarefa; pré-condições; estados begin/update/end/cancel; seleção/subseleção; cursor e hover; snapping; unidades/tolerâncias; preview; operação semântica; resultado de commit; undo/redo; resolução de conflitos; comportamento para locked/hidden/symbol/clip; performance e testes. Não presumir atalhos ou cores definitivas antes do fechamento de GUI/UX.

~~~text
Pointer/Keyboard → ToolController
                  ↓
         Query + Engine Preview
           ↓              ↓
       overlays        Commit Command
                            ↓
                     Prepare/Transaction
                            ↓
                    Core authoring state
                            ↓
                evaluation → render-model → Render
~~~

**Não persistir** hover, seleção, candidatos regionais, tessellation, preview, guides temporárias, overlays ou resultado calculado de live generator.

## Priorização relativa

A ordem segue fundações, não contagem de features:

1. **Base de edição:** Select/Node/Pen, Select Same, Smart Delete, Simplify/Clean, Measure transitório.
2. **Geometria compartilhada:** Direct Bend, Shape Builder, Region Select/Paint, Close Gap, Variable Width.
3. **Generators e cor:** Repeat/Mirror, Symmetry, Objects on Path, Recolor Lab, Pattern Editor, Brand Sheet.
4. **Operações avançadas:** Intertwine, Blend/morph, dimensão associativa, scatter/true vector brushes, Vector Feather, Perspective/Envelope Warp.
5. **Futuro especializado:** Mesh Gradient avançado.

Itens de custo baixo **dependem do motor base**: Region Paint não é barato antes de planar subdivision; Symmetry não é barata antes de Repeat; Blend com morph de topologia diferente não é só interpolar coordenadas.

## Estados

- **Especificado (contrato):** entradas, saídas, invariantes e integração descritas; não significa código implementado.
- **Dependente de fundação:** contrato delineado, implementação após recurso compartilhado.
- **Pós-v0.1-stable / Exploração:** preservado no projeto, mas não promessa de lançamento.

## Critérios de qualidade

Nenhuma ferramenta é “pronta” sem seleção precisa, teclado, cancel, preview consistente, 1 undo lógico, modo headless quando aplicável, estados vazios/degenerados, diagnóstico claro e regressão em documentos reais.

**Interações aprovadas:** [Vector Edit precision](#/docs/04-ui/vector-edit-precision.md) · [Pen](#/docs/04-ui/pen-path-creation.md) · [Smart Path operations](#/docs/04-ui/smart-path-operations.md) · [Smart Region](#/docs/04-ui/smart-region-interaction.md) · [Distribution](#/docs/04-ui/smart-distribution-interaction.md) · [Color](#/docs/04-ui/smart-color-interaction.md) · [Measure](#/docs/04-ui/smart-measure-interaction.md) · [Advanced](#/docs/04-ui/advanced-creative-tools-interaction.md).

[Dados persistentes e tipos](#/docs/01-core/creative-features.md) · [Operações do motor](#/docs/02-engine/creative-operations.md) · [Quality gates](#/docs/00-architecture/verification.md)