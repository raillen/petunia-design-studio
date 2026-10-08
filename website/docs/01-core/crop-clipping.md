# Crop e Clipping não destrutivos

Crop limita a região visível sem destruir a source.

A implementação depende do tipo de conteúdo, mas a regra é única:

> **Cropar altera intenção de exibição; pixels, paths e recursos originais permanecem preservados até Command destrutivo explícito.**

## Dois casos

~~~text
Placed Image
→ source_rect

Scene content / Group / PixelLayer
→ ClipBinding
~~~

Não criar um terceiro sistema de crop se source rect e clipping já resolvem a semântica.

## Image crop

`ImageObject` usa crop no espaço da própria source.

Direção:

~~~rust
pub struct ImageObject {
    pub resource: ResourceId,
    pub source_rect: Option<ImageSourceRect>,
    pub sampling: ImageSamplingPolicy,
}
~~~

`source_rect = None` significa source completa.

`ImageSourceRect` é validado contra as dimensões lógicas do recurso resolvido.

A representação persistente deve ser independente de cache decodificado.

## Coordenadas de source_rect

A v0.1 usa coordenadas normalizadas da source:

~~~text
left/top/right/bottom em 0..1
~~~

Motivos:

- crop sobrevive a mudança de resolução da preview/cache;
- não depende de mip level;
- é estável para linked image substituída por versão de mesma intenção;
- evita confundir pixel de source com device pixel.

Direção:

~~~rust
pub struct ImageSourceRect {
    pub min: NormalizedPoint,
    pub max: NormalizedPoint,
}
~~~

Invariantes:

~~~text
0 <= min.x < max.x <= 1
0 <= min.y < max.y <= 1
~~~

Se a source externa muda aspect ratio/dimensões, o mesmo rect normalizado continua válido, mas a aparência pode mudar; isso é consequência explícita de atualizar o linked resource.

## Local geometry da imagem

A geometria local da ImageObject é derivada da source colocada + crop e transform do SceneNode.

Se futuramente precisarmos dissociar source crop de destination frame, introduziremos um placement spec explícito. A v0.1 não cria um segundo retângulo autoral antes de existir necessidade real.

## Reset crop

~~~text
source_rect = Some(...)
↓ Reset Crop
source_rect = None
~~~

É Command e gera Undo.

Nenhum byte de Resource é restaurado porque nunca foi removido.

## Generic crop

Para Group, vector content, PixelLayer e outros SceneItems, Crop reutiliza `ClipBinding`.

~~~text
content
+
clip source
↓
evaluation
↓
visible intersection
~~~

A source continua inteira.

## Rectangular crop source

Um crop retangular genérico materializa uma Shape Rectangle normal usada como clip source.

~~~text
Rectangle ShapeObject
source_use = BindingOnly
↓
ClipBinding
↓
target
~~~

Isso evita “crop geometry escondida” com um formato paralelo.

A geometria de crop é um objeto real, com ObjectId e Undo normal.

A UI futura pode apresentá-lo como crop frame sem expor complexidade desnecessária.

## BindingOnly

O clip source criado exclusivamente para crop usa:

~~~text
BindingSourceUse::BindingOnly
~~~

Portanto ele não pinta normalmente.

Se o mesmo objeto também precisar aparecer, a relação precisa declarar `AlsoVisible` explicitamente.

## Crop de PixelLayer

Cropar PixelLayer por default **não apaga tiles**.

Usa ClipBinding.

~~~text
PixelSurface source
↓ Clip
derived visible pixels
~~~

Um Command destrutivo separado pode existir para:

~~~text
Trim Pixels to Crop
~~~

Esse Command:

1. calcula região materializada;
2. cria/ajusta PixelSurface;
3. remove pixels fora da região;
4. atualiza transform/bounds conforme policy;
5. guarda tile refs/inverse data para Undo.

Não chamar isso simplesmente de “Crop”, porque a semântica é diferente.

## Crop de grupo

Group crop é clip normal.

Children permanecem:

- editáveis;
- movíveis;
- fora dos bounds do crop;
- preservados no PTND.

Alterar o crop não move children.

## Crop versus mask

Crop retangular usa Clip porque é cobertura geométrica binária/antialiased pela edge.

Mask é usada quando coverage depende de alpha/luminance/gradiente.

Não implementar crop simples como raster mask se um clip geométrico resolve.

## Crop versus Artboard

`Artboard.clip_to_bounds` limita a visualização/output do Artboard conforme sua spec.

Isso não transforma cada child em cropped object.

Crop individual e clip do Artboard são níveis diferentes.

## Crop versus export region

Export Slice/Rect apenas escolhe região de saída.

Não altera Scene visibility.

~~~text
Crop
→ authorial visibility relation

Export Slice
→ output area
~~~

## Bounds

Crop normalmente intersecta visual bounds:

~~~text
input visual bounds
∩
clip bounds
→ conservative cropped bounds
~~~

Se clip/effect exige cálculo mais complexo, bounds permanece conservador.

Nunca usar crop para mutar geometric bounds da source permanentemente.

## Hit-test

Hit-test do conteúdo cropped precisa respeitar clip.

Objeto fora da região visível não deve ser selecionado por clique normal através do crop, salvo modo de edição interno explícito da UI.

A policy de “editar conteúdo dentro do crop” será discutida com Tools; o Engine já fornece hit-test com/sem clip conforme request mode.

## Effects order

Crop/Clip respeita a ordem semântica definida na pipeline.

~~~text
Source
↓ Geometry Effects
↓ Appearance
↓ Post-Paint Effects
↓ Clip / Mask
↓ Node composite
~~~

Se uma feature futura exigir crop antes de effect, isso será outro operation contract; não mover clip arbitrariamente por otimização.

## Expand / destructive operations

Operações destrutivas relacionadas precisam de nomes semânticos distintos:

~~~text
Reset Crop
Detach Clip
Trim Pixels
Expand/Intersect Geometry
Rasterize Cropped Result
~~~

Nenhuma delas acontece automaticamente no save/export.

Export pode rasterizar output sem mudar o Document.

## Resource update

Image crop normalizado permanece quando linked Resource é atualizado.

Se source fica unavailable:

~~~text
ImageObject + crop
↓ Resource unresolved
evaluation unavailable/placeholder
~~~

Crop não é apagado.

## Copy/Paste

Crop via source_rect viaja com ImageObject.

Crop via ClipBinding exige incluir clip source na dependency closure do DocumentFragment, mesmo se `BindingOnly`.

Paste remapeia os ObjectIds da relação.

## Serialization

Persistir:

- ImageObject.source_rect;
- ClipBinding;
- clip source object;
- BindingSourceUse.

Não persistir:

- clipped raster cache;
- cropped preview surface;
- derived bounds;
- temporary crop interaction state.

## Invariantes

1. Crop nunca destrói source implicitamente.
2. Image crop usa source_rect normalizado.
3. Reset Crop remove intenção, não restaura bytes.
4. Generic crop reutiliza ClipBinding.
5. Rect crop source é ShapeObject real, não geometria oculta paralela.
6. PixelLayer Crop não apaga tiles.
7. Trim Pixels é Command destrutivo separado.
8. Crop, Mask, Artboard clipping e Export Slice possuem semânticas diferentes.
9. Hit-test normal respeita clip.
10. Clip permanece na fase semântica definida do pipeline.
11. Export não materializa crop no Document.
12. Copy/Paste inclui clip dependencies e remapeia IDs.
