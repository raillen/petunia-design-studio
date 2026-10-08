# Brush + Raster Engine

Pintura precisa ser responsiva sem transformar a UI em algoritmo.

## Pipeline de brush

~~~text
raw pointer samples
→ timestamp/pressure/tilt normalization
→ stabilizer
→ resampling by arc length
→ dynamics
→ dab generation
→ tile compositor
→ dirty tile set
~~~

Cada etapa resolve um problema diferente.

## Raw pointer samples

A UI entrega amostras de entrada:

~~~rust
pub struct StrokeSample {
    pub position: Point,
    pub pressure: f32,
    pub tilt: Vec2f,
    pub rotation: f32,
    pub time: f64,
}
~~~

Esses valores vêm de mouse, caneta ou outro dispositivo.

A UI coleta e normaliza apenas o formato do evento. Ela não decide a forma do pincel.

## Stabilizer

A v0.1 terá dois modos explícitos:

~~~text
Off
OneEuro
~~~

`OneEuro` usa o **One Euro Filter**, um filtro adaptativo para sinais interativos.

A ideia é:

- em movimento lento, aplica mais suavização;
- em movimento rápido, aumenta a resposta para reduzir atraso percebido.

Ele combina um filtro passa-baixa com cutoff dinâmico calculado a partir da velocidade estimada do sinal. Assim evita o compromisso fixo “linha suave porém atrasada” de uma média móvel simples.

O preset persiste parâmetros semânticos do filtro, não buffers internos.

Um **stabilizer** suaviza pequenas variações do movimento do ponteiro antes de gerar o stroke.

A ideia básica é introduzir uma pequena inércia ou janela de suavização:

~~~text
movimento bruto
· ·  ·   · ·
     ↓
trajetória estabilizada
────────────
~~~

Quanto maior a estabilização, mais suave a linha e maior a sensação de atraso.

Parâmetros principais do One Euro são:

~~~text
min_cutoff → suavização em baixa velocidade
beta       → quanto o cutoff cresce com velocidade
d_cutoff   → suavização da derivada/velocidade
~~~

Todos precisam ser finitos e validados.

A política concreta equilibra suavidade, latência e previsibilidade. Nenhum valor padrão vira contrato arquitetural antes de teste com mouse/caneta reais.

## Resampling by arc length

Eventos de ponteiro chegam em intervalos de tempo irregulares. Se cada evento virasse um dab, a densidade do pincel dependeria da velocidade do computador e do dispositivo.

**Resampling by arc length** cria novas amostras espaçadas pela distância percorrida ao longo do stroke, não pelo tempo.

~~~text
eventos recebidos:
A----B-C--------D

reamostrado por distância:
A--·--·--·--·--D
~~~

**Arc length** é o comprimento acumulado do caminho.

Essa reamostragem torna spacing e textura mais consistentes.

## Dynamics

**Brush dynamics** transforma propriedades do input em propriedades do pincel.

Exemplos:

~~~text
pressure → size
pressure → opacity
tilt     → angle
speed    → scatter
~~~

Essas relações são configuráveis no preset e avaliadas pelo Engine.

## Dab

Um **dab** é uma única impressão do pincel na superfície.

Um stroke normalmente contém muitos dabs:

~~~text
● ● ● ● ● ● ●
      ↓
parecem um traço contínuo
~~~

Cada dab pode variar:

- posição;
- tamanho;
- opacity;
- angle;
- texture;
- color;
- scatter.

## BrushPreset

Persistir parâmetros, não estado interno do algoritmo:

- size;
- hardness;
- spacing;
- opacity;
- flow;
- pressure curves;
- angle/rotation;
- scatter;
- texture/nozzle;
- blend mode;
- smoothing/stabilizer policy.

## Flow versus opacity

**Opacity** limita quanto o stroke final pode cobrir o conteúdo abaixo.

**Flow** controla quanto cada dab deposita antes do limite final.

Exemplo conceitual:

~~~text
Opacity 100%, Flow 20%
dab 1 → pouca tinta
dab 2 → acumula
dab 3 → acumula
...
~~~

Misturar os dois conceitos torna presets imprevisíveis.

## Tiles

Uma superfície raster grande é dividida em **tiles**, pequenos blocos independentes de pixels.

