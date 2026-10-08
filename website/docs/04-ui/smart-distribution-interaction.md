# Smart Distribution — UX de Repeat, Symmetry, Objects on Path, Blend e Scatter

**Estado:** decisões de UX aceitas por delegação em 2026-10-08; especificação de comportamento-alvo, **não implementação/teste concluído**. Respeitar [modelo autoral da família](#/docs/04-ui/smart-distribution.md), [Core](#/docs/01-core/creative-features.md) e [Engine](#/docs/02-engine/creative-operations.md).

## 1. Design comum: instâncias virtuais, source sempre acessível

Todo modo começa em seleção de source(s) elegíveis e produz preview derivado de `DistributionObject`. Até `Expand`, as cópias exibidas são instâncias virtuais e não SceneNodes persistentes. O controle de distribuição é um único contrato compartilhado, com estratégias independentes.

**Estrutura visual:** breadcrumb de contexto `Document > Distribution > Source`, seletor `Linear | Grid | Radial | Mirror | Path | Blend | Scatter`, controles principais da estratégia e `Edit Source`, `Release`, `Expand` (ações explícitas com consequências descritas).

**Gesture contract:** modificar parâmetro on-canvas atualiza preview; Escape cancela gesto, depois sai do subcontexto; Up/Enter confirma apenas Command autoral elegível. Changing count, spacing, angle, origin e seed não deve gerar centenas de commits durante o drag. `Undo` retorna a spec anterior em uma transação lógica.

## 2. Linear Repeat

**Entrada:** selecionar objetos → Repeat → Linear. Preview default conserva source e propõe contagem limitada e step visível; não inventar milhares de cópias.

- Arrastar **step handle** ajusta espaçamento/distância entre instâncias; arrastar **count handle** varia contagem inteira, com feedback `N copies` e teto de orçamento.
- Campos: `Count`, `Step X/Y`, `Spacing`, `Rotation per step`, `Reverse`, `Preserve original`.
- `Drag source` edita source somente após entrar em `Edit Source`; arrastar um ghost não seleciona milhares de objects reais.
- `Count 1` é válido e sem multiplicação visual; step zero repete coincidente intencionalmente com aviso/preview de sobreposição.

## 3. Grid Repeat

**Controles:** rows/columns separados, horizontal/vertical gap, origin/pivot, stagger/brick, alternate mirror/rotation. Render usar handles discretos nas extremidades, não um handle por célula.

- Arrastar borda de grade ajusta row/column count; outro handle ajusta spacing; distinguir por ícone/tooltip/cursor.
- Inputs negativos de gap/células que se sobrepõem são permitidos se geometricamente finitos e previstos no preview; não converter em dimensões NaN.
- Tamanho e rotação da source afetam footprint; opção `Gap based on bounds` vs `Step between origins` deve ser clara.
- Mode `Edit Source` preserva parâmetros da grade; `Expand` mostra contagem de SceneNodes a criar e pode exigir confirmação se ultrapassar budget.

## 4. Radial Repeat

**Controles:** `Count`, `Center X/Y`, `Radius`, `Start angle`, `Total sweep`, `Rotate instance`, `Distribute endpoints`. Preview indica eixo/centro e guides.

- Arrastar centro movimenta o pivô; handle angular modifica início/sweep; handle radial altera raio. Nenhum desses gestos move original silenciosamente.
- `Full circle` e `partial arc` precisam política explícita de endpoints: círculo completo não duplica uma instância no mesmo ângulo do início; arco parcial pode incluir ambas extremidades.
- Count 1/2 e raio zero são degenerate mas válidos, com feedback de sobreposição.
- Rotação das instâncias (`Follow radial orientation`) é opção independente de sua posição angular.

## 5. Mirror Repeat e Symmetry Draw

**Mirror Repeat:** source e eixos pertencem à spec autoral; ghost refletido acompanha alteração. Eixo possui handles para posição e direção, com numeric editor (ângulo/offset).

**Symmetry Draw:** entrada via `Symmetry Drawing` cria contexto que permite Pen, Node ou Brush vetorial editar source enquanto reflete resultado em tempo real. A viewport destaca discretamente área autoral/editável e área virtual.

- `Single Axis` padrão; `Multiple Axes / Radial Symmetry` sob advanced disclosure com limites previsíveis.
- Cursor/pointer sobre ghost apresenta `Virtual instance — Edit source`, com ActionId para revelar source ou entrar em edição refletida mapeada para coordenadas source quando suportado. Não editar o ghost diretamente como segundo objeto autoral escondido.
- `Seam` ao cruzar eixo: `Keep overlapping`, `Clip to axis`, `Weld on Expand` como políticas distintas e explícitas. Padrão conservador não mescla geometria automaticamente.
- No caso dihedral/radial, deduplicar instâncias geometricamente coincidentes na avaliação quando apropriado, sem perder fidelidade autoral nem gerar ordem não determinística.
- `Release` remove generator e mantém source; `Expand` materializa reflexos; sempre mostrar implicação.

## 6. Objects on Path

**Fluxo:** selecionar source(s) → `Objects on Path` → escolher spine por picker semanticamente explícito. `Source` e `Spine` possuem papéis distintos no inspector, com trocas possíveis sem criar cópia autoral.

Controles:
- `Count` ou `Distance` como modos mutuamente claros, mais offsets de início/fim.
- `Rotate to tangent`, `Keep upright`, `Reverse order`, `Include endpoints`, `Corner/cusp policy`.
- `Manual offsets` como ajustes de instâncias virtuais referenciadas por locator de sessão/versão, não por ObjectId persistente fictício.

**Geometria:** espaçamento uniforme percorre comprimento de arco verdadeiro/aproximado com erro controlado, não passos uniformes em parâmetro Bézier t. Em cusps/reversões, exibir regra de orientação e resultado sem rotação abrupta inesperada; permitir desativar tangência.

**Spine edit:** mudar geometria recalcula distribuição; source deletion ou spine dangling mostra `Needs review` e preserva spec até resolução, não anexa automaticamente a outro path próximo.

## 7. Blend 2.0 — distinguir interpolação de morph

**Entrada:** selecionar duas ou mais sources em ordem visível; confirmar endpoints `From/To` e spine opcional. `Reverse` muda ordem explicitamente.

**Attribute Blend (padrão seguro):** interpola transforms, opacidade, cores e attrs compatíveis, com número de steps/easing e espaço de cor definido. Quando conteúdo geométrico não pode ser interpolado, fornecer crossfade nomeado; nunca descrever isso como morph.

**Geometry Morph (opt-in):** painel de compatibilidade de topologia mostra contours pareados, direção, holes, quantidade de nodes e mapping; se incompatível, diagnóstico `Cannot morph geometry` com `Map contours`, `Simplify/Resample (preview)` ou `Switch to Attribute Blend`. Não inferir correspondence por índice sem provenance explícita.

**Controles:** steps/count, spacing, ease (timing curve), color acceleration, interpolation color space, spine mode e `Preview topology`. Easing de posição não deve inadvertidamente alterar easing de cor.

**Safe interactions:** drag on-canvas de position handle altera parâmetro de Blend, não os endpoints sem `Edit Source`. `Expand` indica número de objetos autorais e eventual perda de live linkage. `Release` mantém originals.

## 8. Scatter / Object Brush

**Fluxo:** selecionar motifs → `Scatter` → selecionar região/path → ajustar `Density/Count`, `Min distance`, `Rotation range`, `Scale jitter`, `Seed` e `Collision policy`. Preview determinístico baseado em seed, independentemente de threading.

- `Reseed` é ação explícita com preview, não alteração automática ao mover slider não relacionado.
- Proteção de performance: budgets por contagem, avaliação parcial/LOD do preview, cancelamento, indicação de limite e `Expand` com custo estimado.
- Mesmo que distribua formas vetoriais, Scatter não é automaticamente um True Vector Brush stroke; reutilizar engine de distribution, preservar semântica autoral do Brush quando utilizado.

## 9. Acessibilidade e feedback

- Todos os handles on-canvas têm ActionId equivalente e campos em unidades/documento. Foco sem selecionar/alterar a geometria. "Ajustar grade por teclado" não deve depender de arrastar 40 pixels com mouse.
- Labels textuais `Count, Spacing, Radius, Center, Edit Source, Expand`; tooltip nomeia efeito e estado. Campos numéricos têm draft/Commit/Cancel e incrementos controláveis, com mixed state quando aplicável.
- Ghost vs source diferenciam-se por outline/pattern e rótulo, não apenas cor. Preview de contagem grande pode simplificar elementos sem esconder quantidade real e custo.
- Reduzir competição visual: apenas handles centrais da operação corrente, painel advanced expansível, status `N virtual instances` e aviso claro quando Expand gerará muitos objetos.

## 10. Aceitação/QA

1. `Expand` e `Release` alteram autoral state de maneiras distintas e explícitas.
2. Source edit mantém instâncias virtuais atualizadas; nada gera SceneNodes extras antes de Expand.
3. Radial full circle não duplica endpoint; Grid count/spacing são determinísticos.
4. Symmetry seam usa política escolhida e não duplica strokes involuntariamente.
5. AlongPath respeita arc length; cusps possuem comportamento documentado.
6. Morph incompatível nunca vira falso morph; fallback é claramente `Attribute Blend / Crossfade`.
7. Scatter com seed estável produz resultado reprodutível entre runs e avaliação paralela.
8. Undo/Redo em parâmetros, stale revision, nested generator/cycle rejection e memory budgets.
9. Toolbar/keyboard/screen reader têm paridade funcional e mensagens de erro explicáveis.

## Referências

[Figma Draw — Repeat](https://www.figma.com/blog/introducing-figma-draw/) · [CorelDRAW — Symmetry](https://www.coreldraw.com/en/learn/tutorials/symmetry-tool/) · [Illustrator — Objects on Path](https://helpx.adobe.com/illustrator/using/objects-on-path.html) · [Illustrator — Blend](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blend-panel-overview.html)
