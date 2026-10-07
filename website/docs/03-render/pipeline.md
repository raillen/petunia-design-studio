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

Um **render graph** descreve as etapas de renderização como uma sequência ou grafo de passes e dependências.

Um **pass** é uma etapa que lê recursos e produz outros recursos, como “rasterizar vetores” ou “compor máscara”.

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

**Culling** é descartar trabalho que não pode afetar a imagem final.

Se o bounds visual de um objeto está totalmente fora do viewport, o renderer normalmente não precisa desenhá-lo.

~~~text
viewport
┌───────────┐
│ objeto A  │       objeto B
└───────────┘

A → renderiza
B → pode ser descartado
~~~

Filtros, shadows ou clips podem expandir a região de influência; por isso usamos **visual bounds**, não apenas bounds geométrico.

## Tessellation

**Tessellation** converte geometria vetorial em primitivas simples, normalmente triângulos, que a GPU ou renderer raster consegue processar eficientemente.

É cache derivado. A chave inclui revisão do path, parâmetros de stroke/fill e tolerância/scale quando necessário.

## CPU reference versus GPU

**CPU** executa código geral nos núcleos do processador. **GPU** executa grandes quantidades de operações gráficas/paralelas.

Manter `SoftwareReferenceRenderer` como caminho determinístico de testes: uma implementação simples e previsível usada como referência de correção.

O backend GPU pode otimizar agressivamente, mas não deve alterar a semântica visual além das tolerâncias documentadas.

## Device pixel ratio

**DPR — Device Pixel Ratio** é a relação entre uma unidade lógica de interface e pixels físicos da tela.

Em uma tela HiDPI, DPR 2 significa que uma unidade lógica pode ocupar 2×2 pixels físicos.

~~~text
Document → View → Device Pixels
~~~

DPR afeta qualidade e tamanho do target de render, mas nunca modifica geometria persistente do documento.

## Quality levels

Interactive preview pode usar tolerância mais relaxada; export usa qualidade alta. Diferença não pode alterar topologia ou significado, apenas aproximação visual permitida.
