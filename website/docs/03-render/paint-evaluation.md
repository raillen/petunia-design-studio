# Paint Evaluation

Paint Evaluation transforma `Paint` autoral em cor/coverage amostrada para Render.

Pertence à fronteira Engine/Render: Core guarda parâmetros; Render avalia no output.

## Pipeline

~~~text
Paint
+ geometry/paint space
+ ColorContext
↓
Paint Evaluator
↓
sample color/coverage
↓
compositor
~~~

Paint não modifica a geometria.

# Solid

`Solid(ColorSource)` resolve:

~~~text
ColorSource
↓ linked Swatch ou local ColorValue
↓ Color Management
compositing color
~~~

Swatch resolution failure produz diagnóstico; não substitui silenciosamente por preto.

# Gradient common model

Todo gradient possui:

- stops ordenados;
- midpoint por intervalo;
- interpolation mode;
- spread mode;
- paint space;
- transform/geometria própria.

Stops com mesmo offset são permitidos para transição dura, mas sua ordem precisa ser estável.

## Stop lookup

Para sample parameter `t`:

1. aplicar spread;
2. localizar stops adjacentes;
3. normalizar t no intervalo;
4. aplicar midpoint mapping;
5. converter cores para interpolation space;
6. interpolar;
7. converter resultado para compositing space.

Busca pode usar binary search.

# Spread

## Pad

~~~text
t < 0 → first stop
t > 1 → last stop
~~~

## Repeat

~~~text
t' = t - floor(t)
~~~

Com tratamento canônico para inteiros negativos/positivos.

## Reflect

Repete alternando direção:

~~~text
period = floor(t)
u = t - floor(t)

period par   → u
period ímpar → 1-u
~~~

Implementação precisa tratar t negativo deterministicamente.

# Midpoint

Midpoint `m` em cada intervalo declara onde a mistura chega a 50%.

`m = 0.5` produz interpolação linear normal.

A v0.1 usa remapeamento por expoente:

~~~text
gamma = ln(0.5) / ln(m)

u' = u ^ gamma
~~~

para `0 < m < 1`.

Assim:

~~~text
u = m
→ u' = 0.5
~~~

O contrato persistente é exatamente:

~~~text
midpoint finito
0 < midpoint < 1
~~~

`0` e `1` são inválidos porque tornam a parametrização singular. Não inventar clamp silencioso como `0.01…0.99` no Render.

Para valores internos muito próximos dos extremos, a implementação usa cálculo numericamente estável em log/pow e retorna erro/degradação apenas se o valor deixar de ser finito. A UI futura pode escolher um range ergonômico menor para sliders sem reduzir o domínio autoral permitido.

# Interpolation spaces

## LinearRgb

Converter stops para WorkingLinearRgb e interpolar canais linearly.

É o default técnico da v0.1.

## EncodedRgb

Converter para WorkingEncodedRgb e interpolar os valores codificados.

Produz visual diferente de LinearRgb e por isso é uma opção persistente real.

## Lab

Converter stops para Lab gerenciado pelo Color Engine antes de interpolar.

Não assumir que valores RGB podem ser “tratados como Lab”.

## Oklab

Converter para Oklab derivado e interpolar L/a/b.

Alpha é interpolado separadamente como straight alpha; depois o resultado volta ao compositor premultiplied.

# Linear Gradient

Definido por endpoints `A` e `B` em PaintSpace.

Para point `P`:

~~~text
D = B - A

t =
dot(P - A, D)
/
dot(D, D)
~~~

Se `A == B` dentro da validação geométrica, gradient é degenerado. Core/Command deve evitar criar essa configuração ou Render retorna fallback/error definido; não dividir por zero.

# Radial Gradient

A v0.1 usa radial gradient em espaço local transformável.

Direção canônica:

~~~text
gradient local space
center = (0,0)
unit radius = 1
~~~

A geometria/transform autoral mapeia essa unidade para círculo/ellipse no documento.

Para sample local `P`:

~~~text
t = length(P)
~~~

Depois aplica spread/stops.

Esse modelo representa ellipse através do transform sem duplicar matemática.

Focal radial gradients ficam fora da semântica inicial até terem spec própria.

