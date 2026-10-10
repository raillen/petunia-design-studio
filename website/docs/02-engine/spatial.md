# Spatial + Snapping Engine

Spatial responde perguntas de proximidade, hit-test, alinhamento, measurement e snapping.

> O índice espacial acelera consultas; ele nunca substitui SceneGraph ou geometria autoral.

## Spatial index

A implementação de produção da v0.1 usa **R*-tree** através de adapter, com `rstar` como dependência definida.

R*-tree agrupa AABBs espacialmente e suporta consulta por retângulo, nearest-neighbor e atualização dinâmica.

O tipo externo não atravessa a API Petunia.

~~~rust
pub trait SpatialIndex {
    fn query_aabb(
        &self,
        rect: Rect,
        out: &mut Vec<ObjectId>,
    );

    fn nearest(
        &self,
        point: Point,
        max_distance: f64,
        out: &mut Vec<ObjectId>,
    );
}
~~~

Uma implementação linear simples pode existir apenas como referência/teste e baseline de benchmark.

### Por que `rstar`

A escolha fica fechada para a v0.1 porque o problema já é concreto: precisamos de R*-tree dinâmica, consulta por envelope e nearest-neighbor sem implementar uma árvore espacial proprietária.

`rstar` permanece encapsulada:

~~~text
Petunia Rect/ObjectId
↓ adapter
rstar::RTree
↓ query
ObjectId candidates
~~~

Se profiling futuro mostrar limitação real, o trait `SpatialIndex` permite substituir a implementação sem alterar Scene, PTND, snapping ou hit-test.

Não manter simultaneamente R-tree e BVH de produção sem caso de uso medido.

## AABB

**AABB — Axis-Aligned Bounding Box** é o menor/um retângulo conservador alinhado a X/Y que contém o alvo indexado.

~~~text
object geometry
↓
visual/query bounds
↓
AABB in document space
↓
R*-tree
~~~

AABB é broad phase, não hit-test final.

## Broad phase e narrow phase

**Broad phase** elimina objetos impossíveis rapidamente.

**Narrow phase** executa teste geométrico preciso nos candidatos.

~~~text
pointer
↓ tolerance rect
R*-tree query
↓ candidate IDs
exact hit-test
↓ hits
~~~

Isso evita executar point-in-path em todos os objetos.

## Conteúdo do índice

Entry derivada precisa carregar apenas o necessário:

~~~rust
pub struct SpatialEntry {
    pub object: ObjectId,
    pub bounds: Rect,
}
~~~

Paint order, visibility e tipo podem ser consultados no snapshot ou em metadata derivada adicional.

Não duplicar SceneNode completo no índice.

## Bounds indexado

Para seleção/hit-test de objeto, indexar **visual/query bounds** conservador, incluindo stroke/effects quando eles contam como hittable.

Para snapping de geometria, providers podem usar geometric bounds próprios.

Não usar um único Rect para toda semântica.

## Atualização

Transaction produz conjunto de objetos afetados.

~~~text
commit
↓
affected object IDs
↓
recompute derived bounds
↓
update/remove/insert R-tree entries
~~~

Mudança estrutural grande pode justificar bulk rebuild.

A escolha incremental vs bulk é feita por custo medido; não congelar threshold arbitrário.

## Page scope

Queries incluem PageId/escopo.

Objeto de outra Page não vira candidato acidentalmente apenas porque coordenadas numéricas coincidem.

Artboards dentro da Page continuam filtráveis por policy.

# Paint order

Hit-test precisa ordenar hits pela mesma ordem semântica usada no Render.

Derivar `PaintOrderKey` do traversal:

~~~text
Page.root_children
↓
container children
↓
depth-first paint order
~~~

Não usar ObjectId ou ordem do R-tree como desempate de z-order.

# Hit testing

## Request

~~~rust
pub struct HitTestRequest {
    pub page: PageId,
    pub point_document: Point,
    pub tolerance_px: f64,
    pub mode: HitTestMode,
}
~~~

Tolerance de interação nasce em screen-space e é convertida usando ViewTransform.

## Path fill

Point-in-fill respeita FillRule.

Para EvenOdd: parity de crossings.

Para NonZero: winding number.

O algoritmo trabalha sobre path/flatten derivado com tolerance apropriada, ou backend robusto equivalente.

## Stroke

