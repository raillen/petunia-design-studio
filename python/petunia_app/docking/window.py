from __future__ import annotations

from pathlib import Path
from typing import Any

import PySide6QtAds as ads
from PySide6.QtCore import QByteArray, QEvent, QObject, Qt, QUrl
from PySide6.QtGui import QFontDatabase, QKeySequence
from PySide6.QtQml import QQmlEngine
from PySide6.QtQuickWidgets import QQuickWidget
from PySide6.QtWidgets import (
    QHBoxLayout,
    QMainWindow,
    QVBoxLayout,
    QWidget,
)

from petunia_app.bridge import Bridge
from petunia_app.docking.style import ADS_DARK_THEME


class TitleBarButtonFilter(QObject):
    """Event filter that hides dock buttons when docked and shows them when floating.

    Per professional desktop standards (Affinity / Photoshop), docked panels show
    only clean tabs. Title bar buttons (undock, close, menu) appear dynamically
    only when a panel or group of panels is detached into a floating window.
    """

    def eventFilter(self, watched: QObject, event: QEvent) -> bool:
        if event.type() in (
            QEvent.Type.Show,
            QEvent.Type.LayoutRequest,
            QEvent.Type.Resize,
            QEvent.Type.Paint,
        ):
            if isinstance(watched, ads.CDockAreaTitleBar):
                try:
                    area = watched.dockAreaWidget()
                    if area and area.dockContainer():
                        is_floating = area.dockContainer().isFloating()
                        for btn in watched.findChildren(ads.CTitleBarButton):
                            if not is_floating:
                                if btn.isVisible():
                                    btn.setVisible(False)
                            else:
                                if not btn.isVisible():
                                    btn.setVisible(True)
                except Exception:
                    pass
        return super().eventFilter(watched, event)


