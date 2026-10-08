# Image Analysis + Vectorization

Esta página define motores que analisam raster e produzem dados derivados: **Palette Extraction** e **Image Trace**.

Eles pertencem ao Engine. A UI futura apenas escolhe parâmetros e apresenta preview.

## Princípio

~~~text
ImageResource / PixelSurface snapshot
↓
Image Analysis Engine
↓
derived result
↓
preview
↓ explicit Command
Document State
~~~

Análise nunca modifica a imagem source.

# Color Quantizer

Palette Extraction e Color Trace compartilham um único `ColorQuantizer`.

Isso evita duas implementações que agrupam cores de forma diferente.

## Espaço perceptual

Pixels são convertidos para uma representação perceptual derivada antes de medir distância entre cores.

A v0.1 usa **Oklab** para clustering perceptual.

Oklab representa:

~~~text
L → luminosidade perceptual
a/b → eixos cromáticos
~~~

Distância Euclidiana em Oklab não é uma métrica perceptual perfeita, mas é muito mais coerente para clustering que distância direta em RGB encoded.

A conversão é Derived State; a imagem source permanece no perfil original.

## Sampling

Imagem muito grande não precisa colocar todos os pixels no clustering inicial.

A v0.1 usa sampling determinístico estratificado:

1. dividir a imagem em células regulares;
2. coletar amostras distribuídas pelas células;
3. ponderar pela frequência/coverage;
4. incluir pixels de regiões pequenas quando passam pelo refinement.

Não usar RNG global.

Se sampling aleatório for necessário em uma otimização futura, seed faz parte da request.

## Quantization v0.1

Pipeline:

~~~text
samples Oklab
↓
median-cut initialization
↓
deterministic k-means refinement
↓
palette centers
↓
assign pixels/clusters
~~~

### Median cut

**Median cut** divide recursivamente o conjunto de cores.

Em cada etapa:

1. escolher a caixa com maior range/variância útil;
2. escolher o eixo perceptual dominante;
3. ordenar samples nesse eixo;
4. dividir próximo da mediana ponderada;
5. repetir até chegar ao número desejado de clusters.

Isso produz centros iniciais estáveis.

### K-means refinement

**K-means** alterna:

~~~text
assign sample → centro mais próximo
↓
recalcular centro como média ponderada
↓
repetir
~~~

A inicialização vem do median cut, portanto não depende de escolha aleatória.

Parar quando:

- assignments não mudam significativamente; ou
- deslocamento dos centros <= tolerance; ou
- atingir iteration guard.

Empty cluster é removido/mesclado deterministicamente; não recebe centro aleatório.

## Alpha

Pixels com alpha abaixo de threshold configurado podem ser ignorados na análise de cor.

Para pixels semitransparentes, a política é explícita:

~~~text
IgnoreTransparent
WeightByAlpha
CompositeAgainst(reference color)
~~~

Default técnico para Palette Extraction: `WeightByAlpha`.

Image Trace escolhe conforme TraceSpec.

# Palette Extraction

## Request

~~~rust
pub struct PaletteExtractionSpec {
    pub colors: u16,
    pub alpha_policy: AnalysisAlphaPolicy,
    pub min_population: f32,
    pub merge_delta: f32,
}
~~~

Valores são direção de API; ranges finais são validados no Engine.

## Pipeline

~~~text
image snapshot
↓ color-manage to analysis space
↓ deterministic sampling
↓ quantization
↓ merge near-duplicate centers
↓ population ranking
↓ output palette
~~~

Resultado:

~~~rust
pub struct ExtractedColor {
    pub color: ColorValue,
    pub population: f32,
}
~~~

A extração não cria Swatches automaticamente.

`Create Swatches from Palette` é Command separado, com Undo.

## Near-duplicate merge

Após clustering, centros perceptualmente muito próximos podem ser unidos se distância <= `merge_delta`.

A união é ponderada por população.

Não remover uma cor rara apenas por “parecer pouco usada” sem `min_population` explícito.

# Image Trace

Image Trace preserva a imagem source e armazena parâmetros quando usado como operação live.

