import os
import sys
from pathlib import Path

import PySide6QtAds as ads
from PySide6.QtQuickControls2 import QQuickStyle
from PySide6.QtWidgets import QApplication

from petunia_app.bridge import Bridge
from petunia_app.docking import PetuniaMainWindow


def test_docking_main_window_structure(tmp_path: Path) -> None:
    os.environ["QT_QPA_PLATFORM"] = "offscreen"
    os.environ.setdefault("QT_QUICK_BACKEND", "software")
    app = QApplication.instance() or QApplication(sys.argv)
    QQuickStyle.setStyle("Basic")

    bridge = Bridge()
    win = PetuniaMainWindow(bridge)
    win.resize(1440, 900)
    win.show()
    app.processEvents()

    # Verify central canvas and pinned tool palette
    assert win.dock_canvas is not None
    assert win.dock_tools is not None

    # Verify 11 decoupled tool studios
    assert win.dock_colour is not None
    assert win.dock_swatches is not None
    assert win.dock_stroke is not None
    assert win.dock_appearance is not None
    assert win.dock_layers is not None
    assert win.dock_brushes is not None
    assert win.dock_quickfx is not None
    assert win.dock_styles is not None
    assert win.dock_transform is not None
    assert win.dock_history is not None
    assert win.dock_navigator is not None
    assert len(win.studio_docks) == 11

    # Verify titlebar buttons are hidden when docked
    app.processEvents()
    for area in win.dock_manager.openedDockAreas():
        tb = area.titleBar()
        visible_buttons = [b for b in tb.findChildren(ads.CTitleBarButton) if b.isVisible()]
        assert len(visible_buttons) == 0

    # Verify floating widget is created and widget is floating
    win.dock_swatches.setFloating()
    app.processEvents()
    floating_widgets = win.dock_manager.floatingWidgets()
    assert len(floating_widgets) >= 1
    assert win.dock_swatches.isFloating() is True

    # Verify menu bar and actions
    menubar = win.menuBar()
    actions = [a.text() for a in menubar.actions()]
    assert "&Arquivo" in actions
    assert "&Janela" in actions

    # Test toggle all studios (Tab key behavior)
    assert win.studios_visible is True
    win.toggle_all_studios()
    assert win.studios_visible is False
    win.toggle_all_studios()
    assert win.studios_visible is True

    # Test reset dock layout
    win.reset_dock_layout()
    app.processEvents()
    assert win.dock_colour.dockAreaWidget() is not None

    # Test save and restore layout to a custom temp path
    test_layout_file = tmp_path / "test_dock_layout.xml"
    win.layout_file_path = lambda: test_layout_file  # monkeypatch path for test isolation
    win.save_layout_to_disk()
    assert test_layout_file.exists()
    assert test_layout_file.stat().st_size > 0

    assert win.restore_layout_from_disk() is True
    win.close()
    app.processEvents()
