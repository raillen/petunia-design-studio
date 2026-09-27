# Catálogo de ferramentas

Referência atômica de cada ferramenta interativa. Todas obedecem ao mesmo contrato: gestos previsualizam, `Up` commita via `Ação → Comando → DocumentMutator → ChangeSet`, e **um gesto = um undo**. Os atalhos abaixo são os vínculos canônicos do app (veja [Manual](/pt/manual/)).

## Como ler este catálogo

- **Gesto** — fluxo de ponteiro/teclado (Down / Move / Up, modificadores).
- **Commita via** — o Comando que entra no histórico.
- **Status** — `Wired` (vivo), `Disabled` (declarado, desabilitado com motivo), `Absent` (não implementado, nunca fingido).

## Persona Design

| Ferramenta | Atalho | Gesto | Commita via | Status |
| ---------- | ------ | ----- | ----------- | ------ |
| Seleção (Select) | ++v++ | Clique/marquee; ++shift++ alterna; arrasto com ++alt++ duplica | `SetBounds` / `CreateShapeObject` | Wired |
| Nó (Node) | ++a++ | Agarra âncora em 12 px/zoom, arrasta vértice | `SetShape` (em lote) | Wired |
| PointTransform | — | Move pivô fixo, escala/gira em torno dele | `SetBounds` | Disabled (dobrado no HUD de Transformação) |
| Caneta (Pen) | ++p++ | Clica âncoras, arrasta tangentes simétricas; clicar na origem ≤ 10 px fecha | `Create path` | Wired |
| Lápis (Pencil) | ++n++ | Traço livre ≥ 2 px, suavizado + ajustado ao soltar | `Create path` | Wired |
| Canto (Corner) | — | Arrasta do centro da seleção; ++shift++ nos quatro cantos | `ModifierKind` vivo | Wired |
| Contorno (Contour) | — | Arrasto radial = offset vivo; colapso preserva o anterior | `ModifierKind` vivo | Wired |
| Perspectiva (Perspective) | — | Arrasta um canto do quad; quads degenerados recusam | `ModifierKind` vivo (homografia) | Wired |
| Faca (Knife) | — | Arrasto = linha de corte amostrada em peças reais (fechado segue fechado) | `submit_all` em lote | Wired |
| Tesoura (Scissors) | — | Clique = divide stroke aberto em dois / abre loop em um | `split_path_at_point` | Wired |
| Retângulo / Elipse / Polígono / Estrela | ++m++ | Arrasto + snap; < 2 px → padrão 100×100; ++shift++ 1:1 | `create_shape_commands` | Wired |
| ShapeBuilder | — | Clique cria, ++alt++ subtrai, arrasto funde regiões | `ApplyBoolean` | Wired |
| SmartFill (VectorFloodFill) | — | Clique na face limitada (`moldura − união`); ilimitada = NoOp | `CreateObject` | Wired |
| Texto artístico (ArtisticText) | ++t++ | Clique = manchete 160×32; arrasto = manchete dimensionada; clique no path = texto no path | `CreateObject` + `SetBounds` + `SetShape` + `SetFill` | Wired |
| Caixa de texto (FrameText) | ++t++ | Arrasto de retângulo (mín. 20×20); colunas/fluxo são pós-V1 | `frame_text()` | Wired |
| Gradiente (fill) | ++g++ | Arrasto do vetor; duplo-clique na linha adiciona stop, no stop remove (mín. 2) | `SetAppearance` | Wired |
| Transparência (Transparency) | — | Arrasto do vetor; padrão opaco→transparente; substitui o proxy legado | `SetModifiers` | Wired |
| Conta-gotas (ColorPicker) | ++i++ | Clique no objeto destravado do topo; amostra fill + stroke | `SetAppearance` por alvo | Wired |
| Conta-gotas de estilo (StylePicker) | — | Clique amostra a AppearanceStack inteira | `SetAppearance` | Wired |
| Prancheta (Artboard) | ++a++ | Arrasto + snap; < 10 px → 1280×720; presets são futuros | `create_artboard_commands` | Wired |
| Medida (Measure) | — | Transiente; nunca muta; arrasto = distância/delta/ângulo, modo área no retângulo | — (só leitura) | Wired |
| Zoom | ++z++ | Clique/arrasto aproxima com foco no cursor | `CameraAction::Zoom` | Wired |
| Mão (Hand) | ++space++ | Arrasto pan | `CameraAction::Pan` | Wired |

## Persona Photo

| Ferramenta | Atalho | Gesto | Commita via | Status |
| ---------- | ------ | ----- | ----------- | ------ |
| MarqueeRetângulo | — | Arrasto de retângulo; ++shift++/++alt++ = modos Adicionar/Subtrair | commit `RasterSelection` | Wired |
| MarqueeElipse | — | Arrasto de elipse; mesmos modos | commit `RasterSelection` | Wired |
| Laço (Lasso) | — | Loop livre; fechamento automático | commit `RasterSelection` | Wired |
| Pincel de seleção | — | Pinta diâmetro/dureza; snap de borda | commit `RasterSelection` | Disabled (precisa de pixel layers) |
| Seleção por inundação | — | Clique + tolerância; preview contíguo | commit `RasterSelection` | Disabled (precisa de pixel layers) |
| Pincel de pixels | ++b++ | Dabs registrados; descartados sem pixel layer | — | Disabled (pixel layers pós-V1) |
| Borracha de pixels | ++e++ | Mesmo motor do pincel | — | Disabled (pixel layers pós-V1) |
| Corte (Crop) | ++c++ | Com seleção = modificador de corte vetorial; sem = corte de superfície (mín. 10 px) | `SetSurfaceGeometry` | Wired |

## Regras que toda ferramenta segue

1. **Previsualiza, depois commita** — `Down`/`Move` nunca escrevem histórico; `Up` commita uma vez.
2. **Nunca perda silenciosa** — fragmentos degenerados (< 1 pt) caem com registro; micro-deleção é proibida.
3. **Conversão implícita só dentro do gesto que a exige** (ex.: a faca autoconverte paramétricos no mesmo lote).
4. **Travado/invisível nunca é atingido** — filtro `visible && !locked` em toda parte.

Os contratos UX por ferramenta vivem nos atlas canônicos; o encanamento de gestos para desenvolvedores está em [Arquitetura](/pt/developers/architecture).
