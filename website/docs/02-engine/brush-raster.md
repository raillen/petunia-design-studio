# Brush + Raster Engine

Pintura precisa ser responsiva sem transformar a UI em algoritmo.

## Pipeline de brush

```text
raw pointer samples
→ timestamp/pressure/tilt normalization
→ stabilizer
→ resampling by arc length
→ dynamics
→ dab generation
→ tile compositor
→ dirty tile set
```

## Sample

```rust
pub struct StrokeSample {
    pub position: Point,
    pub pressure: f32,
    pub tilt: Vec2f,
    pub rotation: f32,
    pub time: f64,
}
```

UI apenas coleta dados que o dispositivo fornece.

## BrushPreset

Persistir parâmetros, não estado interno do algoritmo:
- size
- hardness
- spacing
- opacity/flow
- pressure curves
- angle/rotation
- scatter
- texture/nozzle
- blend mode
- smoothing/stabilizer policy.

## Flow versus opacity

Definir semânticas separadas. Opacity limita contribuição final do stroke; flow controla contribuição acumulada por dab. Misturar os dois torna brushes imprevisíveis.

## Tiles

Pixel Engine opera por tiles. Um brush stroke marca conjunto de tiles dirty; Render atualiza somente eles.

## Undo raster

No início de stroke, guardar referências/version das tiles afetadas. Copy-on-write cria novas tiles quando modificadas. Undo troca referências, em vez de copiar canvas inteiro.

## Filtros raster

Contrato recomendado:

```rust
pub trait ImageFilter {
    fn input_region(&self, output: RectI, params: &FilterParams) -> RectI;
    fn evaluate(&self, ctx: &FilterContext, input: &Surface, output: &mut Surface) -> Result<()>;
}
```

CPU e GPU podem ter implementações diferentes sob a mesma semântica.

## Seleções

Selection mask é dado transitório/editorial até ser convertida em Mask persistente. Operações Grow/Shrink/Feather ficam no Engine.

## Liquify e warp

Devem preferir mapa/deformação não destrutiva quando aplicados como live effect. “Apply” materializa.

## Determinismo

Software/reference path precisa gerar resultado estável para golden tests, mesmo se GPU usar aproximações otimizadas.
