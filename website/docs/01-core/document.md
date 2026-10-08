# document.rs

`Document` é a raiz autoral persistente do PTND.

Ele reúne configuração, páginas, SceneGraph e registries, mas **não executa algoritmos** e não conhece UI, renderer ou filesystem.

## Estrutura

~~~rust
pub struct Document {
    pub id: DocumentId,
    pub metadata: DocumentMetadata,
    pub setup: DocumentSetup,
    pub pages: PageCollection,
    pub scene: SceneGraph,
    pub resources: ResourceRegistry,
    pub styles: StyleRegistry,
    pub symbols: SymbolRegistry,
    pub swatches: SwatchRegistry,
    pub guides: GuideRegistry,
    pub grids: GridRegistry,
    pub slices: SliceRegistry,
}
~~~

`SchemaVersion` não precisa ser campo do Domain Document. Ele pertence ao DTO/container de persistência.

## DocumentSetup

~~~rust
pub struct DocumentSetup {
    pub units: Unit,
    pub default_raster_dpi: f64,
    pub color: DocumentColorSpec,
    pub default_page: PageSpec,
}
~~~

`default_raster_dpi` é default para operações que precisam converter medida física para raster. Não representa Device Pixel Ratio da tela.

## Pages

Page é unidade editorial/documental.

~~~rust
pub struct Page {
    pub id: PageId,
    pub name: String,
    pub spec: PageSpec,
    pub root_children: Vec<ObjectId>,
}

pub struct PageSpec {
    pub size: Size2,
    pub margins: Insets,
    pub bleed: Insets,
}
~~~

Cada objeto principal pertence a exatamente uma Page através de parent/root semantics.

A ordem de `root_children` é z-order daquela Page.

## ParentRef

Para evitar root implícito ambíguo, SceneNode deve evoluir para:

~~~rust
pub enum ParentRef {
    Page(PageId),
    Object(ObjectId),
}
~~~

Assim um node sempre possui owner estrutural explícito.

SymbolDefinition usa subtree própria no SymbolRegistry e não reaproveita ParentRef da cena principal sem adapter/modelo dedicado.

## Spread

Spread organiza páginas para layout/editorial.

~~~rust
pub struct Spread {
    pub id: SpreadId,
    pub pages: Vec<PageId>,
    pub arrangement: SpreadArrangement,
}
~~~

Spread não muda ownership dos SceneNodes; apenas organiza Pages.

Isso evita usar o mesmo tipo para Page e Spread com flags obscuras.

## Artboard

Artboard continua sendo SceneItem/container dentro de uma Page.

~~~text
Document
└── Page
    ├── Artboard A
    │   └── objects
    └── Artboard B
        └── objects
~~~

Page e Artboard possuem propósitos diferentes:

- Page: unidade documental/editorial;
- Artboard: região de design dentro da Page/canvas.

## Página mínima

Document válido possui pelo menos uma Page.

Documento “canvas livre” pode ser representado futuramente por PageSpec de canvas/infinite mode, mas não introduzir um segundo root system antes de necessidade concreta.

## Guides

~~~rust
pub struct Guide {
    pub id: GuideId,
    pub axis: GuideAxis,
    pub position: f64,
    pub locked: bool,
    pub scope: GuideScope,
}

pub enum GuideScope {
    Document,
    Page(PageId),
    Artboard(ObjectId),
}
~~~

Guide geometry/lock é Document State.

Visibility de guides é View State.

Snapping é Engine.

## Grids

GridDefinition é persistente quando faz parte da construção.

~~~rust
pub struct GridDefinition {
    pub id: GridId,
    pub scope: GridScope,
    pub origin: Point,
    pub kind: GridKind,
    pub spacing: Vec2,
    pub subdivisions: u32,
}
~~~

GridKind pode evoluir para Cartesian, Isometric, Axonometric e Perspective através de specs tipados.

Grid visibility e snap enabled não pertencem ao Document.

## Slices

Export Slice é intenção autoral reutilizável.

~~~rust
pub struct ExportSlice {
    pub id: SliceId,
    pub source: SliceSource,
    pub name: String,
    pub export_presets: Vec<ExportPresetRef>,
}
~~~

