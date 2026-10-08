# Smart Color — extrair, relacionar e trocar cores

**Estado (2026-10-08):** capacidades aceitas; UX de Palette Extract/Recolor Lab/Global Swatches **aprovada por delegação** e detalhada em [Smart Color — interação](#/docs/04-ui/smart-color-interaction.md). Documentado, não implementado/testado.

## Referências

Illustrator Recolor Artwork, Figma Draw Color Variables e seleção por atributo do Affinity Designer. A meta não é copiar painéis complexos, mas reduzir etapas de branding e ilustração.

## Sistema compartilhado

~~~text
ColorSources / SwatchRefs / Image Resource
↓ Color Management + Palette Analysis
ColorMappingSpec
↓ transient non-destructive preview
Apply via Command / Keep Live Effect
~~~

Uma cor pode aparecer em Fill, Stroke, Gradient Stop, TextStyle, Pattern ou Symbol override. Recolor deve percorrer referências sem quebrá-las silenciosamente.

## Palette Extract

Entradas: imagem ou seleção de objetos, perfil, analysis region, quantidade máxima de cores. Engine converte para espaço de análise definido, agrupa cores similares por quantização, ordena clusters deterministicamente, produz palette com cobertura aproximada. É Query/Job até o Command Create Swatches.

Interação proposta: palette preview, merge/split, reordenar e aceitar; mostrar perfil e diferença perceptual.

## Recolor Lab

1. Selecionar artwork e escopo;
2. coletar cores usadas e referências;
3. apresentar source→target mapping e preview;
4. permitir edit manual, harmonies, rotate hue, randomize com seed, preserve luminance/relative contrast, lock Spot e swatches;
5. Apply como substituição explícita ou Keep Live como ColorMappingEffect.

**Locks:** Spot não vira process color sem consentimento. Preserve luminance e contrast são constraints aproximadas: resultados fora do gamut precisam diagnóstico, nunca promessa perfeita. Assign Profile e Convert Profile não se misturam com Recolor.

## Recolor From Image

Drag imagem → Palette Extract → mapear cores por luminância/hue/contraste ou manualmente → preview → Apply/Keep Live. Imagem é ResourceId, não autoridade de cor após Apply materializado.

## Replace Color / Select Same

Replace Color muda valores/bindings via Command; Select Same Color altera somente Session Selection. Select Similar usa métrica perceptual e threshold nomeado, definidos pelo Engine.

Há quatro operações diferentes: editar Swatch global; alterar color local; aplicar Live Mapping; Bake Mapping. A UI deve distingui-las claramente.

## Casos e testes

Gradients com alpha, ICC ausente, CMYK/Lab, Spot, out-of-gamut, nested symbols, text runs, colorspaces misturados, Undo/Redo, seed, palette clustering determinístico.

[Interação e UX Smart Color](#/docs/04-ui/smart-color-interaction.md) · [Core Color](#/docs/01-core/color.md) · [Color Management](#/docs/02-engine/color-management.md) · [Engine](#/docs/02-engine/creative-operations.md)