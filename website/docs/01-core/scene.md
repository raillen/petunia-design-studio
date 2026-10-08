# scene.rs

Scene é a **estrutura semântica, hierárquica e ordenada** dos objetos autorais.

Ela não é uma lista de draw calls e não contém caches do renderer.

## SceneGraph

SceneGraph armazena nodes por ObjectId. A ordem dos roots pertence à Page correspondente.

~~~rust
pub struct SceneGraph {
    // storage interno indexado por ObjectId
}
~~~

Cada Page possui uma lista ordenada de objetos raiz. Storage pode começar com HashMap e evoluir depois; ordem de storage nunca define z-order.

## SceneNode

~~~rust
pub struct SceneNode {
    pub id: ObjectId,
    pub name: String,
    pub parent: ParentRef,
    pub transform: Transform2D,
    pub flags: NodeFlags,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub geometry_effects: GeometryEffectStack,
    pub post_effects: PostPaintEffectStack,
    pub clip: Option<ClipBinding>,
    pub mask: Option<MaskBinding>,
    pub item: SceneItem,
}
~~~

Propriedades comuns ficam no node quando possuem semântica para todos os itens.

Appearance não fica obrigatoriamente no node comum porque nem todo item é pintado da mesma forma.

## SceneItem

~~~rust
pub enum SceneItem {
    Group(GroupData),
    Layer(LayerData),
    Path(PathObject),
    Shape(ShapeObject),
    Text(TextObject),
    Image(ImageObject),
    PixelLayer(PixelLayerRef),
    Artboard(ArtboardData),
    SymbolInstance(SymbolInstance),
}
~~~

Essa enumeração é persistente e versionada.

Adicionar item novo exige definir serialização, bounds, render semantics, hit-test semantics e comportamento de export.

## Containers

Containers possuem children explicitamente.

~~~rust
pub struct GroupData {
    pub children: Vec<ObjectId>,
}

pub struct LayerData {
    pub children: Vec<ObjectId>,
}

pub struct ArtboardData {
    pub spec: ArtboardSpec,
    pub children: Vec<ObjectId>,
}
~~~

Leaf items não possuem campo `children`. Isso evita estado inválido como Image com filhos.

SceneGraph oferece método interno `children_of(id)` para traversal sem expor detalhes.

## Group versus Layer

Group organiza objetos como unidade transformável.

Layer é container autoral com papel estrutural explícito no documento.

Ambos possuem children, mas não precisam ser o mesmo tipo com flag obscura. Isso permite evoluir comportamento de layer sem quebrar Group.

## Artboard

Artboard é container gráfico com área própria, background/clip policy e children.

Artboard não substitui Page editorial. Pages pertencem ao Document model.

## Parent + children

Parent usa `ParentRef` com dois casos: `Page(PageId)` ou `Object(ObjectId)`.

Se o parent for Object, o container correspondente deve conter o child exatamente uma vez. Se o parent for Page, o child deve aparecer exatamente uma vez em `Page.root_children`.

Esses dois lados mudam juntos. Insert, remove, reparent e reorder são operações atômicas de SceneGraph.

## Z-order

**Z-order** é a ordem de empilhamento visual.

Dentro de um container:

~~~text
children[0]   → mais atrás
...
children[n-1] → mais à frente
~~~

A convenção precisa ser única em Core, Render, hit-test e export.

Não usar HashMap iteration.

## Transform

Cada node guarda local transform relativo ao parent.

~~~text
world(node) =
world(parent) × local(node)
~~~

World transform é Derived State.

Reparent invalida world transforms/bounds descendentes, mas não reescreve geometria local automaticamente.

## Preserve world on reparent

Duas operações semanticamente diferentes:

~~~text
ReparentKeepingLocal
ReparentKeepingWorld
~~~

A segunda calcula:

~~~text
new_local =
inverse(world(new_parent)) × old_world
~~~

Se parent transform não for invertível, Command falha de forma tipada.

Core não adivinha intenção.

## NodeFlags

Persistir apenas flags autorais.

~~~rust
pub struct NodeFlags {
    pub visible: bool,
    pub locked: bool,
    pub exportable: bool,
}
~~~

Selection, hover e focused não pertencem aqui.

`visible = false` afeta output e por isso é Document State.

## PathObject e ShapeObject

