# Smart Measure — medição transitória, dimensões associativas e GUI

**Estado:** especificação de UX aprovada por delegação em 2026-10-08; **não implementado/testado**. Manter contratos de [Core](#/docs/01-core/creative-features.md), [Geometry](#/docs/02-engine/geometry.md) e [Smart Measure](#/docs/04-ui/smart-measure.md).

## 1. Dois tipos de resultado, sem ambiguidade

**Hover Measure** responde a uma dúvida rápida: tamanho, distância, comprimento, ângulo, raio, diâmetro, área ou perímetro. É Query/Overlay transitório, não altera seleção ou documento e nunca suja PTND.

**Pin Dimension** registra uma medição como `DimensionObject` autoral, com anchors associativos e estilo; é uma transação com Undo, render/export sujeitos a policy. Um botão `Pin` explícito transforma medição transitória em anotação persistente quando âncoras são elegíveis.

## 2. Hover Measure — medir sem trocar de ferramenta

Um ActionId `Measure` disponível via toolbar, menu ou acesso temporário traz seleção semântica de objetos/nodes/edges. O tooltip contextual de medida pode surgir a partir da seleção ou por hover deliberado, não deve poluir a viewport por padrão.

- `Measure Between`: escolher ponto/edge A e B com overlay da linha, valor e unidades; saída diferenciada `distance, delta X, delta Y`.
- `Object Size`: alternar `Geometric bounds` e `Visual bounds` com label; não confundir stroke/effects com dimensão geométrica autoral.
- `Path Length`: analisar curve/segment/contour com integrador de arc length do Geometry Engine e tolerância declarada.
- `Area/Perimeter`: reportar área preenchida conforme `FillRule`, incluindo holes; perímetro é comprimento do boundary relevante, não área de bounding box.
- `Angle`: escolha de duas direções ou 3 pontos, com unidade graus/radianos e orientação conforme documento; ângulo reflexo/signed deve ser explícito.
- `Radius/Diameter`: somente quando círculo/arco é suportado pela representação ou ajustado dentro de tolerância com diagnóstico de aproximação; não fingir que cubic genérica tem raio constante.
- `Copy value` e `Send to numeric field` (quando contexto suportar) são ações rápidas que não persistem Dimension.

Preview mostra segmento(s) de referência, label pequeno, direção e snap provider. Ao mover cursor, valores atualizam apenas quando necessário, sem live announcements a cada frame.

## 3. Pin Dimension — transformar leitura em anotação

`Measure → choose anchor A → choose anchor B/feature → preview label/extension lines → choose offset → Pin`.

Dimensões disponibilizadas: `Linear Horizontal`, `Linear Vertical`, `Linear Aligned`, `Angular`, `Radius`, `Diameter`, `Area` e `Perimeter`. A toolbar contextual mostra apenas variantes aplicáveis aos anchors reconhecidos.

**Inspector de Dimension:** `Type`, `Units`, `Precision`, `Prefix/Suffix`, `Text style`, `Arrowheads`, `Line style`, `Offset`, `Display mode`, `Associative vs Fixed`. Opções de aparência pertencem ao `DimensionObject` conforme contrato Core e não precisam persistir dados de medida derivada como verdade.

**Identidade das âncoras:** armazenar referências tipadas (`ObjectId/NodeId/feature ref`) e distinguir de `FixedDocumentPoint`. Ao mover source, valor e overlays se atualizam via avaliação determinística; se source desaparece, marker `Unresolved` e repair picker, nunca reconectar ao objeto mais próximo por conveniência silenciosa.

## 4. Precisão e interpretação geométrica

Todas as medições usam cálculo f64 em unidades documentais. A string exibida é formatação localizada; converter para px/canvas não afeta tamanho físico do documento. `Approximate` deve aparecer quando resultado deriva de fitting/integration tolerante; tooltip opcional explica erro.

Transformações aninhadas e não uniformes exigem medir na geometria avaliada no referencial escolhido. `Distance between local anchors` e `Page distance` podem diferir; UI exibe sistema de referência e unidade.

Para autointerseções, a área usa preenchimento EvenOdd/NonZero configurado da source; no caso de seleção de múltiplos caminhos, calcular soma/união são resultados diferentes e precisam ser rotulados. Nunca oferecer área negativa como mera consequência de winding se usuário pediu `Filled area` sem definição.

## 5. Context bar e defaults

Para **Hover Measure:** `Distance | Size | Angle | Radius | Area | Perimeter`, `Snap`, `Copy`, `Pin` (habilitado só com anchors válidos), unidades. Para **Pinned Dimension:** tipo, offset, estilo, visibilidade, `Relink` quando unresolved e `Detach to fixed` como ação separada.

O valor em foco oferece copiar sem arrastar a anotação, input numérico/precisão editáveis quando fizer sentido e legenda `≈` para aproximações. Usuário pode ocultar medições temporárias sem remover dimensões persistentes.

## 6. Navegação e acessibilidade

- Todos os modos têm ActionIds e acesso pelo teclado: `Choose first anchor`, `Choose second`, `Cycle snap candidates`, `Measure`, `Copy`, `Pin`, `Cancel`.
- Screen reader anuncia `Measurement type, source A, source B, value, unit, precision, approximate/unresolved` em atualização significativa, não a cada pixel de hover.
- Alvos de dimension line/text/arrows têm hit targets maiores que representação, sem alterar geometric accuracy.
- Labels seguem scaling, contraste alto e reduced-motion; não usar somente cor para diferenciar hover versus dimensão persistentemente gravada.
- Ao editar text label, teclado pertence ao campo até confirmação/cancelamento, não dispara atalhos do canvas.

## 7. Critérios de qualidade

1. Hover measure não cria HistoryEntry nem Document dirty.
2. Pin cria `DimensionObject` autoral, com Undo único e refs estáveis.
3. Area/perimeter respeitam FillRule e holes; text oferece método/tolerância se aproximado.
4. Transforms não uniformes, páginas, zoom/DPR/rotated viewport medem corretamente.
5. Delete source deixa dimensão unresolved, não relinka por proximidade.
6. Múltiplos anchors e linhas de medição não entram em conflito com nodes da Vector Edit; desambiguação por candidato.
7. Screen reader/keyboard fazem medição e Pin/Unpin sem pointer.
8. Save/load/export reproduzem dimensões persistentes, não os overlays transitórios.

## Referências

[Illustrator — Dimension Tool](https://helpx.adobe.com/br/illustrator/using/tool-techniques/dimension-tool.html) · [Dimension Objects](https://helpx.adobe.com/br/illustrator/desktop/measure-and-align/plot-and-measure/about-dimension-objects.html)
