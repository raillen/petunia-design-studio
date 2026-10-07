# Roadmap de capacidades

O roadmap registra **capacidades que queremos preservar no horizonte do produto**. Ele não substitui as especificações técnicas e não transforma uma ideia em compromisso de release automaticamente.

A regra é:

> **a feature entra no roadmap agora; a implementação só é fechada quando o domínio arquitetural que a sustenta estiver maduro.**

Isso evita desenhar UI ou APIs sobre fundações ainda instáveis.

## Como ler este roadmap

| Estado | Significado |
|---|---|
| **Planejado** | capacidade aprovada para o produto; fase exata ainda depende da arquitetura |
| **Pós-v0.1.0-stable** | não entra antes da primeira base estável |
| **Exploração futura** | ideia preservada, mas sem compromisso de implementação |
| **Fora de escopo atual** | deliberadamente não faz parte do produto neste ciclo |

## Capacidades planejadas

As capacidades abaixo serão revisitadas quando o tópico técnico relacionado for amadurecido.

| Capacidade | Custo esperado | Domínio que desbloqueia a decisão |
|---|---:|---|
| Live Effects / Effect Stack | médio | Não destrutibilidade · Appearance/Effects · Render |
| Image Trace / Live Trace | baixo–médio | Geometry · Raster · Vectorization |
| Simplify / Smooth / Cleanup | baixo–médio | Path · Geometry |
| Crop não destrutivo | baixo | Scene · Masks/Clipping |
| Linked Images + Relink | baixo | Resources · Document · I/O |
| Palette Extraction | baixo | Color · Raster |
| Remove White / Color to Alpha | baixo | Raster · Effects |
| Invert / Grayscale / Posterize | baixo | Raster · Effects |
| Brightness / Contrast / Levels | baixo | Effects · Color |
| HSL / Vibrance | baixo–médio | Effects · Color |
| Blend Modes | baixo–médio | Compositor |
| Drop Shadow / Glow | médio | Effects · Compositor |
| Live Corners | baixo–médio | Path · Geometry · Non-destructive effects |
| Live Offset / Contour | médio | Geometry · Non-destructive effects |
| Pattern Fill | baixo–médio | Paint · Appearance |
| Conical Gradient | baixo | Paint · Render |
| Clipping / Opacity Masks | médio | Scene · Compositor |
| Export Slices | baixo–médio | Document · Render · Export |
| QR Code / Barcode Generator | baixo | Tools · Vector output |
| Cartesian Grid | baixo | Spatial · Snapping |
| Isometric Grid | baixo | Spatial · Snapping |
| Axonometric Grid | baixo | Spatial · Snapping |
| Pixel Grid | baixo | Spatial · Viewport |
| Baseline Grid | baixo | Text · Layout · Snapping |
| Perspective Grid | médio | Math · Spatial · Snapping |

O custo é uma estimativa relativa. Ele será revisto depois que conhecermos a implementação real das fundações.

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

A biblioteca ou algoritmo concreto será decidido quando chegarmos ao tópico de vectorization. O roadmap não congela uma dependência antecipadamente.

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
- mesh gradients avançados;
- ferramentas avançadas para variable fonts;
- desenvolvimento RAW completo;
- content-aware fill;
- recursos generativos opcionais;
- prepress e separações profissionais mais avançadas;
- perspective/envelope warp avançado;
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

Até essas respostas existirem, o roadmap registra intenção — não uma API congelada.
