# scene.rs

A Scene é a **estrutura semântica e ordenada** de objetos. Ela não é uma lista de draw calls.

## Tipos de node

O `SceneItem` precisa evoluir de Path/Group para um conjunto explícito:

```rust
pub enum SceneItem {
    Group(GroupData),
    Path(PathObject),
    Shape(ParametricShape),
    Text(TextObject),
    Image(ImageObject),
    PixelLayer(PixelLayerRef),
    Artboard(ArtboardData),
    SymbolInstance(SymbolInstance),
}
```

Efeitos, appearance, transform, visibility e blend são propriedades comuns do `SceneNode`, não duplicadas em cada item.

## Estrutura do node

```rust
pub struct SceneNode {
    pub id: ObjectId,
    pub name: String,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
    pub transform: Transform2D,
    pub flags: NodeFlags,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub appearance: Appearance,
    pub effects: EffectStack,
    pub clip: Option<ClipRef>,
    pub mask: Option<MaskRef>,
    pub item: SceneItem,
}
```

Um **container** é um node que pode possuir filhos, como Group ou Artboard. Um **leaf** é um node final, como uma imagem ou path sem filhos.

Container e leaf precisam de invariantes claros. Se `children` fica no node comum, folhas devem rejeitar filhos; se fica em Group/Artboard, **traversal** — percorrer a árvore de nodes — usa trait/match. Escolher um e testar.

## Parent + children

Guardar apenas children torna parent lookup caro; guardar ambos exige consistência. Recomendação: armazenar `parent` no node e `children` ordenados no container, e toda alteração passa por métodos do SceneGraph que atualizam ambos atomicamente.

## Ordem de pintura

**Z-order** é a ordem de empilhamento visual: qual objeto aparece acima ou abaixo de outro.

A ordem de `children` é z-order autoral. Nunca usar ordem de `HashMap` para render.

## Transform

Cada node armazena **local transform**: transformação em relação ao próprio pai.

**World transform** é a transformação acumulada desde a raiz até o objeto e é derivada/cacheável.

```text
world(node) = world(parent) × local(node)
```

Alterar parent invalida world transform e bounds dos descendentes.

## Appearance separada da geometria

`fill` e `stroke` diretamente em `SceneNode` é suficiente para MVP, mas não para múltiplos fills/strokes, styles, gradients e appearance stacking. Migrar para `Appearance`.

## Clip versus mask

- **Clip**: geometria que limita onde o conteúdo pode aparecer.
- **Mask**: modulação contínua de visibilidade por alpha ou luminância.
- **Coverage**: valor de 0 a 1 indicando quanto de um pixel/região está coberto.
- PowerClip-like containers: relação estrutural explicitamente serializada.

Evitar representar tudo como “mask” porque exportadores tratam os conceitos de modo diferente.

## Symbols e instances

`SymbolDefinition` vive em uma **registry** — coleção central indexada por ID — do documento.

`SymbolInstance` guarda referência + **overrides**, isto é, diferenças locais aplicadas à instância sem duplicar toda a definição. Não duplicar toda subárvore em cada instância.

## Revisões

Cada node deve ter `revision` transitória ou um mecanismo de change tracking equivalente. Revisão não é identidade e normalmente não precisa ser persistida no PTND.

## Ciclos

**Reparent** significa mover um node para outro pai.

Reparent, masks, symbols e referências entre nodes devem validar DAG/árvore conforme a relação. Uma **DAG** é um grafo direcionado sem ciclos; referências não podem formar dependências infinitas. Reparent de um grupo para um descendente é erro do Core.

## Remoção

Definir política:
- `remove_node(id)` remove subtree?
- promove children?
- preserva resources órfãos?
- referências externas ficam **dangling** — apontando para um objeto que não existe mais?

Recomendação: Commands explicitam `DeleteSubtree`, `Ungroup` e `Detach`; o método baixo nível não adivinha intenção.

## Snapshot

Um **snapshot** é uma visão imutável e consistente de uma revisão da cena.

Renderer não deve receber `&mut SceneGraph`. Evoluir para snapshot imutável/compilado que separa authoring graph do render graph.
