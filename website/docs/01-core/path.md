# path.rs

O path é a unidade geométrica editável mais importante do editor. A prioridade é preservar editabilidade, IDs estáveis e topologia clara.

## Representação canônica

O modelo atual usa anchors com handles cúbicos. Para um editor profissional, precisamos distinguir **topologia** de **segmento**.

Modelo recomendado:

```rust
pub struct VectorPath {
    pub contours: Vec<Contour>,
    pub fill_rule: FillRule,
}

pub struct Contour {
    pub id: ContourId,
    pub start: NodeId,
    pub segments: Vec<PathSegment>,
    pub closed: bool,
}

pub enum PathSegment {
    Line { to: NodeId },
    Quad { ctrl: Point, to: NodeId },
    Cubic { ctrl1: Point, ctrl2: Point, to: NodeId },
}
```

`NodeId -> PathNode` mantém seleção e histórico estáveis. SVG arcs podem ser preservados como shape/metadata de import ou convertidos determinística e documentadamente; não misturar uma meia implementação de arc no kernel.

## NodeKind

```rust
pub enum NodeKind {
    Cusp,
    Smooth,
    Symmetric,
}
```

Isso é **regra de edição**, não muda a curva já armazenada até um handle ser manipulado.

- Cusp: handles independentes.
- Smooth: tangentes colineares; comprimentos independentes.
- Symmetric: colineares e mesmo comprimento.

## FillRule

Suportar `NonZero` e `EvenOdd`. Eles determinam interior de self-intersections e compound paths; não são “opção do renderer” apenas. Precisam sobreviver a SVG/PDF roundtrip.

## Orientação de contorno

Não depender da orientação para definir hole quando fill rule já carrega a semântica. Geometry Engine pode normalizar orientação para algoritmos específicos, mas não deve reordenar dados autorais silenciosamente.

## Degenerados

Precisamos de política explícita para:
- contour vazio
- 1 node
- segmento zero-length
- handles coincidentes
- contour fechado com último ponto igual ao primeiro
- self-intersection.

O Core pode armazenar alguns degenerados durante edição, mas export/boolean precisa normalização controlada. Evitar “cleanup automático” que mude uma forma sem Command.

## Shapes paramétricas

Rectangle, ellipse, star, polygon, gear e rounded rectangle **não devem nascer como path cru**.

```rust
pub enum ParametricShape {
    Rectangle(RectSpec),
    Ellipse(EllipseSpec),
    Star(StarSpec),
    Polygon(PolygonSpec),
    // ...
}
```

O Engine gera o path avaliável. `Convert to Curves` materializa um `VectorPath` por Command.

## Operações que NÃO ficam em path.rs

- boolean
- offset/contour
- simplify
- intersections
- curve fitting
- flatten
- stroke expansion
- nearest point
- hit-test

Todas são Geometry Engine.

## Curvas e precisão

Para cúbica Bézier, a avaliação pode usar De Casteljau por estabilidade numérica. Bounds exatos consideram extrema internas, não apenas anchors. Arc length é aproximado por tolerância adaptativa.

## IDs internos

Adicionar `ContourId` e `NodeId` antes de ferramentas complexas. Histórico baseado em índice quebra quando inserimos/deletamos nós.

## Migração do modelo atual

`PathNode { point, handle_in, handle_out }` pode continuar como API de edição inicial. A migração deve acontecer antes de boolean live, text-on-path e node-level undo porque esses recursos exigem referências estáveis.
