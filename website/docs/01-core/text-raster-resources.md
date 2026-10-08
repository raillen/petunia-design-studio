# Texto, raster e recursos

Texto, pixel data e resources são **Document State** distintos de seus caches/evaluations.

Esta página define apenas o modelo autoral. Shaping, layout, brush algorithms, decoding e rendering pertencem aos Engines/Render correspondentes.

# Texto

## TextObject

~~~rust
pub struct TextObject {
    pub text: String,
    pub runs: Vec<TextRun>,
    pub paragraphs: Vec<ParagraphRun>,
    pub container: TextContainer,
    pub flow: TextFlow,
}
~~~

O Core guarda Unicode + intenção tipográfica.

Não guarda glyph IDs, glyph positions, line boxes ou glyph atlas.

## TextRange

`TextRange` usa offsets de bytes UTF-8 no runtime/persistência nativa inicial.

Invariantes:

- start <= end;
- ambos dentro da String;
- ambos em boundary UTF-8 válido.

Operações de cursor/backspace não trabalham diretamente por byte; Text Engine/UI usam grapheme segmentation.

**Grapheme** é a unidade visual percebida como caractere, que pode conter mais de um code point.

## TextRun

~~~rust
pub struct TextRun {
    pub range: TextRange,
    pub style: CharacterStyleRef,
}
~~~

Runs são ordenados e não sobrepostos.

Gaps usam o default character style do TextObject/document style policy.

Não representar formatação por markup embutido na String nativa.

## CharacterStyle

Precisa representar, diretamente ou por StyleRef:

- FontRef;
- size;
- weight/stretch/slant;
- variable font axes;
- OpenType features;
- Color/Paint;
- tracking;
- baseline shift;
- language;
- script override quando necessário;
- decoration.

`language` é importante para hyphenation e shaping.

## ParagraphStyle

Preserva:

- alignment;
- leading/line-height;
- space before/after;
- first-line/left/right indents;
- tabs;
- bullets/numbering;
- hyphenation policy;
- keep rules;
- baseline-grid policy.

Paragraph layout é derivado.

## TextContainer

~~~rust
pub enum TextContainer {
    Artistic,
    Frame(TextFrameSpec),
    OnPath(TextPathRef),
}
~~~

### Artistic

Texto cresce conforme conteúdo; não possui frame de quebra obrigatório.

### Frame

Possui geometria/size e overflow policy.

### OnPath

Referencia path/shape autoral e parâmetros como start offset/orientation.

O glyph placement sobre a curva é derivado.

## Text flow

Linked text frames usam referência em uma única direção.

~~~rust
pub struct TextFlow {
    pub next: Option<ObjectId>,
}
~~~

O frame anterior é derivado por index/query.

Guardar next + previous autoritativos criaria duas fontes que podem divergir.

Core valida tipo do alvo e proíbe ciclos.

## Texto e transformação

Font size e text metrics permanecem autorais em unidades documentais.

SceneNode transform continua separado.

Transformar objeto não reescreve cada glyph.

## FontRef

FontRef representa a intenção tipográfica.

Pode referenciar:

- família/style nominal;
- ResourceId de fonte incorporada;
- face identificada por metadata;
- variable font coordinates.

A resolução para arquivo/face disponível é Derived State.

Missing font não reescreve FontRef automaticamente.

# Raster

## PixelLayer

PixelLayer é superfície raster **editável**.

~~~rust
pub struct PixelLayer {
    pub surface: PixelSurfaceRef,
}
~~~

Informações de storage/cor pertencem à PixelSurface referenciada, evitando duplicação.

## PixelSurface

~~~rust
pub struct PixelSurfaceDescriptor {
    pub size: PixelSize,
    pub format: PixelFormat,
    pub color_space: ColorSpaceRef,
}
~~~

PixelSurface bytes vivem em Resource storage próprio, não dentro de SceneNode.

## PixelFormat

Direção inicial:

~~~rust
pub enum PixelFormat {
    Rgba8Unorm,
    Rgba16Unorm,
    Rgba16Float,
    Rgba32Float,
}
~~~

**Unorm** significa inteiro normalizado para faixa 0…1.

Float formats permitem extended range conforme color pipeline.

Adicionar Gray/CMYK raster storage só quando houver pipeline completo que preserve sua semântica.

## Tiles

Superfície grande é dividida em tiles lógicos.

~~~text
surface
┌───┬───┬───┐
│ A │ B │ C │
├───┼───┼───┤
│ D │ E │ F │
└───┴───┴───┘
~~~

Benefícios:

- brush atualiza região;
- undo preserva apenas tiles afetados;
- filtros trabalham por ROI;
- snapshots compartilham tiles;
- documentos grandes não exigem copiar a imagem inteira.

## Tile size

Não congelar 256×256 por intuição.

A implementação começa com um valor candidato configurável internamente e mede:

- cache locality;
- overhead de metadata;
- brush locality;
- blur/ROI;
- export;
- memória.

Se tile size virar parte do container físico, SchemaVersion/migration torna a decisão explícita.

## Sparse surface

Tiles totalmente vazios podem não possuir allocation física.

Ausência de tile representa um valor definido, normalmente transparent black no espaço da surface.

Essa regra precisa ser única em brush, filter, save e render.

