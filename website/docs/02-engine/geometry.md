# Geometry Engine

Geometry transforma, consulta e materializa geometria. Não possui widgets, seleção autoral ou estado de documento mutável.

A regra principal é:

> **Core define a geometria. Geometry Engine calcula sobre snapshots/inputs imutáveis e devolve resultados tipados.**

## Dependências

`kurbo` é adapter/base para operações Bézier e matemática geométrica onde sua semântica atende ao Petunia.

`i_overlay` é backend preferencial para overlay/topologia poligonal robusta.

Tipos dessas crates não atravessam a API pública do Engine.

## Módulos

~~~text
geometry/
├── bezier.rs
├── flatten.rs
├── bounds.rs
├── nearest.rs
├── intersections.rs
├── boolean.rs
├── offset.rs
├── corners.rs
├── simplify.rs
├── stroke_expand.rs
├── curve_fit.rs
├── warp.rs
└── shape_builder.rs
~~~

Não criar um `geometry.rs` monolítico.

# Tolerâncias

Toda operação que aproxima ou decide coincidência recebe contexto de tolerância explícito.

~~~rust
pub struct GeometryTolerance {
    pub coincidence: f64,
    pub flatten: f64,
    pub intersection: f64,
    pub fit: f64,
}
~~~

Essa struct é direção de API, não obrigação de agrupar todos os valores se funções mais estreitas forem melhores.

Não existe EPSILON global.

## Quality presets

UI/Render podem pedir classes:

~~~text
InteractivePreview
Authoring
Export
~~~

Cada classe resolve para tolerâncias documentadas.

Nunca permitir que quality menor altere topologia persistida em Commit. Preview pode aproximar mais, mas materialização usa Authoring/Export contract apropriado.

# Bézier

## Cubic Bézier

~~~text
B(t) =
(1-t)³ P0
+ 3(1-t)²t P1
+ 3(1-t)t² P2
+ t³ P3
~~~

`t` varia de 0 a 1.

P0/P3 são anchors. P1/P2 são controls.

## De Casteljau

De Casteljau avalia/subdivide Bézier usando interpolações lineares sucessivas.

~~~text
Q0 = lerp(P0, P1, t)
Q1 = lerp(P1, P2, t)
Q2 = lerp(P2, P3, t)

R0 = lerp(Q0, Q1, t)
R1 = lerp(Q1, Q2, t)

B  = lerp(R0, R1, t)
~~~

`lerp(A,B,t) = A(1-t) + Bt`.

Para split, os mesmos pontos formam duas cubics:

~~~text
left  = P0, Q0, R0, B
right = B, R1, Q2, P3
~~~

A soma das duas representa exatamente a curva original.

Usar De Casteljau para split, node insertion e subdivisão adaptativa.

# Flattening

**Flattening** aproxima curva por segmentos de reta.

É Derived State.

## Critério de flatness

Para cubic, medir o afastamento dos controls em relação à chord P0→P3.

~~~text
controls suficientemente próximos da chord?
├── sim → aceitar segmento P0→P3
└── não → split em t=0.5 e repetir
~~~

A fórmula exata do bound pode usar implementação validada do backend, mas precisa garantir erro <= tolerância dentro do contrato definido.

## Guards

Flatten adaptativo precisa de:

- tolerância positiva/finita;
- limite de profundidade;
- handling de zero-length;
- saída deterministicamente ordenada.

Se atingir depth guard sem satisfazer tolerância, retornar estado degradado/erro em vez de loop infinito.

## Provenance de flatten

Quando Shape Builder/boolean precisar reconstruir source, cada edge derivada pode carregar:

~~~text
ObjectId
ContourId
source segment/node
t0
t1
~~~

Esse metadata é transitório.

# Bounds

## Line

Bounds = min/max dos endpoints.

## Cubic

Bounds exato considera endpoints e roots da derivada em X/Y dentro de 0..1.

A derivada de cubic é uma quadratic. Resolver suas roots permite encontrar extrema internas.

~~~text
B'(t).x = 0
B'(t).y = 0
~~~

Avaliar B(t) nos roots válidos e unir com P0/P3.

Não usar apenas control-point box como bounds final exato; ele é conservador, não mínimo.

## Visual bounds

Stroke/effects não pertencem ao Geometry bounds puro. Appearance/Render expandem conforme semântica.

