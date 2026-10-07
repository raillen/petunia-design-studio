# document.rs

`Document` é a raiz persistente do arquivo PTND. Ele reúne configuração, scene, páginas, recursos e registries; não deve acumular algoritmos.

## Estrutura-alvo

```rust
pub struct Document {
    pub id: DocumentId,
    pub schema_version: SchemaVersion,
    pub metadata: DocumentMetadata,
    pub setup: DocumentSetup,
    pub pages: PageCollection,
    pub scene: SceneGraph,
    pub resources: ResourceRegistry,
    pub styles: StyleRegistry,
    pub symbols: SymbolRegistry,
    pub swatches: SwatchRegistry,
    pub guides: GuideSet,
}
```

## DocumentSetup

Precisa decidir:

```rust
pub struct DocumentSetup {
    pub units: Unit,
    pub dpi: f64,
    pub color: DocumentColorSpec,
    pub default_page: PageSpec,
}
```

Width/height globais deixam de ser suficientes quando houver páginas/artboards de tamanhos diferentes.

## Pages, spreads e artboards

Separar:
- Page: unidade editorial.
- Spread: arranjo de páginas para layout.
- Artboard: área de design independente dentro de uma página/canvas.

Não usar o mesmo tipo com flags obscuras.

## Guias e grids

Persistir guias e configurações documentais:

```rust
pub struct Guide {
    pub id: GuideId,
    pub orientation: GuideOrientation,
    pub position: f64,
    pub locked: bool,
    pub scope: GuideScope,
}
```

O desenho da guia é Render; drag é UI; snapping é Engine.

## Recursos

Documento guarda referências a imagens, perfis ICC, fontes incorporáveis, padrões e outros assets. Bytes grandes não devem ser duplicados em cada SceneNode.

## Metadata

Separar metadata funcional de metadata editorial:
- title/author
- created/modified
- generator/version
- custom metadata
- document notes.

Timestamp de “modified” deve ser atualizado em save, não em cada pointer move.

## Sessão NÃO pertence ao documento

Não colocar aqui:
- active tool
- selection
- hover
- zoom/pan
- panel layout
- search state
- clipboard.

Esses dados ficam em `StudioSession`/app settings.

## Dirty state

`is_dirty` é derivado de revision/history checkpoint. Não serializar um boolean “dirty”.

## Autosave e recovery

Autosave usa snapshot/journal fora do modelo autoral. Um recovery file precisa registrar schema version, base save revision e transações necessárias para restaurar estado.

## Serialização

`to_json/from_json` atuais servem ao bootstrap. A API futura deve chamar um `serialization` module para separar modelo de estratégia de arquivo, permitir migrations e blobs binários sem transformar `Document` em I/O service.