Stroke hit usa nearest distance ao centerline/expanded outline conforme StrokeStyle.

Variable width, dashes e markers precisam ser considerados quando mode pede appearance-aware hit.

Uma aproximação simples pode ser usada no broad phase, nunca como resposta final se produzir seleção errada acima da tolerance.

## Image

Default hit é source/visual rect transformado.

Alpha-aware hit é opção mais cara e exige decoded resource; não deve bloquear interação se buffer ainda não existe. Pode cair para bounds hit conforme policy explícita.

## Text

Text hit usa layout derivado: line boxes, cluster ranges e glyph bounds quando necessário.

Não reimplementar shaping no Spatial Engine.

## Nodes e handles

Node/handle hit usa geometria do Path + Session selection/tool context.

Esses alvos não entram obrigatoriamente no R-tree global; para objeto ativo normalmente é mais barato consultar nodes diretamente.

# Screen-space tolerance

~~~text
document_tolerance =
screen_tolerance_px / effective_view_scale
~~~

Para transforms não uniformes/rotated, usar transformação conservadora do raio de tela para região no documento, em vez de assumir escala escalar incorreta.

Isso mantém target visual estável.

# Snapping

Snapping recebe intenção de transformação e devolve uma correção; não muta o documento.

~~~text
proposed transform
↓
constraints
↓
candidate generation
↓
ranking / compatible match solve
↓
SnapResult
~~~

## SnapRequest

~~~rust
pub struct SnapRequest {
    pub page: PageId,
    pub moving: MovingGeometry,
    pub proposed: TransformDelta,
    pub settings: SnapSettings,
    pub view: ViewTransformInfo,
    pub previous: Option<SnapLatch>,
}
~~~

`SnapLatch` representa histerese da interação e é Session State.

## Candidate providers

Cada origem implementa provider estreito:

~~~text
GridProvider
GuideProvider
ObjectBoundsProvider
PathNodeProvider
IntersectionProvider
BaselineProvider
SpacingProvider
AngleProvider
TangentProvider
~~~

Todos produzem `SnapCandidate` no mesmo vocabulário.

Isso evita cinco motores de snap diferentes.

## SnapCandidate

~~~rust
pub struct SnapCandidate {
    pub target: SnapTarget,
    pub constraint: SnapConstraint,
    pub correction: TransformDelta,
    pub distance_px: f64,
    pub priority: SnapPriority,
    pub source: SnapSourceId,
}
~~~

Candidate descreve uma possível correção, não decisão final.

## Evitar self-snap

Objetos que estão sendo movidos não entram como target dos próprios anchors, exceto quando uma operation pede explicitamente snapping interno.

Selection/subtree exclusion é parte do request.

## Constraints antes do snap

Quando Shift/constraint limita movimento a eixo/ângulo:

~~~text
raw proposed delta
↓ constrain degrees of freedom
↓ generate/project snap candidates
↓ choose correction compatible with constraint
~~~

Não aplicar snap livre e depois constraint, pois o resultado pode “pular” para outra posição.

## Ranking

Ranking é determinístico e lexicográfico.

Ordem base:

1. candidate permitido pelas settings;
2. compatibilidade com constraint ativa;
3. semantic priority;
4. latch/hysteresis preference;
5. menor distance_px;
6. stable source/target key.

Não usar ordem de HashMap.

## Prioridade semântica

Exemplo inicial:

~~~text
explicit guide / exact node
> intersection
> object edge/center
> grid
> equal spacing suggestion
~~~

Essa tabela precisa ser configurável pela SnapSettings e testada. Não codificar dezenas de números mágicos espalhados.

## Histerese

**Histerese** mantém o snap atual até sair de uma distância de release maior que a distância de acquire.

~~~text
acquire <= 8 px
release > 11 px
~~~

Os números são exemplo, não constantes definidas.

Isso evita oscilar entre targets próximos.

Estado de latch nunca entra no PTND.

## Multi-axis snap

Em transformações simples, X/Y podem ser resolvidos independentemente quando constraints são ortogonais.

~~~text
candidate X
+
candidate Y
→ combined correction
~~~

Se candidates conflitam geometricamente, ranking escolhe combinação compatível de menor score.

Não construir solver genérico não-linear na v0.1 sem necessidade.

## SnapResult

