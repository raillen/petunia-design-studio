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

Stroke persiste intenção; outline expandido é Derived State até `Expand Stroke`.

```rust
pub struct StrokeStyle {
    pub width: f64,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f64,
    pub alignment: StrokeAlignment,
    pub dash: DashPattern,
    pub variable_width: Option<VariableWidthProfile>,
    pub start_marker: Option<MarkerRef>,
    pub end_marker: Option<MarkerRef>,
}
```

**Cap** pode ser Butt, Round ou Square. **Join** pode ser Miter, Round ou Bevel. `miter_limit` limita spikes em ângulos agudos.

`StrokeAlignment` suporta Center/Inside/Outside para contours fechados. Em open paths, v0.1 usa Center; não inventar uma semântica ambígua para inside/outside.

Dash usa comprimentos por arc length. Padrão não pode ter ciclo total zero. Se a lista tiver quantidade ímpar, o evaluator duplica a sequência de forma determinística.

Variable width usa posição normalizada por arc length. Markers são definidos por recurso/definição vetorial e sua colocação usa tangente derivada do path.

## BlendMode

Enum persistente alinhado a um vocabulário estável: Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Hue, Saturation, Color, Luminosity.

Não serializar nomes localizados.

## Effects por fase

Não misturar geometry effects e post-paint effects em um enum sem fase.

```rust
pub struct GeometryEffectStack {
    pub items: Vec<GeometryEffectInstance>,
}

pub enum PostPaintEffect {
    GaussianBlur(BlurParams),
    DropShadow(ShadowParams),
    InnerShadow(ShadowParams),
    Glow(GlowParams),
    Adjustment(AdjustmentRef),
}
```

Geometry Effects operam antes de Appearance. Post-Paint Effects recebem o resultado já pintado.

```rust
pub struct EffectInstance<T> {
    pub id: EffectId,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask: Option<MaskRef>,
    pub operation: T,
}
```

Cada effect possui identidade estável para reorder, history e cache.

## Ordem

A ordem da stack é semântica.

**Downstream** significa “tudo que depende deste resultado depois dele no fluxo”. Se um efeito anterior muda, os efeitos posteriores precisam ser reavaliados.

Drag na UI gera Command de reorder e invalida somente a avaliação downstream afetada.

## Máscara por efeito

`EffectInstance.mask` limita apenas aquela operação. Isso evita grupos artificiais para todo efeito localizado.

## Níveis de opacity

AppearanceItem opacity, Effect opacity, Node opacity e Group opacity ocorrem em pontos diferentes do pipeline. Não colapsar esses valores em um único campo.

## Group isolation

Quando blend/effects do Group exigem isolamento, Render compõe os filhos em uma superfície intermediária antes de misturar o grupo com o parent. Isso é consequência semântica do node, não opção visual de UI.

## Não destrutibilidade

Effect params são fonte da verdade; o resultado nunca substitui pixels/paths originais até Command explícito.

`Expand Appearance` materializa somente partes com representação vetorial equivalente. Effects raster-only exigem `Rasterize` ou `Bake Effect` separado.

## Styles

Style compartilhado pode fornecer Appearance por referência. Objetos podem permanecer linked, ter appearance local ou aplicar overrides explícitos.

Não copiar silenciosamente o style inteiro para o objeto enquanto a relação continua linked.

## Versionamento

Cada effect kind precisa de **schema version** — versão da estrutura persistida de seus parâmetros — ou migrador por versão do documento.

Plugins precisam **namespace** próprio, isto é, um prefixo/identidade que evite colisão entre nomes de efeitos de origens diferentes.


## Invariantes

1. Appearance é uma lista ordenada e permite interleaving Fill/Stroke.
2. AppearanceItem possui identidade persistente.
3. Geometry Effects ocorrem antes de Appearance.
4. Post-Paint Effects operam depois da pintura.
5. Opacity em níveis diferentes mantém semântica própria.
6. Stroke outline é Derived State até Expand Stroke.
7. Inside/Outside stroke não recebe semântica ambígua em open paths.
8. Effect params são autorais; resultado/cache não é.
9. Expand, Bake e Rasterize são Commands distintos.
10. BlendMode é persistente, mas sua matemática pertence ao Render.
