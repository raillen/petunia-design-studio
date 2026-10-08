# Pipeline e Render Graph

Renderer transforma um snapshot avaliado em pixels. Ele não decide regras autorais.

## Contrato de entrada

Render não recebe `SceneGraph` autoral diretamente.

A entrada canônica vem de `petunia-render-model`:

~~~text
Core Document
↓
Engine evaluation
↓
petunia-render-model::RenderSnapshot
↓
petunia-render
↓
pixels
~~~

`RenderSnapshot` é imutável, revisionado e já contém a representação avaliada necessária para desenhar.

Direção de API:

```rust
pub trait RenderBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<RenderStats>;
}

pub struct RenderFrame {
    pub scene: RenderSnapshot,
    pub view: ViewTransform,
    pub target: RenderTarget,
    pub options: RenderOptions,
}
```

`RenderSnapshot` não contém widgets, Commands, history ou estado de sessão. Ele contém somente dados necessários para produzir output visual.

Direção conceitual:

~~~rust
pub struct RenderSnapshot {
    pub revision: DocumentRevision,
    pub pages: Vec<RenderPage>,
    pub resources: RenderResourceTable,
}

pub struct RenderPage {
    pub page: PageId,
    pub primitives: Vec<RenderPrimitive>,
}
~~~

A estrutura real pode usar arenas/arrays compactos internamente, mas z-order e dependências são explícitos.

## Render graph

Um **render graph** descreve as etapas de renderização como uma sequência ou grafo de passes e dependências.

Um **pass** é uma etapa que lê recursos e produz outros recursos, como “rasterizar vetores” ou “compor máscara”.

Passes explícitos permitem:
- offscreen surfaces
- masks
- live filters
- isolated groups
- compositing
- overlays
- color output.

```text
prepare
→ cull
→ evaluate paint primitives
→ allocate intermediates
→ vector/raster/text passes
→ effects
→ composite
→ output transform
→ editor overlays
→ present
```

## Culling

**Culling** é descartar trabalho que não pode afetar a imagem final.

Se o bounds visual de um objeto está totalmente fora do viewport, o renderer normalmente não precisa desenhá-lo.

~~~text
viewport
┌───────────┐
│ objeto A  │       objeto B
└───────────┘

A → renderiza
B → pode ser descartado
~~~

Filtros, shadows ou clips podem expandir a região de influência; por isso usamos **visual bounds**, não apenas bounds geométrico.

## Tessellation

**Tessellation** converte geometria vetorial em primitivas simples, normalmente triângulos, que a GPU ou renderer raster consegue processar eficientemente.

É cache derivado. A chave inclui revisão do path, parâmetros de stroke/fill e tolerância/scale quando necessário.

## CPU reference versus GPU

**CPU** executa código geral nos núcleos do processador. **GPU** executa grandes quantidades de operações gráficas/paralelas.

Manter `SoftwareReferenceRenderer` como caminho determinístico de testes: uma implementação simples e previsível usada como referência de correção.

O backend GPU pode otimizar agressivamente, mas não deve alterar a semântica visual além das tolerâncias documentadas.

## Device pixel ratio

**DPR — Device Pixel Ratio** é a relação entre uma unidade lógica de interface e pixels físicos da tela.

Em uma tela HiDPI, DPR 2 significa que uma unidade lógica pode ocupar 2×2 pixels físicos.

~~~text
Document → View → Device Pixels
~~~

DPR afeta qualidade e tamanho do target de render, mas nunca modifica geometria persistente do documento.

## Quality levels

Interactive preview pode usar tolerância mais relaxada; export usa qualidade alta. Diferença não pode alterar topologia ou significado, apenas aproximação visual permitida.


## RenderPrimitive

O contrato avaliado diferencia primitivas por semântica, não por backend:

~~~rust
pub enum RenderPrimitive {
    Vector(VectorPrimitive),
    Text(TextPrimitive),
    Image(ImagePrimitive),
    Raster(RasterPrimitive),
    Group(RenderGroup),
}
~~~

Geometry Effects já foram avaliados pelo Engine para uma representação adequada. Post-paint effects/composition permanecem descritos para o Render quando sua semântica é raster/compositing.

Isso evita duplicar Geometry Engine dentro do renderer.

## Backend v0.1

A v0.1 terá como backend principal o **software renderer tiled em Rust**.

Motivos:

- headless por padrão;
- comportamento previsível para testes;
- integração simples com export e thumbnails;
- controle fino de memória;
- evita tornar GPU requisito para a primeira versão estável;
- fornece referência semântica para qualquer backend futuro.

GPU permanece backend opcional futuro. Não cria semântica própria.

## Tiled software renderer

A imagem de destino é dividida em tiles de renderização.

~~~text
frame
┌────┬────┬────┐
│ T0 │ T1 │ T2 │
├────┼────┼────┤
│ T3 │ T4 │ T5 │
└────┴────┴────┘
~~~

Para cada primitive:

1. calcular visual bounds em device space;
2. determinar tiles afetados;
3. adicionar primitive aos bins desses tiles;
4. processar tiles independentes em paralelo quando seguro;
5. compor resultados na ordem semântica.

O tamanho do render tile é parâmetro interno medido, não parte do PTND e não precisa ser igual ao tile autoral de PixelSurface.

## Vector rasterization

