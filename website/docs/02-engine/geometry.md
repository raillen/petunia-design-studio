# Geometry Engine

Geometry transforma e consulta geometria; não possui widgets nem estado de seleção.

## Biblioteca base

O workspace já possui `kurbo` e `i_overlay`. Kurbo fornece tipos e operações para curvas Bézier; iOverlay fornece operações booleanas robustas sobre contornos.

Essas bibliotecas ficam atrás de APIs Petunia. O documento não deve depender dos tipos públicos de uma biblioteca substituível.

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
├── simplify.rs
├── stroke_expand.rs
├── curve_fit.rs
├── warp.rs
└── shape_builder.rs
~~~

## Bézier

Uma **curva Bézier** é uma curva definida por pontos de controle. Ela não passa necessariamente por todos esses pontos; os controles internos determinam a direção e a curvatura.

Para uma Bézier cúbica:

- `P0` — ponto inicial;
- `P1` — primeiro controle;
- `P2` — segundo controle;
- `P3` — ponto final;
- `t` — posição ao longo da curva, de `0` a `1`.

~~~text
B(t) = (1-t)³P0
     + 3(1-t)²tP1
     + 3(1-t)t²P2
     + t³P3
~~~

Quando `t = 0`, o resultado é `P0`. Quando `t = 1`, é `P3`.

### De Casteljau

**De Casteljau** é um algoritmo para calcular um ponto de uma curva Bézier por sucessivas interpolações lineares.

Uma **interpolação linear** entre dois pontos `A` e `B` significa caminhar uma fração `t` da distância entre eles:

~~~text
lerp(A, B, t) = A × (1 - t) + B × t
~~~

Para uma curva cúbica, calculamos:

~~~text
Q0 = lerp(P0, P1, t)
Q1 = lerp(P1, P2, t)
Q2 = lerp(P2, P3, t)

R0 = lerp(Q0, Q1, t)
R1 = lerp(Q1, Q2, t)

B  = lerp(R0, R1, t)
~~~

Visualmente:

~~~text
P0 ───── P1 ───── P2 ───── P3
 \       / \       /
  Q0 ─── Q1 ─── Q2
    \    / \    /
      R0 ─── R1
         \ /
          B
~~~

O ponto `B` é o ponto da curva em `t`.

#### Por que usar De Casteljau

Ele é especialmente útil no Petunia porque:

- é numericamente estável;
- funciona para Bézier de qualquer grau;
- permite subdividir uma curva exatamente em duas curvas menores;
- serve de base para flattening adaptativo, hit-testing e vários cálculos geométricos.

Para subdividir, os pontos intermediários calculados por De Casteljau também formam os controles das duas curvas resultantes. Portanto não precisamos aproximar a subdivisão.

## Flattening

**Flattening** converte temporariamente uma curva em vários segmentos de reta.

Isso é necessário porque alguns algoritmos trabalham melhor com linhas do que com curvas contínuas.

O Petunia deve usar flattening **adaptativo**:

1. observar um trecho da curva;
2. medir o quanto ele se afasta de uma reta equivalente;
3. se o erro estiver dentro da tolerância, usar uma reta;
4. se estiver fora, subdividir a curva com De Casteljau;
5. repetir nas duas metades.

~~~text
curva
  ↓
erro <= tolerância? ── sim ─→ segmento
  │
  não
  ↓
subdivide
 ↙       ↘
repete   repete
~~~

A tolerância depende do uso: preview, boolean, hit-test ou export.

O flatten é sempre **derivado**. Ele nunca substitui a curva autoral.

## Bounds

**Bounds** é o menor retângulo que contém uma geometria.

Para uma Bézier, usar apenas os anchors pode produzir um bounds incorreto porque a curva pode atingir máximos ou mínimos entre as extremidades.

O cálculo correto precisa analisar onde a derivada da curva é zero em X ou Y. Esses pontos são chamados de **extrema internas**.

Em termos simples:

> procuramos onde a curva para de crescer e começa a diminuir — ou o contrário — em cada eixo.

Esses pontos, junto com as extremidades, definem o bounds geométrico exato.

## Boolean

Operações booleanas combinam áreas preenchidas.

~~~rust
pub enum BooleanOp {
    Union,
    Intersect,
    Subtract,
    Xor,
    Divide,
}
~~~

- **Union** — mantém tudo que pertence a qualquer forma.
- **Intersect** — mantém apenas a área comum.
- **Subtract** — remove a área de uma forma usando outra.
- **Xor** — mantém áreas que pertencem a apenas uma forma.
- **Divide** — separa as regiões criadas pelas interseções.

