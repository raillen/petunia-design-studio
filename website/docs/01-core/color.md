# color.rs

`color.rs` define **valores de cor e metadados semânticos**. Conversões ICC e transformações entre espaços ficam no Color Engine; composição fica no Render.

## Decisão 1 — representação de canais

Usar `f32` para cores autorais e parâmetros. Não usar `u8` como representação canônica porque 8-bit é formato de armazenamento/output, não precisão de edição.

```rust
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub struct Cmyka {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub a: f32,
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
```

O nome `a_axis` evita confundir o eixo a* de Lab com alpha.

## Decisão 2 — valor versus espaço/perfil

RGB não é um espaço completo por si só. O mesmo triplet representa cores diferentes em sRGB, Display P3 ou Adobe RGB. Portanto valor e perfil não devem ser confundidos.

```rust
pub enum ColorValue {
    Rgb(Rgba),
    Cmyk(Cmyka),
    Lab(Laba),
    Gray(Graya),
    Spot(SpotColorRef),
}

pub struct DocumentColorSpec {
    pub model: ColorModel,
    pub profile: ColorProfileRef,
    pub precision: ChannelPrecision,
}
```

Um documento pode ter perfil de trabalho; recursos colocados podem manter perfil de origem até a etapa definida de conversão.

## Decisão 3 — encoded versus linear

Cor para edição/UI costuma estar codificada com uma transfer function; blending e vários filtros devem ocorrer em espaço linear.

Criar tipos distintos para impedir mistura acidental:

```rust
pub struct EncodedRgba(pub Rgba);
pub struct LinearRgba(pub Rgba);
```

A conversão é função do Engine/Render, não método implícito do Core.

## Decisão 4 — alpha

No modelo de documento, armazenar **straight alpha**. Em buffers de composição, usar preferencialmente **premultiplied alpha**.

Motivo: premultiplicação torna Porter–Duff eficiente e evita várias bordas incorretas, mas blend modes são definidos sobre componentes de cor não premultiplicados. A fronteira deve ser explícita.

## Decisão 5 — range e HDR

Não clamping automático em construtores de `f32`. Operações lineares/HDR podem temporariamente produzir valores fora de 0…1. Clamp só quando um formato/output exigir.

Fornecer:
- `is_finite()`
- validação de alpha
- `clamp_for_encoding()`
- conversões explicitamente nomeadas.

## Decisão 6 — spot colors

Spot color é identidade semântica, não apenas CMYK.

```rust
pub struct SpotColor {
    pub id: SpotColorId,
    pub name: String,
    pub alternate: ColorValue,
    pub tint: f32,
}
```

A tinta spot precisa sobreviver a PDF/export/prepress mesmo quando a tela usa uma cor alternativa para preview.

## Decisão 7 — gradientes

Stops pertencem ao Core:

```rust
pub struct GradientStop {
    pub offset: f32,
    pub color: ColorValue,
    pub midpoint: f32,
}
```

Interpolação pertence ao Engine/Render. A política deve dizer se interpola em linear RGB, perceptual ou outro espaço.

## Invariantes

- Canais devem ser finitos.
- Alpha autoral normalmente 0…1.
- Conversões de perfil nunca acontecem silenciosamente.
- `assign profile` e `convert profile` são Commands diferentes.
- Cor de UI/tema não usa os tipos persistentes do documento.
- Soft proof nunca altera a cor do documento.

## Estado atual e migração

O `ColorRgba` atual já usa `f32` normalizado e deve permanecer como base simples enquanto `ColorValue` e `DocumentColorSpec` são introduzidos. `ColorSpace::Srgb/DisplayP3/LinearSrgb` é insuficiente para workflows ICC/CMYK e deve evoluir sem quebrar serialização por migração versionada.

## Referências técnicas

O ICC define PCS e quatro rendering intents; Affinity diferencia assign de convert e mantém soft proof como ajuste. O compositor deve seguir semântica Porter–Duff/blend em espaço adequado.
