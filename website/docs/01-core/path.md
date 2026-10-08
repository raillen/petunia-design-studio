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

Shapes paramétricas não são armazenadas como paths crus enquanto permanecerem paramétricas.

~~~text
ParametricShape
      ↓ Engine evaluate
VectorPath derivado
~~~

`Convert to Curves` materializa VectorPath por Command.

Detalhes: [Shapes paramétricas](#/docs/01-core/parametric-shapes.md).

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