# Conical Gradient

Conical usa ângulo em torno de center.

Com coordenadas Petunia Y-down e ângulo positivo horário:

~~~text
angle =
atan2(P.y - center.y, P.x - center.x)

t =
normalize_turn(
  (angle - start_angle) / 2π
)
~~~

`normalize_turn` leva o valor para o período requerido antes de aplicar spread.

Conical reutiliza stops/interpolation/midpoint.

Não precisa de novo sistema de cor.

No center exato, direção angular é indefinida. A v0.1 usa deterministicamente `t = 0` para o sample central.

# Pattern

Pattern usa source + inverse transform.

~~~text
document/object point
↓ inverse(pattern transform)
pattern-local point
↓ repeat mapping
source sample
↓ Color Management
compositing color
~~~

Pattern transform precisa ser invertível. Transform singular/near-singular é inválido para PatternPaint.

## Repeat axes

Por eixo:

- Repeat;
- Clamp;
- Mirror.

A mesma matemática de Repeat/Reflect de gradients pode ser reutilizada semanticamente.

## Raster pattern

Sampling policy precisa ser explícita:

~~~text
Nearest
Bilinear
~~~

Bicubic pode entrar depois sob o mesmo contrato.

Source permanece Resource; pattern cache pode gerar tile/color-converted variant.

## Vector pattern

Vector pattern referencia definition/resource vetorial e avalia uma tile lógica reutilizável.

Não duplicar a Scene subtree para cada repetição.

Render pode rasterizar/cachear a tile por scale/color context, mas isso é Derived State.

# Object vs Document space

`PaintSpace::Object` acompanha o objeto.

`PaintSpace::Document` permanece ancorado em document/page coordinates.

Transformar o objeto:

~~~text
Object-space paint → acompanha geometry
Document-space paint → geometry cruza paint fixo
~~~

Essa diferença é autoral e precisa sobreviver save/load.

# Paint transform

Gradient/pattern podem possuir transform adicional.

Composição de transforms segue a convenção canônica de math.rs.

A ordem precisa ser documentada:

~~~text
paint local
↓ paint transform
paint space
↓ object/document transform quando aplicável
document
~~~

Adapters externos convertem convenções, nunca alteram internamente a regra Petunia.

# Extended range

Interpolation interna float pode exceder 0..1 quando stops/working space permitirem HDR.

Spread não é clamp de color channels.

Output pipeline decide gamut/tone mapping.

# Dithering

Dithering não pertence ao Paint autoral.

Quando necessário para output quantizado, ocorre depois da composição/output transform.

# Cache

Gradient LUT é opcional e derivado.

Key:

~~~text
gradient params/stops
+ interpolation mode
+ ColorContext
+ quality
~~~

Pattern cache inclui source content identity + transform scale bucket quando necessário.

Uma LUT nunca substitui stops autorais.

# Tests

Testar:

- t exatamente em stops;
- duplicate stop offsets;
- midpoint 0.5 identity;
- midpoint deslocado;
- Repeat/Reflect com t negativo;
- degenerate linear gradient;
- radial ellipse via transform;
- conical wrap 0↔1;
- center conical;
- Object vs Document space;
- pattern mirror/repeat;
- color interpolation modes.

# Invariantes

1. Paint params são autorais; samples/LUTs são derivados.
2. Gradients compartilham stops/interpolation/spread.
3. Midpoint mapping é determinístico e versionado.
4. Linear gradient usa projection.
5. Radial v0.1 usa unit radial local space + transform.
6. Conical usa angle em coordenadas Y-down e reutiliza stops.
7. Pattern transform precisa ser invertível.
8. Object/Document PaintSpace possuem semânticas diferentes.
9. Vector pattern não duplica source para cada tile.
10. Dithering é output detail, não Paint.

## Verificação de gradientes lineares e radiais (2026-10-10)

Escopo: avaliação de gradientes no compilador do Engine, amostragem no renderer de software e testes golden de pixels. Revisão `bba0837be32ba78bea13b905e867530d3412bfe1` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 394 passed / 0 failed (Render com testes golden de gradientes) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: patterns ainda exigem resolução de assets; mesh gradient segue fora do escopo v0.1.
