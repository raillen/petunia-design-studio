# Cache, overlays e output

Cache acelera; overlay explica interação; output adapta a imagem ao destino.

## O que é cache

**Cache** é uma cópia temporária de um resultado caro de calcular.

Exemplo:

~~~text
path autoral
   ↓
tessellation
   ↓
triângulos
   ↓
cache
~~~

Se o path não mudou, o renderer reutiliza os triângulos em vez de recalculá-los.

Cache é sempre derivado e descartável. Se apagarmos todos os caches, o documento ainda precisa continuar correto.

## Cache layers

Separar caches por responsabilidade:

- evaluated geometry;
- bounds;
- tessellation;
- glyph atlas;
- decoded image;
- raster effect tiles;
- composited group surface.

### Tessellation

**Tessellation** converte formas vetoriais em primitivas que o renderer consegue desenhar eficientemente, normalmente triângulos.

~~~text
curva / fill
     ↓
tessellator
     ↓
triângulos
     ↓
GPU ou software renderer
~~~

Os triângulos não substituem o path original. Eles são apenas uma representação de renderização.

## Invalidation

**Invalidation** significa marcar um cache como desatualizado.

Uma mudança de nome da layer não invalida tessellation. Uma mudança no path sim.

Por isso preferimos revisão e dependências a “limpar tudo”.

~~~text
Path revision 8
   ↓
Tessellation cache revision 8  → válido

Path revision 9
   ↓
Tessellation cache revision 8  → inválido
~~~

## Dependency graph

Um **dependency graph** registra quais resultados dependem de quais entradas.

Se Effect B depende de Effect A, alterar A invalida B. Conteúdo sem dependência de A continua válido.

Isso permite invalidação localizada em vez de reconstrução global.

## Memory budget

Cache acelera, mas usa memória. O renderer precisa de um **memory budget**: limite de memória destinado a caches.

Quando o limite é atingido, resultados menos úteis precisam ser descartados.

## LRU

**LRU** significa *Least Recently Used*.

A ideia é simples:

> quando precisamos liberar espaço, removemos primeiro aquilo que não é usado há mais tempo.

Exemplo:

~~~text
A usado agora
B usado há 2 s
C usado há 20 s
D usado há 3 min
          ↓
remove D primeiro
~~~

LRU é fácil de entender, mas manter uma ordem exata para todo acesso pode ter custo.

## Clock-like eviction

**Clock** é uma aproximação mais barata de LRU.

Cada item possui um marcador de “usado recentemente”. O algoritmo percorre os itens como um ponteiro de relógio:

1. se o item foi usado, limpa o marcador e avança;
2. se já estava sem marcador, ele pode ser removido;
3. continua até liberar memória suficiente.

~~~text
       A*
    ↗      ↘
 D           B
    ↖      ↙
       C*
~~~

`*` representa “usado recentemente”.

A política concreta deve ser escolhida por benchmark; o importante é ter budget e prioridade claros.

## Prioridade de cache

Recursos visíveis e próximos ao viewport têm maior valor imediato.

Uma política pode considerar:

- visibilidade;
- distância do viewport;
- custo de reconstrução;
- tamanho em memória;
- uso recente.

Não transformar essa heurística em estado persistente.

## Overlays

**Overlay** é desenho de interface sobre o canvas que não pertence à arte.

Exemplos:

- selection outline;
- bounding box;
- nodes/handles;
- guides;
- smart guides;
- snap anchors;
- marquee;
- brush cursor;
- measurement labels.

Engine fornece geometria abstrata. Render aplica theme/style.

## Hit feedback

Hover overlay pode atualizar frequentemente sem gerar Command ou nova revisão documental.

Isso é importante porque feedback de ponteiro é estado transitório.

## Output transform

O pipeline final adapta as cores calculadas para o destino:

~~~text
working linear/composite space
→ soft proof opcional
→ tone/gamut mapping quando necessário
→ display transform
→ device surface
~~~

### Tone mapping

**Tone mapping** comprime ou adapta uma faixa de luminância para um destino com capacidade menor.

