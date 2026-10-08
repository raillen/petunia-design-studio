# Guides, Grids e Export Slices

Esses recursos auxiliam construção e saída, mas continuam parte do Document quando expressam intenção reutilizável.

~~~text
Guide / Grid definition
→ Document State

visibility / snap enabled
→ View / Session State

snap candidates / rendered lines
→ Derived State

Export Slice
→ Document State

arquivo exportado
→ Engine output
~~~

## Guides

Guide persistente representa uma referência geométrica simples.

~~~rust
pub struct Guide {
    pub id: GuideId,
    pub axis: GuideAxis,
    pub position: f64,
    pub locked: bool,
    pub scope: GuideScope,
}

pub enum GuideAxis {
    Horizontal,
    Vertical,
}

pub enum GuideScope {
    Document,
    Page(PageId),
    Artboard(ObjectId),
}
~~~

A v0.1 mantém Guides horizontais/verticais.

Guide angular futura precisa de spec própria; não codificar ângulo em um campo escondido.

## Scope

`Document` torna a guide disponível nas Pages compatíveis conforme policy de coordenadas global/editorial.

`Page` fixa a guide à Page.

`Artboard` fixa a guide ao Artboard correspondente.

Engine converte scope para document/page space durante snapping/render.

## Visibility

Guide visibility é View State.

Ocultar guides:

~~~text
não remove
não altera position
não cria HistoryEntry
não deixa Document dirty
~~~

`locked` é Document State porque expressa intenção de edição compartilhável.

## GridDefinition

Grid persistente usa spec tipada.

~~~rust
pub struct GridDefinition {
    pub id: GridId,
    pub scope: GridScope,
    pub origin: Point,
    pub spec: GridSpec,
}
~~~

~~~rust
pub enum GridSpec {
    Affine(AffineGridSpec),
    Baseline(BaselineGridSpec),
    Perspective(PerspectiveGridSpec),
}
~~~

Pixel grid de viewport não exige um quarto motor; ver seção específica.

## Affine Grid

Uma **affine lattice** é definida por dois vetores-base.

~~~text
P(i,j) =
origin + i·u + j·v
~~~

Modelo:

~~~rust
pub struct AffineGridSpec {
    pub kind: AffineGridKind,
    pub basis_u: Vec2,
    pub basis_v: Vec2,
    pub subdivisions_u: u32,
    pub subdivisions_v: u32,
}

pub enum AffineGridKind {
    Cartesian,
    Isometric,
    Axonometric,
    Custom,
}
~~~

`kind` preserva intenção/preset.

`basis_u` e `basis_v` preservam a geometria exata.

Isso evita tentar reconstruir o grid a partir de um nome de preset.

## Validação de Affine Grid

Invariantes:

- basis vectors finitos;
- nenhum basis vector possui comprimento zero;
- os dois vetores não são colineares dentro da tolerância definida;
- subdivisions >= 1.

Dois vectors colineares não formam uma grade 2D invertível.

## Cartesian

Preset:

~~~text
u = (spacing_x, 0)
v = (0, spacing_y)
~~~

A UI futura pode expor spacing diretamente; o documento guarda o basis resultante + kind.

## Isometric

É um AffineGrid preset com eixos de ângulos/escala definidos pela operação de criação.

A spec persistente continua basis-based para evitar depender de convenção visual implícita.

Editar o preset atualiza os basis vectors por Command.

## Axonometric

Também usa AffineGrid, mas permite eixos/escala não isométricos.

Não criar outro algoritmo de snapping.

## Baseline Grid

~~~rust
pub struct BaselineGridSpec {
    pub spacing: f64,
    pub offset: f64,
    pub subdivisions: u32,
}
~~~

~~~text
y_n =
origin.y + offset + n·spacing
~~~

Invariantes:

- spacing finito > 0;
- offset finito;
- subdivisions >= 1.

Text Layout e Spatial usam a mesma definição.

Não manter uma baseline grid no Text Engine e outra no snapping.

## Pixel Grid

A grade de pixels usada para edição raster/zoom alto é, por padrão, **View State derivado**.

~~~text
active raster/pixel coordinate system
+ zoom
+ view transform
↓
Pixel Grid Overlay
~~~

Ela não precisa ser salva no PTND para aparecer.

Quando o usuário quiser uma grade documental equivalente, usa um AffineGrid explícito com spacing correspondente.

Isso evita persistir estado puramente visual apenas porque a feature se chama “grid”.

## Perspective Grid

Perspective Grid é Document State quando criado/configurado.

A representação v0.1 usa uma transformação projetiva do plano lógico da grade para a Page.

~~~rust
pub struct PerspectiveGridSpec {
    pub transform: ProjectiveGridTransform,
    pub spacing: Vec2,
    pub subdivisions: u32,
}
~~~

`ProjectiveGridTransform` representa uma homography 3×3 normalizada.

## Homography

Uma **homography** transforma coordenadas homogêneas:

~~~text
|x'|   |h00 h01 h02| |x|
|y'| = |h10 h11 h12| |y|
|w'|   |h20 h21 h22| |1|

document point =
(x'/w', y'/w')
~~~

Ela preserva linhas retas, mas permite convergência em pontos de fuga.

A matriz pode ser normalizada para remover escala global redundante.

## Validação projetiva

Projective grid exige:

- coeficientes finitos;
- matriz não degenerada;
- spacing finito e positivo;
- subdivisions >= 1.

Ao gerar linhas visíveis, pontos onde `w'` fica próximo de zero estão no/ao redor do horizonte projetivo e precisam de clipping/guard numérico.

