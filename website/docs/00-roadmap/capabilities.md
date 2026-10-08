# Roadmap de capacidades

O roadmap registra **capacidades que queremos preservar no horizonte do produto**. Ele não substitui as especificações técnicas e não transforma uma ideia em compromisso de release automaticamente.

A regra é:

> **a feature entra no roadmap agora; a implementação só é fechada quando o domínio arquitetural que a sustenta estiver maduro.**

Isso evita desenhar UI ou APIs sobre fundações ainda instáveis.

## Como ler este roadmap

| Estado | Significado |
|---|---|
| **Planejado** | capacidade aprovada para o produto; contrato técnico ainda não está fechado |
| **Especificado (motor)** | Core/Engine/Render e invariantes estão definidos; implementação e experiência de ferramenta/GUI ainda podem faltar |
| **Pós-v0.1.0-stable** | não entra antes da primeira base estável |
| **Exploração futura** | ideia preservada, mas sem compromisso de implementação |
| **Fora de escopo atual** | deliberadamente não faz parte do produto neste ciclo |

## Capacidades planejadas

As capacidades abaixo já tiveram o **contrato de motor** amadurecido. Isso não significa que a ferramenta, a interação ou a GUI estejam fechadas; essas camadas serão discutidas em conjunto.

| Capacidade | Estado técnico | Custo esperado | Contrato |
|---|---|---:|---|
| Live Effects / Effect Stack | **Especificado (motor)** | médio | Non-destructive · Appearance/Effects · Render |
| Image Trace / Live Trace | **Especificado (motor)** | baixo–médio | Generated Content · Vectorization · Geometry |
| Simplify / Smooth / Cleanup | **Especificado (motor)** | baixo–médio | Path · Geometry |
| Crop não destrutivo | **Especificado (motor)** | baixo | Crop/Clipping · Scene |
| Linked Images + Relink | **Especificado (motor)** | baixo | Resources · Commands · I/O |
| Palette Extraction | **Especificado (motor)** | baixo | Image Analysis · Color |
| Remove White / Color to Alpha | **Especificado (motor)** | baixo | Adjustments |
| Invert / Grayscale / Posterize | **Especificado (motor)** | baixo | Adjustments |
| Brightness / Contrast / Levels | **Especificado (motor)** | baixo | Adjustments |
| HSL / Vibrance | **Especificado (motor)** | baixo–médio | Adjustments · Color |
| Blend Modes | **Especificado (motor)** | baixo–médio | Compositor |
| Drop Shadow / Glow | **Especificado (motor)** | médio | Appearance/Effects · Compositor |
| Live Corners | **Especificado (motor)** | baixo–médio | Geometry Effects |
| Live Offset / Contour | **Especificado (motor)** | médio | Geometry Effects |
| Pattern Fill | **Especificado (motor)** | baixo–médio | Appearance · Paint Evaluation |
| Conical Gradient | **Especificado (motor)** | baixo | Color · Paint Evaluation |
| Clipping / Opacity Masks | **Especificado (motor)** | médio | Scene · Crop/Clipping · Compositor |
| Export Slices | **Especificado (motor)** | baixo–médio | Guides/Grids/Slices · ExportPlan |
| QR Code / Barcode Generator | **Especificado (motor)** | baixo | Generated Content · Vector Generators |
| Cartesian Grid | **Especificado (motor)** | baixo | AffineGrid · Spatial |
| Isometric Grid | **Especificado (motor)** | baixo | AffineGrid · Spatial |
| Axonometric Grid | **Especificado (motor)** | baixo | AffineGrid · Spatial |
| Pixel Grid | **Especificado (motor)** | baixo | View-derived Grid · Spatial |
| Baseline Grid | **Especificado (motor)** | baixo | BaselineGrid · Text/Layout · Spatial |
| Perspective Grid | **Especificado (motor)** | médio | ProjectiveGrid · Spatial |

O custo é uma estimativa relativa. Ele será revisto depois que conhecermos a implementação real das fundações.

## Ferramentas criativas aprovadas

**Aprovadas para fazer parte do produto e modeladas na documentação.** Isso não significa que já estejam implementadas, que todas tenham o mesmo esforço ou que a v0.1 vá entregá-las integralmente.

