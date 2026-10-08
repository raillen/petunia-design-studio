# path.rs

Path é a unidade vetorial editável fundamental do Petunia.

A representação precisa equilibrar quatro objetivos: precisão, edição previsível, IDs estáveis e kernel geométrico simples.

## Decisão canônica

O path nativo usa **Line + Cubic Bézier** como segmentos canônicos.

Quadratic Bézier importada é convertida **exatamente** para cubic. SVG elliptical arc e outras primitivas especiais são avaliadas por import/generators e convertidas para a representação Petunia quando materializadas.

Essa decisão reduz o número de casos que boolean, offset, hit-test, simplify e Node Tool precisam tratar.

## Modelo

~~~rust
pub struct VectorPath {
    pub contours: Vec<Contour>,
    pub fill_rule: FillRule,
}

pub struct Contour {
    pub id: ContourId,
    pub nodes: Vec<PathNode>,
    pub closed: bool,
}

pub struct PathNode {
    pub id: NodeId,
    pub position: Point,
    pub handle_in: Option<Point>,
    pub handle_out: Option<Point>,
    pub kind: NodeKind,
    pub outgoing: SegmentKind,
}

pub enum SegmentKind {
    Line,
    Cubic,
}

pub enum NodeKind {
    Cusp,
    Smooth,
    Symmetric,
}
~~~

`outgoing` descreve o segmento entre este node e o próximo.

Em contour fechado, o último node conecta implicitamente ao primeiro. Não duplicar o primeiro node no final apenas para indicar fechamento.

## Por que Line + Cubic

Uma cubic Bézier possui:

~~~text
P0 = anchor inicial
P1 = control 1
P2 = control 2
P3 = anchor final
~~~

Uma quadratic `Q0,Q1,Q2` vira cubic exatamente:

~~~text
P0 = Q0
P1 = Q0 + 2/3 × (Q1 - Q0)
P2 = Q2 + 2/3 × (Q1 - Q2)
P3 = Q2
~~~

Portanto manter `Quad` no formato nativo aumentaria branches sem ampliar capacidade geométrica.

## Handles

Para um segmento cubic entre node A e B:

~~~text
P0 = A.position
P1 = A.handle_out ou A.position
P2 = B.handle_in ou B.position
P3 = B.position
~~~

Handle ausente equivale ao anchor naquele lado.

Para `SegmentKind::Line`, handles não alteram o segmento.

## NodeKind

`NodeKind` é uma **restrição de edição**. Ele não reescreve automaticamente uma curva já armazenada.

### Cusp

Handles independentes em direção e comprimento.

### Smooth

Os dois handles permanecem colineares através do anchor, mas podem ter comprimentos diferentes.

### Symmetric

Handles permanecem colineares e com mesmo comprimento.

Ao arrastar um handle, Tool/Engine calcula o outro conforme a constraint e gera a mutação final.

## Continuidade

Duas cubics podem ter níveis de continuidade.

**C0**: as curvas apenas se encontram no mesmo anchor.

**G1**: tangentes são colineares e a direção visual é contínua.

**C1**: além da direção, a derivada parametrizada é compatível.

Para edição visual, `Smooth` representa intenção de continuidade tangencial. C2 não é requisito da v0.1.

## Open e closed contours

`closed = false`:

~~~text
N1 → N2 → N3
~~~

Não existe segmento N3 → N1.

`closed = true`:

~~~text
N1 → N2 → N3
↑         ↓
└─────────┘
~~~

O fechamento é propriedade do contour, não node duplicado.

## Degenerados

Core tolera alguns estados geométricos degenerados porque podem surgir durante edição/import.

Exemplos:

- contour com um node;
- segmento zero-length;
- handles coincidentes;
- dois nodes na mesma posição;
- contour fechado de área zero;
- self-intersection.

**Degenerado** significa estruturalmente representável, mas especial ou sem área/length útil para alguma operação.

Core valida finitude e estrutura; Engine decide se um algoritmo específico aceita aquele caso.

Não fazer cleanup silencioso no Core.

## Self-intersection

Self-intersection é permitida em VectorPath.

~~~text
\ /
 X
/ \
~~~

FillRule define interior. Boolean, offset e outros algoritmos documentam como tratam esse caso.

## FillRule

### EvenOdd

Um ponto está dentro se um raio cruza o boundary um número ímpar de vezes.

