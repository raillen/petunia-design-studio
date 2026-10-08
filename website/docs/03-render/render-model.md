# Render Model

`petunia-render-model` é a crate de contrato imutável entre Engine e Render.

Ela existe para resolver uma fronteira concreta:

~~~text
Engine ──produz──> Render Model <──consome── Render
  │                                      │
  └──────────────> Core <────────────────┘
~~~

Sem essa crate, Engine precisaria depender do Render para construir tipos de renderização, ou Render precisaria depender do Engine para compreender resultados avaliados. Nenhuma das duas opções é aceitável.

## Responsabilidade

Render Model descreve **o que deve ser desenhado**, não **como desenhar**.

Contém:

- snapshots avaliados;
- primitivas vetoriais;
- glyph runs posicionados;
- imagens/raster refs;
- grupos/composite descriptors;
- effect descriptors raster/compositor;
- resource handles imutáveis;
- bounds/ROI necessários ao Render.

Não contém:

- SceneGraph autoral mutável;
- Commands/Transactions;
- History;
- Qt/QML;
- rasterizer;
- cache;
- thread pool;
- GPU handles;
- Geometry algorithms.

## RenderSnapshot

Direção:

~~~rust
pub struct RenderSnapshot {
    pub revision: DocumentRevision,
    pub pages: Vec<RenderPage>,
    pub resources: RenderResourceTable,
}

pub struct RenderPage {
    pub page_id: PageId,
    pub size: Size2,
    pub primitives: Vec<RenderPrimitive>,
}
~~~

A ordem em `primitives` é semântica quando representa paint order.

Se uma implementação futura usar arrays/arenas separados por tipo para performance, precisa preservar uma `PaintOrderKey` ou sequência equivalente.

## RenderPrimitive

~~~rust
pub enum RenderPrimitive {
    Vector(VectorPrimitive),
    Text(TextPrimitive),
    Image(ImagePrimitive),
    Raster(RasterPrimitive),
    Group(RenderGroup),
}
~~~

### VectorPrimitive

Representa geometry já avaliada para pintura.

Direção:

~~~rust
pub struct VectorPrimitive {
    pub source: ObjectId,
    pub geometry: RenderPath,
    pub appearance: RenderAppearance,
    pub transform: Transform2D,
    pub bounds: Rect,
}
~~~

`RenderPath` pode ser uma estrutura compacta própria do Render Model. Não precisa ser igual a `VectorPath` autoral nem a `kurbo::BezPath`.

### TextPrimitive

Contém layout derivado:

~~~text
source ObjectId
glyph runs
resolved font handles/ids
positions
paint
bounds
~~~

Não contém `TextObject` inteiro nem reexecuta shaping no Render.

### ImagePrimitive

Contém:

~~~text
ResourceId/runtime resource handle
source rect
transform
sampling policy
opacity/blend context
bounds
~~~

Decoded pixels continuam Resource/Render cache, não são copiados para cada primitive.

### RasterPrimitive

Referencia PixelSurface/tile version snapshot.

O Render Model não expõe storage mutável da PixelSurface.

## RenderGroup

Group representa uma subtree que precisa preservar semântica de composição.

~~~rust
pub struct RenderGroup {
    pub source: ObjectId,
    pub children: Vec<RenderPrimitive>,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask: Option<RenderMask>,
    pub clip: Option<RenderClip>,
    pub effects: Vec<RenderEffect>,
    pub isolation: IsolationMode,
    pub bounds: Rect,
}
~~~

Uma subtree simples pode ser flattenada pelo compiler/evaluator quando isso for semanticamente idêntico.

## Evaluation boundary

Engine produz Render Model em duas etapas conceituais:

~~~text
Document Snapshot
↓
authorial evaluation
  geometry effects
  parametric shapes
  text layout
  resource resolution metadata
↓
Render compilation
  paint primitives
  group/effect descriptors
  stable paint order
↓
RenderSnapshot
~~~

Essa compilação é Derived State.

Não cria DocumentRevision.

## Preview

Preview usa:

~~~text
base DocumentSnapshot
+
TransientOverrides
↓
Engine evaluation
↓
temporary RenderSnapshot / RenderPatch
~~~

A implementação pode otimizar para patches incrementais, mas a semântica é a mesma.

Preview nunca modifica o RenderSnapshot autoritativo de uma revision.

## RenderPatch

`RenderPatch` é otimização futura permitida, não requisito inicial.

Pode representar:

~~~text
replace primitive O1
invalidate group G4
update resource R7
remove primitive O9
~~~

Primeiro implementar snapshot completo correto. Introduzir patches apenas se profiling mostrar custo relevante na reconstrução.

## Resource handles

Render Model usa referências imutáveis estáveis durante o lifetime do snapshot.

Nunca serializar:

- pointer;
- GPU texture handle;
- atlas slot;
- file descriptor.

Esses detalhes pertencem ao Render runtime.

## Bounds

Toda primitive/group carrega visual bounds conservador quando disponível.

**Conservador** significa que o bounds pode incluir área extra, mas nunca excluir pixels que a primitive possa produzir.

Bounds incorreto para menor causa clipping/culling inválido e é bug.

## Quality

RenderSnapshot pode ser compilado para:

~~~text
InteractivePreview
Authoring
Export
~~~

Quality faz parte da key do resultado derivado.

Diferenças permitidas afetam aproximação/custo, não semântica autoral.

## Versionamento

Render Model é contrato interno do workspace, não formato de arquivo.

Mudanças podem seguir versões do código sem migration PTND.

Mesmo assim, manter tipos coesos e estáveis o suficiente para evitar acoplamento Engine↔backend.

## Erros

Compilação para Render Model pode produzir:

~~~text
MissingResource
UnsupportedEffect
EvaluationFailure
TextLayoutFailure
ComplexityGuardExceeded
Cancelled
~~~

Falha não muta Document.

Quando uma primitive pode ser degradada com segurança, o resultado precisa carregar warning/diagnostic; não esconder perda de fidelidade.

## Invariantes

1. Render Model é contrato, não backend.
2. Engine produz; Render consome.
3. A crate pode depender de tipos estáveis do Core, nunca de UI.
4. Não contém algoritmos de Geometry/Text layout.
5. Não contém caches/handles de GPU autoritativos.
6. RenderSnapshot pertence a uma DocumentRevision.
7. Paint order é explícito e determinístico.
8. Bounds são conservadores.
9. Preview usa a mesma semântica com overrides transitórios.
10. RenderPatch só entra se profiling justificar.
11. Render Model não é serializado em PTND.
12. Falha de compilação não altera Document.
