from __future__ import annotations

import os
import sys
from pathlib import Path


def main() -> int:
    smoke = "--smoke" in sys.argv
    no_dock = "--no-dock" in sys.argv
    if smoke:
        os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

    from PySide6.QtGui import QFontDatabase
    from PySide6.QtQuickControls2 import QQuickStyle
    from PySide6.QtWidgets import QApplication

    app = QApplication.instance() or QApplication(sys.argv)
    QQuickStyle.setStyle("Basic")

    font_path = Path(__file__).parent / "qml" / "assets" / "fonts" / "Phosphor.ttf"
    if font_path.exists():
        QFontDatabase.addApplicationFont(str(font_path))

    from petunia_app.bridge import Bridge

    bridge = Bridge()

    if no_dock:
        from PySide6.QtQml import QQmlApplicationEngine

        engine = QQmlApplicationEngine()
        engine.rootContext().setContextProperty("bridge", bridge)
        qml = Path(__file__).parent / "qml" / "main.qml"
        engine.load(qml.as_uri())

        if not engine.rootObjects():
            return 1
        if smoke:
            return 0
        return app.exec()

    from petunia_app.docking import PetuniaMainWindow

    win = PetuniaMainWindow(bridge)
    win.show()

    if smoke:
        app.processEvents()
        win.close()
        app.processEvents()
        return 0

    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
