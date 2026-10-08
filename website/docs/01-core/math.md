# math.rs

`math.rs` contém apenas **vocabulário matemático estável do domínio**. Algoritmos geométricos pesados — boolean, intersections, simplify, curve fitting, offset e Shape Builder — pertencem ao Engine.

> Core define tipos e invariantes matemáticos persistentes. Engine executa algoritmos sobre esses tipos.

## Escalar padrão

Geometria canônica usa `f64`.

Isso preserva precisão em documentos com muitas transformações, operações booleanas, interseções e medidas físicas. Render/GPU pode converter para `f32` em uma fronteira controlada depois da transformação para espaço de view.

~~~text
Document / Engine
      f64
       ↓
View transform
       ↓
Render / GPU
      f32
~~~

A conversão para menor precisão é derivada e nunca reescreve a geometria autoral.

## Tipos mínimos

Direção de modelo:

~~~rust
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

pub struct Size2 {
    pub width: f64,
    pub height: f64,
}

pub struct Rect {
    pub min: Point,
    pub max: Point,
}

pub struct Insets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub struct Angle(pub f64);
~~~

Esses tipos entram incrementalmente conforme substituem o modelo atual.

## Point, Vec2 e Size2

`Point` representa posição. `Vec2` representa deslocamento/direção. `Size2` representa extensão.

Operações válidas:

~~~text
Point + Vec2 → Point
Point - Vec2 → Point
Point - Point → Vec2

Vec2 + Vec2 → Vec2
Vec2 - Vec2 → Vec2
Vec2 × scalar → Vec2
~~~

Evitar disponibilizar `Point + Point` apenas porque os dois possuem `x` e `y`.

Para `Size2`, a invariante recomendada é:

~~~text
width  >= 0
height >= 0
~~~

Drag invertido é normalizado antes de produzir Size2; width/height negativos não viram uma segunda representação implícita.

## Rect

`Rect` usa `min` e `max` canonicalizados:

~~~text
min.x <= max.x
min.y <= max.y
~~~

API mínima:

~~~text
from_points
from_origin_size
width
height
center
union
intersection
expand
contains_point
contains_rect
intersects
is_empty
~~~

Um rect com `min == max` pode ser vazio e válido. Valores não finitos ou uma representação não canonicalizada são inválidos na fronteira do domínio.

## Bounds geométrico e visual

**Geometric bounds** contém a geometria base.

**Visual bounds** inclui aparência que pode expandir a região:

~~~text
Path
 ↓
Stroke
 ↓
Shadow
 ↓
Blur
~~~

Portanto:

~~~text
geometric bounds
≠
visual bounds
~~~

`Rect` não conhece stroke, shadow ou blur. Engine/Render calculam visual bounds.

## Insets

`Insets` representa expansão ou margem por lado.

~~~text
Rect
+
Insets
↓
Expanded Rect
~~~

É útil para padding, expansão de ROI e bounds derivados. A unidade vem do contexto; não misturar pixel de tela e document unit implicitamente.

## Angle

Internamente, `Angle` usa radianos.

~~~text
2π radians = 360°
~~~

Conversão:

~~~text
radians = degrees × π / 180
~~~

A UI pode mostrar graus, mas não manter um segundo valor autoritativo para o mesmo ângulo.

## Transform2D

Uma **transformação afim** representa translation, rotation, scale e skew sem alterar os pontos originais.

~~~text
| a c tx |
| b d ty |
| 0 0  1 |
~~~

Aplicada a um ponto:

~~~text
x' = a·x + c·y + tx
y' = b·x + d·y + ty
~~~

### Ordem de multiplicação

Decisão:

> `A * B * p` significa aplicar `B` primeiro e depois `A`.

Essa convenção precisa ser idêntica em Core, Engine, Render, import/export e testes.

Não herdar silenciosamente a convenção de uma biblioteca externa.

### API mínima

~~~text
identity
translation
scale
rotation
skew
multiply
determinant
inverse
transform_point
transform_vector
transform_rect
~~~

`transform_rect` produz um bounds axis-aligned conservador; não significa que o retângulo rotacionado continue geometricamente um Rect.

## Determinante e singularidade

Para:

~~~text
| a c |
| b d |
~~~

o determinante é:

~~~text
det = a·d - b·c
~~~

Se `det = 0`, a transformação é **singular**: colapsou ao menos uma dimensão e não possui inversa.

Em floating point, testar apenas `det == 0.0` não basta. Uma matriz pode ser matematicamente invertível e numericamente instável quando o determinante é muito pequeno.