## Modes

A v0.1 define:

~~~rust
pub enum TraceMode {
    Monochrome,
    Grayscale { levels: u16 },
    Color { colors: u16 },
}
~~~

Não criar dezenas de modes especializados antes de existir necessidade.

## TraceSpec

Direção:

~~~rust
pub struct TraceSpec {
    pub mode: TraceMode,
    pub alpha_policy: AnalysisAlphaPolicy,
    pub threshold: f32,
    pub min_region_area_px: f64,
    pub corner_sensitivity: f32,
    pub smoothing: f32,
    pub curve_tolerance: f64,
    pub ignore_background: Option<ColorValue>,
}
~~~

Parameters são autorais quando Live Trace existir.

Derived contours/paths não são persistidos enquanto a operação continuar live.

## Monochrome pipeline

~~~text
image
↓ color management / luminance
coverage field
↓ threshold
binary region mask
↓ cleanup by explicit params
contour extraction
↓ simplify
corner detection
↓ cubic curve fitting
VectorPath result
~~~

### Luminance

Threshold trabalha em luminância derivada no espaço de análise definido pelo Color Engine.

Não usar média `(r+g+b)/3`.

### Threshold

~~~text
luminance < threshold → foreground
luminance >= threshold → background
~~~

A polaridade pode ser invertida por parâmetro quando necessário.

Alpha policy participa antes da decisão.

## Color pipeline

~~~text
image
↓ ColorQuantizer
cluster assignment map
↓
one region label per pixel
↓
connected regions per color
↓
contour extraction
↓
geometry cleanup/fitting
↓
filled VectorPaths + colors
~~~

Cada pixel pertence a um único cluster na representação base. Isso evita múltiplas masks concorrentes para a mesma amostra.

## Grayscale

Grayscale trace usa quantização ordenada de luminância em níveis e depois o mesmo pipeline de regions.

Os níveis são estáveis e deterministicamente ordenados.

## Region connectivity

A v0.1 usa conectividade 8-neighbor para agrupar pixels de uma mesma região.

**8-neighbor** considera laterais e diagonais adjacentes.

A escolha reduz fragmentação diagonal em raster antialiasado.

Contour extraction continua responsável por produzir boundary geométrica consistente.

## Contour extraction

A boundary subpixel usa **Marching Squares** sobre um campo de coverage/labels.

Marching Squares examina cada célula formada por quatro samples e decide como a boundary atravessa essa célula.

~~~text
p00 ---- p10
 |        |
 | cell   |
 |        |
p01 ---- p11
~~~

Cada corner está dentro/fora da região.

Os 16 padrões possíveis produzem segmentos locais.

### Ambiguous cases

Casos diagonais podem ter duas conexões possíveis.

A v0.1 resolve pela conectividade escolhida e pelo valor do campo no centro da célula de forma determinística; não escolhe aleatoriamente.

## Holes

Contours são classificados por nesting/FillRule.

Uma região com ilha vazia produz hole real, não dois objetos preenchidos sobrepostos por acaso.

A orientação final segue a convenção Y-down do Petunia e a FillRule definida.

## Region cleanup

`min_region_area_px` remove regiões menores que o limite solicitado.

Isso é parâmetro explícito de trace, nunca cleanup silencioso global.

Morphological close/open pode ser adicionado futuramente como parâmetro separado; não aplicar automaticamente.

## Simplification

Contour raster inicialmente possui muitos vertices.

Pipeline:

~~~text
raw contour polyline
↓ RDP with trace tolerance
↓ corner protection
↓ curve fitting
~~~

RDP trabalha somente na polyline derivada.

Não simplificar novamente cubics finais com flatten+RDP sem necessidade.

## Corner detection

Calcular mudança angular/local curvature na polyline simplificada.

Pontos acima da sensibilidade de corner viram boundaries de curve fit.

`corner_sensitivity` é convertido para threshold matemático por função versionada do Engine.

Preview e Expand usam a mesma conversão.

## Curve fitting

Usar o mesmo fitter cubic do Geometry Engine baseado no algoritmo Schneider.

