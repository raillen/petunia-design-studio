# color.rs

`color.rs` define valores autorais de cor, espaços/perfis referenciados, swatches e parâmetros de gradiente.

Conversão ICC pertence ao Color Management Engine. Composição pertence ao Render.

## Regra central

> **Canal numérico sem espaço/perfil não é uma cor gerenciada completa.**

`RGB(1,0,0)` em sRGB e Display P3 não representa exatamente a mesma cor.

## Canais

Cores autorais usam `f32`.

~~~rust
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
}

pub struct Cmyka {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub alpha: f32,
}

pub struct Laba {
    pub l: f32,
    pub a_axis: f32,
    pub b_axis: f32,
    pub alpha: f32,
}

pub struct Graya {
    pub gray: f32,
    pub alpha: f32,
}
~~~

`a_axis` evita confundir o eixo a* de Lab com alpha.

8-bit/16-bit inteiros são formatos de armazenamento/output, não a representação canônica de edição.

## Process color

Cores processuais carregam valor + referência de espaço.

~~~rust
pub enum ProcessColorValue {
    Rgb(Rgba),
    Cmyk(Cmyka),
    Lab(Laba),
    Gray(Graya),
}

pub struct ProcessColor {
    pub value: ProcessColorValue,
    pub space: ColorSpaceRef,
}
~~~

`ColorSpaceRef` pode referenciar:

- espaço built-in conhecido;
- perfil ICC incorporado;
- perfil ICC externo resolvido;
- espaço definido pelo documento.

A representação concreta pode evoluir, mas a referência é explícita.

## ColorValue

~~~rust
pub enum ColorValue {
    Process(ProcessColor),
    Spot(SpotColorRef),
}
~~~

Spot não é tratado como “CMYK especial”; possui identidade própria.

## DocumentColorSpec

Documento define defaults/working spaces, não reinterpreta silenciosamente toda cor existente.

~~~rust
pub struct DocumentColorSpec {
    pub rgb_working: ColorSpaceRef,
    pub cmyk_working: Option<ColorSpaceRef>,
    pub gray_working: Option<ColorSpaceRef>,
    pub rendering_defaults: RenderingDefaults,
}
~~~

Recursos colocados podem preservar perfil próprio.

## Assign versus Convert

**Assign profile** muda a interpretação do mesmo número.

**Convert profile** recalcula números para preservar aparência tanto quanto possível.

São Commands diferentes.

~~~text
Assign
numbers same
profile changes
appearance may change

Convert
appearance target preserved
numbers may change
~~~

Nunca fazer Convert implicitamente porque um painel foi aberto ou o working space mudou.

## Encoded e linear

Espaços como sRGB usam transfer function.

**Encoded RGB** é valor após essa curva.

**Linear RGB** é proporcional à intensidade luminosa representada.

Criar distinção de tipos onde reduzir erro:

~~~rust
pub struct EncodedRgba(pub Rgba);
pub struct LinearRgba(pub Rgba);
~~~

A conversão depende do espaço/perfil e pertence ao Engine/Render.

## Alpha

No Document, alpha é **straight**.

~~~text
red = 1
alpha = 0.5
~~~

Em buffers de compositor, preferir **premultiplied alpha**.

~~~text
premultiplied red = 0.5
alpha = 0.5
~~~

Não persistir cor premultiplicada como valor autoral comum.

## Range

Alpha autoral precisa ser finito e normalmente fica em 0…1.

RGB linear intermediário pode ultrapassar 0…1 em HDR/effects.

Não clamp automaticamente um valor apenas porque a UI tradicional espera 0…1.

Clamping ocorre na fronteira que exige range específico.

## Spot colors

~~~rust
pub struct SpotColor {
    pub id: SpotColorId,
    pub name: String,
    pub alternate: ProcessColor,
}
~~~

A identidade da tinta é `SpotColorId`.

Tint pertence ao uso da spot, não necessariamente à definição global:

~~~rust
pub struct SpotColorRef {
    pub id: SpotColorId,
    pub tint: f32,
}
~~~

