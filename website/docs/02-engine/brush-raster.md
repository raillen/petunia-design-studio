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

A política concreta precisa equilibrar:

- suavidade;
- latência;
- previsibilidade.

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