# Arc length

Cubic geralmente não possui primitive elementar simples para comprimento.

Usar integração/aproximação adaptativa.

Direção:

1. estimar comprimento grosseiro;
2. subdividir;
3. comparar soma refinada;
4. aceitar quando diferença <= tolerance;
5. repetir.

Guardar lookup table derivada quando muitos queries usam distance→t.

## Distance-to-t

Dash, text-on-path e variable width precisam converter distância acumulada em parâmetro t.

Usar tabela monotônica de amostras + busca binária + refinamento local.

Não assumir `t = distance / total_length`.

# Nearest point

Objetivo:

~~~text
query point P
↓
nearest position on path
↓
(segment, t, point, distance)
~~~

Pipeline robusto:

1. broad phase por bounds;
2. subdividir cubics candidatas;
3. obter intervalos t promissores;
4. refinar localmente;
5. comparar distância final.

Refinamento pode usar iteração numérica sobre derivada da distância ao quadrado, com fallback para subdivisão quando não converge.

Nunca depender apenas de uma única estimativa Newton-like sem bracket/fallback.

# Intersections

## Line-line

Usar predicate robusto de orientação/cross product com tratamento explícito para:

- crossing;
- endpoint touch;
- parallel;
- collinear overlap.

Overlap não é “um único ponto”; precisa de resultado próprio.

## Cubic-cubic

Direção:

1. testar bounds;
2. subdividir curva com maior incerteza;
3. descartar pairs de bounds disjuntos;
4. quando ambos trechos estão suficientemente flat/small, intersectar chords;
5. refinar parâmetros t/u contra curvas originais;
6. deduplicar roots dentro de IntersectionTolerance.

Resultado:

~~~rust
pub struct CurveIntersection {
    pub t_a: f64,
    pub t_b: f64,
    pub point: Point,
    pub kind: IntersectionKind,
}
~~~

Kinds precisam distinguir cross/touch/overlap quando semanticamente possível.

## Tangential intersections

Quando curvas apenas se tocam, sinais podem não trocar de lado.

Algoritmo não pode depender somente de crossing sign.

Tangência exige derivadas/tolerância e deduplicação cuidadosa.

# Boolean

Boolean combina regiões preenchidas:

~~~text
Union
Intersect
Subtract
Xor
Divide
~~~

## Pipeline v0.1

`i_overlay` é backend topológico preferencial atrás de adapter.

Como o path nativo contém cubics e overlay trabalha sobre contornos poligonais, a avaliação usa:

~~~text
VectorPath
↓ adaptive flatten(BooleanTolerance)
polylines + provenance
↓ overlay/topology
polygon result
↓ cleanup/canonicalization
↓ optional curve reconstruction/refit
VectorPath result
~~~

Source nunca é alterada.

## FillRule

Adapter precisa respeitar NonZero/EvenOdd.

Orientação de contours pode ser normalizada internamente apenas se a conversão preservar a mesma região semântica.

## Canonicalização do resultado

Antes de materializar:

- remover edges numéricas zero-length dentro da tolerance;
- unir vértices equivalentes de forma controlada;
- ordenar contours deterministicamente;
- normalizar start point do contour apenas quando isso não afeta provenance/IDs;
- preservar fill semantics.

Não usar “cleanup” genérico que altera forma acima da tolerance.

## Curve reconstruction

Boolean live pode inicialmente retornar polygonal derived path se quality/tolerance permitir.

Para resultado autoral/Expand, preferir reconstrução:

1. usar provenance para recuperar trechos que correspondem a source cubics;
2. cortar source cubic por t0/t1 via De Casteljau;
3. para edges realmente novos/aproximados, usar curve fitting com max error explícito.

Assim evitamos transformar toda curva original em centenas de lines no Expand.

## Erros

Boolean retorna erro tipado para:

- non-finite input;
- tolerance inválida;
- topologia não resolvida;
- resource/adapter failure;
- guard de complexidade excedido.

Não retornar partial success como se fosse completo.

# Offset / Contour

Offset exato de uma Bézier geral não é, em geral, outra Bézier do mesmo grau.

Portanto offset é aproximação controlada.

## Pipeline

~~~text
source segment
↓ adaptive subdivision por curvatura/erro
samples + tangents
↓ deslocar pela normal
offset samples
↓ joins
↓ curve fit
↓ self-intersection cleanup
↓ result
~~~

