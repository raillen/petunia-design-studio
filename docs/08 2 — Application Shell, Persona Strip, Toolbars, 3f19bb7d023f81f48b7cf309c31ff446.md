# 08.2 — Application Shell, Persona Strip, Toolbars, Tabs & Status Bar

# Vertical anatomy

1. native title/window frame;
2. platform menu bar;
3. document tab strip;
4. persona/workspace strip + primary toolbar;
5. context toolbar;
6. main workspace;
7. optional bottom dock;
8. status bar.

# Document tabs

Cada tab mostra nome, dirty indicator e estados read-only/recovery quando existirem. Drag reordena; overflow preserva legibilidade. Context menu: Close, Close Others, Close Right, Duplicate View, Move to New Window, Reveal in File Manager. Fechamento dirty abre Save/Discard/Cancel com nome do documento.

# Persona strip

Design e Photo ficam visualmente separados de tabs. Persona é workspace mode, não arquivo. Clique/shortcut troca tool registry e panel preset sem converter conteúdo. Tooltip explica o efeito da troca.

# Primary toolbar

Até três grupos visuais: history/navigation, document/global operations, view/export. Undo/redo exibem tooltip com próximo command quando conhecido. Toda action tem QAction/ActionId correspondente.

# Context toolbar

Muda por tool/selection, mantendo positions estáveis por família. Exemplos:

- Move: X/Y/W/H, anchor, rotation, lock aspect, snapping;
- Pen/Node: mode, node type, join/break, close, snap;
- Shape: numeric live parameters, fill/stroke summary;
- Text: font, style, size, alignment, leading;
- Brush: preset, size, hardness, opacity, flow, stabilizer;
- Selection: mode, feather, anti-alias, refine.

Overflow nunca remove acesso funcional.

# Status bar

Left: Surface/page, units, document color mode. Center: contextual tool hint/progress/selection summary. Right: zoom, Snap/Grid/Guides, Proof Colors, Pixel Preview, background jobs.

# Focus Canvas

Tab shortcut pode ocultar chrome/panels; Esc ou Tab retorna. Unsaved state e blocking progress continuam acessíveis.

# Multi-window

Tabs são default. "Move to New Window" cria nova view/session shell sem duplicar documento. Floating palettes pertencem a workspace/window.