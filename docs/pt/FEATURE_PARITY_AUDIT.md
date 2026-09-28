# Auditoria de Cobertura e Paridade de Recursos (Personas Vetor e Foto)

Este documento apresenta a auditoria canônica de todas as ferramentas, modificadores não-destrutivos, filtros, ajustes, configurações globais e painéis de inspeção nas Personas **Vetor (Design)** e **Foto (Raster)** do Petunia.

Todas as classificações seguem rigorosamente a **Taxonomia Canônica de Escopo (ADR 12.8)**:
- `V1 Required`: Mandatório para a versão 1.0 do produto.
- `Milestone Required`: Requisito obrigatório para o marco atual de integração da interface.
- `Post-V1 Candidate`: Totalmente especificado, mas planejado para após a versão 1.0.
- `Out of Scope`: Fora do escopo do produto atual.

---

## 1. Ferramentas Interativas — Persona Vetor (Design Persona)

A Persona Vetorial gerencia curvas Bézier, nós, formas paramétricas, tipografia, operações booleanas e modificadores geométricos não-destrutivos.

| Ferramenta (`ToolKind`) | Ação Canônica (`ActionId`) | Escopo | Estado no Motor/Shell | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Select / Move** | `ptnd.tool.select` | `V1 Required` | ✅ Implementado | ✅ Exposto | Finalizar alças de rotação interativa nos cantos da caixa delimitadora e seleção por cruzamento. |
| **Node (Nós)** | `ptnd.tool.node` | `V1 Required` | ✅ Implementado | ✅ Exposto | Expor botões de conversão de nós (Cúspide, Suave, Simétrico) na Context Toolbar dinâmica. |
| **Point Transform** | `ptnd.tool.point_transform` | `Post-V1 Candidate` | ⚠️ Incorporado | ⚠️ Incorporado | Dobrado no HUD de transformação padrão pelo ADR 08.33; não requer botão avulso no rail. |
| **Pen (Caneta)** | `ptnd.tool.pen` | `V1 Required` | ✅ Implementado | ✅ Exposto | Adicionar modos secundários no HUD de contexto (Modo Inteligente, Polígono e Linha). |
| **Pencil (Lápis)** | `ptnd.tool.pencil` | `V1 Required` | ✅ Implementado | ✅ Exposto | Adicionar slider de tolerância de encaixe de curvas (Ramer-Douglas-Peucker) e estabilizador de traço no HUD. |
| **Corner (Cantos)** | `ptnd.tool.corner` | `V1 Required` | ✅ Implementado | ✅ Exposto | Expor controles numéricos de raio por nó no inspetor e comando "Bake Corner Geometry". |
| **Contour (Contorno)** | `ptnd.tool.contour` | `V1 Required` | ✅ Implementado | 🟡 No Flyout | Botão presente no grupo "modify"; falta alça de arraste radial sobreposta ao canvas. |
| **Perspective (Perspectiva)** | `ptnd.tool.perspective` | `V1 Required` | ✅ Implementado | ✅ Exposto | Desenhar alças interativas dos 4 vértices do quad no overlay do canvas. |
| **Knife (Faca)** | `ptnd.tool.knife` | `V1 Required` | ✅ Implementado | ✅ Exposto | Renderizar linha guia tracejada de corte e trava de ângulo com `Shift`. |
| **Scissors (Tesoura)** | `ptnd.tool.scissors` | `V1 Required` | ✅ Implementado | 🟡 No Flyout | Cursor em tesoura com atração magnética ao nó mais próximo do caminho. |
| **Rectangle (Retângulo)** | `ptnd.tool.shape.rectangle` | `V1 Required` | ✅ Implementado | ✅ Exposto | Edição individual do raio dos 4 cantos no painel e no HUD. |
| **Ellipse (Elipse)** | `ptnd.tool.shape.ellipse` | `V1 Required` | ✅ Implementado | ✅ Exposto | Controles paramétricos de ângulos de setor/torta e anel (Pie e Donut) no HUD. |
| **Polygon (Polígono)** | `ptnd.tool.shape.polygon` | `V1 Required` | ✅ Implementado | ✅ Exposto | Controle numérico de lados (3 a 32) na Context Toolbar dinâmica. |
| **Star (Estrela)** | `ptnd.tool.shape.star` | `V1 Required` | ✅ Implementado | ✅ Exposto | Controle de número de pontas e proporção do raio interno na Context Toolbar. |
| **Line (Linha)** | `ptnd.tool.line` | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Desenho de linha absorvido pela ferramenta Caneta em modo linha para a V1. |
| **Shape Builder** | `ptnd.tool.shape_builder` | `V1 Required` | ⚠️ Core Parcial | 🟡 No Flyout | Pré-visualização de região destacada no hover e síntese booleana de partições planares. |
| **Vector Flood Fill** | `ptnd.tool.vector_flood_fill` | `V1 Required` | ⚠️ Core Parcial | 🟡 No Flyout | Detecção de limites planares para preenchimento de regiões fechadas sem união destrutiva. |
| **Artistic Text** | `ptnd.tool.text.artistic` | `V1 Required` | ✅ Implementado | ✅ Exposto | Seleção e edição in-place de trechos de texto diretamente no canvas. |
| **Frame Text (Texto de Caixa)** | `ptnd.tool.text.frame` | `V1 Required` | ✅ Implementado | 🟡 No Flyout | Diagramação de parágrafos com quebra de linha automática no retângulo delimitador. |
| **Gradient (Gradiente)** | `ptnd.tool.gradient` | `V1 Required` | ✅ Implementado | ✅ Exposto | Linha de vetor interativa [início, fim] com alças de paradas de cor (stops) no canvas. |
| **Transparency (Transparência)** | `ptnd.tool.transparency` | `V1 Required` | ✅ Implementado | 🟡 No Flyout | Linha de vetor de opacidade com alças de gradiente alfa no canvas. |
| **Color Picker (Conta-gotas)** | `ptnd.tool.color_picker` | `V1 Required` | ✅ Implementado | ✅ Exposto | Lente de aumento (loupe 9x9 pixels) sob o cursor durante a amostragem no canvas. |
| **Style Picker (Conta-estilos)** | `ptnd.tool.style_picker` | `V1 Required` | ⚠️ Core Parcial | 🟡 No Flyout | Filtro seletivo de propriedades a transferir (apenas traço, preenchimento, efeitos ou texto). |
| **Vector Brush (Pincel Vetorial)** | `ptnd.tool.vector_brush` | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Aplicação de pontas texturizadas ao longo de caminhos vetoriais. |
| **Artboard / Surface** | `ptnd.tool.artboard` | `V1 Required` | ✅ Implementado | ✅ Exposto | Alças de redimensionamento da prancheta e seletores de predefinição (A4, 1080p, Mobile). |
| **Measure (Medição)** | `ptnd.tool.measure` | `V1 Required` | ✅ Implementado | ✅ Exposto | Linhas de cota temporárias na tela (distância Euclidiana, DX, DY e ângulo). |
| **Zoom & Hand (Navegação)** | `ptnd.tool.zoom` / `pan` | `V1 Required` | ✅ Implementado | ✅ Exposto | Ferramentas de navegação plenamente funcionais. |

