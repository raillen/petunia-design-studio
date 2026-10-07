# math.rs

`math.rs` contém apenas **vocabulário matemático estável**. Algoritmos geométricos pesados ficam no Engine.

## Escalar padrão

Geometria canônica usa `f64`. GPU pode receber `f32` após transformação para espaço de view.

## Tipos mínimos

```rust
pub struct Point { pub x: f64, pub y: f64 }
pub struct Vec2  { pub x: f64, pub y: f64 }
pub struct Size2 { pub width: f64, pub height: f64 }
pub struct Rect  { pub min: Point, pub max: Point }
pub struct Insets { pub left: f64, pub top: f64, pub right: f64, pub bottom: f64 }
pub struct Angle(pub f64); // radianos internamente
```

Ponto e vetor permanecem tipos diferentes: ponto + vetor = ponto; ponto - ponto = vetor.

## Rect e bounds

Evitar `Rect { x, y, width, height }` sem política para width negativo. Preferir rect normalizado por min/max. Fornecer `Rect::from_points`, `union`, `intersect`, `expand`, `contains`.

Bounds geométrico e bounds visual são diferentes. Stroke, blur e shadow expandem visual bounds; essa expansão não pertence ao `Rect` básico.

## Transform2D

Manter matriz afim 2×3:

```text
| a c tx |
| b d ty |
| 0 0  1 |
```

Precisamos decidir e documentar uma única convenção de multiplicação. Recomendação: `A * B * p` significa aplicar B e depois A.

API mínima:
- identity
- translation
- scale
- rotation
- skew
- multiply
- inverse -> Result/Option
- determinant
- transform_point
- transform_vector
- transform_rect conservador
- decompose/recompose

Matriz singular não pode retornar NaN silenciosamente.

## Sistemas de coordenadas

Definir explicitamente:

```text
Object Local
   ↓ local transform
Parent / Scene
   ↓ page/artboard transform
Document
   ↓ viewport transform
View
   ↓ device scale
Device Pixels
```

Transforms de viewport/device não são serializados no documento.

## Unidades

Unidade física não deve contaminar toda matemática. Geometria canônica usa document units; conversão fica em `units.rs`.

```rust
pub enum Unit { Px, Pt, Mm, Cm, Inch, Pica }
pub struct UnitValue { pub value: f64, pub unit: Unit }
```

DPI é necessário para conversões que envolvem pixel físico/documental. Não confundir “pixel unit” do documento com pixel de tela em zoom atual.

## Tolerâncias

Não criar `const EPSILON: f64` global para todas as operações.

Usar contextos:
- igualdade de UI
- flattening tolerance
- boolean tolerance
- hit-test tolerance
- snap tolerance em screen-space.

Uma tolerância boa para hit-test em zoom 10% é diferente da usada para decidir se dois pontos geométricos são coincidentes.

## Pontos não finitos

Na fronteira de Commands e import, rejeitar NaN/±Inf. Dados não finitos destroem ordering, bounds, spatial indices e serialização.

## Kurbo

O workspace já depende de `kurbo`. Devemos decidir se os tipos públicos do Core serão próprios ou wrappers/aliases. Recomendação: **tipos próprios na API persistente**, conversores para Kurbo no Engine. Isso evita acoplar o formato PTND à semântica/versionamento de uma dependência.