Isso é uma transformação **near-singular**.

Direção de erro:

~~~rust
pub enum TransformError {
    NonFinite,
    Singular,
    NearSingular,
}
~~~

A tolerância de inversão é específica dessa operação. Nunca retornar NaN silenciosamente.

## Decompose e recompose

**Decompose** tenta separar uma matriz em translation, rotation, scale e skew. **Recompose** reconstrói uma matriz a partir desses componentes.

Não existe decomposição intuitiva única em todos os casos, especialmente com:

- reflexão;
- escala negativa;
- skew;
- transforms near-singular.

A representação autoral final de transform deve considerar essas ambiguidades quando chegarmos ao modelo de objetos. Não assumir que decomposição é trivial.

## Sistemas de coordenadas

~~~text
Object Local
   ↓ local transform
Parent / Scene
   ↓ hierarchy transforms
Document
   ↓ viewport transform
View
   ↓ device scale
Device Pixels
~~~

**Object Local** é o espaço natural do objeto.

**Document** é o espaço autoral compartilhado pelo documento/página conforme o modelo final.

**View** aplica pan, zoom e canvas rotation.

**Device Pixels** é o espaço do backend de saída.

Conversões devem declarar origem e destino. Preferir nomes como:

~~~rust
document_to_view(...)
view_to_document(...)
~~~

a funções ambíguas como `convert_point(...)`.

Se bugs de mistura de espaço surgirem, newtypes como `DocumentPoint` e `ViewPoint` são candidatos, mas não criá-los preventivamente sem benefício concreto.

## Unidades

Geometria canônica usa **document units**. Conversões de unidade ficam em `units.rs`.

~~~rust
pub enum Unit {
    Px,
    Pt,
    Mm,
    Cm,
    Inch,
    Pica,
}

pub struct UnitValue {
    pub value: f64,
    pub unit: Unit,
}
~~~

### Pixel não é uma única coisa

“Pixel” pode significar:

- unidade documental;
- pixel lógico da UI;
- pixel físico da tela;
- texel de textura;
- sample de raster exportado.

Usar o termo com contexto quando houver risco de ambiguidade.

### DPI

**DPI — Dots Per Inch** representa densidade de saída/conversão física.

~~~text
1 inch × 300 DPI = 300 output pixels
~~~

DPI de documento/export não é Device Pixel Ratio da tela.

## Formatação não é mutação

A UI pode mostrar:

~~~text
12.345678901 mm
→ 12.35 mm
~~~

O valor autoral continua `12.345678901` até o usuário efetivamente editar o campo.

Trocar unidade ou abrir um painel nunca quantiza a geometria.

## Floating point

`f64` usa ponto flutuante IEEE-754. Nem todo decimal possui representação binária exata.

Por isso igualdade numérica precisa de contexto.

### Exact equality

Serve para identidades e valores discretos:

- IDs;
- enums;
- inteiros;
- revision IDs.

### Semantic equality

Dois valores geométricos podem ser equivalentes dentro da tolerância de uma operação:

~~~text
|a - b| <= tolerance
~~~

Isso é uma decisão algorítmica, não uma mutação.

### Visual equality

Dois resultados podem ser indistinguíveis na saída atual apesar de numericamente diferentes.

Pode depender de zoom, device scale, antialiasing e render scale. É principalmente conceito de Render/testes.

## Tolerâncias

Não existe `EPSILON` universal.

Contextos diferentes exigem tolerâncias diferentes:

~~~text
CoincidenceTolerance
FlattenTolerance
BooleanTolerance
HitTestTolerancePx
SnapTolerancePx
InverseTransformTolerance
~~~

Esses nomes podem virar newtypes quando melhorarem segurança e legibilidade.

### Screen-space

Hit-test e snapping usam normalmente tolerância visual:

~~~text
document_tolerance =
screen_tolerance_px / view_scale
~~~

Assim um handle continua clicável em diferentes níveis de zoom.

### Tolerância não é edição

Se:

~~~text
A = 10.000000
B = 10.000005
~~~

um algoritmo pode tratá-los como coincidentes sem executar:

~~~text
B := A
~~~

Fundir ou mover pontos é operação autoral explícita, como Merge by Distance.

## Valores não finitos

**NaN — Not a Number**, `+Inf` e `-Inf` não entram no Document.

Commands, import, plugins e outras fronteiras validam `is_finite()` antes do commit.

Esses valores quebram premissas de ordering, bounds, spatial indexes, hashing, serialização e render.

