# Pipeline e Render Graph

Renderer transforma um snapshot avaliado em pixels. Ele não decide regras autorais.

## Entrada

Evoluir `RenderBackend::render_scene(&SceneGraph)` para algo próximo de:

```rust
pub trait RenderBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<RenderStats>;
}

pub struct RenderFrame {
    pub scene: RenderSnapshot,
    pub view: ViewTransform,
    pub target: RenderTarget,
    pub options: RenderOptions,
}
```

`RenderSnapshot` já contém dados derivados necessários ou handles imutáveis.

## Render graph

Passes explícitos permitem:
- offscreen surfaces
- masks
- live filters
- isolated groups
- compositing
- overlays
- color output.

```text
prepare
→ cull
→ evaluate paint primitives
→ allocate intermediates
→ vector/raster/text passes
→ effects
→ composite
→ output transform
→ editor overlays
→ present
```

## Culling

Usar visual bounds no espaço de view. Objetos totalmente fora do viewport não geram draw work, salvo dependências de filtros/clip.

## Tessellation

É cache derivado. Key inclui path revision, stroke/fill parameters e tolerância/scale quando necessário.

## CPU reference versus GPU

Manter `SoftwareReferenceRenderer` como caminho determinístico de testes. GPU backend otimiza, mas sem alterar semântica.

## Device pixel ratio

Document coordinates → view → device. DPR só entra no limite de output/quality e nunca modifica documento.

## Quality levels

Interactive preview pode usar tolerância mais relaxada; export usa qualidade alta. Diferença não pode alterar topologia ou significado, apenas aproximação visual permitida.
