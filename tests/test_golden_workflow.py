from pathlib import Path

from petunia_app.exporters import export_svg
from petunia_app.model import (
    AddObject,
    Document,
    GroupObjects,
    History,
    MoveObject,
    MoveObjects,
    RemoveObjects,
    SceneObject,
    SetProperty,
    Snap,
    Ungroup,
)


def test_golden_workflow_create_move_edit_undo() -> None:
    doc = Document()
    history = History()

    rect = SceneObject(type="rectangle", name="Hero", width=200, height=120)
    history.execute(AddObject(rect), doc)
    assert len(doc.objects) == 1

    ellipse = SceneObject(type="ellipse", name="Orb", x=40, y=40, width=80, height=80)
    history.execute(AddObject(ellipse), doc)
    assert len(doc.objects) == 2

    history.execute(MoveObject(rect.id, 50, 30), doc)
    assert doc.objects[0].x == 50 and doc.objects[0].y == 30

    history.undo(doc)
    assert doc.objects[0].x == 0 and doc.objects[0].y == 0
    history.redo(doc)
    assert doc.objects[0].x == 50

    history.execute(SetProperty(rect.id, "fill", "#ff0000"), doc)
    assert doc.objects[0].fill == "#ff0000"
    history.undo(doc)
    assert doc.objects[0].fill == "#9bb8ff"

    svg = export_svg(doc)
    assert "<rect" in svg and "<ellipse" in svg
    doc.width = 1280
    assert 'width="1280"' in export_svg(doc)

    assert Snap(8.0).snap(13) == 16

    history.execute(RemoveObjects([rect.id, ellipse.id]), doc)
    assert len(doc.objects) == 0
    history.undo(doc)
    assert len(doc.objects) == 2


def test_group_ungroup_roundtrip() -> None:
    doc = Document()
    history = History()
    a = SceneObject(type="rectangle", name="A", x=0, y=0, width=10, height=10)
    b = SceneObject(type="ellipse", name="B", x=20, y=0, width=10, height=10)
    history.execute(AddObject(a), doc)
    history.execute(AddObject(b), doc)

    cmd = GroupObjects([a.id, b.id], "G")
    history.execute(cmd, doc)
    assert len(doc.objects) == 1
    group = doc.objects[0]
    assert group.type == "group"
    assert len(group.children) == 2
    assert group.bounds() == (0.0, 0.0, 30.0, 10.0)

    history.undo(doc)
    assert len(doc.objects) == 2
    assert doc.objects[0].id == a.id and doc.objects[1].id == b.id
    history.redo(doc)
    assert len(doc.objects) == 1

    history.execute(Ungroup(group.id), doc)
    assert len(doc.objects) == 2
    history.undo(doc)
    assert len(doc.objects) == 1


def test_move_objects_atomic() -> None:
    doc = Document()
    history = History()
    a = SceneObject(type="rectangle", name="A")
    b = SceneObject(type="rectangle", name="B", x=100, y=0)
    history.execute(AddObject(a), doc)
    history.execute(AddObject(b), doc)

    history.execute(MoveObjects([a.id, b.id], 10, 5), doc)
    assert doc.objects[0].x == 10 and doc.objects[0].y == 5
    assert doc.objects[1].x == 110 and doc.objects[1].y == 5
    history.undo(doc)
    assert doc.objects[0].x == 0 and doc.objects[1].x == 100


def test_png_export(tmp_path: Path) -> None:
    import os

    from petunia_app.raster import export_png

    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    os.environ.setdefault("QT_QUICK_BACKEND", "software")
    from PySide6.QtWidgets import QApplication

    _app = QApplication.instance() or QApplication([])
    doc = Document()
    doc.objects.append(SceneObject(type="ellipse", width=40, height=40))
    out = tmp_path / "out.png"
    export_png(doc, out)
    assert out.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
