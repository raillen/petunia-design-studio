# appearance + effects

**Appearance** descreve como uma geometria é pintada: fills, strokes, opacity e blend.

**Effect** descreve uma operação reavaliável aplicada antes ou depois da pintura, como blur, shadow ou ajuste de cor.

## Appearance

Suportar múltiplos paints no futuro sem quebrar modelo:

```rust
pub struct Appearance {
    pub fills: Vec<FillLayer>,
    pub strokes: Vec<StrokeLayer>,
}

pub struct FillLayer {
    pub paint: Paint,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub enabled: bool,
}

pub struct StrokeLayer {
    pub paint: Paint,
    pub style: StrokeStyle,
    pub opacity: f32,
    pub enabled: bool,
}
```

## Paint

```rust
pub enum Paint {
    Solid(ColorValue),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    MeshGradient(MeshGradientRef),
    Pattern(PatternRef),
}
```

Um gradiente precisa declarar em qual sistema de coordenadas vive:

- **object space** — relativo ao próprio objeto;
- **document space** — relativo ao documento.

Um **stop** é um ponto do gradiente que define posição e cor. O renderer não pode “adivinhar” a unidade ou o espaço dos stops.

## StrokeStyle

Um stroke é a linha desenhada ao redor ou ao longo de um path.

Decisões obrigatórias:

- **width** — espessura;
- **cap** — acabamento das extremidades abertas: butt, round ou square;
- **join** — como dois segmentos se conectam em um canto: miter, round ou bevel;
- **miter limit** — limite que impede pontas excessivamente longas em cantos agudos;
- **dash pattern** — sequência de traço/espaço;
- **dash offset** — deslocamento inicial dessa sequência;
- **alignment** — center/inside/outside quando suportado;
- **variable width profile** — variação da espessura ao longo do path;
- **start/end markers** — elementos como setas nas extremidades.

## BlendMode

Enum persistente alinhado a um vocabulário estável: Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Hue, Saturation, Color, Luminosity.

Não serializar nomes localizados.

## EffectStack

Cada node pode possuir uma stack. Groups também podem possuir effects.

```rust
pub struct EffectStack {
    pub items: Vec<EffectInstance>,
}

pub enum EffectKind {
    GaussianBlur(BlurParams),
    DropShadow(ShadowParams),
    InnerShadow(ShadowParams),
    OuterGlow(GlowParams),
    ColorAdjustment(AdjustmentRef),
    Geometry(GeometryEffect),
}
```

## Ordem

A ordem da stack é semântica.

**Downstream** significa “tudo que depende deste resultado depois dele no fluxo”. Se um efeito anterior muda, os efeitos posteriores precisam ser reavaliados.

Drag na UI gera Command de reorder e invalida somente a avaliação downstream afetada.

## Máscara por efeito

Permitir `EffectInstance.mask` evita obrigar o usuário a criar uma camada intermediária para todo efeito localizado.

## Não destrutibilidade

Effect params são fonte da verdade; o resultado nunca substitui pixels/paths originais até um Command explícito de bake/expand.

## Versionamento

Cada effect kind precisa de **schema version** — versão da estrutura persistida de seus parâmetros — ou migrador por versão do documento.

Plugins precisam **namespace** próprio, isto é, um prefixo/identidade que evite colisão entre nomes de efeitos de origens diferentes.
