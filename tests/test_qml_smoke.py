import sys


def test_qml_smoke(monkeypatch) -> None:
    import os

    os.environ["QT_QPA_PLATFORM"] = "offscreen"
    os.environ.setdefault("QT_QUICK_BACKEND", "software")
    monkeypatch.setattr(sys, "argv", ["petunia", "--smoke"])
    from petunia_app.__main__ import main

    assert main() == 0