### Normal

Para tangent T=(x,y), uma normal 2D possível é:

~~~text
N = normalize(-y, x)
~~~

O sinal da distância escolhe o lado.

## Joins

- Miter: interseção das tangentes/offset lines, limitado por miter limit;
- Bevel: conecta endpoints diretamente;
- Round: arco circular aproximado por Cubic dentro da tolerance.

## Self-intersection

Offsets de concavidades frequentemente se cruzam.

Depois de gerar candidate outline, usar overlay/topology cleanup para selecionar a boundary semanticamente correta.

Não “apagar loops pequenos” por área arbitrária sem parâmetro/política.

# Stroke expansion

Stroke expansion reutiliza offset semantics:

~~~text
center path
├→ offset +width/2
└→ offset -width/2
      ↓
joins + caps
      ↓
combine contours
      ↓
topology cleanup
~~~

Inside/Outside alteram as distâncias em closed paths.

Dash é aplicado por arc length **antes** da expansão de cada dash segment.

Variable width fornece distância local variável e exige sampling/refinement apropriado.

# Live Corners

Live Corners é Geometry Effect compartilhado.

## Line-line

Para duas edges que se encontram em vertex:

1. calcular angle;
2. limitar radius ao espaço disponível;
3. encontrar tangent points em ambas edges;
4. remover a parte próxima do corner;
5. inserir arco/cubic round ou corner style escolhido.

Radius autoral pode exceder o máximo geométrico. O evaluator pode usar effective radius menor sem reescrever o parâmetro, permitindo que corner volte a crescer se o shape mudar.

## Curved neighbors

Para curve-line/curve-curve, tangent points exigem solve por arc length/geometria local.

Suporte pode entrar incrementalmente; o contrato do effect permanece o mesmo.

Não duplicar algoritmo em Rectangle, Polygon e Star.

# Simplificação

“Simplify” não é uma única operação.

## Merge by Distance

Une vertices distintos apenas quando o usuário/Command pede cleanup e a distância <= tolerance.

Não executar implicitamente em save.

## Collinear cleanup

Para polyline, remover vertex intermediário quando sua distância à line dos vizinhos <= tolerance e a mudança de direção não representa corner protegido.

## Ramer-Douglas-Peucker

RDP simplifica **polyline** com limite de erro perpendicular.

~~~text
A ........ X .... B
\______________/

X = maior distância à chord AB
~~~

Se max distance <= tolerance, todos os intermediários podem ser removidos.

Caso contrário, dividir em X e repetir.

Decisão v0.1:

> RDP é o simplificador padrão de amostras/polylines derivadas quando precisamos de erro geométrico explícito.

Visvalingam-Whyatt não entra inicialmente; adicioná-lo só se benchmark visual demonstrar vantagem concreta.

## Curve-native simplify

Para VectorPath cubic já autoral, não flatten + RDP + re-fit indiscriminadamente.

Usar estratégia local:

1. escolher node candidato;
2. tentar substituir segmentos adjacentes por uma cubic;
3. medir max deviation contra trecho original;
4. preservar corners protegidos;
5. aceitar apenas se erro <= tolerance.

Isso preserva melhor intenção e reduz drift.

# Curve fitting

Para converter amostras em cubics, usar como base o método clássico de fitting iterativo associado a Philip J. Schneider/Graphics Gems.

## Ideia

1. receber pontos ordenados;
2. estimar tangentes inicial/final;
3. atribuir parâmetros iniciais por chord length;
4. resolver controls da cubic que melhor se ajustam;
5. medir ponto de maior erro;
6. se erro <= tolerance, aceitar;
7. tentar reparameterization;
8. se ainda falhar, dividir no maior erro e repetir.

## Chord-length parameterization

Cada sample recebe t proporcional à distância acumulada entre samples.

~~~text
u_i =
distance acumulada até i
/
distance total
~~~

É estimativa inicial, não verdade final.

## Reparameterization

Ajustamos os t dos samples para que correspondam melhor à cubic atual.

Pode usar iteração Newton-Raphson: método numérico que aproxima root usando valor e derivada local.

Se a iteração sair do intervalo ou não melhorar, manter/fallback para o parâmetro anterior.

## Corner detection