~~~text
1 → dentro
2 → fora
3 → dentro
~~~

Orientação do contour não altera EvenOdd.

### NonZero

Cada crossing contribui com sinal conforme a direção do contour. A soma é o **winding number**.

~~~text
winding != 0 → dentro
winding == 0 → fora
~~~

Para NonZero, orientação dos contours **faz parte da semântica**.

Portanto o Core preserva orientação; algoritmos não podem revertê-la silenciosamente se isso alterar fill.

## Orientação

Engine pode normalizar orientação internamente para um algoritmo, mas essa normalização é derivada.

`Reverse Contour` é Command autoral explícito.

## IDs

`ContourId` e `NodeId` sobrevivem a edições normais.

~~~text
move node → mantém NodeId
move handle → mantém NodeId
split segment → nodes existentes mantêm IDs; novo node recebe novo ID
delete + undo → ID original volta
reverse contour → mesmos NodeIds, nova ordem
~~~

Operações que geram nova geometria sem correspondência inequívoca podem criar IDs novos.

## Insert e split segment

Inserir node em cubic precisa preservar a curva.

Usar **De Casteljau** no parâmetro `t`.

~~~text
cubic original
      ↓ split(t)
left cubic + new node + right cubic
~~~

A geometria resultante é a mesma curva, apenas dividida em segmentos.

## Delete node

Remover node é edição geométrica, não simples remoção de vetor.

Ferramentas podem oferecer:

- delete direto;
- delete + curve refit;
- dissolve node.

São Commands/algoritmos diferentes. Core oferece mutações estruturais validadas.

## Reverse contour

Reverter contour exige:

- inverter ordem dos nodes;
- trocar `handle_in ↔ handle_out`;
- reassociar `SegmentKind` ao novo outgoing;
- preservar NodeIds;
- manter closed/open.

A operação precisa de testes específicos para NonZero.

## Bounds e length

Bounds exato de cubic considera extrema internas.

Arc length é derivado e calculado no Geometry Engine por aproximação adaptativa.

Não persistir bounds ou length dentro de VectorPath como fonte da verdade.

## Arc-length parameterization

O parâmetro Bézier `t` não representa distância uniforme.

~~~text
t = 0.5
≠ necessariamente metade do comprimento
~~~

Dash, text-on-path e variable width usam lookup/solver de arc length derivado.

## Shapes paramétricas

Shapes paramétricas preservam intenção geométrica de alto nível enquanto continuarem editáveis como shapes.

> Rectangle continua Rectangle; Ellipse continua Ellipse; Star continua Star. Converter para curves é uma ação explícita.

### Modelo

```rust
pub enum ParametricShape {
    Rectangle(RectangleSpec),
    Ellipse(EllipseSpec),
    Polygon(PolygonSpec),
    Star(StarSpec),
}
```

O SceneNode guarda transform separadamente. Shape specs trabalham em espaço local.

```text
ShapeSpec
   ↓ evaluate
VectorPath derivado
   ↓ local transform
Document geometry
```

### Rectangle

```rust
pub struct RectangleSpec {
    pub size: Size2,
    pub corners: CornerRadii,
}
```

Corner radii podem ser independentes por canto. Cada raio possui componentes X/Y não negativos e finitos.

Se a soma de raios adjacentes ultrapassa width/height, o evaluator reduz os raios proporcionalmente até caber. Essa normalização é derivada e determinística.

Rounded Rectangle não precisa ser tipo separado: é Rectangle com CornerRadii.

### Ellipse

```rust
pub struct EllipseSpec {
    pub radii: Vec2,
    pub arc: EllipseArc,
}

pub enum EllipseArc {
    Full,
    Open { start: Angle, sweep: Angle },
    Chord { start: Angle, sweep: Angle },
    Pie { start: Angle, sweep: Angle },
}
```

Full cobre ellipse/circle normal. Open produz arco aberto; Chord fecha entre endpoints; Pie fecha endpoints ao centro.

Circle é Ellipse com raios iguais, não um tipo persistente separado.

### Polygon

```rust
pub struct PolygonSpec {
    pub sides: u32,
    pub radius: f64,
    pub rotation: Angle,
}
```

Invariantes:

```text
sides >= 3
radius >= 0
finite values
```

Vertices são distribuídos uniformemente no círculo circunscrito:

```text
angle_i = rotation + i × 2π / sides
```

### Star

