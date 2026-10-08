# Adjustments

Adjustments são operações não destrutivas de cor/pixel avaliadas no Render/Effects pipeline.

O Core persiste parâmetros tipados. O Render aplica a matemática. O Color Management Engine fornece conversões de espaço.

## Regra central

> Adjustment nunca altera a source autoral até um Command explícito de Bake/Rasterize.

~~~text
painted/evaluated surface
↓
Adjustment(params)
↓
derived surface
~~~

## AdjustmentSpace

Cada adjustment declara o domínio em que sua matemática é definida.

Direção:

~~~rust
pub enum AdjustmentSpace {
    WorkingEncodedRgb,
    WorkingLinearRgb,
    PerceptualOklab,
}
~~~

A v0.1 não permite que backend escolha espaço implicitamente.

Se um adjustment só possui uma semântica suportada, seu params fixa essa semântica.

## Alpha

Por padrão, adjustments tonais/cromáticos alteram RGB e preservam alpha.

Exceções, como Color-to-Alpha, declaram explicitamente que modificam alpha.

Nunca aplicar HSL/Levels diretamente em RGB premultiplicado. Primeiro recuperar componentes de cor válidos, executar adjustment e voltar ao fluxo premultiplied.

# Invert

Semântica inicial em `WorkingEncodedRgb`:

~~~text
out = 1 - in
~~~

por canal RGB.

Alpha é preservado.

Valores extended-range precisam de policy explícita; para v0.1, Invert trabalha no range normalizado 0..1 após transformação para o domínio encoded do adjustment.

# Grayscale

Default: **relative luminance** em WorkingLinearRgb.

~~~text
RGB linear
↓ luminance weights do working RGB space
Y
↓
R = G = B = Y
~~~

Não usar média simples `(R+G+B)/3`.

Para spaces ICC não descritos por uma matriz simples, o Color Engine fornece conversão para o working linear RGB usado pelo compositor.

Alpha é preservado.

# Posterize

`levels >= 2`.

Em WorkingEncodedRgb:

~~~text
out =
round(in × (levels - 1))
/
(levels - 1)
~~~

por canal.

Isso gera exatamente `levels` valores possíveis em 0..1.

Não quantizar alpha.

# Brightness / Contrast

A v0.1 usa params normalizados:

~~~rust
pub struct BrightnessContrastParams {
    pub brightness: f32, // -1..1
    pub contrast: f32,   // -1..1
}
~~~

Domain: WorkingEncodedRgb.

### Brightness

~~~text
b = in + brightness
~~~

### Contrast

Usar ganho exponencial em torno do midpoint 0.5:

~~~text
gain = 2 ^ contrast
out = (b - 0.5) × gain + 0.5
~~~

Isso fornece:

~~~text
contrast = 0  → gain 1
contrast = 1  → gain 2
contrast = -1 → gain 0.5
~~~

Clamping para 0..1 ocorre ao final desse adjustment porque sua semântica v0.1 é SDR encoded normalizada.

A fórmula é versionada. Alterá-la depois exige migration/semantic version do effect para não mudar aparência de documentos antigos.

# Levels

Direção:

~~~rust
pub struct LevelsChannel {
    pub input_black: f32,
    pub input_white: f32,
    pub gamma: f32,
    pub output_black: f32,
    pub output_white: f32,
}
~~~

Invariantes:

~~~text
0 <= input_black < input_white <= 1
gamma > 0
0 <= output_black <= output_white <= 1
~~~

Para channel value `x`:

~~~text
n =
clamp(
  (x - input_black)
  /
  (input_white - input_black),
  0, 1
)

g = n ^ (1 / gamma)

out =
output_black
+
g × (output_white - output_black)
~~~

Pode existir master channel + channels individuais.

A ordem definida é:

~~~text
master transform
↓
per-channel transform
~~~

ou outra única ordem explicitamente versionada; a v0.1 usa master primeiro.

# HSL

Semântica inicial em WorkingEncodedRgb.

Pipeline:

~~~text
RGB
↓ deterministic RGB↔HSL conversion
HSL
↓ adjust
RGB
~~~

Params:

~~~rust
pub struct HslParams {
    pub hue_shift: f32,       // turns or normalized signed range
    pub saturation: f32,      // -1..1
    pub lightness: f32,       // -1..1
}
~~~

Hue wraps circularmente.

Saturation e Lightness permanecem em range válido após adjustment.

A conversão RGB↔HSL é implementação compartilhada/testada; não duplicar uma versão em CPU e outra em futuro shader.

# Vibrance

Vibrance aumenta saturação de forma adaptativa: cores já muito saturadas recebem menos aumento.

