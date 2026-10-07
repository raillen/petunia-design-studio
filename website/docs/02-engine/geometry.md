# Geometry Engine

Geometry transforma e consulta geometria; não possui widgets nem estado de seleção.

## Biblioteca base

O workspace já possui `kurbo` e `i_overlay`. Kurbo fornece vocabulário e algoritmos Bézier; iOverlay fornece boolean robusto em polygons, inclusive fill rules e simplificação. Encapsular ambos atrás de APIs Petunia para evitar vazar tipos externos.

## Módulos

```text
geometry/
├── bezier.rs
├── flatten.rs
├── bounds.rs
├── nearest.rs
├── intersections.rs
├── boolean.rs
├── offset.rs
├── simplify.rs
├── stroke_expand.rs
├── curve_fit.rs
├── warp.rs
└── shape_builder.rs
```

## Bézier

Para cubic Bézier P0..P3:

```text
B(t) = (1-t)³P0
     + 3(1-t)²tP1
     + 3(1-t)t²P2
     + t³P3
```

Preferir De Casteljau para subdivision/evaluation robusta e derivadas para tangentes/extrema.

## Flattening

Converter curva em segmentos de reta apenas com tolerância explícita. Tolerância depende da finalidade:
- render preview
- boolean
- hit-test
- export.

Não salvar o flatten como geometria autoral.

## Bounds

Bounds de curva precisa resolver extrema da derivada. Bounds de anchors apenas é incorreto para handles que passam além dos endpoints.

## Boolean

API semântica:

```rust
pub enum BooleanOp { Union, Intersect, Subtract, Xor, Divide }
pub fn boolean(inputs: &[EvaluatedPath], op: BooleanOp, tol: GeometryTolerance)
    -> Result<Vec<VectorPath>>;
```

Live Boolean mantém referências no Core e usa esta função na avaliação; “Expand Boolean” materializa o resultado.

## Offset / Contour

Offset precisa política de join, cap e miter. Curvas podem ser aproximadas adaptativamente. Resultados com self-intersection passam por cleanup topológico explícito.

## Simplificação

Não existe um único “simplify”.

Separar:
- merge coincident/near points
- remove collinear
- polyline simplification, como RDP/Visvalingam
- curve refit para Bézier
- contour cleanup depois de tracing.

Parâmetros devem ser expressos em erro máximo visual/documental, não “percentual mágico”.

## Curve fitting

Image Trace e Pencil precisam fitting de amostras para curvas. Usar algoritmo que minimize erro geométrico e limite número de segmentos, com corner detection separada.

## Shape Builder

Pipeline:
1. flatten/evaluate candidate contours
2. planar subdivision/intersections
3. construir regions
4. hit-test region sob pointer
5. union/subtract regiões escolhidas
6. reconstruir paths e provenance.

Preview de região é transitório; commit gera Command.

## Robustez

Todo algoritmo recebe tolerância e retorna erro tipado. Nada de NaN, loops infinitos em self-intersections ou “best effort silencioso” em topologia inválida.
