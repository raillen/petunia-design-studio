# Conteúdo vetorial gerado

O Core precisa representar algumas fontes autorais que **produzem vetor**, mas não são `VectorPath` nem `ParametricShape`.

A v0.1 possui duas famílias:

~~~text
TraceObject
GeneratedVectorObject
~~~

A regra comum é:

> Persistir intenção e parâmetros; geometria avaliada continua Derived State.

## TraceObject

`TraceObject` representa vetor derivado de uma source raster.

Direção:

~~~rust
pub struct TraceObject {
    pub source: ResourceId,
    pub spec: ImageTraceSpec,
}
~~~

`source` identifica a imagem/raster lógico. O Engine resolve os bytes/pixels através do Resource system.

`ImageTraceSpec` pertence ao modelo persistente porque alterar threshold, número de cores, smoothing ou tolerância muda a intenção autoral.

A geometria produzida não é persistida enquanto o trace continuar live.

~~~text
TraceObject
├── ResourceId
└── ImageTraceSpec
        ↓
Image Analysis Engine
        ↓
derived vector regions
~~~

### Identidade

Editar parâmetros mantém o `ObjectId` do SceneNode.

`Expand Trace` materializa novos PathObjects/contours/nodes com IDs persistentes novos. O Command pode preservar o ObjectId do container/objeto principal somente quando a semântica de substituição continuar sendo “o mesmo objeto”; subentidades geradas recebem novas identidades.

### Source ausente

Se o Resource existe no Document mas está unresolved externamente, o TraceObject continua estruturalmente válido e sua avaliação fica `Unavailable`.

Nunca substituir o resultado por vazio permanentemente.

## ImageTraceSpec

O Core persiste apenas parâmetros semânticos; algoritmos pertencem ao Engine.

Direção:

~~~rust
pub struct ImageTraceSpec {
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

`TraceMode` começa com:

~~~rust
pub enum TraceMode {
    Monochrome,
    Grayscale { levels: u16 },
    Color { colors: u16 },
}
~~~

Ranges e finitude são validados no Core/Command boundary; a conversão desses parâmetros em thresholds matemáticos fica no Engine e precisa ser versionada semanticamente.

## GeneratedVectorObject

`GeneratedVectorObject` representa vetor criado por um standard/gerador estruturado.

~~~rust
pub struct GeneratedVectorObject {
    pub generator: GeneratorSpec,
    pub appearance: Appearance,
}
~~~

~~~rust
pub enum GeneratorSpec {
    QrCode(QrCodeSpec),
    Barcode(BarcodeSpec),
}
~~~

Ele não é `ParametricShape`: QR e Barcode codificam dados e regras de standards, não uma primitiva geométrica.

## QR Code

Direção persistente:

~~~rust
pub struct QrCodeSpec {
    pub payload: QrPayload,
    pub error_correction: QrErrorCorrection,
    pub version: QrVersionPolicy,
    pub mask: QrMaskPolicy,
    pub quiet_zone_modules: u8,
}
~~~

O Core valida ranges e estrutura. Reed-Solomon, mask scoring, data placement e escolha de encoder pertencem ao Engine/backend especializado.

A matrix QR é Derived State.

## Barcode

Direção:

~~~rust
pub struct BarcodeSpec {
    pub symbology: BarcodeSymbology,
    pub data: String,
    pub quiet_zone_modules: f64,
    pub bar_height_modules: f64,
}
~~~

Charsets, checksum e encoding são avaliados pelo Engine conforme a symbology.

Human-readable text não fica embutido como outlines no generator; usa o sistema normal de Text quando a feature solicitar label.

## Appearance

QR/Barcode usam `Appearance` normal, permitindo cor e efeitos sem acoplar o encoder ao sistema de pintura.

Trace colorido pode produzir appearances derivados por região/cluster. O `TraceObject` não força uma Appearance única que destruiria as cores da source.

Uma feature futura pode adicionar override global de trace através de parâmetros tipados, mas não entra como campo genérico agora.

## Convert / Expand

~~~text
GeneratedVectorObject
↓ Convert to Curves
PathObject(s)

TraceObject
↓ Expand Trace
PathObject(s)
~~~

Ambos são Commands atômicos e undoable.

O resultado materializado usa qualidade Authoring, nunca aproximação temporária de preview.

## Serialização

Persistir:

- source/GeneratorSpec;
- parâmetros;
- Appearance quando parte do objeto;
- IDs autorais.

Não persistir:

- QR matrix;
- barcode module pattern;
- trace regions;
- contour extraction intermediária;
- fitted paths live;
- bounds;
- tessellation;
- caches.

## Versionamento

Specs persistentes precisam ser versionáveis pelo SchemaVersion/DTO.

Mudanças de algoritmo que alterem output para os mesmos parâmetros exigem uma das opções:

1. manter comportamento antigo para documentos antigos;
2. migrar parâmetros explicitamente;
3. carregar semantic version da operação quando necessário.

Não alterar silenciosamente um QR, Barcode ou Trace já salvo porque o backend mudou.

## Invariantes

1. Trace e GeneratedVector são SceneItems próprios.
2. Nenhum dos dois é ParametricShape.
3. Source + params são autorais; paths avaliados são derivados.
4. Resource unresolved não corrompe TraceObject.
5. QR/Barcode encoding pertence ao Engine, não ao Core.
6. GeneratedVector reutiliza Appearance normal.
7. Trace colorido pode produzir paint derivado por região sem forçar Appearance única.
8. Expand/Convert materializam em qualidade Authoring.
9. Geometria materializada recebe IDs persistentes apropriados.
10. Mudança de backend não pode alterar silenciosamente semântica de documentos existentes.
