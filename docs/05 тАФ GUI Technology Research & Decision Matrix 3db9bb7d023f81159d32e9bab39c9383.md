# 05 — GUI Technology Research & Decision Matrix

<aside>
🎯

**Decisão Oficial (Setembro 2026):** **Slint é a interface primária e canônica do Aubrieta Design Suite.** O **egui** permanece como interface experimental secundária e workbench técnico de diagnóstico. O **Iced** é mantido ativamente como reserva estratégica e contingência de longo prazo (salvaguarda de licenciamento MIT). O **GPUI foi formalmente descontinuado**, juntamente com Floem e Xilem. O core permanece estritamente GUI-agnóstico através do `AubrietaGuiBridge`.

</aside>

# Critérios de Seleção Aubrieta

Prioridade máxima:
1. **Compatibilidade de Hardware no Mundo Real**: Execução fluida em GPUs modernas e legadas (Intel Ivy Bridge HD 4000 com OpenGL 4.2 / Wayland / X11), sem dependência exclusiva de Vulkan 1.2+ não-conformante.
2. **Separação Arquitetural Rígida (`AGENTS.md`)**: Domínio e motores jamais importam tipos de toolkit GUI. A UI recebe DTOs/view models e despacha `ActionRequest`/`CommandRequest` pelo `AubrietaGuiBridge`.
3. **Ergonomia e Produtividade Visual (DX)**: Facilidade para iterar rapidamente no visual de inspectors, toolbars e design system sem gargalos de compilação.
4. **Canvas Vetorial Interativo**: Capacidade de integrar renderização de alta performance, manipulação de caminhos Bézier e zoom/pan infinitos.
5. **Segurança Jurídica e Continuidade**: Proteção contra riscos de licenciamento e dependência excessiva de ferramentas monolíticas de terceiros.

---

# Decisão Oficial de GUI — Slint Primário, Egui Secundário, Iced Contingência

### 1. Shell Primária: Slint (`apps/aubrieta-slint`)
- **Papel**: Interface oficial de produção do usuário final.
- **Tecnologia**: DSL declarativa (`.slint`) compilada estaticamente para Rust, modelo reativo de propriedades e callbacks.
- **Renderização**: Múltiplos backends intercambiáveis: **FemtoVG (OpenGL 4.2/3.3)** com aceleração por hardware, **Skia**, e **Software Renderer** nativo (CPU puro).
- **Vantagem Competitiva**: **Live Preview em tempo real** via extensão de editor, permitindo desenhar inspectors, barras de ferramentas e componentes do Design System sem necessidade de recompilar o código Rust a cada ajuste cosmético.
- **Hardware**: Validado com 100% de estabilidade e fluidez imediata na GPU Intel HD 4000.

### 2. Shell Secundária / Experimental: Egui (`apps/aubrieta-egui`) — APOSENTADA
- **Status**: app removido do workspace; Slint é a interface primária e única.
- **Papel anterior**: Workbench de diagnóstico técnico, depuração profunda e ferramentas para desenvolvedores/agentes.
- **Tecnologia**: Immediate-mode via `eframe` configurado com backend `glow` (OpenGL).
- **Recursos**: Painéis de telemetria em tempo real, visualizador de dirty rects, árvore de inspeção semântica do grafo de nós, métricas de memória de rasterização e injeção de comandos MCP.
- **Hardware**: Validado a 60 FPS contínuos no driver `crocus` OpenGL 4.2.

### 3. Contingência Estratégica: Iced (`apps/aubrieta-iced`) — APOSENTADA
- **Status**: app removido do workspace; Slint segue primária, egui como secundária leve pós-V1.
- **Papel anterior**: Salvaguarda de arquitetura e licenciamento de longo prazo.
- **Tecnologia**: The Elm Architecture (TEA), 100% Rust fortemente tipado, licença **MIT irrestrita**.
- **Canvas de 1ª Classe**: Possui o módulo `iced::widget::canvas` nativo para manipulação vetorial direta.
- **Garantia**: Caso ocorra qualquer impasse futuro quanto ao licenciamento do Slint (GPLv3 / termos comerciais), o Iced é o caminho de transição imediato, pois reflete exatamente o mesmo fluxo `Message -> ActionRequest -> CommandRequest -> ChangeSet` do core do Aubrieta.

---

# Tecnologias Descontinuadas

### GPUI (Descontinuado)
- **Motivo Técnico**: O GPUI no Linux apoia-se estritamente na crate `blade-graphics` sobre **Vulkan**. Em placas gráficas integradas mais antigas (como Intel Ivy Bridge HD 4000), o driver Vulkan (`hasvk`) não possui conformidade (`conformanceVersion = 0.0.0.0`) e falha na inicialização de adaptadores.
- **Motivo de Manutenção**: O GPUI é o motor interno proprietário do editor Zed, sem ecossistema auxiliar independente de widgets no `crates.io`, com quebras de API recorrentes a cada versão do Zed.

### Floem e Xilem (Descontinuados)
- **Floem**: Descontinuado após falha de adaptador WGPU por requisito exclusivo de Vulkan moderno.
- **Xilem / Vello**: Descontinuado devido à dependência mandatória de compute shaders Vulkan 1.2+ do motor Vello.

---

# Tabela Histórica e Comparativa de Candidatos

| Candidato | Papel no Aubrieta | Licença | Backend Gráfico | Veredito |
| :--- | :--- | :--- | :--- | :--- |
| **Slint** | **Primário (Oficial)** | GPLv3 / Comercial | FemtoVG (OpenGL) / Skia / Software | **Adotado**. Live preview veloz, estável 1.x, roda suave em OpenGL legado e moderno. |
| **Egui** | **Secundário (Dev/Debug)** | MIT / Apache-2.0 | Glow (OpenGL) | **Adotado**. Ideal para workbench, diagnóstico, inspeção e testes rápidos. |
| **Iced** | **Contingência (Fallback)** | MIT (Livre) | WGPU + TinySkia (CPU) | **Mantido Ativo**. TEA pura, canvas vetorial nativo, proteção estratégica de licença. |
| **GPUI** | *Descontinuado* | Apache-2.0 | Blade-Graphics (Vulkan estrito) | **Descontinuado**. Falha em drivers Vulkan legados; dependência fechada ao Zed. |
| **Floem** | *Descontinuado* | MIT / Apache-2.0 | WGPU | **Descontinuado**. Incompatibilidade de hardware em GPUs legadas. |
| **Xilem** | *Descontinuado* | Apache-2.0 | Vello / Vulkan 1.2+ | **Descontinuado**. Vello exige compute shaders modernos ausentes em Ivy Bridge. |

---

# Regra Imutável: GUI Boundary

Mesmo com a escolha do **Slint** como frontend primário, a regra de ouro do `AGENTS.md` permanece absoluta:
> **Nenhum tipo de Slint, Egui ou Iced cruza a fronteira para os crates de domínio (`aubrieta_document`, `aubrieta_geometry`, `aubrieta_color`, `aubrieta_text`, `aubrieta_raster`, `aubrieta_render`, `aubrieta_commands`).**

Toda a interação flui exclusivamente via DTOs/view-models e eventos semânticos despachados para a porta central [`AubrietaGuiBridge`](file:///home/raillen/Documentos/Projetos/aubrieta-design/crates/aubrieta_shell/src/bridge/gui_bridge.rs).