Isso permite usar a mesma tinta em diferentes percentuais.

`alternate` serve para preview quando o dispositivo não reproduz a tinta real.

## Swatches

Swatch é recurso documental reutilizável.

~~~rust
pub struct Swatch {
    pub id: SwatchId,
    pub name: String,
    pub value: SwatchValue,
}

pub enum SwatchValue {
    Color(ColorValue),
    Gradient(Gradient),
}
~~~

Se Swatch for **linked**, objetos referenciam SwatchId e mudanças propagam.

Se a cor for copiada como valor local, deixa de depender do swatch.

A escolha entre linked/local precisa ser explícita no Paint.

## Gradients

Stops são Document State.

~~~rust
pub struct GradientStop {
    pub offset: f32,
    pub color: ColorSource,
    pub midpoint: f32,
}
~~~

`offset` é normalizado em 0…1. `midpoint` controla a posição perceptual da mistura entre stops adjacentes e precisa de range definido.

### ColorSource

~~~rust
pub enum ColorSource {
    Value(ColorValue),
    Swatch(SwatchId),
}
~~~

Assim Paint pode optar por cor local ou vinculada.

## Interpolação

Gradiente precisa persistir sua política de interpolação.

Direção:

~~~rust
pub enum GradientInterpolation {
    LinearRgb,
    EncodedRgb,
    Lab,
    Oklab,
}
~~~

`LinearRgb` é default técnico inicial por comportamento previsível para luz/composição.

Adicionar modos perceptuais não exige mudar estrutura do gradient.

Interpolation não é inferida pelo renderer.

## Spread mode

~~~rust
pub enum GradientSpread {
    Pad,
    Repeat,
    Reflect,
}
~~~

**Pad** mantém cores extremas fora dos stops.

**Repeat** repete o gradiente.

**Reflect** repete alternando direção.

## Geometry de gradient

Gradiente declara espaço de coordenadas.

~~~rust
pub enum PaintSpace {
    Object,
    Document,
}
~~~

Linear e radial persistem seus pontos/radii no espaço escolhido.

Transformar objeto não deve “adivinhar” se gradient acompanha o objeto: PaintSpace torna isso explícito.

## Conical gradient

Conical gradient é compatível com o mesmo modelo de stops/spread/space e entra como capacidade planejada.

Não precisa de novo sistema de cor.

## Mesh gradient

Mesh gradient avançado continua futuro.

Não persistir `MeshGradient` incompleto apenas para reservar enum na v0.1.

Quando especificado, deve ter modelo próprio e migration.

## HDR

Arquitetura permite RGB float extended range.

HDR completo exige posteriormente definir:

- working space;
- transfer function;
- luminance semantics;
- metadata de output;
- tone mapping.

Não prometer HDR profissional apenas porque `f32` aceita valores > 1.

## Finitude e validação

Todos os canais persistentes são finitos.

Ranges específicos são validados por modelo.

Exemplos:

~~~text
alpha: 0..1
CMYK process channels: 0..1
GradientStop offset: 0..1
Spot tint: 0..1
~~~

Lab usa ranges semânticos próprios; não normalizar eixos a* e b* arbitrariamente só para “caber” em 0…1.

## Cor de UI

Theme/UI colors não usam ColorValue persistente do Document.

Elas pertencem ao domínio UI.

## Soft proof

Soft proof é View State.

Ele transforma a apresentação através do Color Engine/Render e nunca altera ColorValue do documento.

## Invariantes

1. Cor processual sempre referencia espaço/perfil.
2. Cores autorais usam float, não u8 canônico.
3. Straight alpha é persistente; premultiplied é representação de composição.
4. Assign e Convert são operações distintas.
5. Conversão ICC nunca acontece silenciosamente.
6. Spot possui identidade independente do alternate preview.
7. Gradient persiste interpolation/spread/space.
8. Mesh gradient avançado não entra incompleto na v0.1.
9. HDR é arquitetura preparada, não promessa de workflow completo.
10. UI theme colors não contaminam o modelo documental.