## Negative zero e canonicalização

IEEE-754 possui `0.0` e `-0.0`.

Para a semântica geométrica comum eles são equivalentes, mas podem gerar texto, hashes ou cache keys diferentes.

Em fronteiras canônicas:

~~~text
-0.0 → 0.0
~~~

**Canonicalizar** significa escolher uma representação estável para valores semanticamente equivalentes.

Isso não autoriza aproximar coordenadas por tolerância.

## Round-trip de float

Persistência deve preservar precisão suficiente para:

~~~text
f64 original
↓ serialize
decimal
↓ deserialize
mesmo f64
~~~

Não serializar com a quantidade de casas usada pela UI.

## Determinismo

**Determinismo semântico** significa:

> mesma entrada + mesmos parâmetros → mesmo resultado autoral relevante.

Não permitir que ordem acidental de HashMap defina:

- z-order;
- ordem persistida;
- ordem lógica de resultado boolean;
- manifests;
- IDs derivados.

Quando necessário:

~~~text
resultado paralelo/não ordenado
↓
canonicalize
↓
stable sort
↓
commit
~~~

## Floating point e paralelismo

Ponto flutuante não é perfeitamente associativo:

~~~text
(a + b) + c
pode diferir levemente de
a + (b + c)
~~~

Paralelismo pode mudar a ordem de combinação.

A regra é:

> ordem interna de execução não pode vazar como semântica autoral acidental.

Determinismo padrão é semântico, não promessa bit-a-bit em qualquer CPU, GPU ou driver.

## Random seed

Operações autorais aleatórias usam seed persistente.

~~~rust
pub struct RandomSeed(pub u64);
~~~

Exemplos futuros:

- brush jitter;
- scatter;
- procedural noise;
- procedural patterns.

~~~text
input + parameters + seed
→ resultado reproduzível
~~~

`Randomize` gera nova seed explicitamente. Relógio do sistema não define silenciosamente um resultado autoral estático.

## Predicates geométricos

Um **predicate geométrico** responde uma pergunta discreta que pode controlar topologia.

~~~text
orientation(A, B, C)
→ clockwise
→ counter-clockwise
→ collinear
~~~

Quando o resultado numérico está muito próximo de zero, erro de floating point pode inverter a decisão.

Em topologia isso pode produzir contorno invertido, self-intersection artificial, região perdida ou boolean inconsistente.

Operações críticas devem usar predicates robustos e bibliotecas especializadas quando apropriado.

## Cache keys

Não inferir “mudou?” comparando coleções grandes de floats quando uma revision já representa mudança autoral.

Preferir:

~~~text
revision
+
operation parameters
→ cache key
~~~

Revisions locais mais granulares, como GeometryRevision, podem ser introduzidas quando cache fino exigir. Não criar antecipadamente.

## Testes numéricos

| Teste | Estratégia |
|---|---|
| IDs/enums | igualdade exata |
| ordem autoral | igualdade exata |
| Point/Rect calculado | tolerância específica |
| topologia | estrutura esperada |
| serialization | round-trip/canonical form |
| CPU renderer | reference/golden contract |
| GPU vs CPU reference | tolerância visual documentada |

Evitar `assert_eq!` indiscriminado em floats e também tolerâncias arbitrariamente grandes que escondam regressões.

## Kurbo

O workspace já depende de `kurbo`.

Decisão:

> tipos persistentes e públicos do Core permanecem tipos Petunia; conversões para Kurbo ficam no Engine/adapter.

~~~text
Petunia math/path
      ↓
adapter
      ↓
kurbo
      ↓
algorithm
      ↓
adapter
      ↓
Petunia result
~~~

Isso evita acoplar PTND e invariantes públicas ao versionamento de uma dependência externa.

## Invariantes de math.rs

1. Geometria canônica usa `f64`.
2. Point, Vec2 e Size2 possuem semânticas diferentes.
3. Rect usa representação canonicalizada por min/max.
4. A ordem de multiplicação de Transform2D é única e documentada.
5. Inversão rejeita transform singular ou numericamente inseguro.
6. Espaços de coordenadas não são misturados implicitamente.
7. UI nunca quantiza valores autorais.
8. Não existe tolerância global universal.
9. NaN e infinito não entram no domínio persistente.
10. Persistência canonicaliza representações irrelevantes sem alterar geometria.
11. Operações autorais aleatórias possuem seed explícita.
12. Tipos Kurbo não definem o modelo persistente Petunia.