Antes de fitting suave, detectar mudanças de direção acima de threshold/tolerance.

Corner verdadeiro vira boundary de fit; não arredondar só para reduzir node count.

# Shape Builder

Shape Builder usa a mesma topologia do boolean, mas expõe **faces locais** das sobreposições.

É uma feature central e precisa preservar curvas/aparência melhor que “flatten e esquecer”.

## Arrangement

**Arrangement** é a subdivisão do plano criada por todas as edges/intersections.

~~~text
input boundaries
↓
intersections
↓
split edges
↓
vertices + half-edges
↓
faces
~~~

## Half-edge graph

Usar uma estrutura transitória inspirada em **DCEL — Doubly Connected Edge List**.

Cada edge geométrica possui duas half-edges em direções opostas.

~~~text
A -------- B
A→B      B→A
~~~

Cada half-edge conhece conceitualmente:

- origin vertex;
- twin;
- next around face;
- source provenance.

Isso permite caminhar boundaries de cada face sem persistir uma malha no Document.

## Pipeline

1. avaliar shapes/paths selecionados;
2. flatten com ShapeBuilderTolerance carregando provenance;
3. detectar/split intersections;
4. canonicalizar vertices coincidentes;
5. construir half-edges;
6. ordenar outgoing edges por angle;
7. ligar next edges para formar faces;
8. classificar faces contra FillRule dos sources;
9. construir spatial lookup de faces para hover;
10. aplicar escolha union/subtract;
11. extrair boundary resultante;
12. reconstruir trechos de curva a partir da provenance;
13. gerar Transaction somente no commit.

## Face walking

Em cada vertex, outgoing half-edges são ordenadas por direção angular.

Para caminhar uma face, escolhemos consistentemente a próxima edge que mantém a face de um lado definido.

A convenção clockwise/counter-clockwise precisa ser única e testada.

## Face classification

Escolher um ponto interior seguro da face e avaliar membership nos source fills.

Não usar simplesmente o centroid aritmético se ele puder cair fora de face côncava.

Pode usar ponto derivado de edge + pequeno deslocamento para dentro, validado pelo próprio face polygon.

## Provenance

Cada derived edge registra origem:

~~~text
ObjectId
ContourId
source segment
source t interval
appearance source
~~~

Quando boundary final segue uma curva source, reconstruir o slice da cubic original via De Casteljau em vez de curve fitting.

Isso é crucial para preservar precisão.

## Aparência

Quando todas as edges de uma face vêm de uma única source, aparência dessa source é candidata natural.

Quando sources diferentes contribuem, política precisa ser determinística e o Command precisa escolher herança explícita.

Engine produz provenance; Tool/UI futura decide exposição da política.

## Complexidade

Arrangement pode crescer aproximadamente com o número de intersections.

Aplicar guards:

- número máximo de input segments por operação interativa;
- cancellation;
- backpressure de preview;
- fallback de qualidade preview;
- diagnóstico em vez de freeze.

Não impor limite pequeno arbitrário ao Document; o guard é operacional.

# Warp

Warp é domínio de Geometry, mas v0.1 mantém contrato estreito.

Affine transform continua Transform2D do Core.

Envelope/mesh warp avançado fica futuro e deve ser operation live, não reescrita silenciosa de nodes.

# Robustez

Todos os algoritmos:

- rejeitam NaN/Inf;
- recebem tolerance explícita;
- possuem guards de recursion/complexity;
- retornam erro tipado;
- não mutam Document;
- não usam UI state;
- produzem ordem determinística quando resultado vira autoral.

## Invariantes

1. Geometry opera sobre tipos Petunia via adapters.
2. De Casteljau é base de split/subdivision.
3. Flatten é derivado e bounded por tolerance.
4. Bounds cubic considera extrema.
5. Boolean usa overlay robusto com aproximação explicitamente controlada.
6. Offset/stroke expansion são aproximações com erro documentado.
7. RDP é usado em polyline; curve-native simplify preserva cubics.
8. Curve fitting usa erro máximo explícito e corner detection.
9. Shape Builder usa arrangement/half-edge graph transitório.
10. Provenance é preservada o suficiente para reconstruir source curves quando possível.
11. Nenhum cleanup altera Source sem Command.
12. Resultado parcial nunca é apresentado como sucesso total.
