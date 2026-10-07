# 08.3 — Docking, Studio Panels, Floating Palettes & Workspace Presets

# Authority

Docking é **workspace state**, nunca document state. A serialização usa WorkspaceDockModel próprio, ainda que o adapter V1 use QMainWindow/QDockWidget.

# Regions

left, right, bottom, center canvas, floating palette layer, detached window layer.

# Dock tree

SplitNode(horizontal|vertical, ratio, children) + TabStack(panelInstances, active) + PanelLeaf. IDs são PanelId + instance UUID. Layout desconhecido/provider ausente deve sobreviver como placeholder metadata.

# Drag behavior

Arrastar tab/panel mostra ghost, legal drop targets, tab insertion e split preview. Esc cancela e restaura posição. Drop fora cria floating palette se permitido. Keyboard commands: Dock Left/Right/Bottom, Float, Move to Next Group.

# Panel sizing

Panel registra min/preferred/max optional, shrink priority, allowed orientations e multi-instance. Baseline: Layers ~300 px, Properties ~320, Color ~280, History ~280, Assets ~360 — tokens/preferred, nunca hard constraints.

# Tabs

Reorder, overflow, middle-click optional close, context menu Float/Dock/Move/Close/Reset. Active state usa indicator + text contrast.

# Floating

Mesma component tree do docked panel, janela utility não modal, monitor-aware restore, pin within-app optional.

# Collapse

Dock pode recolher para icon rail ou zero-width reveal strip. Hover preview é opcional; click fixa.

# Workspace presets

Design Default, Design Compact, Photo Default, Photo Retouch, Minimal Canvas, Dual Monitor. User can Save As, Update, Duplicate, Rename, Delete user preset, Reset built-in, Import/Export.

# Persistence

Workspace schema armazena dock tree, panels, floats, toolbars, persona association, visibility, density override e monitor metadata. Não armazena seleção/document data.

# Missing plugin panel

Se plugin some: manter placement metadata, exibir unavailable placeholder/diagnostic, não quebrar splits, restaurar automaticamente quando provider retorna.