O software renderer usa pipeline:

~~~text
VectorPath avaliado
↓ transform para device space
adaptive flatten
↓
edges
↓
tile binning
↓
scan conversion com coverage antialias
↓
premultiplied surface
~~~

### Scan conversion

**Scan conversion** transforma boundaries vetoriais em cobertura de pixels.

A v0.1 usa edge walking/scanline determinístico com subpixel coverage.

**Subpixel coverage** estima qual fração do pixel é coberta pela forma, produzindo antialiasing sem depender de blur posterior.

O algoritmo precisa respeitar EvenOdd/NonZero exatamente no mesmo sentido do Core.

A resolução/subdivisão interna de coverage pode evoluir por benchmark; o contrato é coverage estável dentro da tolerância visual do reference renderer.

## Edge binning

Antes de rasterizar todos os edges contra todos os pixels, edges são associados aos tiles que intersectam.

~~~text
edge bounds
↓
intersected tile range
↓
append edge refs to tile bins
~~~

Isso reduz custo em documentos grandes e combina bem com Rayon.

## Text rendering

TextPrimitive já contém glyph IDs/positions resolvidos pelo Text Engine.

Render:

1. resolve/rasteriza glyph coverage;
2. usa `fontdue` para bitmap coverage de glyphs outline convencionais;
3. aplica transform/paint;
4. reutiliza glyph cache quando key compatível.

Convert to Curves não passa por fontdue; essa materialização já acontece no Engine via outlines.

Fontes coloridas seguem o contrato produzido pelo Text Engine:

~~~text
Outline        → fontdue coverage + Paint
COLR/CPAL      → derived vector/paint layers
Raster glyph   → decoded raster primitive
SVG glyph      → safe SVG-derived primitive ou diagnóstico/fallback
~~~

Glyph atlas pode conter coverage ou raster variants, mas atlas slot continua Runtime State. O Render não transforma color glyph em estado autoral.

## Image/Raster rendering

ImagePrimitive referencia recurso/decoded image cache.

RasterPrimitive referencia PixelSurface/tile versions.

Sampling policy é explícita:

~~~text
Nearest
Bilinear
Bicubic
~~~

A v0.1 precisa de Nearest e Bilinear. Bicubic entra quando houver caso real/benchmark que justifique custo.

Não escolher sampling implicitamente por zoom.

## Render graph compilado

O Render Graph é construído a partir do snapshot para representar offscreen dependencies.

~~~text
primitive passes
↓
isolated groups
↓
effect passes
↓
mask passes
↓
composite passes
↓
output transform
↓
overlay
~~~

O graph é Derived State e pode ser cacheado por revision + view/output contract.

Não persistir Render Graph.

## Allocation de intermediários

Offscreen surfaces usam pool interno com lifetime limitado ao frame/graph.

Passes declaram:

- extent;
- pixel format;
- color space;
- read dependencies;
- write target.

O allocator pode reutilizar memória quando lifetimes não se sobrepõem.

Não manter uma surface intermediária viva só porque um efeito existiu em frame anterior; cache de efeito é mecanismo separado.

## Culling por ROI

Culling usa visual bounds + ROI propagation.

Um objeto fora do viewport ainda pode precisar ser processado se um efeito/mask dentro da região visível depender dele.

Portanto:

~~~text
viewport
↓ backward ROI through graph
↓ required primitives/resources
~~~

é mais correto que simplesmente testar geometry bounds isoladamente.

## Quality contract

Três classes:

~~~text
InteractivePreview
Authoring
Export
~~~

InteractivePreview pode:

- usar flatten tolerance maior;
- reduzir resolução temporária de certos efeitos raster;
- adiar caches caros.

Não pode:

- mudar FillRule;
- mudar topologia commitada;
- alterar ordem de composição;
- usar outro BlendMode;
- materializar aproximação no Document.

Authoring é qualidade normal do canvas parado/commitado.

Export usa contrato máximo solicitado pelo destino.

## Error behavior

Render falha por frame/target, não corrompe Document.

Erros típicos:

~~~text
OutOfMemory
UnsupportedTarget
MissingResource
InvalidRenderSnapshot
BackendFailure
~~~

Missing resource pode produzir placeholder diagnosticável quando a semântica permitir. Invalid snapshot é bug/erro de pipeline e deve ser registrado.

## Headless

Software renderer não depende de QApplication, QML ou display server.

~~~text
RenderSnapshot
↓
SoftwareRenderer
↓
offscreen target
↓
PNG/PDF-raster/export/test
~~~

Essa é a base de CI e batch processing.

## Invariantes

1. Render consome `petunia-render-model::RenderSnapshot`, não SceneGraph mutável.
2. Engine e Render não dependem um do outro.
3. Software tiled renderer é o backend primário/reference da v0.1.
4. GPU é backend futuro opcional sob o mesmo contrato.
5. Vector rasterization usa flatten + scan conversion antialias determinística.
6. FillRule do renderer é idêntica à semântica do Core.
7. Render tiles são runtime detail e independem de PixelSurface tiles.
8. Render Graph é Derived State.
9. Intermediários possuem lifetime controlado e pool reutilizável.
10. ROI/culling respeitam efeitos, masks e dependências.
11. Preview quality nunca altera semântica autoral.
12. Headless render é requisito, não caminho secundário.
