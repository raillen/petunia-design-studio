# Spatial + Snapping Engine

Este domínio responde “o que está perto de quê?”: hit-test, snapping, alignment, guides, measurement e índice espacial.

## Spatial index

Um **índice espacial** é uma estrutura que evita testar todos os objetos do documento quando procuramos algo próximo de um ponto ou região.

A API pode começar simples:

~~~rust
pub trait SpatialIndex {
    fn query_aabb(&self, rect: Rect, out: &mut Vec<ObjectId>);
    fn nearest(&self, point: Point, max_distance: f64, out: &mut Vec<ObjectId>);
}
~~~

### AABB

**AABB** significa *Axis-Aligned Bounding Box*: um retângulo alinhado aos eixos X/Y que contém um objeto.

Ele é barato de comparar e serve como primeiro filtro:

~~~text
pointer
  ↓
AABB query
  ↓
poucos candidatos
  ↓
teste geométrico preciso
~~~

O AABB não substitui o hit-test exato; ele apenas reduz o número de objetos que precisam ser examinados.

### R-tree e BVH

Para o MVP, um scan linear pode ser suficiente. Em documentos grandes, candidatos naturais são **R-tree** ou **BVH**.

**R-tree** agrupa retângulos próximos em uma árvore. Uma consulta descarta rapidamente grupos inteiros que não intersectam a área pesquisada.

**BVH** (*Bounding Volume Hierarchy*) também organiza objetos em uma árvore de volumes envolventes. Cada nó cobre seus filhos; se o volume do pai não interessa à consulta, todos os descendentes podem ser ignorados.

Ambos têm o mesmo objetivo para o Petunia:

> reduzir uma busca de “testar tudo” para “testar apenas regiões plausíveis”.

A escolha entre R-tree e BVH permanece aberta até profiling com documentos reais.

O índice guarda bounds derivados e nunca substitui o SceneGraph.

## Hit testing

**Hit-test** responde qual elemento está sob o ponteiro.

Ordem:

1. converter o pointer de View para Document;
2. consultar o índice espacial com a tolerância visual;
3. ordenar candidatos pelo z-order;
4. executar teste geométrico preciso por tipo;
5. aplicar a política da ferramenta ativa.

### Z-order

**Z-order** é a ordem visual de empilhamento. O objeto desenhado por último normalmente aparece por cima e deve ser testado primeiro para seleção.

Path hit-test distingue:

- fill;
- stroke;
- node;
- handle.

Esses alvos possuem geometrias e tolerâncias diferentes.

## Tolerância em screen-space

**Screen-space** significa medir em pixels visuais da tela, não em unidades do documento.

O usuário espera que um node continue fácil de clicar tanto em 20% quanto em 800% de zoom.

~~~text
document_threshold = screen_px_threshold / view_scale
~~~

Exemplo:

- tolerância desejada: 8 px;
- zoom: 200% = escala 2;
- tolerância no documento: 4 unidades equivalentes.

Assim o alvo visual permanece aproximadamente constante.

## SnapCandidate

Um **SnapCandidate** é uma possibilidade de encaixe encontrada pelo Engine. Encontrar um candidato ainda não significa escolhê-lo.

~~~rust
pub struct SnapCandidate {
    pub target: SnapTarget,
    pub point: Point,
    pub distance_px: f64,
    pub priority: SnapPriority,
    pub source_object: Option<ObjectId>,
}
~~~

## Alvos

Candidatos podem vir de:

- grid;
- guide;
- page/artboard edge;
- object bounds;
- center;
- corner/node;
- midpoint;
- path nearest;
- intersection;
- baseline;
- equal spacing;
- angle;
- tangent.

Cada tipo declara prioridade e condição de ativação.

## Ranking

Quando vários candidatos são válidos, o Engine precisa escolher de forma determinística.

Ordem sugerida:

1. alvo explicitamente habilitado;
2. prioridade semântica;
3. menor distância em pixels;
4. estabilidade em relação ao snap anterior;
5. desempate estável por ID.

## Histerese

**Histerese** evita que o snap fique trocando rapidamente entre dois alvos quase equivalentes.

Sem histerese:

~~~text
A  B
↔ ↔ ↔ ↔
cursor oscila entre os dois
~~~

Com histerese, o alvo atual recebe uma pequena preferência até o ponteiro se afastar o suficiente.

Isso produz sensação de estabilidade sem “grudar” permanentemente.

A histerese é estado transitório da interação, não dado do documento.

## SnapResult

O resultado precisa explicar o que ocorreu para o Render desenhar feedback:

~~~rust
pub struct SnapResult {
    pub transformed: TransformDelta,
    pub matches: Vec<SnapMatch>,
    pub guide_visuals: Vec<GuideVisual>,
}
~~~

Engine descreve linhas, pontos e relações. Render/UI escolhem cor, espessura e aparência.

## Smart guides

**Smart guides** são guias temporárias inferidas da geometria próxima, como alinhamento de centros ou espaçamento igual.

Elas não são adicionadas ao documento.

~~~text
objetos vizinhos
   ↓
consulta espacial
   ↓
relações possíveis
   ↓
SnapCandidates temporários
   ↓
feedback visual
~~~

## Guides permanentes

Guide persistente é dado do Core.

- Core guarda a guia.
- Engine calcula snap.
- UI manipula.
- Render desenha.

## Measurement

Engine retorna valores geométricos, não strings localizadas.

Exemplo:

~~~text
distance = 12.538 document units
angle = 0.523598... rad
~~~

A Interface decide mostrar “12,54 mm” ou “30°” de acordo com unidade, locale e precisão configurados.
