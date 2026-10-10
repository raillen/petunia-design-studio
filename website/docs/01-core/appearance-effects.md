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
    pub paint: ColorSource,
    pub variable_width: Option<VariableWidthProfile>,
    pub start_marker: Option<MarkerRef>,
    pub end_marker: Option<MarkerRef>,
}
```

O stroke carrega a própria tinta: um item de stroke nunca toma a tinta do fill por implicitude, então a cor renderizada continua explícita na ordem da stack.

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


## Pattern

Pattern é um Paint reutilizável, não uma imagem “colada” no objeto.

Direção:

~~~rust
pub struct PatternPaint {
    pub source: PatternSourceRef,
    pub transform: Transform2D,
    pub space: PaintSpace,
    pub repeat_x: PatternRepeat,
    pub repeat_y: PatternRepeat,
}
~~~

`PatternSourceRef` pode apontar para recurso raster ou definição vetorial autorizada.

`PatternRepeat` começa com:

~~~text
Repeat
Clamp
Mirror
~~~

Não duplicar tiles/padrões fisicamente só para preencher uma área.

O transform do pattern é autoral. Cache de pattern rasterizado é Derived State.

## Variable width

Variable width precisa ser descrita em coordenada normalizada de arc length:

~~~rust
pub struct VariableWidthProfile {
    pub points: Vec<WidthPoint>,
}

pub struct WidthPoint {
    pub position: f32,   // 0..1 ao longo do arc length
    pub scale: f32,      // multiplicador da largura base
}
~~~

Invariantes:

- points ordenados por position;
- position finita em 0..1;
- scale finita e >= 0;
- duplicate positions seguem política explícita de merge/reject.

Interpolation inicial é linear entre points.

O perfil não armazena distância absoluta em document units porque editar o comprimento do path deveria reparametrizar a distribuição relativa, não mover arbitrariamente todos os control points do width profile.

## Markers

Marker é definição vetorial reutilizável, normalmente arrowhead ou endpoint symbol.

Marker placement usa:

~~~text
path endpoint
+ tangent
+ stroke width
+ marker transform policy
↓
derived marker geometry
~~~

O Core persiste somente referência + parâmetros.

Expansão para path é Derived State até `Expand Appearance`.

Marker nunca altera os nodes do path só para “caber” no endpoint.

## Stroke alignment

Para closed contours:

~~~text
Center
Inside
Outside
~~~

são suportados semanticamente.

Para open contours:

~~~text
Center
~~~

é a única semântica definida na v0.1.

Import de formato externo com inside/outside em open path deve preservar metadata quando possível, mas o Core não inventa uma interpretação visual silenciosa.

## Dash normalization

Dash pattern usa comprimentos positivos/finitos em document units ou unidade relativa explicitamente declarada pelo modelo final.

Regras:

- valores negativos → erro;
- soma total zero → erro;
- lista vazia → stroke contínuo;
- quantidade ímpar → sequência duplicada para formar período par;
- dash offset é normalizado modularmente pelo comprimento total do padrão.

A normalização é derivada e determinística.

## Geometry Effect params

Os parâmetros built-in são structs tipadas.

Direção:

~~~rust
pub enum GeometryEffect {
    Offset(OffsetParams),
    Corners(CornerParams),
    LiveBoolean(LiveBooleanParams),
}
~~~

Não usar:

~~~text
effect_name + HashMap<String, Value>
~~~

para built-ins.

Isso preserva validação, migration e discoverability.

## Gaussian Blur params

Gaussian Blur persiste `sigma_x` e `sigma_y` como parâmetros canônicos.

A UI futura pode apresentar radius, mas a conversão radius↔sigma é camada de apresentação.

~~~rust
pub struct BlurParams {
    pub sigma_x: f64,
    pub sigma_y: f64,
}
~~~

Sigma precisa ser finito e >= 0.

Sigma zero é passthrough válido.

## Shadow params

Direção:

~~~rust
pub struct ShadowParams {
    pub offset: Vec2,
    pub sigma: Vec2,
    pub color: ColorSource,
    pub spread: f64,
}
~~~

`spread` altera a coverage antes do blur, não “aumenta sigma”.

A ordem semântica é fixa e documentada no compositor.

## Adjustment params

Adjustments built-in também são tipos próprios:

~~~text
LevelsParams
HslParams
CurvesParams
BrightnessContrastParams
VibranceParams
PosterizeParams
InvertParams
GrayscaleParams
~~~

Nem todos precisam ser implementados simultaneamente, mas cada um entra com schema próprio quando implementado.

Não criar um Adjustment “universal” com mapa de parâmetros.

## Styles e overrides

Um objeto pode usar:

~~~text
Local Appearance
Linked Style
Linked Style + typed overrides
~~~

A representação precisa distinguir esses estados explicitamente.

Direção:

~~~rust
pub enum AppearanceSource {
    Local(Appearance),
    Style(StyleId),
    StyleWithOverrides {
        style: StyleId,
        overrides: AppearanceOverrides,
    },
}
~~~

Overrides são tipados por identidade de AppearanceItem/Effect quando possível.

Se o style remove um item referenciado por override, a migration/update policy precisa resolver isso deterministicamente; override dangling não é mantido silenciosamente.

## Appearance equality

Não usar igualdade de floats para inferir “mesmo estilo” em runtime.

Identidade de Style é `StyleId`.

Dois Appearances locais visualmente idênticos continuam valores independentes até um Command explícito criar/vincular Style compartilhado.

## Invariantes adicionais

11. Pattern persiste source/transform/repeat semantics; tiles rasterizados são cache.
12. Variable width usa posição normalizada por arc length.
13. Marker placement é derivado da tangente/width e não muta Path.
14. Dash normalization é determinística e validada.
15. Blur persiste sigma, não radius de UI.
16. Effects/Adjustments built-in usam params tipados/versionáveis.
17. Linked Style e Local Appearance são estados distintos.
18. Igualdade visual não substitui identidade de Style.
