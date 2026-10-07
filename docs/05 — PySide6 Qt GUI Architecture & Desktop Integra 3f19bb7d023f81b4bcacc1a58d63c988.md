# 05 — PySide6/Qt GUI Architecture & Desktop Integration

# Shell

Qt Widgets é a baseline do desktop profissional: QMainWindow, QAction/QShortcut, menus, model/view, accessibility e windowing. A aparência é proprietária do Petunia Design System, não o estilo default do Qt.

# Docking

QDockWidget pode ser usado como primeiro adapter, mas a autoridade é WorkspaceDockModel próprio: DockTree, PanelId, splits, tabs, floats e monitor placement. Assim o backend de docking pode evoluir sem alterar plugins, documento ou workspace schema.

# Canvas

CanvasWidget/CanvasWindow é uma view nativa sobre RenderSession C++. Eventos são normalizados para PointerEvent, PenEvent, KeyChord e Gesture. Canvas não usa QGraphicsScene como documento.

# Tool architecture

ToolController Python recebe input semântico, mantém uma state machine leve e solicita preview/commit ao core. Hot previews como brushes, node hit-testing massivo e snapping rodam em C++.

# Models

QAbstractItemModel adapters projetam Layers, Assets, History e registries. Eles consomem snapshots/diffs; não expõem containers C++ diretamente.

# Signals

Signals/slots servem à presentation layer. Domain events chegam como event batches tipados e são convertidos para Qt signals. Nenhuma cadeia de signals deve substituir Transaction/ChangeSet.

# Themes

QPalette + stylesheet limitado + custom paint/delegates. Tokens são gerados de JSON design tokens. Cores, radius, spacing e metrics não aparecem como literais espalhados.

# Accessibility

Accessible names/descriptions, keyboard focus, high contrast, screen reader semantics, tab order, shortcuts discoverable e non-color states são requisitos de cada componente.

# Platform adapters

Windows: Win32/WinRT where needed; macOS: Cocoa bridges; Linux: Wayland/X11 portal-first. File pickers, URI opening, clipboard e color profile discovery têm interfaces próprias.

# Multi-window

Single-window multi-tab default; detached document windows e floating palettes suportados. DocumentSession pode ter múltiplas views com viewport/selection independentes quando explicitamente configurado.

# Testing

QtTest/pytest-qt para componentes; semantic UI inspection IDs; visual snapshots por platform profile; synthetic pointer/keyboard only em test harness.

[05.1 — QMainWindow Shell, Menus, Tabs, Toolbars, Status & Multi-Document Windows](05%201%20%E2%80%94%20QMainWindow%20Shell,%20Menus,%20Tabs,%20Toolbars,%20S%203f19bb7d023f814f904adb74c7b67934.md)

[05.2 — Custom Docking Model on Qt: QDockWidget Adapter, DockTree, Floating Palettes & Restoration](05%202%20%E2%80%94%20Custom%20Docking%20Model%20on%20Qt%20QDockWidget%20Adap%203f19bb7d023f81718b27ce3dbc36a738.md)

[05.3 — Native Canvas Host, QWindow/QWidget Integration, Swapchain Lifecycle & Frame Scheduling](05%203%20%E2%80%94%20Native%20Canvas%20Host,%20QWindow%20QWidget%20Integra%203f19bb7d023f812ba72fe4f444a1c8d0.md)

[05.4 — Unified Input System: Mouse, Tablet, Touch, Gestures, Keyboard & Pointer Capture](05%204%20%E2%80%94%20Unified%20Input%20System%20Mouse,%20Tablet,%20Touch,%20%203f19bb7d023f8125b3eeda3c9836b2df.md)

[05.5 — QAction, Shortcut Registry, Command Palette & Context-Sensitive Availability](05%205%20%E2%80%94%20QAction,%20Shortcut%20Registry,%20Command%20Palette%203f19bb7d023f8135844fe9a078abd678.md)

[05.6 — Qt Model/View for Layers, Assets, History, Data Merge, Jobs & Large Collections](05%206%20%E2%80%94%20Qt%20Model%20View%20for%20Layers,%20Assets,%20History,%20%203f19bb7d023f81c297e7f42a34091c53.md)

[05.7 — Property Editor Framework, Generic Inspector Widgets, Validation & Mixed Values](05%207%20%E2%80%94%20Property%20Editor%20Framework,%20Generic%20Inspecto%203f19bb7d023f81d8b72fcd69db197dfa.md)

[05.8 — Clipboard, Drag & Drop, MIME Types, Paste Semantics & Cross-App Interoperability](05%208%20%E2%80%94%20Clipboard,%20Drag%20&%20Drop,%20MIME%20Types,%20Paste%20S%203f19bb7d023f816b9b38d9a7ce28ab23.md)

[05.9 — Native Dialogs, File Portals, System Integration, URLs, Clipboard & Notifications](05%209%20%E2%80%94%20Native%20Dialogs,%20File%20Portals,%20System%20Integr%203f19bb7d023f81399ce1d442fe057449.md)

[05.10 — Accessibility Bridge: QAccessible, Semantic Canvas, Focus, Screen Readers & Keyboard Navigation](05%2010%20%E2%80%94%20Accessibility%20Bridge%20QAccessible,%20Semantic%203f19bb7d023f8121b6b1f7b71ccc0001.md)

[05.11 — Theme Engine, QPalette/QSS, Custom Painting, SVG Icons & Density Scaling](05%2011%20%E2%80%94%20Theme%20Engine,%20QPalette%20QSS,%20Custom%20Paintin%203f19bb7d023f810cb29af20fedd505b4.md)

[05.12 — Qt Event Loop, Native Jobs, Asyncio/RPC Integration & Responsiveness Contract](05%2012%20%E2%80%94%20Qt%20Event%20Loop,%20Native%20Jobs,%20Asyncio%20RPC%20In%203f19bb7d023f8184bb88dc6b46116f61.md)