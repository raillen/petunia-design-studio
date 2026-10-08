# Smart Region — experiência de Build, Paint, Gap e Weave

**Estado:** decisões de UX aprovadas por delegação em 2026-10-08, subordinadas à arquitetura previamente fechada. **Documentado, não implementado/testado.** Consulte [Smart Region overview](#/docs/04-ui/smart-region.md) e [Creative operations Engine](#/docs/02-engine/creative-operations.md).

## 1. Modelo mental: quatro tarefas, uma análise de regiões

Usuário inicia a partir de uma seleção de shapes/paths elegíveis. O Engine avalia geometria, interseções, FillRule, clipping e transformações, produzindo um **grafo derivado de regiões com provenance**. Esse grafo serve Build, Paint, Region Select e gap detection, mas não transforma regiões em novos SceneItems até uma operação autoral explicitamente confirmada.

| Ação | Intenção do usuário | Modelo autoral |
|---|---|---|
| Shape Builder | reunir, subtrair, extrair faces | generator live ou Paths materializados por Expand |
| Region Paint | pintar uma face delimitada por múltiplos paths | `RegionPaintObject` com bindings e paints |
| Close Gap | detectar, simular ponte ou fechar desenho | análise/virtual bridge ou Command geométrico distinto |
| Region Select | escolher uma face para agir | somente sub-selection até Extract/Expand |
| Intertwine/Weave | alterar ordem visual **local** de sobreposições | grupo/bindings locais de oclusão; não altera z-order global |

## 2. Estados comuns

`Idle → ResolvingSources → Analyzing → Ready → HoverFace → Selecting/Building/Painting/Weaving → Preview → Commit/Cancel`.

- `Analyzing`: trabalho cancelável com progressivo; UI continua responsiva e mostra status contextual não bloqueante.
- Se `source revision` mudar durante análise/preview, invalidar cache/preview antigo; não aplicar resultado stale.
- Nenhuma seleção de face por hover cria IDs autorais.
- Canvas deixa resto do documento visível em baixa ênfase **sem esconder o contexto espacial da arte**; oferecer Focus Mode opcional para ilustração intrincada.
- Clique em face ambígua dá acesso a lista de region candidates; não escolhe aleatoriamente por ordem de hash.

## 3. Shape Builder — padrão live, expansão explícita

**Decisão:** criação não destrutiva **Live Build** é o padrão recomendado no Petunia, com `Expand to Paths` como transformação destrutiva e `Release` como retorno das fontes originais. Isso protege iniciantes que só perceberiam a necessidade de mudar as formas depois.

**Entrada:** selecionar dois ou mais caminhos/objetos adequados ou um path self-intersecting que produza faces elegíveis; `Shape Builder` tem ActionId próprio. Escopo e pré-condições aparecem no painel se alguns objetos são Text, Symbol, masked/locked/hidden.

**Ferramenta:** a barra contextual mostra `Add`, `Subtract`, `Extract`, `Live / Expand`, e contagem de faces. `Add` é default; `Subtract` não depende exclusivamente de Alt ou Shift. Um gesto Shift durante drag não pode mudar silenciosamente a seleção de nodes do Vector Edit subjacente: contexto da operação captura a intenção.

**Gestos:**
1. Hover mostra face única com preview translúcido e tooltip breve (`Add region`).
2. Click Add seleciona face; Click Extract gera resultado separado no live generator, ou no materializado somente após Expand.
3. Drag atravessando faces agrega uma sequência de faces elegíveis, com ghost do resultado combinado. Não basta cruzar bounding box; usar adjacency/face crossing exato.
4. Subtract marca face removida, com overlay/ícone distinto e label textual `Subtract`.
5. Confirmar gera uma única Transaction com operação live autoral. Escape cancela toda preview da sessão Build, sem destruir fontes.
6. Expand é Command posterior com before/after e contagem de novos PathObjects; Release volta às sources conforme bindings, sem perda inesperada.

**Styles conflitantes:** quando faces de fontes diferentes possuem fill/stroke/style distintos, usar estilo do `Primary source` visível e escolhido (default deterministicamente o active object) com possibilidade `Choose source style`, `Keep per-region styling` quando suportado. Nunca escolher silenciosamente a primeira ocorrência casual de R-tree.

**Topo e limits:** holes, nested contours, open paths, converted strokes, compound paths e FillRule EvenOdd/NonZero precisam pré-condições claras. `Keep source` não é o mesmo que `Expand` e deve ficar explicado.

## 4. Region Paint — pintar regiões sem partir o desenho

**Entrada:** `Region Paint` a partir de paths selecionados, live Build ou Region Select. O usuário vê faces delimitadas por paths/arestas mesmo quando faces dependem de vários objetos.

**Fluxo:** selecionar fonte → entrar Region Paint → hover realça face e `current paint` → clique aplica preenchimento via binding → continuar pintando → confirmar uma Transaction lógica por stroke/sessão controlada. A paleta pode ser solid, gradient ou pattern via Paint API; paint sources não são reescritas por mero clique em região virtual.

**Comportamento avançado:**
- `Paint current`, `Eyedropper paint`, `Recent colors`, `Swap current paint`, `Fill all similar regions` por ações coerentes, com preview da quantidade de faces.
- `Alt hover` não é obrigatório: color picker acessível como ferramenta/ActionId, e os mesmos targets via teclado.
- Hover de face mostra bounds/borda sem alterar stroke source; cursor mostra estado Paint/Unavailable.
- `FillRule` e zoom não alteram a identidade lógica de face dentro da mesma revisão; resoluções de IDs derivados seguem provenance.
- Se edição posterior do source divide/une face, remapear apenas quando correspondência for **inequívoca**; do contrário exibir `Needs Review`, manter vínculo e pintura anteriores em estado recuperável sem aplicar cor arbitrariamente numa nova face.
- `Expand` materializa Paths com Paint, com confirmação explícita por poder abandonar reatividade do live graph.

## 5. Close Gap — Detect, Virtual Bridge e Close Geometry separados

O produto deve apresentar os três caminhos como escolhas claramente diferentes, evitando que `Detect gaps` seja confundido com alterar o arquivo.

### Detect

Selecionar sources → `Detect gaps` → painel com contagem, endpoints, distância (document units), ângulo das tangentes e potencial de cruzamento. Navegar por ocorrências no canvas, com preview de conector. Analisar por tolerância configurável, com preset conservador. **Nenhuma mutação**.

### Virtual Bridge

Selecionar proposta → `Use virtual bridge for regions`. O Engine adiciona uma ponte **somente no grafo derivado** para conter flood/leak de Region Paint; paths originais permanecem abertos. A ponte deve se vincular a endpoints e escopo e aparecer com marcador virtual ao inspecionar regiões. Alteração de source revalida bridge e pode marcá-la como unresolved.

### Close Geometry

Selecionar gap → `Close geometry` exibe segmento/nova geometria proposta, risco topológico e referências afetadas. Só confirmar cria/edita `Line/Cubic` e produz HistoryEntry. Não escolher automaticamente entre `Join nodes`, `Add segment` e `Close contour` sem validar a identidade e a topologia.

**Recusas necessárias:** não criar bridge atravessando terceiro path, self-intersection indesejada, máscara ou grupo fora do escopo; não fechar gap com distance alone sem verificar orientação/tangentes, FillRule e limits. Permitir `Skip` e explicar motivo, sem toast por falha durante hover.

## 6. Region Select — seleção transitória de faces

Click numa face seleciona `RegionRef` derivado para aplicar Paint, Build, Extract, mask ou medida; **não** ObjectId persistente. Shift+click alterna seleção de faces, Lasso/Marquee reutilizam políticas aprovadas de seleção com regras próprias para geometria de faces (Contenção/Interseção explícitas).

Após editar sources, sub-selection derivada deve validar revision/provenance; região que deixou de existir não vira outra por proximidade visual. Focus/keyboard usa `Region N of M`, área/perímetro quando disponível, associação às sources e ActionIds contextuais.

## 7. Intertwine / Weave — ordem de oclusão local, sem cortar sources

**Default não destrutivo.** Selecionar objetos sobrepostos → `Create Weave` → hover destaca crossings/overlap areas → click `Bring in Front/Send Behind` **na região apontada**, com preview. Em sobreposições de três ou mais objetos, abrir escolha de par e ordem relativa, sem promover uma ordem impossível.

- Use contornos de região fechada para aplicar mudança em múltiplos crossings, além de click pontual.
- Mostrar `current local order` legível na context bar; não modificar `Page.root_children` global.
- `Release` remove regra de oclusão e retorna à composição original; `Expand` materializa recortes/masks somente quando solicitado/export necessário.
- `opacity/blend/mask/effect` podem tornar ambíguo o aspecto final; avisar que troca de oclusão não equivale a reorganizar objetos no documento.
- Regras de ordem que criam ciclo/impossibilidade nas mesmas regiões precisam diagnóstico de conflito, não fallback visual falso.
- Não forçar exclusões antecipadas por limites de outros aplicativos; implementar verificação de capacidades Petunia por fonte/estrutura e tratar aninhamento de Weave via política/diagnóstico.

## 8. Context bar, inspector e organização de ferramentas

`Smart Region` apresenta operação ativa `Build | Paint | Gap | Select | Weave` e sempre deixa visível `Finish/Apply/Cancel`. Em cada operação, mostrar os 1–3 controles mais usados; avançado/diagnóstico em inspector.

| Operação | Prioridade de controles |
|---|---|
| Build | Add/Subtract/Extract, live, Apply, source styling |
| Paint | Paint target, fill/current color, recent paints, Gap indicator |
| Gap | Detect threshold + count, Virtual/Geometry method, Review |
| Select | Face count, selection mode, Extract |
| Weave | Front/Behind, local order, Release/Expand |

Toolbar e command palette apontam aos mesmos ActionIds. Quando a operação muda, preservar seleção de source e faces se ainda elegíveis; caso contrário explicar o que foi invalidado. Não reaproveitar handles de node com significado diferente de face sem rotulação.

## 9. Feedback/acessibilidade

- Cor **não** é único canal: sobreposição usa hatch/outline/ícone e etiqueta `Add`, `Subtract`, `Paint`, `Blocked`, `Virtual Bridge`.
- Targets de face usam query geométrica + threshold em pixels lógicos quando for boundary; touch/stylus recebem alvos acessíveis e alternativa por lista.
- Canvas semântico apresenta região, sources, ação disponível, status de análise e resultado; leitor de tela não anuncia cada face ao movimentar pointer.
- Navegação por teclado percorre faces/crossings em ordem determinística e permite confirmar/cancelar, selecionar cor, escolher ação, expand/release.
- Jobs longos incluem `Cancel analysis` e `Last stable preview`, respeitando reduced-motion e autonomia do usuário.

## 10. Quality gates

- Build self-intersection e múltiplos paths: união/subtração não eliminam sources nem aparecem como mudança antes de Commit.
- Paint em fontes abertas com gap: Detect/Virtual Bridge/Close Geometry geram **três** resultados semânticos diferentes.
- Remap de faces após source edit: ambíguo mostra Needs Review, sem "vazamento" de paint.
- Weave com 2/3 objetos: ordem regional correta, z-order global original intacto e Release idempotente.
- Style conflict: source/style explícitos, sem seleção casual determinada por hash/z-order inesperado.
- Invalid inputs: NaN/Inf, tiny gaps, concave faces, nearly tangent, collinear overlaps, holes, FillRules, masks.
- Revisão/stale analysis e jobs cancelados não deixam documento parcial.
- Screen reader/keyboard operam Build/Paint/Gap/Weave, sem exigir hover.
- Cache por geometry revision + tolerâncias/FillRule, protection contra explosion de arrangement; profiling com targets reais e hardware modesto.

## 11. Referências externas

- [Figma — Shape Builder](https://help.figma.com/hc/en-us/articles/31616004109847-Create-custom-shapes-with-the-shape-builder-tool): regiões editáveis e gestos de merge/extract/subtract; Petunia diverge ao preferir fluxo live não destrutivo.
- [Inkscape — Shape Builder](https://wiki.inkscape.org/wiki/Release_notes/1.3): gesto aditivo/subtrativo + stroke.
- [Illustrator — Find and close Live Paint gaps](https://helpx.adobe.com/illustrator/desktop/paint-and-fill/learn-painting-basics/find-and-close-gaps-in-live-paint-groups.html): distinguir detectar, simular fechamento e criar paths.
- [Illustrator — Intertwine](https://helpx.adobe.com/uk/illustrator/desktop/manage-objects/reshape-transform-objects/create-intertwined-objects.html): ordem de composição em overlap local sem destruir sources.

**Implementação:** seguir infraestrutura de planar subdivision/provenance do Engine. Não adicionar `RegionRef` a Core persistent sem um modelo autoral explícito de binding.
