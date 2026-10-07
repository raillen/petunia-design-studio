# Spatial + Snapping Engine

Este domínio responde “o que está perto de quê?”: hit-test, snapping, alignment, guides, measurement e índice espacial.

## Spatial index

Começar com trait:

```rust
pub trait SpatialIndex {
    fn query_aabb(&self, rect: Rect, out: &mut Vec<ObjectId>);
    fn nearest(&self, point: Point, max_distance: f64, out: &mut Vec<ObjectId>);
}
```

MVP pode usar scan; documentos grandes devem migrar para R-tree/BVH sem alterar callers.

Index armazena bounds derivados, nunca substitui SceneGraph.

## Hit testing

Ordem:
1. converter pointer View → Document
2. spatial query por tolerância
3. candidatos em z-order reverso
4. teste preciso por tipo
5. aplicar política da ferramenta.

Path hit-test distingue fill, stroke, node e handle.

## Tolerância em screen-space

O usuário espera aproximadamente o mesmo alvo visual em qualquer zoom.

```text
document_threshold = screen_px_threshold / view_scale
```

Snapping atual usa threshold em unidades documentais fixas; deve migrar.

## SnapCandidate

```rust
pub struct SnapCandidate {
    pub target: SnapTarget,
    pub point: Point,
    pub distance_px: f64,
    pub priority: SnapPriority,
    pub source_object: Option<ObjectId>,
}
```

## Alvos

- grid
- guide
- page/artboard edge
- object bounds
- center
- corner/node
- midpoint
- path nearest
- intersection
- baseline
- equal spacing
- angle
- tangent, quando suportado.

## Ranking

Escolha determinística:
1. target explicitamente habilitado
2. prioridade semântica
3. menor distância em pixels
4. estabilidade/histerese com snap anterior
5. desempate estável por ID.

Sem histerese, cursor pode oscilar entre dois alvos próximos.

## SnapResult

Resultado precisa explicar o que ocorreu para o Render desenhar feedback:

```rust
pub struct SnapResult {
    pub transformed: TransformDelta,
    pub matches: Vec<SnapMatch>,
    pub guide_visuals: Vec<GuideVisual>,
}
```

Engine descreve linhas/anchors sem cor de tema. Render/UI escolhem estilo visual.

## Smart guides

Alignment e equal-distance são consultas espaciais. Gerar candidatos temporários, não inserir Guides permanentes no documento.

## Guides permanentes

Guide data está no Core. Engine calcula snap; UI manipula; Render desenha.

## Measurement

Distância e ângulo precisam unidades e precisão formatada na UI. Engine retorna valores dimensionais, não strings localizadas.