Slice pode referenciar Artboard/Object/Rect documental.

A exportação em si pertence ao Engine.

## Resources

ResourceRegistry centraliza imagens, pixel surfaces, perfis ICC, fontes incorporadas, patterns e outros assets.

SceneNodes guardam ResourceId; não duplicam blobs.

## Styles

StyleRegistry mantém estilos compartilhados.

Mudança em style linked pode afetar múltiplos objetos sem copiar Appearance para cada um.

Styles possuem IDs próprios e payload tipado.

## Symbols

SymbolRegistry mantém definitions separadas das instances do SceneGraph.

Definition é Document State e usa IDs estáveis em sua subtree.

## Swatches

Swatches são recursos de cor/gradient reutilizáveis.

`SwatchId` permite vínculo explícito entre Paint e palette documental.

## Metadata

Separar metadata autoral/editorial de dados operacionais.

Exemplos:

- title;
- author;
- created timestamp;
- modified timestamp;
- document notes;
- custom metadata namespaced.

`modified` é atualizado no save bem-sucedido, não a cada pointer move.

Application version que salvou pode existir em manifest/tooling metadata, mas não é semântica do Document.

## Units

Document unit define apresentação/entrada padrão.

A geometria continua em document units canônicas; mudar preferência de exibição não converte silenciosamente todas as coordenadas.

## Dirty state

Não existe boolean autoritativo `Document.is_dirty`.

~~~text
is_dirty =
current_revision != saved_revision
~~~

History/DocumentSession controla checkpoint.

Dirty não é serializado.

## Session não pertence ao Document

Não armazenar:

- active tool;
- selection;
- hover;
- zoom/pan;
- canvas rotation;
- panel layout;
- focused widget;
- search state;
- clipboard;
- progress de jobs.

Esses dados são Session/Application State.

## Autosave e recovery

Autosave trabalha sobre snapshot consistente.

Recovery é infraestrutura separada.

~~~text
last safe PTND
+
recovery journal/snapshot
↓
recovered Document
~~~

Recovery metadata pode guardar base revision, timestamps e transactions necessárias.

Não adicionar estado de ferramenta transitório ao PTND só para recovery.

## Registries e ordem

Registry é lookup por ID, não necessariamente ordem autoral.

Quando ordem possui semântica, ela é armazenada explicitamente em coleção separada.

Exemplo:

~~~text
SwatchRegistry lookup
+
Palette order Vec<SwatchId>
~~~

Não deixar HashMap iteration definir UI, export ou serialização.

## Ownership

Uma entidade persistente possui owner claro.

Exemplos:

~~~text
SceneNode → Page/Object parent
SymbolDefinition → SymbolRegistry
Resource → ResourceRegistry
Style → StyleRegistry
Guide → GuideRegistry
~~~

Não duplicar a mesma entidade autoritativa em dois registries.

## Validação do Document

Construção/commit precisam garantir:

- DocumentId válido;
- pelo menos uma Page;
- PageIds únicos;
- Page root children consistentes com SceneGraph;
- referências de registries existentes;
- SceneGraph válido;
- Guide/Grid scopes válidos;
- ColorSpec válido;
- nenhum número não finito;
- nenhum dangling reference obrigatório.

## Serialização

Domain Document não implementa estratégia de container diretamente.

~~~text
Document
↓ DTO adapter
Current DocumentDto
↓ serializer
PTND container
~~~

I/O Engine coordena save/load, migrations e atomic replace.

## Invariantes

1. Document é aggregate root autoral, não service.
2. SchemaVersion pertence à persistência, não ao Domain Document.
3. Page, Spread e Artboard são conceitos distintos.
4. Page possui root children ordenados.
5. SceneNode possui owner estrutural explícito por ParentRef.
6. Document válido possui pelo menos uma Page.
7. Guide/Grid geometry pode ser documental; visibility é View State.
8. Registries não usam ordem de HashMap como semântica.
9. Session/Dirty/Job state não entra no PTND.
10. Resource blobs não são duplicados em SceneNodes.
11. Recovery é infraestrutura externa ao modelo autoral.
12. Serialization passa por DTO/I/O layer.