---

## 2. Ferramentas Interativas — Persona Raster (Photo Persona)

A Persona Foto gerencia pintura raster, seleções por mapa de bits, máscaras e alterações no nível dos pixels.

| Ferramenta (`ToolKind`) | Ação Canônica (`ActionId`) | Escopo | Estado no Motor | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Marquee Rect** | `ptnd.tool.photo.marquee_rect` | `V1 Required` | ✅ Implementado | 🟡 No Rail Foto | Renderizar contorno pontilhado animado ("marching ants") no canvas Vello. |
| **Marquee Ellipse** | `ptnd.tool.photo.marquee_ellipse` | `V1 Required` | ✅ Implementado | 🟡 No Rail Foto | Renderizar elipse animada com marching ants no canvas. |
| **Lasso (Laço)** | `ptnd.tool.photo.lasso` | `V1 Required` | ✅ Implementado | 🟡 No Rail Foto | Fechamento automático no mouse up e rasterização da máscara poligonal. |
| **Selection Brush** | `ptnd.tool.photo.selection_brush` | `V1 Required` | ⛔ Bloqueado | 🟡 No Rail Foto | Requer algoritmo de atração por bordas (Edge-Snapping) no buffer de seleção. |
| **Flood Select (Varinha)** | `ptnd.tool.photo.flood_select` | `V1 Required` | ⛔ Bloqueado | 🟡 No Rail Foto | Requer preenchimento por inundação com tolerância de cor conectado ao buffer raster. |
| **Pixel Paint Brush** | `ptnd.tool.photo.brush` | `V1 Required` | ⛔ Bloqueado | 🟡 No Rail Foto | Requer pipeline de camadas raster em mosaico (`PixelLayer`) para pintura destrutiva. |
| **Pixel Eraser (Borracha)** | `ptnd.tool.photo.eraser` | `V1 Required` | ⛔ Bloqueado | 🟡 No Rail Foto | Requer pipeline de camadas raster para remoção de alfa e cores RGBA. |
| **Photo Gradient** | `ptnd.tool.photo.gradient` | `V1 Required` | ✅ Implementado | 🟡 No Rail Foto | Renderização de gradiente diretamente na máscara de seleção ou camada de pixels. |
| **Crop (Corte)** | `ptnd.tool.photo.crop` | `V1 Required` | ✅ Implementado | 🟡 No Rail Foto | Restrições de proporção (1:1, 16:9, Livre) e corte não-destrutivo do canvas. |
| **Clone / Healing / Inpainting** | — | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Retoque fotográfico avançado e síntese de textura (Post-V1). |
| **Dodge / Burn / Smudge / Blur** | — | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Pincéis de exposição, queima e desfoque localizado (Post-V1). |