É especialmente relevante em conteúdo HDR ou quando a tela não representa todo o range calculado.

### Gamut mapping

**Gamut** é o conjunto de cores que um dispositivo ou espaço consegue representar.

**Gamut mapping** adapta cores fora desse conjunto para cores reproduzíveis de acordo com uma política definida.

Essas operações são de visualização/output; não devem alterar silenciosamente as cores autorais do documento.

## Thumbnail

Thumbnail usa o mesmo renderer com overlays desligados e target menor.

Evitar criar um “mini renderer” separado porque duas implementações tenderiam a divergir.

## Headless

**Headless** significa executar sem janela ou interface gráfica.

Export, testes e batch processing precisam funcionar com RenderTarget offscreen sem QApplication.

Isso permite CI, CLI e processamento automatizado usando a mesma lógica de renderização.


## Política de cache v0.1

A v0.1 usa caches **renderer-owned**, não um cache global mutável compartilhado pela aplicação.

~~~text
RendererContext
├── Geometry/Tessellation Cache
├── Glyph Cache
├── Decoded Image Cache
├── Effect Tile Cache
└── Intermediate Surface Pool
~~~

Cada cache possui key própria. Compartilhamento entre renderer contexts só entra quando houver ganho medido.

## Soft budget e hard budget

Memória de cache usa dois limites conceituais:

- **soft budget** — ao ultrapassar, iniciar eviction;
- **hard budget** — nova alocação grande precisa liberar memória antes ou falhar/degradar explicitamente.

Esses budgets pertencem à aplicação/runtime, nunca ao PTND.

O valor padrão é calculado pela aplicação conforme memória disponível e plataforma. A arquitetura não congela um número absoluto universal.

## Eviction v0.1

A política inicial é **clock-like eviction**, porque evita manter uma lista LRU exata atualizada em todo acesso.

Cada entrada guarda pelo menos:

~~~text
size_bytes
recently_used
rebuild_cost_class
last_revision/key
~~~

Durante pressão de memória, o clock percorre entradas e favorece retenção de dados:

- visíveis/recentes;
- caros de reconstruir;
- pequenos com alto reuse.

Não manter cache por “idade” apenas: uma decoded image enorme pouco usada pode valer menos que centenas de glyphs pequenos.

A heurística pode ser refinada por profiling sem afetar semântica.

## Cache key rules

Toda key precisa conter somente dependências semânticas reais.

Exemplos:

~~~text
TessellationKey =
Path/GeometryRevision
+ transform scale bucket quando necessário
+ fill/stroke geometry params
+ flatten tolerance

GlyphKey =
font content identity
+ glyph id
+ raster size/subpixel mode
+ hinting/raster options

EffectTileKey =
input evaluation key
+ effect params
+ tile coordinate
+ quality
+ color context
~~~

Nome de layer, seleção ou tema não entram nessas keys quando não alteram o resultado.

## Scale buckets

Alguns caches dependem da escala de saída, como tessellation/flattening de preview.

Evitar uma entrada diferente para cada valor minúsculo de zoom.

Pode-se usar **scale buckets**: intervalos discretos de escala cuja tolerância de rasterização continua visualmente válida.

~~~text
zoom 101%
zoom 102%
zoom 103%
→ mesmo bucket, se erro visual continuar dentro do contrato
~~~

Buckets são optimization detail e precisam de benchmark. Export pode usar key exata/qualidade própria.

## Intermediate Surface Pool

Surfaces temporárias de render graph são reutilizadas por tamanho/formato compatível.

Elas não entram no cache semântico de efeito automaticamente.

~~~text
frame N
pass blur usa Surface A
↓ lifetime termina
pool
↓
frame N+1
outro pass compatível reutiliza A
~~~

Isso reduz alocação sem transformar buffer temporário em resultado persistente.

## Glyph cache

Glyph cache separa:

~~~text
font outline/shaping identity
↓
glyph raster key
↓
coverage bitmap/atlas slot
~~~

