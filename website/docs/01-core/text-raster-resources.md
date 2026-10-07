# texto, raster e recursos

Este arquivo de documentação cobre três modelos persistentes que não devem ser confundidos com seus engines.

## Texto — modelo autoral

```rust
pub struct TextObject {
    pub text: String,
    pub runs: Vec<TextRun>,
    pub container: TextContainer,
}

pub struct TextRun {
    pub range: TextRange,
    pub style: CharacterStyleRef,
}

pub enum TextContainer {
    Artistic,
    Frame(TextFrameSpec),
    OnPath(TextPathRef),
}
```

O Core guarda caracteres e intenção tipográfica; glyph IDs e positions são derivados pelo Text Engine.

## CharacterStyle

Precisa preservar:
- font family/reference
- size
- weight/stretch/style
- variable font axes
- OpenType features
- fill/stroke
- tracking
- baseline shift
- language/script metadata.

ParagraphStyle preserva alignment, leading, spacing, indents, tabs, bullets/numbering e hyphenation policy.

## Text flow

Frames conectados precisam IDs estáveis e uma relação explícita de next/previous flow. Engine detecta ciclos.

## Raster — storage semântico

```rust
pub struct PixelLayer {
    pub surface: PixelSurfaceRef,
    pub pixel_format: PixelFormat,
    pub color_profile: ColorProfileRef,
}

pub enum PixelFormat {
    Rgba8,
    Rgba16,
    Rgba16Float,
    Rgba32Float,
}
```

A camada não deve armazenar uma gigantesca `Vec<u8>` diretamente dentro do SceneNode.

## Tiled surface

Recomendação inicial: tiles 256×256 com benchmark antes de congelar formato.

Benefícios: undo por região, pintura parcial, cache, mipmaps e documentos grandes. Tiles podem ser sparse e compartilhados copy-on-write entre snapshots.

## Resources

```rust
pub enum ResourceSource {
    Embedded,
    Linked { uri: String },
}

pub struct ResourceRecord {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub source: ResourceSource,
    pub content_hash: ContentHash,
    pub metadata: ResourceMetadata,
}
```

Linked resource mantém identidade mesmo após reload. Hash detecta mudança de conteúdo; path sozinho não é identidade.

## Imagem colocada versus pixel layer

Uma `ImageObject` referencia uma imagem original e transformações não destrutivas. Uma `PixelLayer` é uma superfície editável. “Rasterize image” converte explicitamente a primeira na segunda.

## Fontes

Não serializar glyph cache. Documento guarda FontRef e, se permitido/licenciado, pacote pode incorporar bytes como Resource. Resolver fallback é responsabilidade do Engine.