A interface dessas ferramentas será detalhada e fechada em discussão conjunta. Os contratos de Core/Engine/Render que já puderam ser estabelecidos estão em [Modelos autorais](#/docs/01-core/creative-features.md), [Creative Operations](#/docs/02-engine/creative-operations.md) e [Catálogo de ferramentas](#/docs/04-ui/creative-tools-overview.md).

| Família | Capacidade | Etapa técnica | Referência prioritária |
|---|---|---|---|
| Smart Path | Select/Node/Pen aprimorados | Base | Figma Draw / Illustrator |
| Smart Path | Direct Bend (arrastar segmento) | Base geométrica | Figma Draw / edição direta |
| Smart Path | Smart Delete Node | Base geométrica | Illustrator / curve fitting |
| Smart Path | Simplify e Smooth | Base geométrica | Inkscape / Illustrator |
| Smart Path | Clean Vector + relatório/previews | Base geométrica | Inkscape cleanup + diferencial Petunia |
| Smart Path | Select Same/Similar | Base | Affinity / Illustrator |
| Smart Region | Shape Builder | Após planar subdivision | Illustrator / Affinity / Inkscape / Figma |
| Smart Region | Region Select | Após region graph | Figma / Petunia |
| Smart Region | Region Paint | Após region graph | Illustrator Live Paint / Affinity Flood Fill |
| Smart Region | Detect Gap + Virtual Bridge + Close Geometry | Após region graph | Live Paint + diferencial Petunia |
| Smart Region | Intertwine/Weave | Após region graph + masks | Illustrator Intertwine |
| Smart Distribution | Linear/Grid/Radial Repeat | Após live generators | Figma Draw / Illustrator |
| Smart Distribution | Mirror Repeat | Após live generators | Illustrator / CorelDRAW |
| Smart Distribution | Symmetry Draw | Após Mirror | CorelDRAW Symmetry |
| Smart Distribution | Objects on Path | Após arc-length layout | Illustrator |
| Smart Distribution | Blend: transforms/colors/opacity | Após live generators | Illustrator Blend |
| Smart Distribution | Blend Geometry Morph + manual correspondence | Avançado | Illustrator Blend |
| Smart Distribution | Scatter / Distribution Brush | Após distribution | Figma Draw |
| Smart Color | Palette Extraction | Color Engine | Illustrator / ferramentas de paleta |
| Smart Color | Recolor Lab | Color/Appearance | Illustrator Recolor |
| Smart Color | Recolor from Image | Após Palette Extract | diferencial Petunia |
| Smart Color | Replace Color | Color/Appearance | Illustrator / Affinity |
| Smart Color | Harmonies, locks e contrast/luminance mapping | Color Management | Illustrator / diferencial Petunia |
| Smart Color | Global Swatches / linked styles | Core Styles | Illustrator / Figma variables |
| Smart Measure | Hover Measure: W/H, angle, length | Spatial/Geometry | Affinity / diferencial Petunia |
| Smart Measure | Area / Perimeter / Radius / Diameter | Geometry | Affinity / Illustrator |
| Smart Measure | Pin Dimension: linear/angular/radial | Após DimensionObject | Illustrator Dimension |
| Smart Measure | Persistent associative dimensions | Após anchor contracts | Illustrator + diferencial Petunia |
| Advanced | Variable Width / Width Profiles | Stroke Engine | Illustrator / CorelDRAW |
| Advanced | Pattern Editor on-canvas | Após PatternDefinition | Inkscape / Figma |
| Advanced | Brand Sheet Generator | Após Styles + Fragment | diferencial Petunia |
| Advanced | Vector Feather / Variable Edge Softness | Após Render/Mask | experimental Petunia |
| Advanced | Perspective/Envelope Warp | Pós-v0.1-stable | Illustrator / CorelDRAW |
| Advanced | True Vector Brushes (Stretch/Scatter/Art) | Após Brush + Distribution | Illustrator / Figma |
| Advanced | Mesh Gradient avançado | Exploração pós-v0.1-stable | Illustrator / Inkscape |
| Advanced | Knife / Scissors / Vector Eraser / Raster Eraser | Conforme Geometry/Raster | Illustrator / Affinity / Inkscape |

**Etapa técnica** indica dependências, não data de entrega. “Base” não implica funcionalidade codificada hoje. “Exploração pós-v0.1-stable” registra a capacidade para investigação, sem compromisso de implementação enquanto modelo/custo não estiverem maduros.

### Estratégia de implementação

~~~text
Select / Node / Pen / Select Similar
    ↓
Smart Delete + Simplify/Smooth + Clean Vector + Hover Measure
    ↓
Shape Builder / Region Graph → Region Paint / Close Gap
    ↓
Stroke Width + Pattern + Recolor
    ↓
Live Distribution → Repeat/Mirror → Symmetry / Objects on Path
    ↓
Intertwine + Blend + Associative Dimension
    ↓
Brand Sheet + Vector Brushes + Vector Feather
    ↓
Perspective/Envelope Warp + Mesh Gradient avançado (pós-stable)
~~~

**Reutilização não pode disfarçar custo.** O Region Graph ajuda Region Paint, mas remap de faces durante source edits é difícil; Blend geométrico exige correspondence real; annotations associativas exigem referências estáveis; Vector Feather precisa suporte de máscaras/ROI.

### Killer features escolhidas

As diferenciações de produto que mais combinam alto valor com infraestrutura compartilhada são:

**Clean Vector com diagnóstico**, **Smart Delete com limite de desvio**, **Direct Bend**, **Close Gap não destrutivo**, **Recolor from Image**, **Smart Measure associado** e **Brand Sheet Generator**.

Não serão shortcuts destrutivos disfarçados de assistentes: sempre apresentar preview, diagnóstico, custo quando relevante e Apply/Cancellation controlados.

## Sequência técnica preferencial

Esta ordem descreve dependência arquitetural, não obrigatoriamente release:

```text
Effect Stack
    ↓
Masks / Clip / Blend
    ↓
Adjustments e efeitos simples
    ↓
Live geometry effects
    ↓
Image Trace
    ↓
Simplify / Smooth / Cleanup
    ↓
Palette / Pattern / gradients
    ↓
Slices / utilitários
```

A razão é simples: várias funcionalidades aparentemente diferentes compartilham o mesmo evaluator não destrutivo, compositor e sistema de recursos.

## Image Trace

Image Trace deve nascer como operação editável quando possível:

```text
ImageResource
    ↓
Trace parameters
    ↓
Derived vector result
    ↓
preview
```

O usuário só materializa paths normais quando executar **Expand Trace**.

O contrato de vectorization já está definido em `Image Analysis + Vectorization`: quantização perceptual compartilhada, contour extraction determinística, simplificação controlada e curve fitting reutilizado do Geometry Engine. Dependências concretas adicionais continuam sujeitas à política de dependências.

## Efeitos e ajustes

Os ajustes simples são valiosos porque exercitam a infraestrutura geral sem criar subsistemas separados.

Exemplos:

```text
Brightness / Contrast
Levels
HSL
Vibrance
Invert
Grayscale
Posterize
Drop Shadow
Glow
```

Todos devem usar o mesmo modelo de efeito parametrizado e não destrutivo, salvo uma ação explícita de Bake/Flatten.

## Grids e precisão

Grid não é uma ferramenta isolada. Todos os grids devem alimentar o mesmo sistema de candidatos de snapping.

```text
Grid
  ↓
SnapCandidate
  ↓
Spatial Engine
  ↓
SnapResult
```

A diferença entre cartesiano, isométrico, axonométrico, pixel e perspectiva está na geração geométrica dos candidatos, não na criação de cinco motores de snap.

## Placed 3D

**Estado: Pós-v0.1.0-stable.**

O Petunia poderá importar e compor um elemento 3D dentro do documento 2D, preservando a fonte e parâmetros de visualização.

Escopo pretendido:

- colocar recurso 3D;
- transformar;
- controlar câmera;
- iluminação simples;
- materiais básicos;
- compor o resultado com masks, effects e blend modes 2D.

Regra de produto:

> **Petunia pode posicionar, visualizar e compor conteúdo 3D; Petunia Design Studio não é um modelador 3D.**

Modelagem, rigging, animação esquelética, sculpt, física e sistemas equivalentes continuam fora desse escopo.

A especificação de formato, renderer e modelo de dados só será feita depois de `v0.1.0-stable`.

## Exploração futura

As capacidades abaixo ficam preservadas para estudo, mas **não são compromisso do roadmap ativo**:

- suporte de alta fidelidade a PSD;
- edição PDF mais profunda;
- ferramentas avançadas para variable fonts;
- desenvolvimento RAW completo;
- content-aware fill;
- recursos generativos opcionais;
- prepress e separações profissionais mais avançadas;
- animation/motion.

Cada uma será avaliada por utilidade real, custo, manutenção, dependências e coerência com a filosofia do produto.

## Fora de escopo atual

Alguns caminhos são explicitamente evitados para proteger a identidade do projeto:

- transformar o Petunia em modelador 3D completo;
- introduzir uma game engine;
- criar uma DAW ou editor de vídeo dentro do mesmo produto;
- adotar uma feature apenas para buscar paridade nominal com concorrentes.

Paridade funcional só é desejável quando a função melhora o fluxo de criação do usuário sem violar simplicidade e modularidade.

## Regra de promoção

Uma feature sai de “planejada” e ganha especificação definitiva apenas quando conseguimos responder:

1. Qual problema do usuário resolve?
2. Qual domínio é dono dos dados?
3. Qual Engine executa o algoritmo?
4. O que é persistente, derivado e transitório?
5. Como funciona undo/redo?
6. Como permanece não destrutiva?
7. Qual é o custo de CPU/memória?
8. Que dependência é realmente necessária?
9. Como funciona sem UI quando aplicável?
10. Quais testes provam que está correta?

Quando essas respostas existem, a feature pode ser marcada **Especificado (motor)**. Isso congela o contrato técnico necessário para implementação, mas não congela Tools, acessibilidade ou GUI/UX antes da discussão correspondente.