class PetuniaMainWindow(QMainWindow):
    """Professional desktop window hosting the Qt Advanced Docking System."""

    def __init__(self, bridge: Bridge | None = None, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setWindowTitle("Petunia Design Studio")
        self.resize(1440, 900)

        self.bridge = bridge or Bridge()
        self._studios_visible = True
        self._btn_filter = TitleBarButtonFilter(self)

        # Setup shared QML engine and register fonts
        self.qml_engine = QQmlEngine(self)
        self.qml_engine.rootContext().setContextProperty("bridge", self.bridge)

        font_path = Path(__file__).parent.parent / "qml" / "assets" / "fonts" / "Phosphor.ttf"
        if font_path.exists():
            QFontDatabase.addApplicationFont(str(font_path))

        # Setup Qt Advanced Docking System Manager
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.ActiveTabHasCloseButton, False)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.DockAreaHasUndockButton, True)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.DockAreaHasTabsMenuButton, True)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.DragPreviewIsDynamic, True)
        ads.CDockManager.setConfigFlag(
            ads.CDockManager.eConfigFlag.DragPreviewShowsContentPixmap, True
        )
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.OpaqueSplitterResize, True)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.FocusHighlighting, True)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.EqualSplitOnInsertion, True)
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.DoubleClickUndocksWidget, True)
        ads.CDockManager.setConfigFlag(
            ads.CDockManager.eConfigFlag.MiddleMouseButtonClosesTab, True
        )
        ads.CDockManager.setConfigFlag(
            ads.CDockManager.eConfigFlag.HideSingleCentralWidgetTitleBar, True
        )
        ads.CDockManager.setConfigFlag(ads.CDockManager.eConfigFlag.XmlAutoFormattingEnabled, True)

        self.dock_manager = ads.CDockManager(self)
        self.dock_manager.setStyleSheet(ADS_DARK_THEME)
        self.setStyleSheet(ADS_DARK_THEME)

        # Hook dynamic button filter on area/float creation
        self.dock_manager.dockAreaCreated.connect(self._on_dock_area_created)
        self.dock_manager.floatingWidgetCreated.connect(self._on_floating_widget_created)

        self._build_top_and_bottom_bars()
        self._build_dock_widgets()
        self._build_default_layout()
        self._build_menu_bar()

        # Try to restore previously saved dock layout
        self._layout_restored = self.restore_layout_from_disk()

    # -------------------------------------------------------------------------
    # Dynamic Titlebar Button Management
    # -------------------------------------------------------------------------
    def _on_dock_area_created(self, area: ads.CDockAreaWidget) -> None:
        try:
            tb = area.titleBar()
            if tb:
                tb.installEventFilter(self._btn_filter)
            self._update_all_titlebar_buttons()
        except Exception:
            pass

    def _on_floating_widget_created(self, floating_widget: Any) -> None:
        self._update_all_titlebar_buttons()

    def _update_all_titlebar_buttons(self) -> None:
        try:
            for area in self.dock_manager.openedDockAreas():
                tb = area.titleBar()
                if not tb:
                    continue
                tb.installEventFilter(self._btn_filter)
                is_floating = False
                container = area.dockContainer()
                if container:
                    is_floating = container.isFloating()
                for btn in tb.findChildren(ads.CTitleBarButton):
                    btn.setVisible(is_floating)
        except Exception:
            pass

    # -------------------------------------------------------------------------
    # QQuickWidget Factory
    # -------------------------------------------------------------------------
    def _create_qml_widget(self, rel_path: str, min_w: int = 0, min_h: int = 0) -> QQuickWidget:
        qw = QQuickWidget(self.qml_engine, None)
        qw.setResizeMode(QQuickWidget.ResizeMode.SizeRootObjectToView)
        qw.rootContext().setContextProperty("bridge", self.bridge)
        qml_file = (Path(__file__).parent.parent / "qml" / rel_path).resolve()
        qw.setSource(QUrl.fromLocalFile(str(qml_file)))
        if min_w > 0:
            qw.setMinimumWidth(min_w)
        if min_h > 0:
            qw.setMinimumHeight(min_h)
        return qw

    # -------------------------------------------------------------------------
    # Top, Bottom & Pinned Left Toolbar Layout
    # -------------------------------------------------------------------------
    def _build_top_and_bottom_bars(self) -> None:
        # Central container holding Top Bars, Middle Workspace, and Bottom Status Bar
        container = QWidget(self)
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # 1. Top Header Bar (Persona Switcher + Context Toolbar = 68px)
        self.top_header_widget = self._create_qml_widget("views/TopHeaderComposite.qml", min_h=68)
        self.top_header_widget.setFixedHeight(68)
        layout.addWidget(self.top_header_widget)

        # 2. Middle Workspace Row: [Pinned Left Toolbar (42px)] + [ADS Dock Manager]
        middle_widget = QWidget(self)
        middle_layout = QHBoxLayout(middle_widget)
        middle_layout.setContentsMargins(0, 0, 0, 0)
        middle_layout.setSpacing(0)

        # Pinned Left Toolbar (Never squished by ADS splitters or ruined with ADS headers)
        self.left_toolbar_widget = self._create_qml_widget("components/LeftToolbar.qml", min_w=42)
        self.left_toolbar_widget.setFixedWidth(42)
        middle_layout.addWidget(self.left_toolbar_widget)

        # Expose dock_tools alias for backward compatibility with tests/code
        self.dock_tools = self.left_toolbar_widget

        # Dock Manager central container (Canvas + Dockable Studios)
        middle_layout.addWidget(self.dock_manager, 1)
        layout.addWidget(middle_widget, 1)

        # 3. Bottom Status Bar (24px)
        self.status_bar_widget = self._create_qml_widget("components/StatusBar.qml", min_h=24)
        self.status_bar_widget.setFixedHeight(24)
        layout.addWidget(self.status_bar_widget)

        self.setCentralWidget(container)

    # -------------------------------------------------------------------------
    # Docks Construction (11 Fully Decoupled Tool Panels)
    # -------------------------------------------------------------------------
    def _build_dock_widgets(self) -> None:
        # Central Canvas Viewport
        self.dock_canvas = ads.CDockWidget(self.dock_manager, "Pranchetas & Canvas")
        self.dock_canvas.setWidget(self._create_qml_widget("views/CanvasViewport.qml"))
        self.dock_canvas.setFeature(ads.CDockWidget.DockWidgetFeature.DockWidgetClosable, False)
        self.dock_canvas.setFeature(ads.CDockWidget.DockWidgetFeature.DockWidgetMovable, False)
        self.dock_manager.setCentralWidget(self.dock_canvas)

        # Tier 1 Studios (Colour & Appearance Group)
        self.dock_colour = ads.CDockWidget(self.dock_manager, "Cor")
        self.dock_colour.setWidget(self._create_qml_widget("panels/ColourPanel.qml", min_w=260))

        self.dock_swatches = ads.CDockWidget(self.dock_manager, "Amostras")
        self.dock_swatches.setWidget(self._create_qml_widget("panels/SwatchesPanel.qml", min_w=260))

        self.dock_stroke = ads.CDockWidget(self.dock_manager, "Traço")
        self.dock_stroke.setWidget(self._create_qml_widget("panels/StrokePanel.qml", min_w=260))

        self.dock_appearance = ads.CDockWidget(self.dock_manager, "Aparência")
        self.dock_appearance.setWidget(
            self._create_qml_widget("panels/AppearancePanel.qml", min_w=260)
        )

        # Tier 2 Studios (Layers & Effects Group)
        self.dock_layers = ads.CDockWidget(self.dock_manager, "Camadas")
        self.dock_layers.setWidget(self._create_qml_widget("panels/LayersPanel.qml", min_w=260))

        self.dock_brushes = ads.CDockWidget(self.dock_manager, "Pincéis")
        self.dock_brushes.setWidget(self._create_qml_widget("panels/BrushesPanel.qml", min_w=260))

        self.dock_quickfx = ads.CDockWidget(self.dock_manager, "Efeitos")
        self.dock_quickfx.setWidget(self._create_qml_widget("panels/QuickFxPanel.qml", min_w=260))

        self.dock_styles = ads.CDockWidget(self.dock_manager, "Estilos")
        self.dock_styles.setWidget(self._create_qml_widget("panels/StylesPanel.qml", min_w=260))

        # Tier 3 Studios (Transform, History & Navigator Group)
        self.dock_transform = ads.CDockWidget(self.dock_manager, "Transformar")
        self.dock_transform.setWidget(
            self._create_qml_widget("panels/TransformStudio.qml", min_w=260)
        )

        self.dock_history = ads.CDockWidget(self.dock_manager, "Histórico")
        self.dock_history.setWidget(self._create_qml_widget("panels/HistoryStudio.qml", min_w=260))

        self.dock_navigator = ads.CDockWidget(self.dock_manager, "Navegador")
        self.dock_navigator.setWidget(
            self._create_qml_widget("panels/NavigatorStudio.qml", min_w=260)
        )

        self.studio_docks = [
            self.dock_colour,
            self.dock_swatches,
            self.dock_stroke,
            self.dock_appearance,
            self.dock_layers,
            self.dock_brushes,
            self.dock_quickfx,
            self.dock_styles,
            self.dock_transform,
            self.dock_history,
            self.dock_navigator,
        ]

    # -------------------------------------------------------------------------
    # Layout Arrangement (3 Distinct Tabbed Dock Tiers on the Right)
    # -------------------------------------------------------------------------
    def _build_default_layout(self) -> None:
        # Tier 1 (Top right): Colour, Swatches, Stroke, Appearance
        area_colour = self.dock_manager.addDockWidget(ads.RightDockWidgetArea, self.dock_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_swatches, area_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_stroke, area_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_appearance, area_colour)

        # Tier 2 (Middle right, below Tier 1): Layers, Brushes, Quick FX, Styles
        area_layers = self.dock_manager.addDockWidget(
            ads.BottomDockWidgetArea, self.dock_layers, area_colour
        )
        self.dock_manager.addDockWidgetTabToArea(self.dock_brushes, area_layers)
        self.dock_manager.addDockWidgetTabToArea(self.dock_quickfx, area_layers)
        self.dock_manager.addDockWidgetTabToArea(self.dock_styles, area_layers)

        # Tier 3 (Bottom right, below Tier 2): Transform, History, Navigator
        area_transform = self.dock_manager.addDockWidget(
            ads.BottomDockWidgetArea, self.dock_transform, area_layers
        )
        self.dock_manager.addDockWidgetTabToArea(self.dock_history, area_transform)
        self.dock_manager.addDockWidgetTabToArea(self.dock_navigator, area_transform)

        # Make primary tab in each tier active by default
        self.dock_colour.raise_()
        self.dock_layers.raise_()
        self.dock_transform.raise_()

        # Ensure filter and button visibility is updated
        self._update_all_titlebar_buttons()

    def apply_default_splitter_proportions(self) -> None:
        """Configures clean Affinity proportions (studios ~300px wide)."""
        root_sp = self.dock_manager.rootSplitter()
        if root_sp:
            studio_w = 300
            canvas_w = max(400, self.width() - 42 - studio_w)
            root_sp.setSizes([canvas_w, studio_w])

        colour_area = self.dock_colour.dockAreaWidget()
        if colour_area and colour_area.parentSplitter():
            total_h = max(self.height() - 92, 600)
            t1 = int(total_h * 0.35)
            t2 = int(total_h * 0.40)
            t3 = max(100, total_h - t1 - t2)
            colour_area.parentSplitter().setSizes([t1, t2, t3])

        self._update_all_titlebar_buttons()

    def showEvent(self, event: Any) -> None:
        super().showEvent(event)
        if not getattr(self, "_layout_restored", False):
            self.apply_default_splitter_proportions()
        self._update_all_titlebar_buttons()

    def reset_dock_layout(self) -> None:
        """Restores the pristine 11-panel 3-tier Affinity arrangement."""
        for dock in self.studio_docks:
            dock.toggleView(True)

        area_colour = self.dock_manager.addDockWidget(ads.RightDockWidgetArea, self.dock_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_swatches, area_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_stroke, area_colour)
        self.dock_manager.addDockWidgetTabToArea(self.dock_appearance, area_colour)

        area_layers = self.dock_manager.addDockWidget(
            ads.BottomDockWidgetArea, self.dock_layers, area_colour
        )
        self.dock_manager.addDockWidgetTabToArea(self.dock_brushes, area_layers)
        self.dock_manager.addDockWidgetTabToArea(self.dock_quickfx, area_layers)
        self.dock_manager.addDockWidgetTabToArea(self.dock_styles, area_layers)

        area_transform = self.dock_manager.addDockWidget(
            ads.BottomDockWidgetArea, self.dock_transform, area_layers
        )
        self.dock_manager.addDockWidgetTabToArea(self.dock_history, area_transform)
        self.dock_manager.addDockWidgetTabToArea(self.dock_navigator, area_transform)

        self.dock_colour.raise_()
        self.dock_layers.raise_()
        self.dock_transform.raise_()

        self.apply_default_splitter_proportions()
        self._update_all_titlebar_buttons()

    def toggle_all_studios(self) -> None:
        """Toggle visibility of all studios (like the Tab shortcut in Affinity/Photoshop)."""
        self._studios_visible = not self._studios_visible
        for dock in self.studio_docks:
            dock.toggleView(self._studios_visible)

    # -------------------------------------------------------------------------
    # Native Menu Bar
    # -------------------------------------------------------------------------
    def _build_menu_bar(self) -> None:
        menubar = self.menuBar()

        # Arquivo
        menu_arquivo = menubar.addMenu("&Arquivo")
        act_new = menu_arquivo.addAction("&Novo Documento...")
        act_new.setShortcut(QKeySequence.StandardKey.New)
        act_new.triggered.connect(self.bridge.requestNewDocument)

        act_open = menu_arquivo.addAction("&Abrir Documento PTND...")
        act_open.setShortcut(QKeySequence.StandardKey.Open)
        act_open.triggered.connect(lambda: self.bridge.open("/tmp/document.ptnd"))

        act_save = menu_arquivo.addAction("&Salvar")
        act_save.setShortcut(QKeySequence.StandardKey.Save)
        act_save.triggered.connect(lambda: self.bridge.save("/tmp/document.ptnd"))

        act_save_as = menu_arquivo.addAction("Salvar &Como...")
        act_save_as.setShortcut(QKeySequence.StandardKey.SaveAs)
        act_save_as.triggered.connect(lambda: self.bridge.save("/tmp/document.ptnd"))

        menu_arquivo.addSeparator()
        act_place = menu_arquivo.addAction("&Inserir Imagem...")
        act_place.triggered.connect(self.bridge.requestPlaceImage)

        act_preset_art = menu_arquivo.addAction("Inserir &Prancheta Preset")
        act_preset_art.triggered.connect(lambda: self.bridge.insertPresetArtboard("doc"))

        menu_arquivo.addSeparator()
        act_png = menu_arquivo.addAction("&Exportar Imagem PNG...")
        act_png.triggered.connect(lambda: self.bridge.exportPng("/tmp/petunia-design-export.png"))

        act_svg = menu_arquivo.addAction("Exportar Vetor &SVG...")
        act_svg.triggered.connect(self.bridge.exportSvg)

        menu_arquivo.addSeparator()
        act_doc_setup = menu_arquivo.addAction("Configuração do &Documento...")
        act_doc_setup.triggered.connect(self.bridge.requestDocumentSetup)

        menu_arquivo.addSeparator()
        act_quit = menu_arquivo.addAction("&Sair")
        act_quit.setShortcut(QKeySequence.StandardKey.Quit)
        act_quit.triggered.connect(self.close)

        # Editar
        menu_editar = menubar.addMenu("&Editar")
        act_undo = menu_editar.addAction("&Desfazer")
        act_undo.setShortcut(QKeySequence.StandardKey.Undo)
        act_undo.triggered.connect(self.bridge.undo)

        act_redo = menu_editar.addAction("&Refazer")
        act_redo.setShortcut(QKeySequence.StandardKey.Redo)
        act_redo.triggered.connect(self.bridge.redo)

        menu_editar.addSeparator()
        act_dup = menu_editar.addAction("&Duplicar Seleção")
        act_dup.setShortcut("Ctrl+D")
        act_dup.triggered.connect(self.bridge.duplicateSelected)

        act_del = menu_editar.addAction("&Excluir")
        act_del.setShortcut(QKeySequence.StandardKey.Delete)
        act_del.triggered.connect(self.bridge.deleteSelected)

        menu_editar.addSeparator()
        act_snap = menu_editar.addAction("Alternar &Alinhamento Magnético (Snap)")
        act_snap.setShortcut("Ctrl+;")
        act_snap.triggered.connect(self.bridge.toggleSnapping)

        act_prefs = menu_editar.addAction("&Preferências do Aplicativo...")
        act_prefs.setShortcut("Ctrl+,")
        act_prefs.triggered.connect(self.bridge.requestAppSettings)

        # Visualizar
        menu_view = menubar.addMenu("&Visualizar")
        act_zin = menu_view.addAction("Aumentar Zoom (&+)")
        act_zin.setShortcut("Ctrl+=")
        act_zin.triggered.connect(self.bridge.zoomIn)

        act_zout = menu_view.addAction("Diminuir Zoom (&-)")
        act_zout.setShortcut("Ctrl+-")
        act_zout.triggered.connect(self.bridge.zoomOut)

        act_fit = menu_view.addAction("&Ajustar Pranchetas à Tela")
        act_fit.setShortcut("Ctrl+0")
        act_fit.triggered.connect(self.bridge.fitAllArtboards)

        act_100 = menu_view.addAction("Tamanho Real (&100%)")
        act_100.setShortcut("Ctrl+1")
        act_100.triggered.connect(lambda: self.bridge.setZoom(1.0))

        menu_view.addSeparator()
        act_pv = menu_view.addAction("Persona &Vetor")
        act_pv.triggered.connect(
            lambda: (self.bridge.setPersona("vector"), self.bridge.setTool("select"))
        )
        act_pp = menu_view.addAction("Persona &Pixel")
        act_pp.triggered.connect(
            lambda: (self.bridge.setPersona("pixel"), self.bridge.setTool("pixel_brush"))
        )
        act_pe = menu_view.addAction("Persona &Exportação")
        act_pe.triggered.connect(
            lambda: (self.bridge.setPersona("export"), self.bridge.setTool("slice"))
        )

        # Camada
        menu_camada = menubar.addMenu("&Camada")
        act_grp = menu_camada.addAction("&Agrupar Objetos")
        act_grp.setShortcut("Ctrl+G")
        act_grp.triggered.connect(self.bridge.groupSelected)

        act_ungrp = menu_camada.addAction("&Desagrupar Objeto")
        act_ungrp.setShortcut("Ctrl+Shift+G")
        act_ungrp.triggered.connect(self.bridge.ungroupSelected)

        menu_camada.addSeparator()
        act_curves = menu_camada.addAction("&Converter em Curvas")
        act_curves.setShortcut("Ctrl+Return")
        act_curves.triggered.connect(
            lambda: self.bridge.convertToCurves(str(self.bridge.selectedId))
        )

        menu_camada.addSeparator()
        act_front = menu_camada.addAction("&Trazer para a Frente")
        act_front.setShortcut("Ctrl+Shift+]")
        act_front.triggered.connect(lambda: self.bridge.bringToFront(""))

        act_fwd = menu_camada.addAction("&Avançar um Nível")
        act_fwd.setShortcut("Ctrl+]")
        act_fwd.triggered.connect(lambda: self.bridge.moveForward(""))

        act_back = menu_camada.addAction("&Recuar um Nível")
        act_back.setShortcut("Ctrl+[")
        act_back.triggered.connect(lambda: self.bridge.moveBackward(""))

        act_bottom = menu_camada.addAction("&Enviar para o Fundo")
        act_bottom.setShortcut("Ctrl+Shift+[")
        act_bottom.triggered.connect(lambda: self.bridge.sendToBack(""))

        act_lock = menu_camada.addAction("&Bloquear/Desbloquear Camada")
        act_lock.setShortcut("Ctrl+L")
        act_lock.triggered.connect(lambda: self.bridge.toggleLock(str(self.bridge.selectedId)))

        menu_camada.addSeparator()
        menu_camada.addAction("Alinhar à &Esquerda", lambda: self.bridge.alignSelected("left"))
        menu_camada.addAction("Alinhar ao &Centro", lambda: self.bridge.alignSelected("center"))
        menu_camada.addAction("Alinhar à &Direita", lambda: self.bridge.alignSelected("right"))
        menu_camada.addAction("Alinhar ao &Topo", lambda: self.bridge.alignSelected("top"))
        menu_camada.addAction("Alinhar ao &Meio", lambda: self.bridge.alignSelected("middle"))
        menu_camada.addAction("Alinhar ao &Fundo", lambda: self.bridge.alignSelected("bottom"))

        # Geometria
        menu_geom = menubar.addMenu("&Geometria")
        menu_geom.addAction("&Adicionar (União Booleana)", lambda: self.bridge.boolean("union"))
        menu_geom.addAction("&Subtrair", lambda: self.bridge.boolean("difference"))
        menu_geom.addAction("&Intersecção", lambda: self.bridge.boolean("intersection"))
        menu_geom.addAction("&XOR (Exclusão Mútua)", lambda: self.bridge.boolean("xor"))
        menu_geom.addAction("&Dividir Regiões", lambda: self.bridge.boolean("divide"))
        menu_geom.addSeparator()
        menu_geom.addAction("Ativar Ferramenta &Shape Builder", self.bridge.shapeBuilderStart)

        # Ferramentas
        menu_ferramentas = menubar.addMenu("&Ferramentas")
        menu_ferramentas.addAction("&Mover (Select)", lambda: self.bridge.setTool("select")).setShortcut("V")
        menu_ferramentas.addAction("&Prancheta", lambda: self.bridge.setTool("artboard")).setShortcut("A")
        menu_ferramentas.addAction("&Nó", lambda: self.bridge.setTool("node")).setShortcut("N")
        menu_ferramentas.addAction("&Canto", lambda: self.bridge.setTool("corner")).setShortcut("C")
        menu_ferramentas.addAction("Ca&neta", lambda: self.bridge.setTool("pen")).setShortcut("P")
        menu_ferramentas.addAction("&Lápis", lambda: self.bridge.setTool("pencil")).setShortcut("B")
        menu_ferramentas.addAction("Pincel &Vetorial", lambda: self.bridge.setTool("brush"))
        menu_ferramentas.addAction("Preenchimento (&Gradiente)", lambda: self.bridge.setTool("fill")).setShortcut("G")
        menu_ferramentas.addAction("&Transparência", lambda: self.bridge.setTool("transparency")).setShortcut("Y")
        menu_ferramentas.addSeparator()
        menu_ferramentas.addAction("&Retângulo", lambda: self.bridge.setTool("rectangle")).setShortcut("M")
        menu_ferramentas.addAction("&Elipse", lambda: self.bridge.setTool("ellipse")).setShortcut("O")
        menu_ferramentas.addAction("&Triângulo", lambda: self.bridge.setTool("triangle"))
        menu_ferramentas.addAction("E&strela", lambda: self.bridge.setTool("star"))
        menu_ferramentas.addAction("&Polígono", lambda: self.bridge.setTool("polygon"))
        menu_ferramentas.addAction("&Losango", lambda: self.bridge.setTool("diamond"))
        menu_ferramentas.addAction("&Seta", lambda: self.bridge.setTool("arrow"))
        menu_ferramentas.addAction("&Coração", lambda: self.bridge.setTool("heart"))
        menu_ferramentas.addAction("&Engrenagem (Cog)", lambda: self.bridge.setTool("cog"))
        menu_ferramentas.addSeparator()
        menu_ferramentas.addAction("&Texto", lambda: self.bridge.setTool("text")).setShortcut("T")
        menu_ferramentas.addAction("Re&corte Vetorial", lambda: self.bridge.setTool("crop")).setShortcut("X")
        menu_ferramentas.addAction("&Faca (Knife)", lambda: self.bridge.setTool("knife")).setShortcut("K")
        menu_ferramentas.addSeparator()
        menu_ferramentas.addAction("&Pincel de Pixel", lambda: self.bridge.setTool("pixel_brush"))
        menu_ferramentas.addAction("&Lápis de Pixel (1px)", lambda: self.bridge.setTool("pixel_pencil"))
        menu_ferramentas.addAction("&Borracha de Pixel", lambda: self.bridge.setTool("pixel_eraser")).setShortcut("E")
        menu_ferramentas.addAction("&Laço de Pixel", lambda: self.bridge.setTool("pixel_lasso")).setShortcut("L")
        menu_ferramentas.addAction("&Carimbo Clone", lambda: self.bridge.setTool("pixel_clone"))
        menu_ferramentas.addAction("&Balde de Tinta", lambda: self.bridge.setTool("pixel_fill"))
        menu_ferramentas.addAction("&Subexposição (Dodge)", lambda: self.bridge.setTool("pixel_dodge"))
        menu_ferramentas.addAction("S&uperexposição (Burn)", lambda: self.bridge.setTool("pixel_burn"))
        menu_ferramentas.addAction("&Borrar (Smudge)", lambda: self.bridge.setTool("pixel_smudge"))
        menu_ferramentas.addAction("&Desfoque (Blur)", lambda: self.bridge.setTool("pixel_blur"))
        menu_ferramentas.addSeparator()
        menu_ferramentas.addAction("&Conta-gotas", lambda: self.bridge.setTool("eyedropper")).setShortcut("I")
        menu_ferramentas.addAction("&Medição", lambda: self.bridge.setTool("measure"))
        menu_ferramentas.addAction("&Mão (Pan)", lambda: self.bridge.setTool("hand")).setShortcut("H")
        menu_ferramentas.addAction("&Zoom", lambda: self.bridge.setTool("zoom")).setShortcut("Z")
        menu_ferramentas.addAction("&Fatia (Exportação)", lambda: self.bridge.setTool("slice")).setShortcut("S")

        # ---------------------------------------------------------------------
        # Menu Janela / Estúdios (Docking Controls)
        # ---------------------------------------------------------------------
        menu_janela = menubar.addMenu("&Janela")

        menu_studios = menu_janela.addMenu("&Estúdios")
        # Add toggleViewAction for all 11 individual dock widgets
        for dock in self.studio_docks:
            menu_studios.addAction(dock.toggleViewAction())

        menu_janela.addSeparator()
        act_reset_layout = menu_janela.addAction("&Restaurar Layout dos Estúdios")
        act_reset_layout.triggered.connect(self.reset_dock_layout)

        act_save_layout = menu_janela.addAction("&Salvar Layout dos Estúdios...")
        act_save_layout.triggered.connect(self.save_layout_to_disk)

        act_load_layout = menu_janela.addAction("&Carregar Layout dos Estúdios...")
        act_load_layout.triggered.connect(self.restore_layout_from_disk)

        menu_janela.addSeparator()
        act_toggle_studios = menu_janela.addAction("&Ocultar / Exibir Todos os Estúdios")
        act_toggle_studios.setShortcut(Qt.Key.Key_Tab)
        act_toggle_studios.triggered.connect(self.toggle_all_studios)

        menu_janela.addSeparator()
        act_demo = menu_janela.addAction("Carregar Demonstração &Inioluwa Abiri")
        act_demo.triggered.connect(self.bridge.loadDemoDocument)

        # Ajuda
        menu_ajuda = menubar.addMenu("A&juda")
        act_help_shortcuts = menu_ajuda.addAction("&Atalhos de Teclado...")
        act_help_shortcuts.setShortcut(QKeySequence.StandardKey.HelpContents)
        act_help_shortcuts.triggered.connect(self.bridge.requestHelp)

        act_about = menu_ajuda.addAction("&Sobre o Petunia Design Studio...")
        act_about.triggered.connect(self.bridge.requestHelp)

    @property
    def studios_visible(self) -> bool:
        """Returns True if studios are currently visible."""
        return self._studios_visible

    # -------------------------------------------------------------------------
    # Layout State Persistence
    # -------------------------------------------------------------------------
    def layout_file_path(self) -> Path:
        config_dir = Path.home() / ".config" / "petunia-design"
        config_dir.mkdir(parents=True, exist_ok=True)
        return config_dir / "dock_layout_v2.xml"

    def save_layout_to_disk(self) -> None:
        """Persists the current docking state to disk."""
        path = self.layout_file_path()
        state = self.dock_manager.saveState()
        with open(path, "wb") as f:
            f.write(bytes(state))

    def restore_layout_from_disk(self) -> bool:
        """Restores docking state from disk if present."""
        path = self.layout_file_path()
        if not path.exists():
            return False
        try:
            with open(path, "rb") as f:
                data = f.read()
            restored = bool(self.dock_manager.restoreState(QByteArray(data)))
            if restored:
                self._update_all_titlebar_buttons()
            return restored
        except Exception:
            return False

    def closeEvent(self, event: Any) -> None:
        self.save_layout_to_disk()
        if hasattr(self, "dock_manager"):
            self.dock_manager.deleteLater()
        super().closeEvent(event)
