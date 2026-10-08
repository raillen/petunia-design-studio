# Smart Region — construir, pintar e entrelaçar áreas

**Estado:** sistemas aceitos, UX final em discussão conjunta. Compartilham análise de regiões, mas **não** um único modelo autoral para operações diferentes.

## Referências

- [Inkscape Shape Builder](https://wiki.inkscape.org/wiki/Release_notes/1.3): clique/regiões, drag união, subtract, Enter/Escape.
- [Adobe Live Paint](https://helpx.adobe.com/br/illustrator/desktop/paint-and-fill/learn-painting-basics/about-live-paint.html): pintar faces delimitadas por interseções, inclusive gaps mediante política.
- [Adobe Intertwine](https://helpx.adobe.com/pt/illustrator/desktop/manage-objects/reshape-transform-objects/create-intertwined-objects.html): inversão de z-order **local** não destrutiva.
- [Figma Draw](https://www.figma.com/blog/introducing-figma-draw/): shape building e edição vetorial no canvas.

## Infraestrutura compartilhada

~~~text
selected path sources
↓ evaluate transforms/geometry effects
↓ intersection + planar subdivision
↓ region graph / provenance / adjacency
↓ region hit-test + virtual gap analysis
        ├→ Build operations
        ├→ Region Paint bindings
        ├→ Region Select query
        └→ Weave overlap rules (distinct semantic model)
~~~

Uma região calculada não recebe ObjectId persistente até materialização. Source continua editável em modos live.

## Shape Builder — Build

1. Selecionar dois ou mais paths ou self-intersecting path.
2. Entrar no modo Build, com região sob hover destacada e ação atual legível.
3. Click marca face; drag atravessa faces e agrupa; subtração é ação distinta.
4. Preview mostra resultado e quantidade de faces/contours, sem esconder definitivamente source.
5. Confirmar = Command atômico; Cancel = nenhum Document change.
6. **Live Build** mantém binding/operations; **Expand** gera paths normais. Escolha de default de UI permanece aberta.

**Policy:** FillRule, holes, open paths, masked objects e strokes expandidos devem ser explicitados antes da operação. Evitar cor herdada arbitrariamente de uma source quando múltiplos styles divergem: perguntar/aplicar first-selected/default de modo configurável.

## Region Paint

Uma área pode ser delimitada por pedaços de paths diferentes. Hover identifica face, clique aplica Paint. Fill pode ser solid/gradient/pattern e depois mudar; source strokes permanecem intactos.

~~~text
Source Objects
   ↓
Live Region Graph
   ↓
RegionPaintObject {
  source refs,
  paint per region,
  gap policy
}
~~~

**Edits posteriores:** tentar remapear region by provenance; se um corte/fusão tornar o alvo ambíguo, exibir *Needs review* e preservar último paint binding sem atribuir a outra região. **Expand** materializa shapes separados.

## Close Gap

Há três ações diferentes:

- **Detect:** mostrar endpoint pairs e possíveis microfissuras, sem alterar desenho.
- **Virtual Bridge:** completar somente o fechamento usado por Live Region evaluation; não altera source.
- **Close Geometry:** Command explícito que cria segmentos/edita nodes, com preview, tolerância e undo.

Não criar bridge que cruze contornos sem validar; distância não basta — testar tangentes/oclusão, contexto, self-intersections e unidade. Gap threshold ajustável document-space, apresentado também em screen-space.

## Region Select

Click seleciona a face calculada como sub-selection transitória; não cria ObjectId até Extract/Expand. Pode ser usado como máscara de paint, Shape Builder, área de fechamento e inspeção. Não usar como substituto de selecionar objetos inteiros.

## Intertwine / Weave

Uma área de overlap tem ordem local diferente do z-order global. Proposta: selecionar elementos → Create Weave group → hover crossings → flip front/back local com preview; opção de desenhar região fechada para múltiplos crossings.

Persistir **local occlusion specification** e refs, não fragmentos de path cortados. Release restaura sources. Expand materializa resultado, quando formato exigir.

Limites: stroke/opacity/blend/masks complicam occlusion; N objetos podem produzir ordens conflitantes, exigindo resolver/erro tipado. Isso **não é “barato” automaticamente** por já existir Shape Builder.

## Estados de interação comuns

Idle → Analyzing (cancelável) → Ready → HoverRegion → Painting/Building/Weaving → Preview → Commit/Cancel. Se source revision mudar durante processamento, descartar análise anterior e reavaliar.

## Performance e testes

Cache por refs/revision/tolerance/FillRule, subgraph incremental quando possível. Proteger contra explosion de edges; fallback diagnóstico em pathological geometry. Testar overlaps, holes, collinear touches, self-intersections, tiny gaps, locked layers, undo, preview/commit parity e region remap após source edits.

[Core autoral](#/docs/01-core/creative-features.md) · [Geometry](#/docs/02-engine/geometry.md)