## Authorial tile versus cache

Pixel tile editado é Document State.

Não confundir com:

- GPU texture;
- mipmap;
- effect tile;
- decoded preview;
- thumbnail.

Esses são Derived State.

## Copy-on-write

Snapshots/Undo podem compartilhar tile bytes imutáveis.

~~~text
revision 10 ─┐
             ├→ Tile A
revision 11 ─┘

edit Tile A
↓
revision 11 → Tile A'
~~~

COW é estratégia runtime; não precisa determinar o formato público do PTND.

## Dirty tiles

`dirty tiles` é estado derivado de processamento: quais tiles precisam de recompute/upload/save incremental.

Não é um campo autoral do PixelLayer.

# Resources

## ResourceRecord

~~~rust
pub struct ResourceRecord {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub source: ResourceSource,
    pub content_hash: Option<ContentHash>,
    pub metadata: ResourceMetadata,
}
~~~

`ResourceId` é identidade lógica. `ContentHash` descreve bytes conhecidos.

## Embedded

Resource incorporado vive dentro do PTND/container.

~~~rust
pub enum ResourceSource {
    Embedded { entry: ResourceEntryRef },
    Linked { uri: ResourceUri },
}
~~~

`ResourceEntryRef` é referência lógica à entry; não expor path arbitrário do filesystem.

## Linked

Linked resource preserva ResourceId mesmo se URI mudar via Relink.

~~~text
ResourceId R
uri old
↓ Relink
uri new
ResourceId R
~~~

Relink é Command.

## URI

ResourceUri precisa de parsing/normalização próprios.

Não concatenar URI externa diretamente em filesystem path.

Loader aplica permissões e regras de segurança antes de abrir conteúdo.

## Missing linked resource

Quando arquivo externo some:

~~~text
ResourceId existe
source = Linked
resolve falha
↓
ResourceState::Unresolved
~~~

O documento continua válido.

Render pode mostrar placeholder/último preview se disponível, sem substituir a fonte.

## ContentHash

Hash ajuda a:

- detectar alteração externa;
- compartilhar caches;
- deduplicar blobs;
- validar integridade.

Hash não é identidade.

Mudança de bytes não cria ResourceId novo automaticamente se semanticamente o mesmo linked resource foi atualizado.

## ImageObject

ImageObject referencia ResourceId de imagem original.

Direção:

~~~rust
pub struct ImageObject {
    pub resource: ResourceId,
    pub source_rect: Option<ImageSourceRect>,
    pub sampling: ImageSamplingPolicy,
}
~~~

SceneNode transform posiciona a imagem.

Crop não destrutivo modifica source rect/clip intent, não reescreve bytes da imagem original.

## ImageObject versus PixelLayer

~~~text
ImageObject
→ placed immutable source

PixelLayer
→ editable raster surface
~~~

`Rasterize` materializa uma imagem/evaluated object em PixelLayer por Command explícito.

## Decode

`image` crate ou outro decoder produz buffers derivados a partir do Resource.

Decoded buffer é cache quando a fonte original continua preservada.

Import que cria PixelLayer materializa bytes na PixelSurface de forma explícita.

## Fonts

Font bytes podem ser Embedded Resource quando licença permitir.

Documento guarda FontRef e resource metadata; não guarda glyph cache.

Resolver face, fallback e shaping pertence ao Text Engine.

## Licensing

Embeddability de fontes não deve ser ignorada.

Resource metadata precisa permitir registrar política/flags necessárias para export/package.

Petunia não deve incorporar automaticamente uma fonte proibida apenas porque o arquivo está instalado.

## Profiles

ICC profiles incorporados também são Resources.

ColorSpaceRef pode apontar para ResourceId compatível.

Parser ICC e CMM ficam no Color Management Engine.

## Patterns e outros blobs

Pattern bitmap/vector assets, future LUTs e outros dados grandes usam ResourceRegistry quando possuírem identidade documental.

Não duplicar blobs por AppearanceItem.

## Resource lifecycle

Delete de SceneNode não apaga Resource imediatamente.

Garbage collection de resources verifica:

- Scene references;
- Styles;
- Symbols;
- History/recovery policy quando aplicável.

`Purge Unused Resources` pode ser Command/maintenance explícita.

## Mipmaps e previews

Não serializar mipmap/cache como fonte autoral.

Preview opcional no PTND é acelerador descartável e precisa ser marcado como derivado da source revision/hash.

## Invariantes

1. Text Core guarda Unicode/intenção, não glyph layout.
2. TextRange é validado em UTF-8 boundaries.
3. Runs autorais não se sobrepõem.
4. Text flow armazena uma direção e proíbe ciclos.
5. Font fallback nunca reescreve FontRef silenciosamente.
6. PixelLayer referencia PixelSurface; não contém Vec gigante no SceneNode.
7. Pixel data autoral e raster cache são estados diferentes.
8. Tile size é decisão medida, não constante arquitetural arbitrária.
9. Linked resource pode ficar unresolved sem corromper Document.
10. ResourceId e ContentHash possuem papéis diferentes.
11. Rasterize é Command explícito.
12. Delete de objeto não purga Resource automaticamente.
13. Font embedding respeita política/licença.
14. Glyph cache, mipmaps e previews são Derived State.