~~~rust
pub struct SnapResult {
    pub corrected: TransformDelta,
    pub matches: Vec<SnapMatch>,
    pub visuals: Vec<GuideVisual>,
    pub latch: Option<SnapLatch>,
}
~~~

`GuideVisual` descreve geometry/meaning; Render/UI escolhem style.

# Smart guides

Smart guides são relações derivadas temporárias:

- alinhamento de edges/centers;
- equal spacing;
- shared baseline;
- angle relation.

Pipeline:

~~~text
nearby objects
↓ spatial query
relation generation
↓ candidates
↓ chosen matches
↓ guide visuals
~~~

Não criar Guide persistente automaticamente.

# Equal spacing

Para objetos ordenados em um eixo:

1. obter intervalos/bounds;
2. calcular gaps vizinhos;
3. gerar target gap compatível;
4. converter diferença em correction;
5. rankear em screen-space.

Usar tolerance visual; não comparar floats por igualdade exata.

# Grids

Todos os grids alimentam SnapCandidate.

## Affine grid

Cartesian, Isometric e Axonometric podem compartilhar uma lattice definida por basis vectors.

~~~text
P(i,j) =
origin + i·u + j·v
~~~

Cartesian:

~~~text
u = (sx, 0)
v = (0, sy)
~~~

Isometric/axonometric mudam direction/length de u/v.

Nearest lattice point pode ser obtido convertendo o ponto para coordinates da basis, arredondando i/j e voltando para document space.

## Pixel grid

Pixel grid é affine lattice vinculada à unidade raster/document policy, mas sua visibilidade pode depender de zoom.

Visibility é View State; geometry/snap definition pode ser Document State quando configurada.

## Baseline grid

É lattice 1D:

~~~text
y_n = origin + n·spacing
~~~

Text Layout usa a mesma definição de baseline que Spatial usa para snap.

## Perspective grid

Perspective grid não cabe em affine basis.

Modelar como spec persistente de horizon/vanishing geometry e derivar uma transformação projetiva.

Uma **homography** é matriz 3×3 que mapeia pontos de um plano para outro preservando linhas retas, mas permitindo convergência em perspective.

~~~text
logical grid
↓ projective transform
document lines
~~~

Engine gera visible grid lines/intersections sob demanda, sem materializar uma grade infinita.

# Guides permanentes

Guide é Core Document State.

Spatial Engine converte GuideDefinition em candidates.

Guide visibility continua View State.

# Measurement

Engine retorna valores estruturados:

~~~rust
pub struct Measurement {
    pub distance: f64,
    pub angle: Angle,
    pub delta: Vec2,
}
~~~

UI faz unit conversion/locale.

## Area e perimeter

Quando solicitado, Geometry fornece area/perimeter; Spatial apenas coordena relação/measurement.

Não duplicar integração de path.

# Performance

Objetivo é proteger latência.

- R-tree query broad phase;
- candidate providers recebem região limitada;
- não gerar todos os intersections do documento por pointer move;
- expensive intersection candidates podem ser lazy/nearby-only;
- reuse de snapshot/index por revision;
- backpressure em previews caros.

# Invariantes

1. R*-tree é o índice espacial de produção, encapsulado por adapter.
2. AABB é broad phase, não resposta geométrica final.
3. Paint order vem do Scene/Page, nunca do índice.
4. Hit-test usa tolerance de screen-space.
5. Snapping retorna correction; não muta Document.
6. Constraints são aplicadas antes/na geração de snap compatível.
7. Candidate providers compartilham o mesmo modelo.
8. Ranking e desempate são determinísticos.
9. Histerese é Session State.
10. Grids affine compartilham basis-vector model.
11. Perspective grid usa evaluation projetiva separada.
12. Smart guides não viram Guides persistentes automaticamente.

## Integração verificada em 2026-10-10

O índice e narrow phase compartilham world transforms, página ativa e visibilidade efetiva. Hit de snapshots cobre texto materializado, imagens recortadas e instâncias de símbolos. Contours respeitam fill rule e strokes usam o evaluator compartilhado. Ranking e hysteresis possuem testes de saída; a sessão mantém snap em coordenadas do documento e tolerância de ponteiro em view.

Evidência: `scheduler_spatial_regressions.rs` (13 testes), `render_bridge_regressions.rs` e `session_boundary.rs` (4 testes). Providers baseline/perspective, guides de artboard e markers continuam pendentes. [Gates](#/docs/00-architecture/verification.md).