~~~text
canvas
┌───┬───┬───┐
│ A │ B │ C │
├───┼───┼───┤
│ D │ E │ F │
└───┴───┴───┘
~~~

Um stroke que toca apenas B e E não precisa copiar/processar A, C, D e F.

## Dirty tiles

Uma **dirty tile** é uma tile que mudou e precisa ser recalculada ou reenviada ao renderer.

~~~text
stroke
 ↓
tiles B e E mudaram
 ↓
dirty = {B, E}
~~~

Isso reduz trabalho por frame.

## Copy-on-write

**Copy-on-write — COW** permite compartilhar dados enquanto ninguém os modifica.

Exemplo:

~~~text
Snapshot antigo ─┐
                 ├→ Tile A
Snapshot novo  ──┘
~~~

Enquanto Tile A não muda, ambos apontam para o mesmo conteúdo.

Quando o novo estado precisa editar a tile:

~~~text
Snapshot antigo → Tile A original
Snapshot novo   → cópia de Tile A → modifica
~~~

A cópia só acontece **na escrita**.

Isso é especialmente útil para undo raster e snapshots porque evita duplicar imagens inteiras.

## Undo raster

No início do stroke:

1. identificar tiles que poderão mudar;
2. preservar referências/versões antigas;
3. modificar cópias somente quando necessário;
4. registrar a troca de referências na Transaction.

Undo restaura as versões anteriores sem copiar o canvas inteiro.

## Filtros raster

Contrato recomendado:

~~~rust
pub trait ImageFilter {
    fn input_region(
        &self,
        output: RectI,
        params: &FilterParams,
    ) -> RectI;

    fn evaluate(
        &self,
        ctx: &FilterContext,
        input: &Surface,
        output: &mut Surface,
    ) -> Result<()>;
}
~~~

`input_region` responde qual área de entrada é necessária para produzir uma área de saída.

Isso permite processamento por ROI em filtros como blur e shadow.

CPU e GPU podem ter implementações diferentes, mas devem obedecer à mesma semântica.

## Seleções

Uma **selection mask** é uma superfície de cobertura que indica onde uma edição raster deve atuar.

~~~text
0.0 → fora da seleção
1.0 → completamente selecionado
0.5 → borda parcialmente selecionada
~~~

Enquanto for apenas seleção atual, ela é estado de sessão.

Quando o usuário converte a seleção em uma Mask do documento, ela se torna persistente.

## Grow, Shrink e Feather

- **Grow** expande a região selecionada;
- **Shrink** contrai;
- **Feather** suaviza a transição da borda.

Feather normalmente produz coverage gradual em vez de uma borda binária dura.

## Liquify e warp

**Warp** desloca posições segundo um campo/deformação.

**Liquify** aplica deformações locais interativas, como empurrar ou expandir regiões.

Quando usados como live effect, o mapa de deformação e seus parâmetros permanecem editáveis.

`Apply` ou `Bake` materializa o resultado.

## Determinismo

Uma operação é **determinística** quando a mesma entrada e os mesmos parâmetros produzem a mesma saída.

O software/reference path deve ser determinístico para testes.

Uma implementação GPU pode usar otimizações diferentes, mas deve permanecer dentro das tolerâncias documentadas.


## Interpolação de samples

Resampling cria samples novos entre eventos reais.

Posição usa interpolação linear ao longo do segmento local. Pressure, tilt e rotation também precisam ser interpolados de forma contínua, com tratamento correto para ângulos que cruzam 360°/2π.

Timestamp intermediário é derivado proporcionalmente.

Não copiar simplesmente o último pressure para todos os dabs: isso gera degraus visíveis em dispositivos com baixa frequência de eventos.

## Spacing

Spacing é definido como distância relativa ao diâmetro efetivo do dab:

~~~text
spacing_distance =
effective_diameter × spacing_ratio
~~~

Isso mantém densidade aproximadamente consistente ao mudar brush size.

Presets podem futuramente oferecer spacing absoluto, mas a semântica precisa ser explícita; não misturar as duas unidades no mesmo campo.

## Randomness

Scatter, rotation jitter, size jitter e texture jitter usam gerador pseudoaleatório determinístico com seed explícita por stroke/operação.

~~~text
stroke seed
+ dab index
+ dynamics parameters
→ deterministic jitter
~~~