A v0.1 **não** implementa heurística de proteção de skin tone; isso adicionaria comportamento subjetivo difícil de prever.

Usar HSL saturation `S` e amount `a` em -1..1:

Para `a >= 0`:

~~~text
S' = S + a × (1 - S)
~~~

Para `a < 0`:

~~~text
S' = S × (1 + a)
~~~

Assim cores pouco saturadas recebem aumento maior em termos absolutos quando `a > 0`.

Hue e lightness permanecem.

Essa fórmula é contract/versioned.

# Color to Alpha

Color-to-Alpha remove uma cor de referência preservando, tanto quanto possível, a aparência quando o resultado é recomposto sobre essa mesma cor.

A matemática opera em WorkingLinearRgb.

Source straight color: `C`.

Reference color: `B`.

Para cada canal `i`:

~~~text
se C_i > B_i:
  a_i = (C_i - B_i) / (1 - B_i)

se C_i < B_i:
  a_i = (B_i - C_i) / B_i

se C_i == B_i:
  a_i = 0
~~~

Com guards para divisores zero.

Escolher:

~~~text
a = max(a_r, a_g, a_b)
~~~

Esse é o menor alpha que permite representar C como composição de um foreground válido sobre B.

Se `a = 0`:

~~~text
output alpha = 0
output color = canonical zero
~~~

Caso contrário, recuperar foreground:

~~~text
F_i =
(C_i - (1 - a) × B_i)
/
a
~~~

e clamp apenas resíduos numéricos pequenos para o range esperado.

Se source já possui alpha `αs`:

~~~text
output alpha = αs × a
~~~

A conversão precisa ser validada com teste:

~~~text
ColorToAlpha(C, B)
↓ composite over B
≈ original C
~~~

dentro da tolerância numérica.

## Remove White

Remove White é simplesmente:

~~~text
ColorToAlpha(reference = white)
~~~

Não possui algoritmo separado.

Isso garante que dois features equivalentes não divirjam.

# Curves

Curves persiste control points, não LUT.

Direção:

~~~rust
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}
~~~

Points:

- finitos;
- x em 0..1;
- ordenados por x;
- endpoints/policy definidos;
- duplicate x rejeitado ou resolvido deterministicamente.

A v0.1 usa interpolação cúbica monotônica quando o user curve for monotônico, evitando overshoot que criaria inversões involuntárias.

Quando os control points explicitamente não forem monotônicos, o mode precisa permitir isso conscientemente; não “corrigir” a curva silenciosamente.

A LUT derivada é cache.

# Opacity / Color to Alpha ordering

Adjustment order é a ordem da Effect Stack.

~~~text
Levels → HSL
≠
HSL → Levels
~~~

Reorder é alteração autoral.

Color-to-Alpha colocado antes/depois de outros adjustments também produz resultados diferentes e segue exatamente a stack.

# Remove color / threshold utilities

Ferramentas futuras que selecionam por cor não devem reutilizar Color-to-Alpha como se fosse selection algorithm.

Color-to-Alpha modifica coverage; color selection produz Session selection mask.

São semânticas diferentes.

# Quality

Adjustments básicos não possuem versão “preview matematicamente diferente”.

Preview pode usar tiles/resolução reduzida em efeitos espaciais, mas Levels/HSL/Invert/Posterize usam a mesma fórmula em todas as qualities.

# Cache

Adjustment tile key:

~~~text
input evaluation key
+ AdjustmentId/EffectId
+ params
+ AdjustmentSpace/color context
+ tile
~~~

LUT/cache interno faz parte do derived result.

# Tests

Cada adjustment possui:

- valores conhecidos;
- endpoints;
- alpha 0/1;
- neutral params;
- roundtrip/invariant quando aplicável;
- NaN/Inf rejection;
- CPU reference golden scene.

Neutral params precisam ser identidade:

~~~text
Brightness 0 / Contrast 0
HSL 0
Vibrance 0
Levels defaults
→ input unchanged
~~~

# Invariantes

1. Adjustment é não destrutivo até Bake/Rasterize.
2. Espaço matemático de cada adjustment é explícito.
3. RGB adjustments comuns preservam alpha.
4. Nenhum adjustment cromático opera diretamente em valores premultiplied.
5. Invert/Posterize/Brightness-Contrast/HSL usam contract versionado.
6. Grayscale default usa luminância linear, não média RGB.
7. Levels usa input range + gamma + output range validados.
8. Vibrance é adaptive saturation sem skin heuristic na v0.1.
9. Remove White reutiliza Color-to-Alpha.
10. Color-to-Alpha possui teste de recomposição sobre a cor removida.
11. Curves persiste control points; LUT é cache.
12. Reorder de adjustments é semântica autoral.
