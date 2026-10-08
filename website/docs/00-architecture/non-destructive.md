# Não destrutibilidade

Não destrutibilidade é uma **regra estrutural do documento**.

> A fonte autoral permanece preservada; edições reavaliáveis são armazenadas como intenção e parâmetros. Materialização destrutiva só ocorre por Command explícito.

## Modelo de avaliação

O fluxo conceitual é:

~~~text
Source
  ↓
Geometry Operations
  ↓
Appearance
  ↓
Image Effects / Adjustments
  ↓
Mask / Clip
  ↓
Blend / Composite
  ↓
Output
~~~

Cada etapa possui semântica própria. Nem toda operação pode ser movida livremente para outra fase.

## Source

**Source** é o dado autoral que uma operação consome.

Exemplos:

- VectorPath original;
- ParametricShape + parâmetros;
- texto Unicode + estilos;
- imagem colocada;
- PixelLayer;
- SymbolDefinition.

A fonte não é substituída pelo resultado derivado de uma operação live.

## Operation

Uma **Operation** é uma transformação parametrizada e reavaliável.

Exemplos:

~~~text
Live Offset(distance = 8)
Live Corners(radius = 12)
Gaussian Blur(sigma = 4)
Levels(...)
Live Boolean(Union)
~~~

A operação persiste intenção, não seu cache de resultado.

## Evaluation

**Evaluation** calcula o resultado atual de Source + Operations.

~~~text
Source + parameters
        ↓
Evaluator
        ↓
Derived Result
~~~

O resultado pode ser geometry, paint primitives ou surface raster dependendo da fase.

Evaluation nunca altera silenciosamente Source.

## Fases

| Fase | Entrada típica | Exemplos | Saída |
|---|---|---|---|
| Geometry | vetor/shape | live corners, offset, warp, live boolean | geometria |
| Appearance | geometria | fills, strokes, markers | primitives de pintura |
| Image Effect | surface | blur, shadow, glow | surface |
| Adjustment | pixels/cor | levels, HSL, curves | surface/cor transformada |
| Composite | surfaces | opacity, mask, blend | surface final |

A fronteira é semântica. Blur não edita nodes; Offset não opera sobre pixels já rasterizados.

## Stack e DAG

Para operações lineares, uma stack ordenada é suficiente:

~~~text
Source
 ↓
Offset
 ↓
Blur
 ↓
Levels
~~~

Algumas relações possuem múltiplas entradas. O evaluator deve suportar uma **DAG — Directed Acyclic Graph**, ou grafo direcionado sem ciclos.

~~~text
Path A ─────┐
            ├→ Live Boolean → Offset
Path B ─────┘
~~~

“Sem ciclos” significa que nenhuma cadeia de dependências volta ao mesmo nó.

Inválido:

~~~text
A → B → C → A
~~~

O Core rejeita ciclos na mutação. O evaluator não tenta resolver ciclos por timeout.

## Modelo persistente

Não usar “nome do efeito + HashMap<String, Value>” para operações built-in.

Operações nativas usam tipos versionáveis:

~~~rust
pub enum GeometryEffect {
    Offset(OffsetParams),
    Corners(CornerParams),
    Warp(WarpParams),
}

pub enum ImageEffect {
    GaussianBlur(BlurParams),
    DropShadow(ShadowParams),
    Glow(GlowParams),
}
~~~

Cada instância possui identidade estável:

~~~rust
pub struct EffectInstance<T> {
    pub id: EffectId,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask: Option<MaskRef>,
    pub operation: T,
}
~~~

A implementação concreta pode usar enums separados por fase para impedir combinações inválidas no tipo.

## Ordem semântica

A ordem altera resultado:

~~~text
Offset → Blur
≠
Blur → Offset
~~~

Reorder é mutação autoral e gera Command.

Ao mover uma operação, apenas resultados downstream são invalidados quando possível.

**Downstream** significa operações que dependem direta ou indiretamente daquele resultado.

## Habilitar/desabilitar

Desabilitar efeito preserva:

- EffectId;
- parâmetros;
- posição na stack;
- máscara;
- metadata.

~~~text
enabled = false
↓
evaluator faz passthrough
~~~

**Passthrough** significa devolver semanticamente a entrada sem aplicar a operação.

## Failure semantics

Uma operação live pode falhar por geometria degenerada, resource ausente ou configuração inválida.

Falha não deve destruir Source.

O evaluator retorna estado tipado, por exemplo:

~~~text
Success(result)
Degraded(result + warning)
Unavailable(reason)
Failed(error)
~~~

A representação exata pode variar, mas “falhou → substituir source por vazio” é proibido.

## Live Boolean

Live Boolean referencia suas entradas e a operação:

~~~text
inputs = [Object A, Object B, Object C]
operation = Union
~~~

O resultado vetorial é derivado.

**Expand Boolean** materializa o resultado como VectorPath novo e remove/substitui a relação live conforme o Command escolhido.

## Live geometry

Live Offset, Live Corners, Contour e futuros geometry effects seguem:

~~~text
Source Geometry
   ↓ operation params
Derived Geometry
~~~

O Core persiste Source + params. Engine calcula Derived Geometry.

## Appearance não destrutiva

Múltiplos fills e strokes permanecem parâmetros.

Stroke expandido usado para render/hit-test é derivado.

**Expand Stroke** cria paths novos explicitamente.

## Masks e clips

Clip e mask são relações persistentes, não pixels pré-aplicados.

~~~text
Content ─────┐
             ├→ Composite
Mask/Clip ───┘
~~~

Clip limita cobertura geometricamente. Mask modula cobertura/alpha/luminância conforme sua semântica.