A implementação do PRNG precisa ser versionada se seu output for necessário para replay/recovery. Se o histórico raster guarda tiles materializados, Undo não depende de rerodar o PRNG; preview e commit ainda precisam compartilhar a mesma seed.

Não usar RNG global ou relógio em cada dab.

## Dab compositor

Cada dab é avaliado em espaço local e composto somente nos tiles intersectados.

Pipeline:

~~~text
DabSpec
↓ bounds
candidate tiles
↓
coverage/nozzle evaluation
↓
blend in working raster space
↓
modified tile versions
~~~

Dabs que não intersectam um tile nunca devem tocar sua memória.

## Hardness

Para brush redondo básico, hardness controla o tamanho do núcleo opaco/forte antes do falloff.

A curva exata precisa ser única entre software renderer e preview. A v0.1 usa falloff monotônico e contínuo; presets de textura podem fornecer coverage próprio.

Hardness não é sinônimo de opacity.

## Blend semântico

Brush blend usa o mesmo vocabulário de BlendMode do compositor, mas a implementação raster deve obedecer ao mesmo contrato de alpha/color space.

Não criar uma matemática de Multiply exclusiva do brush.

## Selection mask no brush

Coverage final do dab é multiplicada pela selection mask quando existe:

~~~text
dab coverage
× selection coverage
→ effective coverage
~~~

Essa operação acontece antes do composite no destination tile.

Selection mask continua Session State enquanto não for materializada como Mask documental.

## Grow / Shrink

A v0.1 implementa Grow/Shrink de selection mask com **distance transform**.

**Distance transform** calcula, para cada pixel, a distância até a fronteira/região oposta.

~~~text
binary/coverage mask
↓ distance field
↓ threshold at ±radius
grown/shrunk mask
~~~

Isso é mais previsível para raios grandes do que aplicar dilatação/erosão unitária repetidamente.

Bordas e coverage precisam ser definidas em pixel space da selection surface.

## Feather

Feather usa blur Gaussiano da coverage mask com sigma derivado do radius definido pelo contrato.

O mesmo kernel/semântica do Gaussian Blur deve ser reutilizado; não criar um blur especial incompatível só para seleção.

## Filter edge modes

Filtros que consultam pixels fora da surface precisam declarar edge behavior:

~~~text
Transparent
Clamp
Repeat
Mirror
~~~

O default de efeitos documentais é parte do Effect contract. Nunca deixar backend CPU/GPU escolher comportamentos diferentes.

## Liquify

Para live Liquify, a representação autoral é um **displacement field** ou operação equivalente versionada.

**Displacement field** armazena um vetor de deslocamento por região/amostra:

~~~text
position
↓
(dx, dy)
↓
sample source at displaced coordinate
~~~

Brush interativo atualiza o campo transitório; commit grava a operação/field quando o modo é live.

Modo destrutivo é Command separado que materializa pixels.

A resolução/representação exata do field será escolhida por profiling; não embutir uma grade gigante fixa no SceneNode.

## Raster transaction

Um stroke raster commitado precisa ser uma única Transaction.

Direção:

~~~text
begin stroke
↓ capture original tile refs lazily
preview/edit COW tiles
↓ pointer up
commit tile-version replacements
↓
1 HistoryEntry
~~~

Cancel descarta versões transitórias e restaura references sem reprocessar o stroke.

## Memory guards

Operações raster validam:

- surface dimensions;
- tile count;
- bytes por tile;
- temporary ROI size;
- blur/filter expansion;
- maximum allocation por job.

Erro de allocation/guard é tipado e não deixa PixelSurface parcialmente atualizada.

## Invariantes

1. UI fornece samples; Engine define stroke.
2. One Euro é o stabilizer padrão disponível na v0.1; Off sempre existe.
3. Resampling é por arc length e interpola dynamics continuamente.
4. Spacing relativo usa diâmetro efetivo.
5. Jitter usa seed determinística por stroke/operação.
6. Brush altera somente tiles intersectados.
7. Undo raster troca versões/refs de tiles, não copia canvas inteiro.
8. Selection mask modula coverage e continua Session State até materialização.
9. Grow/Shrink usam distance transform; Feather reutiliza Gaussian semantics.
10. Filtros declaram edge mode e ROI.
11. Live Liquify persiste deformação, não pixels já deformados.
12. Stroke commitado corresponde a uma Transaction.
13. CPU/GPU obedecem à mesma matemática de blend/filter.