~~~rust
pub struct PathObject {
    pub path: VectorPath,
    pub appearance: Appearance,
}

pub struct ShapeObject {
    pub shape: ParametricShape,
    pub appearance: Appearance,
}
~~~

Geometry effects comuns permanecem no SceneNode para permitir mesma infraestrutura em ambos.

## Text

TextObject possui modelo tipográfico próprio. Object-level effects/opacity continuam no SceneNode.

Não converter texto em paths para render normal.

## Image versus PixelLayer

Image referencia recurso colocado e preserva fonte.

PixelLayer referencia superfície raster editável.

Rasterize é Command explícito entre esses conceitos.

## Clip e Mask

Clip e Mask são bindings persistentes, não “um child especial escondido por posição”.

~~~rust
pub struct ClipBinding {
    pub source: ObjectId,
}

pub struct MaskBinding {
    pub source: ObjectId,
    pub mode: MaskMode,
}
~~~

`MaskMode` diferencia alpha/luminance quando aplicável.

O binding precisa definir se o source também pinta normalmente ou é consumido apenas como mask; essa política fica explícita no binding/Command.

## Ciclos de referência

A árvore parent/children é acíclica.

Bindings, symbols e future live relations formam grafos adicionais que também precisam de cycle validation.

Exemplo proibido:

~~~text
A masked by B
B masked by A
~~~

Core rejeita a mutação antes do commit.

## Symbols

SymbolDefinition vive em `SymbolRegistry`, separado da árvore principal.

~~~rust
pub struct SymbolInstance {
    pub definition: SymbolId,
    pub overrides: SymbolOverrides,
}
~~~

Definition usa a mesma semântica de nodes, mas possui seu próprio root/subtree autoral.

Instance não duplica toda a árvore.

Overrides referenciam entidades estáveis da definition.

## Overrides

Override é diferença explícita sobre a definição.

Exemplos futuros:

~~~text
Text content override
Color override
Visibility override
Resource override
~~~

Não usar mapa JSON arbitrário para built-ins. Overrides nativos precisam de schema tipado/versionado.

## Revisions

Node/relation pode ganhar revisions locais runtime para caches.

Essas revisions:

- não são identidade;
- não precisam ser PTND;
- não substituem DocumentRevision;
- entram somente quando cache fino justificar.

## Remoção

Baixo nível não deve possuir uma ambígua `remove_node()` que adivinhe intenção.

Commands distintos:

~~~text
DeleteSubtree
Ungroup
DetachMask
DetachClip
DeleteKeepingChildren
~~~

Cada Command prepara referências e inverses antes do commit.

Document válido não termina com dangling reference obrigatória.

## Resources órfãos

Remover último ImageObject que usa Resource não precisa apagar bytes imediatamente.

Resource garbage collection é operação separada/segura porque History ou outros objetos podem ainda referenciar aquele Resource.

## Traversal

Traversal precisa ser determinístico.

~~~text
root_children em ordem
↓
depth-first conforme children
~~~

Algoritmos que não dependem de ordem podem usar estratégias diferentes internamente, mas output persistente não depende de scheduling.

## Snapshot

Renderer e workers recebem snapshot imutável.

~~~text
Authoring SceneGraph
↓ compile/snapshot
SceneSnapshot
↓
Engine / Render / Jobs
~~~

Snapshot pode futuramente usar storage otimizado diferente do authoring graph.

Não obrigar SceneGraph autoral a virar render graph.

## Validação

SceneGraph válido garante:

- ObjectId único;
- parent existente ou root;
- children existentes;
- parent/child consistency;
- nenhuma child duplicada no mesmo container;
- árvore sem ciclos;
- leaf sem children;
- referências obrigatórias válidas;
- transforms finitos;
- opacity finita no range definido.

## Invariantes

1. Scene é modelo autoral, não draw list.
2. Z-order vem de children/root_children ordenados.
3. Leaf items não armazenam children.
4. Parent + children mudam atomicamente.
5. Local transform é persistente; world transform é derivado.
6. Reparent keeping local e keeping world são Commands diferentes.
7. Visibility/lock são Document State; selection/hover não.
8. Clip e Mask são bindings explícitos.
9. Ciclos proibidos são rejeitados no Core.
10. Symbol instance referencia definition; não duplica subtree.
11. Delete não destrói Resource automaticamente.
12. Renderer nunca recebe `&mut SceneGraph`.