Live Boolean mantém as formas originais e apenas avalia o resultado. **Expand Boolean** materializa paths novos.

## Offset / Contour

**Offset** cria uma curva paralela a uma distância definida da curva original.

Em cantos, o algoritmo precisa decidir como unir os lados:

- `round` — arco arredondado;
- `bevel` — corte reto;
- `miter` — prolonga as bordas até elas se encontrarem.

Um **miter limit** impede que cantos muito agudos criem pontas extremamente longas.

Offsets podem criar auto-interseções. Por isso o resultado precisa passar por limpeza topológica antes de ser aceito como geometria final.

## Simplificação

“Simplificar” não é um único algoritmo.

### Merge de pontos próximos

Une pontos cuja distância está abaixo de uma tolerância definida. É útil depois de import, tracing ou operações que geram vértices quase coincidentes.

### Remoção de pontos colineares

Três pontos são **colineares** quando estão praticamente na mesma linha. Se o ponto central não altera a forma dentro da tolerância, ele pode ser removido.

### Ramer–Douglas–Peucker — RDP

**RDP** simplifica uma polyline preservando os pontos que mais alteram sua forma.

Passos:

1. ligar o primeiro e o último ponto por uma reta;
2. encontrar o ponto intermediário mais distante dessa reta;
3. se essa distância for menor que a tolerância, remover todos os pontos intermediários;
4. caso contrário, manter o ponto mais distante e repetir o processo dos dois lados.

~~~text
A · · · X · · B
\_____________/

X é o ponto que mais se afasta da reta A–B.
~~~

Quanto maior a tolerância, menos pontos permanecem.

RDP trabalha originalmente com polylines. Em curvas autorais ele deve atuar sobre amostras ou contornos derivados, não destruir handles diretamente.

### Visvalingam–Whyatt

**Visvalingam–Whyatt** mede a importância de cada ponto pela área do triângulo formado com seus dois vizinhos.

~~~text
A
|\
| \
B--C
~~~

Quanto menor a área do triângulo `A-B-C`, menor a contribuição de `B` para a forma. O algoritmo remove progressivamente os pontos de menor área e recalcula os vizinhos.

Ele costuma produzir uma simplificação visual mais gradual que RDP.

### Decisão ainda aberta

RDP e Visvalingam são candidatos, não uma escolha fechada.

A decisão deve ser tomada separadamente para polyline importada, contour de Image Trace, Pencil e cleanup pós-boolean.

O critério principal deve ser erro visual máximo e preservação de cantos, não quantidade arbitrária de pontos.

## Curve fitting

**Curve fitting** transforma uma sequência de pontos amostrados em uma quantidade menor de curvas Bézier.

~~~text
amostras do Pencil
· · · · · · · · · ·
        ↓
   Bézier cúbica
~~~

O algoritmo tenta encontrar controles `P1` e `P2` que mantenham a curva dentro de um erro máximo em relação às amostras.

Quando uma única curva não consegue respeitar a tolerância:

1. localizar a região de maior erro;
2. dividir as amostras;
3. ajustar curvas menores;
4. repetir até o erro ficar aceitável.

**Corner detection** identifica mudanças bruscas de direção. Um canto real não deve virar uma curva suave apenas para reduzir a quantidade de segmentos.

## Shape Builder

Shape Builder precisa descobrir as regiões fechadas produzidas por várias formas sobrepostas.

### Planar subdivision

**Planar subdivision** significa dividir a geometria exatamente nos pontos onde segmentos se cruzam, formando uma rede de arestas e regiões.

~~~text
formas sobrepostas
      ↓
encontrar interseções
      ↓
quebrar contornos nesses pontos
      ↓
identificar regiões fechadas
~~~

Pipeline:

1. avaliar os contornos participantes;
2. detectar interseções;
3. dividir os segmentos nas interseções;
4. construir as regiões fechadas;
5. fazer hit-test para descobrir em qual região o ponteiro está;
6. marcar regiões para union ou subtract;
7. reconstruir os paths resultantes.

### Provenance

**Provenance** registra de qual objeto original cada trecho resultante veio.

Isso permite preservar atributos quando possível:

~~~text
segmento resultante
→ veio do objeto A
→ pode herdar aparência compatível
~~~

Preview de região é transitório. Somente o commit gera Command.

## Robustez

Todo algoritmo geométrico recebe tolerância explícita e retorna erro tipado quando não consegue produzir resultado válido.

Não aceitar silenciosamente NaN, infinito, loops de topologia inválida ou resultado parcial apresentado como sucesso.