---

## 3. Modificadores Vivos Não-Destrutivos (ADR 09.31)

Modificadores transformam a geometria e a renderização em tempo de execução sem destruir os nós e parâmetros originais.

| Modificador (`ModifierKind`) | Escopo | Motor (`petunia_design_document`) | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- |
| **ContourOffset** | `V1 Required` | ✅ Implementado | ⚠️ Parcial | Expor estilos de junção (Miter, Round, Bevel) no inspetor de Propriedades. |
| **TransparentGradient** | `V1 Required` | ✅ Implementado | ⚠️ Parcial | UI para configurar múltiplos pontos de parada de opacidade ao longo do vetor. |
| **Perspective** | `V1 Required` | ✅ Implementado | ✅ Exposto | Alças de tela interativas nos 4 cantos com malha 3x3 e ação de fixação (bake) no HUD de contexto. |
| **CropRect** | `V1 Required` | ✅ Implementado | ⚠️ Parcial | Recorte retangular não-destrutivo de caixas delimitadoras. |
| **Warp / Envelope Mesh** | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Deformação por malha Bézier livre NxN. |
| **Modifier Stack Inspector** | `V1 Required` | ⚠️ Pronto no núcleo | ❌ Ausente | Seção no Dock para listar, reordenar, ocultar e dar **Bake** (converter em curvas). |

---

## 4. Filtros Vivos e Efeitos de Camada (Layer FX — 10.4 & 10.10)

| Filtro / Efeito (`EffectKind`) | Família | Escopo | Motor | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Drop Shadow (Sombra Externa)** | Layer FX | `V1 Required` | ✅ Implementado | ⚠️ Apenas presets | Editor completo: deslocamento (dx, dy), raio de desfoque, cor e opacidade. |
| **Inner Shadow (Sombra Interna)** | Layer FX | `V1 Required` | ✅ Implementado | ⚠️ Apenas presets | Editor completo confinado ao preenchimento do objeto. |
| **Gaussian Blur (Desfoque)** | Live Filter | `V1 Required` | ✅ Implementado | ⚠️ Apenas presets | Slider de raio com opção de máscara de exclusão. |
| **Sharpen (Nitidez)** | Live Filter | `V1 Required` | ✅ Implementado | ✅ Exposto | Modelado no motor com controles interativos de raio e intensidade no dock de propriedades. |
| **Noise (Ruído)** | Live Filter | `V1 Required` | ✅ Implementado | ✅ Exposto | Modelado no motor com controles de intensidade e alternador monocromático/colorido no dock. |
| **Outer Glow / Inner Glow** | Layer FX | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Brilho radial uniforme em 360 graus. |
| **Bevel & Emboss (Chanfro)** | Layer FX | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Iluminação simulada de relevo tridimensional nas bordas. |
| **Liquify (Distorção Líquida)** | Persona Retoque | `Out of Scope` | ❌ Não modelado | ❌ Ausente | Fora do escopo da v1 (requer shaders de computação gráfica). |

---

## 5. Camadas de Ajuste Tonais e Análise de Imagem (Spec 10.10)