~~~text
polyline section
↓ tangent estimates
cubic fit
↓ max deviation
accept or split
~~~

`curve_tolerance` limita erro máximo permitido.

Isso evita manter um segundo curve fitter exclusivo do Image Trace.

## Path reconstruction

Output usa os tipos canônicos Petunia:

~~~text
ContourId/NodeId novos
Line/Cubic segments
FillRule
Appearance with Solid fill
~~~

Live result usa IDs derivados/transitórios.

`Expand Trace` cria IDs persistentes novos na Transaction.

## Layer/order policy

Para Color/Grayscale, result paths são ordenados deterministically:

1. region nesting;
2. cluster order;
3. area;
4. stable geometric tie-break.

A ordem não depende de HashMap ou worker scheduling.

A composição visual precisa ser testada contra o raster source.

## Background removal

`ignore_background` remove regiões cujo cluster corresponde ao background dentro de tolerance perceptual explícita.

“Remove White” de Image Trace não é o mesmo que Color-to-Alpha raster effect.

Trace simplesmente não gera vector region para aquele cluster.

## Live Trace model

Direção conceitual:

~~~rust
pub struct LiveTrace {
    pub source: ResourceId,
    pub params: TraceSpec,
}
~~~

Evaluation:

~~~text
ImageResource
↓ LiveTrace params
Image Analysis Engine
↓ derived VectorPaths
Appearance
↓ Render
~~~

Mudar a imagem source ou params invalida o trace.

## Expand Trace

Command:

1. valida source revision;
2. avalia em qualidade Authoring;
3. materializa PathObjects;
4. gera IDs persistentes;
5. preserva colors/fill semantics;
6. substitui ou insere resultado conforme Command;
7. guarda inverse data para Undo.

Nunca materializa preview de baixa qualidade.

## Cancellation e progress

Trace é Background job para imagens relevantes.

Phases publicáveis:

~~~text
Preparing
Quantizing
Segmenting
TracingContours
FittingCurves
Finalizing
~~~

Cancelamento ocorre entre batches/regions e dentro de loops caros.

## Complexity guards

Validar:

- source dimensions;
- requested color count;
- maximum regions;
- contour vertex count;
- curve fitting recursion;
- output object/node count;
- temporary allocation.

Se limite operacional for excedido, retornar diagnóstico e sugestão de parâmetros menos custosos. Não travar UI nem produzir resultado parcial como completo.

## Determinism

Mesma source bytes/profile + mesma TraceSpec + mesma engine semantic version produz:

- mesma palette order;
- mesma region classification;
- mesma contour order;
- mesma geometry dentro do contrato numérico.

Parallelism pode mudar scheduling, não output autoral.

## Cache

Key:

~~~text
source content hash/revision
+ source color context
+ TraceSpec
+ analysis engine semantic version
+ quality
~~~

Cache pode guardar quantization/region intermediates para previews sucessivos.

É descartável.

## Testing

Fixtures precisam incluir:

- high-contrast logo;
- antialiased line art;
- holes;
- diagonal boundaries;
- tiny regions;
- gradients;
- transparent background;
- limited color illustration;
- noisy/photo source;
- very large source with cancellation.

Métricas:

- max geometric deviation;
- region count;
- node count;
- color error/population;
- runtime/memory;
- determinism.

## Invariantes

1. Palette Extraction e Image Trace compartilham ColorQuantizer.
2. Clustering acontece em espaço perceptual derivado, inicialmente Oklab.
3. Quantization é determinística: median cut + k-means refinement.
4. Palette Extraction não cria Swatches sem Command.
5. Live Trace persiste source + params, não derived paths.
6. Marching Squares extrai boundaries com ambiguity policy determinística.
7. Regions usam connectivity explícita.
8. Simplify usa RDP em polylines derivadas; cubics usam Geometry curve fitting.
9. Curve tolerance limita erro de fitting.
10. Expand Trace usa qualidade Authoring e IDs novos.
11. Background removal de trace não é Color-to-Alpha.
12. Jobs são canceláveis e possuem complexity guards.
13. Parallelism não altera output semântico.
