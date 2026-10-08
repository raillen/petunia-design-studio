# shape.rs

`shape.rs` define formas vetoriais autorais descritas por parâmetros de alto nível. Elas são diferentes de `VectorPath`: o usuário ainda pode editar intenção como tamanho, número de lados, raios e arco sem manipular nodes individuais.

Shapes paramétricas preservam intenção geométrica de alto nível enquanto continuarem editáveis como shapes.

> Rectangle continua Rectangle; Ellipse continua Ellipse; Star continua Star. Converter para curves é uma ação explícita.

## Modelo

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

## Rectangle

```rust
pub struct RectangleSpec {
    pub size: Size2,
    pub corners: CornerRadii,
}
```

Corner radii podem ser independentes por canto. Cada raio possui componentes X/Y não negativos e finitos.

Se a soma de raios adjacentes ultrapassa width/height, o evaluator reduz os raios proporcionalmente até caber. Essa normalização é derivada e determinística.

Rounded Rectangle não precisa ser tipo separado: é Rectangle com CornerRadii.

## Ellipse

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

## Polygon

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

## Star

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

## Corners compartilhados

Polygon e Star não ganham algoritmos próprios de rounded corners na v0.1.

```text
Polygon/Star
↓ evaluated path
↓ Live Corners
↓ result
```

Live Corners é Geometry Effect compartilhado.

## Outros generators

Gear, Spiral, Wave e similares só entram no formato persistente quando possuírem intenção autoral própria, parâmetros estáveis, avaliação determinística e valor suficiente para justificar schema.

Não transformar `ParametricShape` em catálogo infinito.

## Avaliação

Shapes são avaliados em VectorPath derivado.

Ellipse/arcs usam Cubic Bézier. Como ellipse não possui representação exata com número finito de cubics, a aproximação é determinística e respeita tolerância documentada.

A fonte paramétrica permanece exata; a aproximação não substitui os parâmetros.

## Geometry effects

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

## Convert to Curves

`Convert to Curves`:

1. avalia a shape;
2. materializa VectorPath;
3. cria ContourId/NodeId;
4. substitui `SceneItem::Shape` por `SceneItem::Path` em uma Transaction;
5. preserva ObjectId do SceneNode;
6. preserva Appearance/effects compatíveis;
7. registra inverse data para Undo restaurar a shape paramétrica.

`Convert to Curves` materializa somente geometria base. `Expand Appearance` é outro Command.

## Hit-test, snapping e bounds

Engine pode usar fórmulas analíticas quando forem mais baratas, desde que sejam semanticamente equivalentes ao path avaliado.

Bounds, evaluated path e tessellation são Derived State.

## Serialização

Persistir apenas parâmetros autorais finitos. Não persistir VectorPath derivado, tessellation, bounds, handles gerados ou caches.

## Invariantes de shapes

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

## Fronteira Core × Engine

Core persiste apenas o `ParametricShape` e seus parâmetros válidos.

Engine é responsável por:

- avaliar shape em `VectorPath`;
- aplicar tolerância de aproximação quando necessário;
- calcular bounds derivados;
- fornecer hit-test/snapping equivalentes;
- aplicar Geometry Effects;
- materializar `Convert to Curves`.

Render nunca precisa interpretar parâmetros autorais se já receber geometria avaliada pelo Render Model.

## Compatibilidade e evolução

Adicionar uma nova shape built-in altera o schema autoral e exige:

1. parâmetros com significado estável;
2. validação explícita;
3. avaliação determinística;
4. comportamento definido para valores-limite;
5. serialização/migração;
6. testes analíticos ou golden geometry;
7. benefício real que justifique ampliar o formato.

Não adicionar generators apenas para aumentar catálogo.

## Invariantes finais

1. `ParametricShape` e `VectorPath` são modelos autorais distintos.
2. Shape permanece paramétrica até Command explícito.
3. Transform do SceneNode permanece separado do ShapeSpec.
4. Avaliação da shape é Derived State.
5. `Convert to Curves` preserva o `ObjectId` do objeto quando ele continua sendo a mesma entidade de cena.
6. Novos contours/nodes materializados recebem IDs novos.
7. Rounded Rectangle é Rectangle com corner radii.
8. Circle é Ellipse com raios iguais.
9. Polygon/Star não duplicam algoritmos gerais de Live Corners.
10. Caches, handles gerados e tessellation nunca são persistidos como source.