```rust
pub struct StarSpec {
    pub points: u32,
    pub outer_radius: f64,
    pub inner_ratio: f64,
    pub rotation: Angle,
}
```

Invariantes:

```text
points >= 2
outer_radius >= 0
inner_ratio >= 0
finite values
```

Vertices alternam outer/inner. Não impor clamp arbitrário de `inner_ratio <= 1`; valores maiores continuam matematicamente válidos.

### Corners compartilhados

Polygon e Star não ganham algoritmos próprios de rounded corners na v0.1.

```text
Polygon/Star
↓ evaluated path
↓ Live Corners
↓ result
```

Live Corners é Geometry Effect compartilhado.

### Outros generators

Gear, Spiral, Wave e similares só entram no formato persistente quando possuírem intenção autoral própria, parâmetros estáveis, avaliação determinística e valor suficiente para justificar schema.

Não transformar `ParametricShape` em catálogo infinito.

### Avaliação

Shapes são avaliados em VectorPath derivado.

Ellipse/arcs usam Cubic Bézier. Como ellipse não possui representação exata com número finito de cubics, a aproximação é determinística e respeita tolerância documentada.

A fonte paramétrica permanece exata; a aproximação não substitui os parâmetros.

### Geometry effects

Effects não forçam conversão definitiva.

```text
RectangleSpec
↓ evaluate
VectorPath
↓ Live Offset
↓ Live Corners
↓ Appearance
```

Editar o Rectangle reavalia a cadeia.

### Convert to Curves

`Convert to Curves`:

1. avalia a shape;
2. materializa VectorPath;
3. cria ContourId/NodeId;
4. substitui `SceneItem::Shape` por `SceneItem::Path` em uma Transaction;
5. preserva ObjectId do SceneNode;
6. preserva Appearance/effects compatíveis;
7. registra inverse data para Undo restaurar a shape paramétrica.

`Convert to Curves` materializa somente geometria base. `Expand Appearance` é outro Command.

### Hit-test, snapping e bounds

Engine pode usar fórmulas analíticas quando forem mais baratas, desde que sejam semanticamente equivalentes ao path avaliado.

Bounds, evaluated path e tessellation são Derived State.

### Serialização

Persistir apenas parâmetros autorais finitos. Não persistir VectorPath derivado, tessellation, bounds, handles gerados ou caches.

### Invariantes de shapes

1. Shapes permanecem paramétricas até Command explícito.
2. Transform do objeto é separado do ShapeSpec.
3. Rounded Rectangle é Rectangle com corner radii.
4. Circle é especialização de Ellipse.
5. Polygon e Star usam parâmetros determinísticos mínimos.
6. Corner rounding compartilhado pertence ao Geometry Engine.
7. Path derivado nunca substitui Source automaticamente.
8. Convert to Curves preserva ObjectId e cria IDs internos novos.
9. Bounds/path/tessellation derivados não são persistidos.
10. Novos generators exigem spec própria e justificativa.

## O que não pertence a path.rs

Não colocar no Core:

- boolean;
- intersections;
- offset;
- simplify;
- flattening;
- curve fitting;
- nearest point;
- stroke expansion;
- hit-test;
- Shape Builder.

Esses algoritmos pertencem ao Geometry Engine.

## Adapters

Tipos persistentes são Petunia.

~~~text
VectorPath
↓ adapter
kurbo / i_overlay representation
↓ algorithm
Petunia result
~~~

Não serializar `kurbo::BezPath`.

## Validação

VectorPath válido garante:

- IDs internos únicos;
- coordinates/handles finitos;
- contours structurally consistent;
- SegmentKind conhecido;
- FillRule conhecido.

Não exige que todo contour possua área, seja simples ou seja renderizável.

## Invariantes

1. Line + Cubic são segmentos canônicos nativos.
2. Quadratic importada converte exatamente para Cubic.
3. Closed contour não duplica o primeiro node.
4. NodeKind é constraint de edição, não transformação automática.
5. Self-intersection é permitida.
6. FillRule é persistente.
7. Orientação é semanticamente relevante para NonZero.
8. Core não executa cleanup geométrico silencioso.
9. NodeId/ContourId permanecem estáveis quando a mesma entidade continua existindo.
10. Split de cubic usa De Casteljau para preservar geometria.
11. Bounds/length são derivados.
12. Algoritmos pesados ficam no Geometry Engine.
