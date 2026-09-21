# 08.15 — GUI Implementation Map & Component Ownership (Slint & Egui)

<aside>
🎯

**Regra de Governança:** O **Slint** é a implementação primária da interface gráfica (`apps/aubrieta-slint`). O **egui** atua como interface secundária leve (pós-V1, low-end) e workbench de depuração técnica (`apps/aubrieta-egui`). O **Iced foi aposentado** (`apps/aubrieta-iced` removido). O **GPUI foi descontinuado**. A biblioteca interna `aubrieta_shell` (ex-`aubrieta_ui_gpui`) atua como a infraestrutura neutra de desacoplamento (`AubrietaGuiBridge` e `AubrietaShell`); sessão, portas e view-models vivem em `aubrieta_application`.

</aside>

# Mapeamento de Responsabilidades por Camada de Interface

| Área | Implementação Primária (Slint) | Implementação Secundária (Egui) | Contingência (Iced) | Propriedade |
| :--- | :--- | :--- | :--- | :--- |
| **Fundação de Janela / Loop** | Slint Runtime (winit/FemtoVG/OpenGL 4.2) | Eframe (glow / OpenGL) | Iced Runtime (winit/wgpu/tiny-skia) | Framework |
| **DSL / Layout de Chrome** | `.slint` declarativo + propriedades tipadas | Código Rust imediato (`egui::Ui`) | The Elm Architecture (`view()`) | **Aubrieta Shell** |
| **Design Tokens & Estilos** | `aubrieta_resources` + Slint global styles | `egui::Visuals` customizado | `iced::theme` customizado | **Aubrieta Design System** |
| **Canvas / Viewport Vetorial** | Framebuffer compartilhado (`SharedPixelBuffer`) / `aubrieta_render` | Textura OpenGL renderizada por `glow`/`tiny-skia` | `iced::widget::canvas::Program` nativo | **Aubrieta Render / Engine** |
| **Barra de Ferramentas (Tool Rail)** | Componentes customizados `.slint` | `egui::Toolbar` com botões de toggle | Widgets de botão Iced mapeados para `Message` | **Aubrieta** |
| **Árvore de Camadas (Layers Panel)** | Slint `ListView` virtualizada | `egui_ltreeview` / `CollapsingHeader` | `iced::widget::scrollable` com lista imutável | **Aubrieta** |
| **Inspetor de Propriedades** | Componentes Slint reutilizáveis | Slint/Egui fields reativos | Widgets de controle tipados Iced | **Aubrieta** |
| **Paleta de Comandos** | Janela modal popup no Slint | `egui_palette` / popup | Modal com text input e lista filtrada | Compartilhado |
| **Tipografia no Documento** | `aubrieta_text` (Cosmic-text / Skrifa) | `aubrieta_text` | `aubrieta_text` / Cosmic-text nativo | **Core (Domínio)** |
| **Seletor de Cores Oklab / P3** | Custom Color Picker em Slint | `egui::color_picker` / shaders | Canvas de gradiente 2D Iced | **Aubrieta Color** |
| **Ícones do Sistema** | SVGs compilados estaticamente no `.slint` | Ícones Phosphor / Lucide integrados | `iced::widget::Svg` | **Aubrieta Assets** |
| **Workbench / Diagnóstico / Telemetria** | Não prioritário (foco na UX final) | **Painel Principal no Egui** (dirty rects, mem, MCP) | Painel auxiliar | **Aubrieta Egui (Dev)** |

---

# Fronteira da Crate `aubrieta_shell` (GUI Bridge & Shell)

A crate `crates/aubrieta_shell` (renomeada de `aubrieta_ui_gpui`; **não depende de GPUI**) define a ponte agnóstica de controle:

```
crates/aubrieta_shell
├── bridge/
│   ├── gui_bridge.rs      (AubrietaGuiBridge unificado)
│   ├── ports.rs           (ActionQueryPort, CommandPort, InspectionPort, etc.)
│   ├── session.rs         (DocumentSession, SelectionSession)
│   └── view_models.rs     (DTOs: LayerRow, PropertyField, ActionState, etc.)
├── canvas/
│   ├── camera.rs          (ViewportCamera com zoom infinito centrado)
│   └── snapping.rs        (SnapEngine com histerese anti-jitter)
├── tools/
│   ├── select_tool.rs     (Seleção, marquee, duplicate drag)
│   ├── pen_tool.rs        (Curvas Bézier e manipulação de tangentes)
│   ├── node_tool.rs       (Edição de nós e âncoras)
│   └── shape_tool.rs      (Retângulos, elipses e polígonos)
├── panels/
│   ├── layers_controller.rs
│   ├── properties_controller.rs
│   ├── history_controller.rs
│   └── data_merge_controller.rs
└── shell/
    ├── desktop_shell.rs   (AubrietaShell orquestrador)
    └── mock_adapter.rs    (MockGuiAdapter headless para CI)
```

Nenhum tipo de toolkit visual (`slint::*`, `egui::*`, `iced::*`) existe dentro dessa crate ou das crates de domínio.

---

# Descontinuação do GPUI

O suporte e pesquisa inicial com GPUI foram oficialmente descontinuados pelos seguintes fatores:
1. **Requisito estrito de Vulkan 1.2+**: Incompatível com máquinas e laptops com GPUs legadas (ex: Intel Ivy Bridge HD 4000), onde o driver Mesa `hasvk` falha por não-conformidade.
2. **Ciclo fechado ao Zed**: Falta de ecossistema independente no `crates.io` e constantes quebras de API internas da Zed Industries.
3. **Substitutos superiores**: O **Slint** atende de forma superior a velocidade de design visual via Live Preview, o **Egui** cobre com excelência os diagnósticos técnicos imediatos, e o **Iced** garante a integridade arquitetural e segurança jurídica sob licença MIT.