from pathlib import Path

from petunia_app.model import Document, SceneObject
from petunia_app.ptnd_scene import open_ptnd, save_ptnd


def test_ptnd_roundtrip(tmp_path: Path) -> None:
    doc = Document(name="Demo", width=800, height=600)
    doc.objects.append(SceneObject(type="rectangle", name="R", x=5, y=6, width=70, height=80))
    doc.objects.append(SceneObject(type="ellipse", name="E", x=20, y=30, width=40, height=40))
    pkg = tmp_path / "demo.ptnd"
    save_ptnd(pkg, doc)
    back = open_ptnd(pkg)
    assert back.name == "Demo"
    assert back.width == 800
    assert len(back.objects) == 2
    assert back.objects[0].type == "rectangle"
    assert back.objects[0].x == 5
    assert back.objects[1].type == "ellipse"
    assert back.objects[1].width == 40


def test_ptnd_semantic_snapshot_equals_reopen(tmp_path: Path) -> None:
    doc = Document(name="Semantic", width=1920, height=1080)
    doc.objects.append(
        SceneObject(
            type="rectangle",
            name="Hero",
            x=10,
            y=20,
            width=320,
            height=200,
            gradient=["#ff0000", "#00ff00", "#0000ff"],
            stroke="#123456",
            stroke_width=3.5,
            visible=True,
        )
    )
    doc.objects.append(
        SceneObject(
            type="ellipse",
            name="Orb",
            x=400,
            y=100,
            width=64,
            height=64,
            fill="#abcdef",
            visible=False,
        )
    )
    group = SceneObject(type="group", name="G")
    group.children = [
        SceneObject(type="rectangle", name="C1", x=0, y=0, width=10, height=10),
        SceneObject(type="ellipse", name="C2", x=30, y=0, width=10, height=10),
    ]
    doc.objects.append(group)

    pkg = tmp_path / "semantic.ptnd"
    save_ptnd(pkg, doc)
    back = open_ptnd(pkg)

    assert back.to_json() == doc.to_json()
    assert back.document_id == doc.document_id
    assert back.objects[2].type == "group"
    assert len(back.objects[2].children) == 2
    assert back.objects[2].bounds() == (0.0, 0.0, 40.0, 10.0)


def test_ptnd_group_roundtrip(tmp_path: Path) -> None:
    doc = Document(name="Groups", width=500, height=500)
    g = SceneObject(type="group", name="Pair")
    g.children = [
        SceneObject(type="rectangle", name="A", x=0, y=0, width=10, height=10),
        SceneObject(type="rectangle", name="B", x=50, y=0, width=10, height=10),
    ]
    doc.objects.append(g)
    pkg = tmp_path / "g.ptnd"
    save_ptnd(pkg, doc)
    back = open_ptnd(pkg)
    assert back.objects[0].type == "group"
    assert [c.name for c in back.objects[0].children] == ["A", "B"]
