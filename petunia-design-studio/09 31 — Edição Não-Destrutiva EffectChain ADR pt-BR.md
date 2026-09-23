# 09.31 — Edição Não-Destrutiva Universal: EffectChain, Modificadores Vivos, Bake Explícito

<aside>

**Status:** `ACCEPTED_V1` (espelho pt-BR do canônico en-US
`09 31 — Non-Destructive Editing EffectChain ADR`). Implementa a doutrina de
`AGENTS.md` / `01` / `02` / `03` (não-destrutivo por padrão via EffectChain
tipada e ordenada; Bake/Expand/Rasterize/Convert-to-Curves só explícitos).
Primeiro modificador vivo: `ContourOffset`. Supersede: nada (primeiro ADR de EffectChain).

</aside>

# Pergunta de decisão

Como todas as edições — vetor e bitmap, presentes e futuras — ficam
não-destrutivas sem travar a velocidade das ferramentas?

# Contexto

Toda edição transformadora reescrevia a geometria-fonte: `offset_path` inflava
bounds e escalava vértices no lugar, cantos colapsavam quatro raios em um,
sculpt substituía paths sem rastro. Undo restaurava, mas undo é rede de
segurança, não modelo vivo. A arquitetura exigia EffectChain tipada, mas nenhum
`EffectChain` ou `LiveModifier` existia em `crates/`.

# Restrições

- `AGENTS.md:9`, `01:75`, `02:29`, `03:31`: não-destrutivo por padrão; Bake e
  cia. só explícitos.
- F-01 (um gesto, um undo), F-21 (tolerância explícita; booleanos exatos POST_V1),
  arquivos `NATIVE_SCHEMA_VERSION = 1` continuam abrindo.
- Sem acesso lateral entre features; UI recebe DTOs.

# Opções

1. **Fundação agora, Contour primeiro (escolhida).** `ModifierKind` tipado +
   cadeia ordenada em `DocumentObject`, doutrina base-vs-avaliada, matemática
   real de offset, comandos `SetModifiers`/`BakeContour`, ferramenta Contour
   migrada, retângulos paramétricos por canto com fix de render.
2. **Híbrida (rejeitada por ora).** ADR agora, código depois. Rejeitada: cada
   batch novo aumentaria a dívida que a correção veio estancar.
3. **Só diretriz (rejeitada).** Adiaria a própria correção.

# Decisão

- `petunia_design_document::modifiers`: `ModifierItem { id, kind, enabled }`,
  `ModifierKind::ContourOffset { distance, join, cap }`; novos tipos estendem o
  enum, nunca uma operação destrutiva genérica.
- `DocumentObject.modifiers` (`#[serde(default)]`: arquivos v1 abrem sem migração).
- **Doutrina base vs. avaliada:**

  | Leitor | Lê | Por quê |
  |---|---|---|
  | Edição de nós/Pen/Pencil, `convert_to_curves` | `to_path()` (base) | ferramentas editam a fonte |
  | Render, hit-test, seleção, booleanos, exportação, preview | `evaluated_path()` / `evaluated_bounds()` | todos veem a mesma geometria viva |
  | `BakeContour` (só explícito) | avaliada → `Path` base, limpa entradas | o único congelamento permitido |

- Matemática honesta: expansão segue a borda externa do stroke (preserva curvas);
  inset erosiona pelo motor de offset (sem spikes; curvas achatam em 0.25pt, F-21).
  Colapso mantém o resultado anterior em vez de destruir.
- `Change::ModifiersChanged` passa por revert e replay. `OffsetPath` legado virou
  upsert do modificador (mesmos chamadores, zero destruição). `SetModifiers`
  commita cadeias em um undo.
- Corner: raios paramétricos por canto em retângulos (`[TL, TR, BR, BL]`),
  render `rect_corners` honra os quatro (antes só o índice 0), clamp físico, um undo.
  Não-retângulos ficam intactos — conversão continua ação explícita.
- Contour: arrasto commita um offset vivo por objeto selecionado em um undo, com
  preview do contorno pendente; botão `Bake Contour` espelha Bake Corners.

# Consequências

- Geometria, documento, comandos/undo, bridge, seleção/propriedades, canvas/SVG/PDF:
  ver seção equivalente no canônico en-US.
- Auditoria de 51 trechos normativos: as páginas de doutrina já exigiam isto;
  nenhuma página contradiz a decisão.

# Pendências (não bloqueiam)

- `CornerType` por canto (chamfer/concave): exige modelo + builder.
- Inset exato em curvas: expansão preserva, inset achata (F-21). Unificar POST_V1.
- Preview vivo de arrasto para todas as ferramentas.
- ND raster (adjustments, filtros vivos, máscaras) quando o pixel sair do stub.

# Gatilho de revisita

Um segundo tipo de modificador deve generalizar identidade/ordem na UI e
confirmar custo linear de avaliação. Só gargalo medido ou migração de formato
podem superseder este ADR, explicitamente.