## Bake, Expand, Rasterize e Flatten

Esses Commands possuem significados distintos.

| Command | Materializa |
|---|---|
| **Expand Stroke** | stroke → paths |
| **Expand Appearance** | aparência/effects vetoriais suportados → geometria |
| **Expand Boolean** | live boolean → paths |
| **Rasterize** | conteúdo avaliável → PixelLayer |
| **Bake Effect** | effect live → pixels/geometria conforme effect |
| **Flatten** | várias entradas compostas → surface única |

Eles nunca acontecem automaticamente só para facilitar implementação.

## Provenance em materialização

Quando um resultado materializado nasce de várias fontes, o Engine pode carregar **provenance** durante avaliação.

Provenance registra de onde uma parte do resultado veio.

~~~text
segment resultante
→ source Object A / Contour C / Segment S
~~~

Isso ajuda a preservar aparência e metadata quando semanticamente possível.

Provenance é derivado; não precisa virar formato persistente geral.

## Evaluator

O **Evaluator** percorre dependências, resolve operações e produz resultados derivados.

Responsabilidades:

- validar tipo de entrada/saída;
- ordenar dependências;
- detectar estados indisponíveis;
- escolher/cachear implementação;
- aplicar tolerâncias;
- propagar bounds;
- calcular ROI;
- respeitar revision;
- oferecer cancelamento em operações caras.

Não é responsabilidade do evaluator:

- mutar Document;
- registrar Undo;
- decidir UX;
- acessar QML;
- salvar PTND.

## Evaluation key

Cache não deve depender apenas de ObjectId.

Chave conceitual:

~~~text
operation identity
+ source revisions
+ operation parameters/revision
+ evaluation quality
+ render scale quando relevante
+ color context quando relevante
+ backend semantic version quando necessário
~~~

Não incluir parâmetros irrelevantes a uma fase.

Exemplo: mudar nome da layer não invalida offset geometry.

## Revisões locais

DocumentRevision protege consistência global.

Caches finos podem futuramente usar revisions locais como:

~~~text
GeometryRevision
AppearanceRevision
ResourceRevision
~~~

Somente introduzir quando profiling demonstrar benefício. Não antecipar complexidade.

## Bounds propagation

Cada operação precisa declarar como transforma bounds.

Exemplos:

~~~text
Transform → transforma bounds
Offset(8) → expande aproximadamente pela distância
Blur → expande pelo alcance do kernel
Crop → intersecta
~~~

Quando o bounds exato for caro, pode existir bounds conservador.

**Conservador** significa que pode ser maior que o resultado, mas nunca menor a ponto de cortar conteúdo válido.

## Region of Interest

**ROI — Region of Interest** é a região mínima de entrada necessária para gerar determinada região de saída.

~~~text
output tile
   ↓
Blur precisa vizinhos
   ↓
input ROI expandido
~~~

O evaluator propaga ROI de trás para frente.

Isso permite efeitos em documentos grandes sem processar toda a superfície.

## Qualidade de avaliação

Preview interativo pode permitir qualidade menor se a semântica final for preservada.

Exemplo:

~~~text
InteractivePreview
Final
Export
~~~

Diferenças permitidas precisam ser documentadas por operação.

O commit nunca armazena “preview approximation” como Source.

## Preview

Interação contínua usa transient override:

~~~text
base operation params
+
preview override
↓
Evaluator
↓
preview result
~~~

Pointer move não cria EffectInstance novo nem modifica PTND.

No commit, o mesmo cálculo semântico recebe os parâmetros finais.

## Determinismo

Operações que geram resultado autoral materializável precisam de determinismo semântico.

Se usam aleatoriedade, seed é parâmetro explícito.

Se processamento paralelo produz ordem arbitrária, o resultado é canonicalizado antes de virar Document State.

## Plugins

Operações de plugin não ganham acesso livre ao Document.

Um plugin effect recebe inputs/snapshots definidos e produz resultado de contrato.

Payload persistente de plugin:

- usa namespace;
- possui versionamento;
- pode ser preservado opacamente se plugin ausente;
- não executa durante load apenas por estar no arquivo.

## CPU e GPU

CPU e GPU podem implementar a mesma operação.

Isso não cria duas semânticas.

~~~text
Effect Contract
├── CPU implementation
└── GPU implementation
~~~

O contrato define resultado, color space, bounds, edge behavior e tolerância aceitável.

Se a implementação GPU não suporta uma operação, fallback é explícito; não muda o documento.

## Contrato mínimo de operação

Toda operação live precisa documentar:

1. fase;
2. tipo de entrada;
3. tipo de saída;
4. parâmetros persistentes;
5. versionamento/migração;
6. bounds propagation;
7. ROI;
8. tolerâncias;
9. determinismo;
10. color-space semantics;
11. cache dependencies;
12. cancelamento se cara;
13. CPU/GPU support;
14. failure behavior;
15. materialização explícita equivalente.

## Invariantes

1. Source autoral não é substituída por resultado derivado.
2. Operation persiste intenção e parâmetros, não cache.
3. Evaluation é read-only em relação ao Document.
4. DAGs de dependência nunca contêm ciclos.
5. Fase de operação é explícita.
6. Reorder é mudança autoral.
7. Falha live preserva Source.
8. Bake/Expand/Rasterize/Flatten só ocorrem por Command explícito.
9. ROI e bounds fazem parte do contrato de efeitos caros.
10. Preview usa a mesma semântica do resultado final.
11. Implementações CPU/GPU obedecem ao mesmo contrato.
12. Plugins não podem introduzir estado executável opaco durante load.
