# P03 Dossier — Sistema de Extensão, Recursos e Plataforma (P03-G01 a P03-G04)

## Status / scope

- P03-G01 `REVIEWING` (Pacotes de Recursos, Tokens e i18n — DTCG tokens, temas Light/Dark, ciclo de alias, catálogo en-US e pt-BR, IconMap)
- P03-G02 `REVIEWING` (Portas e Adaptadores de Plataforma — ClipboardService, FileDialogService, EnvironmentService e adaptadores Headless)
- P03-G03 `REVIEWING` (Runtime e Host de Extensões — API neutra de plugins, capability broker e sandbox Lua 5.4/5.5 via `mlua`)
- P03-G04 `PLANNED` (Servidor e Ferramentas MCP — automação e inspeção via Model Context Protocol)
- Escopo: `V1_REQUIRED` (camada de plataforma, recursos e desacoplamento do SO).

## Goal e outcomes alcançados nesta onda

1. **Recursos e Tokens (`aubrieta_resources`)**:
   - Modelo de design tokens compatível com DTCG (primitivos, semânticos e componentes).
   - Resolução transitiva de aliases com detecção estrita de referências circulares (`CircularAlias`).
   - Suporte nativo a temas (`Light` e `Dark`) com sobrescrita controlada de tokens semânticos (`surface.canvas`, `text.primary`, `border.subtle`).
   - Módulo de internacionalização (`i18n`): chaves estáveis `TextId`, catalogo canônico em `en-US` com espelho obrigatório em `pt-BR`, interpolação nomeada de parâmetros (`{count}`, `{format}`) e mecanismo defensivo de fallback.
   - Pacote de recursos canônico (`ResourcePack::core_pack()`) integrando tokens, strings localizadas e mapa semântico de ícones (`IconMap`).

2. **Serviços de Plataforma (`aubrieta_platform`)**:
   - `ClipboardService`: abstração de área de transferência para texto puro, SVG vetorial, bitmaps RGBA8 e fragmentos estruturados de documento.
   - `FileDialogService`: filtros de extensão tipados (`AubrietaPackage`, `SvgVector`, `RasterImage`, `PdfDocument`) para diálogos de abertura e salvamento.
   - `EnvironmentService`: leitura de métricas do sistema operacional (HiDPI scale factor, modo escuro, idiomas preferidos).
   - Adapters *Headless* determinísticos (`HeadlessClipboard`, `HeadlessFileDialog`, `HeadlessEnvironment`) que garantem testes e automação em CI sem necessidade de servidor gráfico (X11/Wayland/Windows).

3. **Sistema de Extensões & Plugins (`aubrieta_extension`)**:
   - Manifestos de plugins (`PluginManifest`) com taxonomia fina de permissões (`DocumentRead`, `DocumentWrite`, `Clipboard`, `ScopedStorage`, `Network`).
   - `CapabilityBroker`: autorização em tempo de execução com razão descritiva de recusa, garantindo a invariante "capability ausente nunca causa pânico".
   - `PluginHost`: runtime isolado em Lua 5.4 via `mlua` com stripping de bibliotecas perigosas do SO (`os`, `io`, `debug`, `package.loadlib`).
   - API de script `aubrieta`: consultas de documento e solicitações de mutação emitindo `ActionRequest` canônicos.

4. **Conformidade E2E e Gauntlet (`apps/aubrieta-cli` & `xtask`)**:
   - `aubrieta-cli` expandido para os passos 10, 11 e 12 (resolução de tokens, i18n bilíngue, clipboard e execução de ação via plugin Lua em sandbox).
   - Regras de arestas proibidas em `xtask` estendidas para `aubrieta_resources`, `aubrieta_platform` e `aubrieta_extension`.

## Evidência de verificação P03

- `cargo test --workspace`: **76 passed** (60 testes unitários + 16 testes de propriedade `proptest`), 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: **0 warnings** (limpo).
- `cargo fmt --all --check`: **100% formatado**.
- `cargo xtask gauntlet`: **verde total** (fmt, clippy, testes unitários, testes de propriedade, arestas arquiteturais, fixtures e conformidade CLI).
- `prumo doctor .`: **todos os checks passaram sem erros**.

## Handoff e Próximos Passos (P03 Conclusão)

1. `P03-G04`: Implementação do servidor de inspeção e mutação MCP (`aubrieta_mcp`).
