# Appearance + Effects

**Appearance** descreve como uma geometria é pintada. **Effect** descreve uma operação reavaliável aplicada antes ou depois dessa pintura.

A ordem semântica do objeto é:

```text
Source Geometry
↓
Geometry Effects
↓
Appearance Items
↓
Post-Paint Effects / Adjustments
↓
Clip / Mask
↓
Node Opacity / Blend into Parent
```

## Appearance

Appearance é uma lista ordenada para permitir interleaving de fills e strokes:

```rust
pub struct Appearance {
    pub items: Vec<AppearanceItem>,
}

pub struct AppearanceItem {
    pub id: AppearanceItemId,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub kind: AppearanceKind,
}

pub enum AppearanceKind {
    Fill(Fill),
    Stroke(Stroke),
}
```

Separar fills e strokes em dois vetores impediria ordens como Fill → Stroke → Fill.

`AppearanceItemId` é identidade persistente para reorder, undo e referência futura.

## Paint

```rust
pub enum Paint {
    Solid(ColorSource),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Pattern(PatternRef),
}
```

Gradients persistem stops, interpolation, spread e espaço geométrico. **Object space** acompanha o objeto; **document space** permanece referenciado ao documento.

Conical Gradient é extensão planejada do mesmo contrato. Mesh Gradient avançado fica fora da v0.1 até ter modelo próprio; não manter um enum incompleto apenas para reservar espaço.

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
