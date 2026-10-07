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

Container e leaf precisam de invariantes claros. Se `children` fica no node comum, folhas devem rejeitar filhos; se fica em Group/Artboard, traversal usa trait/match. Escolher um e testar.

## Parent + children

Guardar apenas children torna parent lookup caro; guardar ambos exige consistência. Recomendação: armazenar `parent` no node e `children` ordenados no container, e toda alteração passa por métodos do SceneGraph que atualizam ambos atomicamente.

## Ordem de pintura

A ordem de `children` é z-order autoral. Nunca usar ordem de `HashMap` para render.

## Transform

Cada node armazena **local transform**. World transform é derivado e cacheável.

```text
world(node) = world(parent) × local(node)
```

Alterar parent invalida world transform e bounds dos descendentes.

## Appearance separada da geometria

`fill` e `stroke` diretamente em `SceneNode` é suficiente para MVP, mas não para múltiplos fills/strokes, styles, gradients e appearance stacking. Migrar para `Appearance`.

## Clip versus mask

- Clip: geometria binária/coverage que limita região.
- Mask: modulação contínua alpha/luminance.
- PowerClip-like containers: relação estrutural explicitamente serializada.

Evitar representar tudo como “mask” porque exportadores tratam os conceitos de modo diferente.

## Symbols e instances

`SymbolDefinition` vive em uma registry do documento; `SymbolInstance` guarda referência + overrides. Não duplicar toda subárvore em cada instância.

## Revisões

Cada node deve ter `revision` transitória ou um mecanismo de change tracking equivalente. Revisão não é identidade e normalmente não precisa ser persistida no PTND.

## Ciclos

Reparent, masks, symbols e referências entre nodes devem validar DAG/árvore conforme a relação. Reparent de um grupo para um descendente é erro do Core.

## Remoção

Definir política:
- `remove_node(id)` remove subtree?
- promove children?
- preserva resources órfãos?
- referências externas ficam dangling?

Recomendação: Commands explicitam `DeleteSubtree`, `Ungroup` e `Detach`; o método baixo nível não adivinha intenção.

## Snapshot

Renderer não deve receber `&mut SceneGraph`. Evoluir para snapshot imutável/compilado que separa authoring graph do render graph.
