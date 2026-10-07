# appearance + effects

Appearance descreve **como a geometria é pintada**. Effects descrevem operações reavaliáveis aplicadas em fases bem definidas.

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

Gradiente guarda geometry em object/document space claramente indicado. O renderer não pode “adivinhar” a unidade dos stops.

## StrokeStyle

Decisões obrigatórias:
- width
- cap: butt/round/square
- join: miter/round/bevel
- miter limit
- dash pattern
- dash offset
- alignment: center/inside/outside quando suportado
- variable width profile
- start/end markers.

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

A ordem da stack é semântica. Drag na UI gera Command de reorder e invalida downstream evaluation.

## Máscara por efeito

Permitir `EffectInstance.mask` evita obrigar o usuário a criar uma camada intermediária para todo efeito localizado.

## Não destrutibilidade

Effect params são fonte da verdade; o resultado nunca substitui pixels/paths originais até um Command explícito de bake/expand.

## Versionamento

Cada effect kind precisa de schema version ou migrador por versão do documento. Plugins precisam namespace próprio para evitar colisão com efeitos built-in.