Eviction de atlas não invalida layout de texto: GlyphRun guarda glyph IDs/positions, não atlas coordinates autoritativas.

Atlas slot é Runtime State.

## Decoded image cache

Resource original continua fonte da verdade.

~~~text
Resource bytes
↓ decode
DecodedImage
↓ cache
~~~

Cache pode manter variantes por:

- content hash/resource revision;
- decoded pixel format;
- color transform context;
- mip/scale level se houver.

Relink/update do Resource invalida variantes dependentes.

## Effect cache

Efeito caro é cacheado por tile/ROI, não necessariamente como uma surface gigante inteira.

Mudança em uma pequena região da fonte pode invalidar somente tiles downstream cujo ROI toca a região alterada.

Essa granularidade entra incrementalmente; v0.1 pode invalidar mais amplamente desde que nunca reutilize resultado stale.

## Overlays após output documental

Overlays de edição são UI/presentation.

A ordem final definida é:

~~~text
document compositing
↓
soft proof / gamut preview quando ativo
↓
display/output transform
↓
editor overlays em espaço de apresentação
↓
present
~~~

Isso evita que cor de selection/handles seja reinterpretada pelo perfil de impressão do documento.

Export/thumbnail desligam editor overlays por padrão.

## Overlay primitives

Engine/UI descreve overlays abstratos:

~~~text
Line
Polyline
Rect
Handle
Anchor
TextLabel
CursorShape
HighlightRegion
~~~

Render decide rasterização final.

Overlay não referencia diretamente QML object nem vira SceneNode.

## Pixel-snapped overlays

Linhas de UI de 1 device pixel podem exigir alinhamento ao pixel grid para permanecer nítidas.

Essa correção acontece em device/view space e nunca modifica guides/geometry documentais.

## Soft proof pipeline

Soft proof é View State e usa:

~~~text
document compositing space
↓ proof transform / paper simulation
↓ display transform
↓ gamut warning overlay opcional
↓ editor overlays
~~~

A posição exata de gamut warning em relação ao display transform deve preservar sua função de aviso; a cor usada para warning é UI/presentation color, não authorial color.

## SDR output

Para targets SDR comuns:

1. compositor mantém float/linear;
2. aplica output/display transform;
3. aplica quantização para target;
4. dithering opcional reduz banding;
5. escreve surface final.

**Dithering** adiciona ruído de baixa amplitude controlado antes da quantização para reduzir faixas visíveis em gradientes.

Dithering é output detail e nunca altera Document.

## HDR output

HDR permanece capacidade arquitetural preparada.

Quando implementado, o Output contract precisará declarar:

- target color space;
- transfer function;
- reference white;
- peak luminance;
- tone mapping policy;
- metadata exigida pelo formato/display.

Não inferir HDR apenas porque buffers internos são float.

## Thumbnail contract

Thumbnail:

~~~text
mesmo RenderSnapshot
+ target pequeno
+ quality apropriada
+ overlays off
→ thumbnail
~~~

Pode usar cache próprio por document revision + target spec.

Preview armazenado no PTND é descartável e precisa carregar a revision/content identity de origem para não ser exibido como se fosse atual quando stale.

## Headless contract

Nenhum módulo de Render exige Qt para:

- export raster;
- thumbnail;
- golden tests;
- batch;
- CI;
- software render.

A integração Qt apenas apresenta uma surface/frame já produzido ou conecta um backend compatível.

## Invariantes de cache/output

1. Caches são renderer-owned e descartáveis.
2. Soft/hard memory budgets são runtime settings, não Document State.
3. Clock-like eviction é a política inicial da v0.1.
4. Cache key contém somente dependências semânticas reais.
5. Intermediate Surface Pool é diferente de effect cache.
6. Atlas slot e GPU handle nunca viram identidade autoral.
7. Overlay é composto depois do output documental para preservar semântica de UI.
8. Pixel snapping de overlay acontece apenas em device space.
9. Thumbnail usa o mesmo pipeline sem overlays.
10. Headless output não depende de Qt.