| Recurso de Ajuste / Análise | Escopo | Motor | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- |
| **Levels (Níveis)** | `V1 Required` | ✅ Implementado | ✅ Exposto | Função de transferência não-destrutiva, gama e recorte preto/branco com controles no dock. |
| **Curves (Curvas)** | `V1 Required` | ✅ Implementado | ✅ Exposto | Spline Monotone Cubic Hermite com presets (Curva S, Linear, Alto Contraste) e avaliação precisa. |
| **HSL** | `V1 Required` | ✅ Implementado | ✅ Exposto | Rotação de matiz (-180° a +180°), saturação e luminância em tempo real. |
| **Exposure (Exposição)** | `V1 Required` | ✅ Implementado | ✅ Exposto | Multiplicador de paradas EV, deslocamento de preto e expoente de gama. |
| **White Balance (Balanço Branco)**| `V1 Required` | ✅ Implementado | ✅ Exposto | Ajustes cromáticos de temperatura (quente/frio) e matiz (verde/magenta). |
| **Histogram (Histograma)** | `V1 Required` | ⚠️ Cálculo em CPU | ❌ Ausente | Widget gráfico dinâmico com distribuição de luminância e canais RGB. |
| **Channel View (Canais)** | `V1 Required` | ⚠️ Lógica base | ❌ Ausente | Inspeção não-mutante de componentes Vermelho, Verde, Azul e Alfa. |
| **Soft Proofing (Prova de Cor)** | `V1 Required` | ✅ Em `petunia_color`| ❌ Ausente | Alternador na barra/menu de visualização simulando perfis ICC de impressão. |

---

## 6. Configurações Globais, HUD de Ferramentas e Painéis

| Área / Recurso | ID de Superfície | Escopo | Motor | Estado na UI Freya | Lacuna de Implementação / Trabalho Restante |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Janela de Preferências** | `ptnd.window.preferences` | `Post-V1 Candidate` | ⚠️ Parcial | 🟡 CustomizeDialog | Formulário de configurações com persistência em `preferences.toml`. |
| **Diálogo de Novo Documento** | `ptnd.dialog.new_document` | `V1 Required` | ✅ Implementado | 🟡 Modal Existente | Incluir opções de sangria (bleed), margens e espaços de cor. |
| **Diálogo de Exportação** | `ptnd.dialog.export` | `V1 Required` | ✅ Implementado | 🟡 Modal Existente | Seleção de resolução (DPI 72/150/300), perfil ICC e seleção de pranchetas. |
| **Painel Camadas** | `ptnd.panel.layers` | `V1 Required` | ✅ Implementado | ✅ Exposto | Suporte a reordenamento por arrastar e soltar (drag & drop) e renomeação inline. |
| **Inspetor de Propriedades** | `ptnd.panel.properties` | `V1 Required` | ✅ Implementado | ✅ Exposto | Integrar estilos de traço avançados, controle tipográfico e pilha de efeitos (FX). |
| **Cores e Amostras** | `ptnd.panel.color` / `swatches` | `V1 Required` | ✅ Implementado | ✅ Exposto | Controles deslizantes (RGB, HSL, CMYK) junto às paletas de amostras. |
| **Painel Histórico** | `ptnd.panel.history` | `V1 Required` | ✅ Implementado | ✅ Exposto | Navegação e ramificação temporal não-destrutiva de estados de desfazer. |
| **Painel Navegador** | `ptnd.panel.navigator` | `Post-V1 Candidate` | ✅ Implementado | ✅ Exposto | Miniatura com retângulo indicador de viewport navegável. |
| **Painel de Ativos (Assets)** | `ptnd.panel.assets` | `Post-V1 Candidate` | ❌ Não modelado | ❌ Ausente | Biblioteca de componentes gráficos reutilizáveis para arrastar ao canvas. |
| **Mesclagem de Dados** | `ptnd.panel.data_merge` | `Post-V1 Candidate` | ✅ Core em 10.11 | ⛔ Desabilitado | Interface para carregar CSV/JSON e vincular dados a nós de documento. |
| **HUD de Opções de Ferramenta** | `ptnd.surface.context_toolbar` | `V1 Required` | ✅ Implementado | ✅ Exposto | Opções dinâmicas no topo (pontas da estrela, lados do polígono, Contorno, Perspectiva, Medição, Gradiente, Texto, Seleção). |
