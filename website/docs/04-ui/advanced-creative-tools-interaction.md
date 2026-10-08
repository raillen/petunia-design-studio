# Ferramentas avançadas — especificações de interação, segurança e acessibilidade

**Estado:** decisões de UX aceitas por delegação em 2026-10-08, sem alterar os estágios de entrega definidos no [roadmap](#/docs/00-roadmap/capabilities.md). **Contratos documentados; funcionalidades não consideradas implementadas nem testadas.** Base técnica: [Catálogo avançado](#/docs/04-ui/advanced-creative-tools.md), [Core](#/docs/01-core/creative-features.md) e [Engine](#/docs/02-engine/creative-operations.md).

## 1. Princípio comum

Ferramenta avançada deve ser inicialmente compreensível por um gesto direto e preservar opções profissionais no Inspector. A toolbar mostra família + operação ativa, não todos os recursos de uma vez.

Em todas: `Entry/Eligibility → Preview → Modify/Options → Apply or Keep Live → Expand/Bake when available → Undo`. `Escape` cancela preview antes de sair do contexto. Locked/hidden/instance/text/raster têm elegibilidade por ferramenta, com motivo quando operação não suportada. Nenhuma função autoral é disparada por hover.

## 2. Variable Width Stroke

**Referência mental:** manipular espessura do traço sobre o próprio path, sem reconstruir contorno manualmente. Core Path permanece Line+Cubic, enquanto perfil de largura autoral pertence ao StrokeStyle/WidthProfile.

**Flow:** selecionar Stroke elegível → `Vector Edit → Width` → hover spine destaca posição de inserção `s` em comprimento de arco → click cria width point com preview → drag transversal ajusta largura → arrastar longitudinalmente reposiciona ponto ao longo da curva → Up confirma Transaction.

- `Symmetric` padrão: controla largura total com lados iguais; `Asymmetric` opt-in exibe dois handles, Left/Right, valores não negativos.
- Campos `Position along path` (distância/percentual do arc length), `Left width`, `Right width`, `Interpolation`, `Lock total width`, `Reset`, `Remove width point`.
- Handles de Width devem ser visual e semanticamente distintos de handles Bézier/Node. Modo Width não altera nodes pelo drag em segmento.
- Profile usa `s` por arc length, não parâmetro t; closed contour, dashes, caps, sharp cusp e self-overlap geram preview correspondente, com warning quando offset não for robusto.
- `Expand Stroke` é ação explícita que materializa Path geometry e perde vínculo editável do perfil; `Edit source stroke` permanece possível antes disso.
- Teclado/focus percorre width points em ordem, ajusta posição e larguras com incremento em unidades documentais.

## 3. Pattern Editor — tile editável e preview virtual

**Flow:** selecionar padrão ligado a objeto ou criar Pattern Definition → entrar `Pattern Edit`. Tela mostra tile primário com outline claro e repetição ghost em torno. Alterar conteúdo do tile ocorre em source authoring context; modificar parâmetros usa handles e campos.

**Controles:** `Tile width/height`, `Origin`, `Gap X/Y`, `Rotation`, `Scale`, `Brick/Stagger`, `Mirror`, `Preview repetitions`, `Edit content`.

- `Edit linked pattern` altera PatternDefinition e todos os consumidores; UI informa quantidade de consumers, oferece `Detach this instance` para criar cópia, sem editar global por acidente.
- `Preview repetitions` define apenas densidade visual de UI, não quantia de objetos autorais. Canvas mostra seam e área de recorte, não milhares de nós.
- Quando o tile é modificado, preview usa cache por revision para não interromper manipulação.
- Detectar referências cíclicas (Pattern A contém Pattern A ou cadeia) e rejeitar sem corromper; advertir se custo de render exceder orçamento.
- Release volta ao contexto anterior preservando alteração confirmada; Escape em gesture só descarta a alteração transitória.

## 4. Brand Sheet Generator — artefato editável, não screenshot

**Flow:** `Tools → Generate Brand Sheet` → wizard curto contextual: escolher Logo/Variant(s), Global Swatches, Text Styles, Page preset e optional assets → preview de páginas/frames → `Generate` produz DocumentFragment de shapes/textos editáveis com referências preservadas e uma Transaction.

- A entrada permite `Use current selection` com lista explícita; no scope não inserir logo errado.
- Templates claros (Minimal, Identity Overview, Color & Type, Usage Sheet) com preview; campos obrigatórios têm placeholders com explicação e rotas para dados faltantes.
- Reusar Style/Swatch refs autorais, sem converter texto em curves ou rasterizar sem solicitação.
- Se não houver perfil CMYK, não rotular valores convertidos como prova/medida exata de impressão.
- Futura atualização automática depende de contrato de provenance/reload; inicialmente `Generate Snapshot` deixa claro que layout é editável e não se atualiza magicamente.
- Keyboard/language/reading order e contraste adequados desde a geração, não apenas no painel.

## 5. Vector Feather / Variable Edge Softness

**Flow:** selecionar path/mask elegível → `Appearance → Vector Feather` → preview da borda suavizada → ajustar `Global softness` ou inserir control points em posições de arc length → variar magnitude/falloff → Keep Live. Core Path original permanece editável.

- Handles de softness distintos de handles de width/tangent; controles `Inside/Outside/Both`, `Falloff curve`, `Local softness`, `Global strength`, `Preview quality`.
- Não apresentar isso como Gaussian Blur global; label deve explicar que a suavização varia ao longo da borda.
- O motor usa cobertura/mask derivada (SDF/edge-distance ou equivalente) com ROI e tolerâncias; não carregar resultado de preview como geometry autoral.
- Holes, tiny loops, corners/cusps, overlaps e masks aninhadas exigem testes de continuidade/softness; limites de render descritos no Inspector.
- `Flatten/Export` devem alertar quando formato não suporta o efeito e oferecer rasterização explícita, se autorizada.

## 6. Perspective e Envelope Warp

**Sem misturar dois motores:**
- **Perspective:** controle quadrilateral/homografia projetiva, quatro corner handles e center/pivot; detecção de quadrilátero degenerado.
- **Envelope:** edge/corner handles ou control mesh, deformando a source por field de controle; prioridade de design para manipulação intuitiva com preview de grade de deformação.

**Flow:** selecionar source → escolher Warp mode → overlay claro da forma original e resultado → arrastar ponto/aresta com constraints ou informar coordenadas → `Keep Live`; `Expand Geometry` ou `Rasterize` explícitos dependendo do source type e export.

- Lock de proporção e constraints não escolhem solução matemática inválida; não produzir divisão por zero/Infinity em pontos projetivos.
- `Edit source` versus `Edit warp` separados por subcontexto, para que duplo clique não destrua warp.
- Text deve permanecer editável enquanto live quando backend suportar; se não, explicitar capability limitation e exigir conversão/flatten específico.
- Preview usa adaptive subdivision com error budget; exibir quality performance trade-offs somente no Advanced inspector.
- No caso de inversão de face, sobreposição/fold ou matriz degenerada, mostrar diagnóstico localizado antes de Commit.

## 7. True Vector Brushes — motivos como vetores

**Flow:** escolher brush asset vetorial → Brush Tool → desenhar spine com preview estabilizado → ajustar spacing/scale/jitter e dynamics → manter `BrushStrokeObject` autoral vinculado a spine/asset/seed → opcional `Expand` para paths.

Perfis `Stretch`, `Art`, `Scatter` diferenciam:
- Stretch: source deformada ao longo do spine.
- Art: motivo mantido/orientado conforme spine e regras.
- Scatter: distribui instâncias vetoriais seeded, com variação e colisão.

O brush inspector mostra thumbnail, nome, densidade, spacing, seed/dynamics e estimativa de instâncias. Não disfarçar brush raster com PNG como 'True Vector'.

**Entrada/cancelamento:** pen pressure/tilt só afetam atributos escolhidos no preset; não alteram seleção. Stroke cancelado deixa zero alteração. Performance mede custo com milhares de instâncias, preview LOD/cancel backpressure sem mudar output final.

**Asset handling:** brush não existe quando asset referenciado falta; manter vínculo unresolved e placeholder legível sem substituir por outro asset silenciosamente.

## 8. Mesh Gradient (exploração futura)

Não prometer lançamento próximo nem inserir enum persistente vazio antes do Core model aprovado. A UX-alvo pode prever `Create Mesh` → manipular patches/control points → escolher cores por vertex → preview contínuo e `Apply`. Integração posterior dependerá de modelo explícito de patch continuity e color interpolation, avaliação de GPU/software e export degradation. O sistema de seleção usa foco e alvos ampliados como demais controles da viewport.

**Importante:** esta seção é alvo exploratório; não supera status pós-v0.1 do catálogo/roadmap.

## 9. Knife, Scissors, Vector Eraser e Raster Eraser

São quatro operações com resultados distintos. Não reusar um único modo obscuro com comportamento que depende de qual objeto está abaixo do cursor.

| Ferramenta | Target e gesto | Resultado e proteção |
|---|---|---|
| Scissors | hover path/segment mostra corte pontual, click em parâmetro válido | Divide path/contour com preservação de NodeIds válidos; sem remover área |
| Knife | desenhar cutter trajetória/shape com preview | Divide topologia de Path conforme interseções; FillRule/holes preservados quando possível |
| Vector Eraser | stroke de remoção sobre geometry vetorial | Subtração topológica/recorte com before/after, não altera PixelLayer |
| Raster Eraser | pintar cobertura alpha sobre PixelLayer | Brush Engine com tiles/COW e rollback exato por Undo |

**Scissors:** escolher alvo com target picker quando vários segments coincidem; valor do ponto de corte por nearest t e coordenada document-space, inserir node via De Casteljau preservando silhueta quando cubic. No click hover não secciona.

**Knife:** ghost do cutter e quantidade estimada de pieces; opções `Cut`, `Keep both sides`, `Separate pieces` explícitas, sem ocultar parts. Cursor no locked path indica `Cannot cut locked object`.

**Vector Eraser:** brush visual indica raio real em unidades e scaling; modos `Subtract`, `Split`, `Trim` distintos. Preview conserva source até Commit. Large eraser stroke não explode nodes sem diagnosticar/budget.

**Raster Eraser:** dureza/size/opacity/flow/stabilizer e pressão quando configurados; eraser não edita vectors por acidente e respeita active PixelLayer/alpha lock/masks.

**A11y:** para Scissors/Knife disponibilizar coordenadas/pontos e ação de corte sem gesto livre; Eraser oferece operações por seleção/máscara quando desenhar com pointer for inacessível.

## 10. Regras de Context Bar e classificação das capacidades

As operações de edição direta ficam próximas da família relacionada (`Width` em Vector Edit; `Pattern` no Paint/Appearance; `Warp` em Transform; `Knife/Eraser` em Tools). As ações especializadas vão para overflow/Command Palette, não todos para toolbar fixa.

Mostrar inicialmente: modo ativo, alvo elegível, um ou dois controles principais e `Apply/Cancel`; ao abrir `Advanced`, revelar tolerâncias e opções de backend sem deslocar abruptamente os controles básicos. Tooltips explicam `Keep Live`, `Expand`, `Bake`, `Flatten`, `Rasterize` com palavras diferentes.

## 11. Quality gates comuns

- Operações live preservam source e referencial; expand/bake é explícito e Undo atômico.
- Preview e Commit usam mesmo solver e mesmas tolerâncias sem drift.
- Node/Contour/Resource/Style IDs são mantidos quando a transformação não exige substituir identidade.
- Stale revision cancela ou recalcula, sem mutação parcial.
- Negative scale, transforms não uniformes, locked, masks, symbols, self-intersections, zoom/DPR alto e áreas enormes.
- Performance budgets, cancel latency, resource resolution e fallback seguro de export.
- Teclado/screen reader, nome de controls, focus, contraste e reduced-motion possuem aceitação real.
- Features classificadas `exploratórias` ou `pós-v0.1` não se transformam em promises de roadmap por esta especificação de UX.

## Referências

[Illustrator — Variable Width](https://helpx.adobe.com/br/illustrator/using/stroke-object.html) · [Inkscape — Pattern Editor](https://wiki.inkscape.org/wiki/Release_notes/1.3) · [CorelDRAW — Variable Outline](https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Mac/CorelDRAW-en/CorelDRAW-Variable-outlines.html) · [Figma — Scatter Brushes](https://www.figma.com/blog/figma-draw-scatter-brushes/) · [Advanced tools overview](#/docs/04-ui/advanced-creative-tools.md)
