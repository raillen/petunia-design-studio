# Modelos autorais para ferramentas criativas

**Status:** contrato de dados proposto. Nomes e campos de Rust são direção de implementação; toda inclusão no PTND exige SchemaVersion/DTO/migration e operações atômicas.

## Regra

Ferramentas não persistem resultados gerados como se fossem fonte. O Core registra a intenção estável de objetos live, referências tipadas, parâmetros e fallback semanticamente explícito.

### Novos SceneItems necessários

Proposta de evolução controlada de SceneItem:

~~~rust
enum SceneItem {
    // variantes já existentes ...
    LiveRegion(RegionPaintObject),
    LiveDistribution(DistributionObject),
    Dimension(DimensionObject),
    // BrandSheet pode ser Group + Text + Shape + linked styles
}
~~~

Não acrescentar variantes se uma combinação de Scene/Appearance/Effect existente representar exatamente o mesmo contrato sem ambiguidade. A principal exigência é **semântica persistente distinta**, não inflar enums.

## RegionPaintObject

~~~rust
struct RegionPaintObject {
    sources: Vec<ObjectId>,
    gap_policy: GapPolicy,
    paints: Vec<RegionPaintBinding>,
}
struct RegionPaintBinding {
    region: RegionLocator,
    paint: Paint,
}
struct RegionLocator {
    source_ids: Vec<ObjectId>,
    // assinatura determinística de provenance/topologia,
    // coordenada testemunha somente como fallback controlado
}
enum GapPolicy {
    None,
    VirtualBridges { max_gap_document_units: f64 },
}
~~~

**Provenance** é o vínculo entre uma região derivada e as partes dos objetos que a formaram. O fingerprint geométrico derivado sozinho não pode servir como identidade autoral eterna. Ao alterar source, o Engine tenta remapear o binding; se houver ambiguidade, marca região unresolved e preserva paint/intent, sem pintar outra área silenciosamente.

Region Paint **não** muda a aparência original dos objetos-fonte. Expand é Command explícito.

Intertwine tem contrato separado: local overlap order/occlusion regions para um grupo, nunca altera z-order global nem corta path silenciosamente. Reusar estruturas de provenance quando conveniente; não forçar pintura e occlusão a terem o mesmo tipo autoral.

## DistributionObject

~~~rust
struct DistributionObject {
    sources: Vec<ObjectId>,
    spec: DistributionSpec,
    seed: Option<RandomSeed>,
}
enum DistributionSpec {
    Linear(LinearRepeatSpec),
    Grid(GridRepeatSpec),
    Radial(RadialRepeatSpec),
    Mirror(MirrorRepeatSpec),
    AlongPath(AlongPathSpec),
    Blend(BlendSpec),
    Scatter(ScatterSpec),
}
~~~

Fonte fica autoral e reutilizada; instâncias são derivadas com transform/appearance override, não cópias com ObjectId persistente por instância. Selecionar/editá-las exige locator derivado {generator ObjectId, instance index}; se Expand, **criar IDs persistentes**.

Referências a spine/path e generators formam dependências: ciclos são rejeitados. Alteração da source invalidará somente resultados dependentes.

Blend deve distinguir:
- Transform/color/opacity interpolation entre fontes compatíveis.
- **Geometry morph** exige correspondência explícita entre contornos/nodes; não inventar matching apenas com índice. Se incompatível, erro tipado ou modo crossfade, nunca mutação aleatória.

## DimensionObject

~~~rust
struct DimensionObject {
    kind: DimensionKind,
    anchors: Vec<DimensionAnchor>,
    units: UnitPresentation,
    precision: u8,
    style: DimensionStyle,
    offset: f64,
}
enum DimensionKind {
    Linear,
    Angular,
    Radius,
    Diameter,
    Area,
    Perimeter,
}
enum DimensionAnchor {
    ObjectNode { object: ObjectId, node: NodeId },
    ObjectFeature { object: ObjectId, feature: FeatureReference },
    FixedDocumentPoint(Point),
}
~~~

Medida exibida é **Derived State** e reavaliada dos anchors. Se o alvo deixa de existir ou o feature reference não resolve, mostrar stale/unresolved; não reposicionar para objeto vizinho.

Annotations persistentes participam do render/export conforme propriedades autorais. Measurements de hover são Session State e nunca PTND.

## Stroke width/softness

Variable Width pertence a StrokeStyle ou referencial tipado de WidthProfile; armazena amostras por **posição ao longo do path**, larguras left/right, interpolação e corner discontinuity explícita. WidthProfile não reescreve VectorPath.

Vector Feather pertence a Appearance/Effect, com amostras de softness ao longo do contorno, unidade e edge falloff; qualidade de rasterização derivada. Não chamar de variable width: largura geométrica e suavidade visual são independentes.

## Pattern

PatternDefinition referencia source/asset editável e parâmetros de tile layout (transform, offsets, repeats, mirror, brick); Paint::Pattern é vínculo. Tile raster/cache não entra no PTND.

## Warp e brushes

Perspective/Envelope Warp persiste envelope/control surface e relação com source. Engine fornece avaliação; materialização somente com Expand/Bake.

True Vector Brush persiste spine + BrushPreset/BrushAssetId + spacing/dynamics/seed e atributos mínimos reavaliáveis, nunca milhares de dabs materializados enquanto estiver live. Se a semântica de brush for puramente raster, permanece PixelLayer/brush engine, não finge ser vetor.

## Recolor e Brand Sheet

Recolor Live é adjustment/effect ou vínculo de palette, **não** reescrita arbitrária de cada objeto. ColorMappingSpec referencia swatches/targets/locks/intents; conversão perceptual pertence ao Color Engine.

Brand Sheet inicial é gerado por Command como Scene Group + shapes/text + references de swatches/styles quando aplicável. Um grupo autoatualizável (template regenerável) exigirá especificação própria de conflito e provenance, e não é assumido no MVP.

## Identidade, remoção, serialização

- ObjectId original persiste em editar params/mover generator.
- Referências internas dangling nunca ficam ocultas; remover source dispara policy explícita: reject, detach/materialize ou delete cascade via Transaction.
- Undo/Redo restaura identidade; Expand gera novas identidades.
- Dados de terceiros não atravessam Core.
- Params finitos, ranges limitados, seed explícita, sem dependência de UI.
- DTO versionado para cada feature; arquivo de versão antiga não é reinterpretado silenciosamente com algoritmos novos.

## Abstrações evitadas

Não criar um GenericMagicGenerator, per-instance ObjectId para toda repetição, region ID derivado tratado como permanente, nem armazenar geometria de preview no documento.

[Catálogo das ferramentas](#/docs/04-ui/creative-tools-overview.md) · [Engine](#/docs/02-engine/creative-operations.md)