Nunca gerar coordenadas infinitas e passá-las ao renderer.

## Edição de Perspective Grid

Ferramenta futura pode expor:

- horizon;
- vanishing points;
- origin;
- spacing handles.

Esses controles são UI.

O Engine converte a manipulação em `ProjectiveGridTransform` validado.

Assim a representação matemática fica estável sem congelar a experiência de interação.

## Grid scope

Direção:

~~~rust
pub enum GridScope {
    Document,
    Page(PageId),
    Artboard(ObjectId),
}
~~~

Scope inválido/dangling é rejeitado pelo Core.

## Grid visibility e snapping

Não persistir no GridDefinition:

~~~text
visible
snap_enabled
hovered
selected
~~~

São View/Session State.

Engine recebe settings da sessão e decide quais definitions geram candidates.

## Derived grid geometry

Não materializar infinitas linhas.

Engine recebe viewport/query bounds:

~~~text
GridSpec
+ visible region
↓
finite line/intersection range
↓
Render/Spatial
~~~

Para Affine Grid, calcular intervalo de índices i/j que pode cruzar a região.

Para Perspective Grid, gerar somente famílias de linhas relevantes e clipar.

## SnapCandidate

Todos os grids convergem para a mesma API de snapping:

~~~text
GridDefinition
↓ evaluate nearby points/lines
↓ SnapCandidate
↓ same ranking/hysteresis system
~~~

Nenhum Grid possui “motor de snap próprio”.

## Export Slice

Export Slice é uma intenção de saída reutilizável salva no Document.

~~~rust
pub struct ExportSlice {
    pub id: SliceId,
    pub name: String,
    pub source: SliceSource,
    pub presets: Vec<SliceExportPreset>,
}
~~~

## SliceSource

~~~rust
pub enum SliceSource {
    Page(PageId),
    Artboard(ObjectId),
    Object(ObjectId),
    Rect {
        page: PageId,
        rect: Rect,
    },
}
~~~

Source precisa existir e ser compatível.

Rect é Page-local.

## Por que preset fica no documento

A exportação precisa ser reproduzível ao abrir o arquivo em outra máquina.

Portanto o Slice persiste os parâmetros de output relevantes, em vez de referenciar apenas uma preferência global do usuário.

~~~rust
pub struct SliceExportPreset {
    pub format: ExportFormat,
    pub scale: ExportScale,
    pub color: ExportColorOptions,
    pub suffix: Option<String>,
}
~~~

A estrutura exata acompanha o Export Engine, mas o princípio é definido:

> parâmetros necessários para reproduzir o slice viajam com o documento.

Preset global da aplicação pode servir como template para criar um SliceExportPreset, mas não permanece como dependência externa invisível.

## Output location

Slice não salva absolute filesystem destination como parte autoral obrigatória.

Destino de export é runtime/Application State.

Isso evita:

- paths privados dentro do PTND;
- arquivo inútil em outra máquina;
- sobrescrita acidental de path antigo.

Nome/suffix relativo pode fazer parte do preset.

## Slice evaluation

~~~text
SliceSource
↓ resolve current bounds/content
↓ ExportPlan
↓ Render/Vector exporter
↓ output
~~~

Slice não guarda bitmap/render cache como fonte.

Alterar source atualiza output na próxima exportação.

## Object Slice

Object source usa visual bounds conforme export policy.

Effect/shadow não pode ser cortado porque o engine usou geometric bounds por engano.

A policy precisa declarar bleed/padding adicional quando houver.

## Artboard/Page Slice

Artboard respeita sua spec/clip policy.

Page respeita PageSpec, bleed e export options.

## Rect Slice

Rect representa região autoral explícita.

Pode conter áreas transparentes e não precisa se ajustar automaticamente aos objetos existentes.

## Missing source

Slice com source interna inexistente é dangling e Document inválido.

Delete do alvo precisa:

- remover Slice;
- retarget;
- ou rejeitar;

conforme Command explícito.

Não manter slice quebrado silenciosamente.

## Export presets e capabilities

Antes de export:

~~~text
SliceExportPreset
+ source capabilities
+ exporter capabilities
↓
ExportPlan
~~~

Plan decide preserve/expand/rasterize/warn/error conforme I/O Engine.

## History

Criar/mover/editar Guide/Grid/Slice é Command/Transaction normal.

Alterar visibility/snap settings não entra no History documental.

## Serialization

Persistir:

- IDs;
- geometry/spec;
- scope;
- lock;
- SliceSource;
- SliceExportPreset.

Não persistir:

- visible;
- snap latch;
- generated grid lines;
- SnapCandidates;
- export progress;
- output destination absoluto;
- render cache.

## Invariantes

1. Guide geometry/lock é Document State; visibility é View State.
2. Guides v0.1 são horizontais/verticais.
3. Cartesian/Isometric/Axonometric compartilham AffineGrid.
4. AffineGrid persiste kind + basis vectors.
5. BaselineGrid é uma única definição compartilhada por Layout e Spatial.
6. Pixel Grid padrão é View State derivado.
7. PerspectiveGrid persiste uma homography validada.
8. Grid nunca materializa uma grade infinita.
9. Todo Grid produz o mesmo SnapCandidate model.
10. SliceSource é tipado e Page-local quando usa Rect.
11. Slice persiste parâmetros necessários para export reproduzível.
12. Output filesystem destination não é parte autoral obrigatória.
13. Slice usa visual/export bounds corretos, não geometric bounds por conveniência.
14. Delete de source resolve Slice dependency atomicamente.
