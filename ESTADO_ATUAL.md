# Estado Atual — Petunia Design Studio

**Data**: 2026-09-30  
**Branch**: `work/f6-f7`  
**Framework de UI**: Freya `0.5.0-rc.7` (Skia / Dioxus)  
**Fonte da verdade**: `DOSSIER_V1_2026-09-30.md`, `crates/petunia_design_application/src/surfaces.rs`

---

## 1. Resumo Executivo

O Petunia Design Studio opera sobre a engine Skia + Freya com rendering direto, pipelines estritamente transacionais (`Action` → `Command` → `DocumentMutator` → `ChangeSet`) e zero UI falsa. Toda superfície e ferramenta marcada como `Wired` está conectada ao modelo e com cobertura de testes automatizados.

Documentos históricos pré-Freya foram arquivados em `archive/` (`PROJECT_STATE_2026-09-20.md`, `HANDOFF_LEGACY.md`, `UI_IMPLEMENTATION_LEDGER_SLINT.md`).

---

## 2. Itens Concluídos e Validados (P0 & P1)

- **F6 Compositor Dirty-Rect**: Compositor com culling de bounds, dirty-rect e snapshot cache O(n) invalidado por `revision`.
- **F7 Cosmic-Text (F7.1–F7.3)**: Font system foundation, layout cacheado e text-on-path funcional.
- **P1-1 Hit Stroke-Aware**: Hit test sensível à proximidade do contorno e zoom em paths abertos via `stroke_hit.rs`.
- **P1-2 Inversão de Seleção (`select.invert`)**: Inversão de seleção de objetos vetoriais e de máscara raster.
- **P1-3 Offset de Contorno (`offset_path`)**: Diálogo modal com prompt numérico e modifier `ContourOffset` dinâmico.
- **P1-4 Multi-Documento e Tab Strip**: Multi-documento nativo com abas interativas no chrome, alternância de documento ativo, proteção contra perda de dados não salvos com `ConfirmCloseDialog` em `file.close` e `file.quit`.
- **P1-5 Painéis Color, Swatches e Background Tasks**:
  - `ptnd.panel.color`: Modos sRGB, CMYK, Lab e Spot, soft-proofing SWOP e isolamento de canais.
  - `ptnd.panel.swatches`: Bibliotecas de Sistema, Documento (extração em tempo real) e Favoritos editáveis.
  - `ptnd.panel.background_tasks`: Orquestração de jobs em segundo plano com progresso, estados e cancelamento determinístico via `JobManager`.
- **P1-6 Inserção de Imagens (`file.place`)**: Ferramenta `ptnd.tool.vector.place_image` promovida a `Wired` acoplada ao pipeline de importação e render Skia.
- **P1-7 Novo Documento (`NewDocumentDialog`)**: Diálogo funcional com presets (Web, Mobile, Print, Custom) e despacho real de `file.new`.
- **P1-8 Conflito de Sobrescrita (`OverwriteConflictDialog`)**: Proteção modal contra substituição destrutiva de arquivos existentes (`ptnd.dialog.overwrite_conflict`).
- **P1-9 Histograma Real de Buffer**: Amostragem e cálculo de bins sobre pixels de objetos raster (`RawRasterImage` Rgba8), com suporte a ajustes de tonalidade e fallback para tokens.
- **P2-1 Minimap Navigator Interativo (`NavigatorTab`)**: Componente `NavigatorTab` (`ptnd.tab.navigator`) renderizando objetos em baixa resolução, retângulo de viewport dinâmico em coordenadas de documento e navegação pan-by-drag interativa.
- **P2-2 Validação Numérica Rígida (`numeric.rs`)**: Módulo de validação numérica estrita (`parse_numeric_input`), rejeita-ou-explica com bounds semânticos e mensagens bilíngues, eliminando coerção silenciosa para `0.0`.
- **P2-3 Docks Flexíveis & Splitters Reais (`LeftDock`, `BottomDock`)**: Doca esquerda (`ptnd.surface.dock.left`) e doca inferior (`ptnd.surface.dock.bottom`) promovidas a `Wired`, com splitters verticais e horizontais baseados no padrão `Portal`, persistência de dimensões/abas, toggles na `StatusBar` e fechamento modal.
- **P2-4 Assets & Symbols Browsing (`AssetsTab`, `SymbolsTab`)**: Painel de ativos do documento (`ptnd.panel.assets`) com listagem e importação (`file.place`), e biblioteca de componentes e formas com inserção direta via `Command`.

---

## 3. Próximos Passos (Dossiê V1)

1. **i18n EN+pt-BR e Catálogo de Textos (P2 - M)**: Migração de literais restantes para chaves `TextId`.
2. **Estabilizador Live (StreamLine + velocidade) (P2 - M)**: Suavização móvel e sensibilidade para traço livre.
3. **Clipboard / Portas Headless (P2 - S)**: Suporte a copiar/colar de objetos entre sessões.

---

## 4. Gates de Verificação

```bash
cargo test -p petunia-design --bin petunia-design
cargo test -p petunia-design --test click_smoke
cargo clippy -p petunia_design_jobs -p petunia_design_application -p petunia_design_shell -p petunia-design --all-targets -- -D warnings
cargo check --workspace
```
