"""Unit tests for wave 2 features: layer reordering, blend modes,
stroke styles, swatches, slices, and flood fill.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, cast

from petunia_app.bridge import Bridge
from petunia_app.model import SceneObject


def test_layer_reordering() -> None:
    bridge = Bridge()
    o1 = SceneObject(type="rectangle", name="Obj1", x=0, y=0, width=50, height=50)
    o2 = SceneObject(type="rectangle", name="Obj2", x=50, y=0, width=50, height=50)
    o3 = SceneObject(type="rectangle", name="Obj3", x=100, y=0, width=50, height=50)
    bridge.document.objects.extend([o1, o2, o3])
    bridge.objectsChanged.emit()

    assert [o.name for o in bridge.document.objects] == ["Obj1", "Obj2", "Obj3"]

    # Bring Obj1 to front
    bridge.bringToFront(o1.id)
    assert [o.name for o in bridge.document.objects] == ["Obj2", "Obj3", "Obj1"]

    # Undo reorder
    bridge.undo()
    assert [o.name for o in bridge.document.objects] == ["Obj1", "Obj2", "Obj3"]

    # Redo reorder
    bridge.redo()
    assert [o.name for o in bridge.document.objects] == ["Obj2", "Obj3", "Obj1"]

    # Send Obj1 to back
    bridge.sendToBack(o1.id)
    assert [o.name for o in bridge.document.objects] == ["Obj1", "Obj2", "Obj3"]

    # Move Obj2 forward
    bridge.moveForward(o2.id)
    assert [o.name for o in bridge.document.objects] == ["Obj1", "Obj3", "Obj2"]

    # Move Obj2 backward
    bridge.moveBackward(o2.id)
    assert [o.name for o in bridge.document.objects] == ["Obj1", "Obj2", "Obj3"]


def test_blend_mode_and_stroke_options() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100, 100)
    obj = bridge.document.objects[0]

    # Blend Mode
    bridge.setBlendMode(obj.id, "multiply")
    assert obj.blend_mode == "multiply"

    bridge.setProp("blendMode", "screen")
    assert obj.blend_mode == "screen"

    # Stroke cap & join
    bridge.setStrokeCap("square")
    assert obj.stroke_cap == "square"

    bridge.setStrokeJoin("bevel")
    assert obj.stroke_join == "bevel"

    # Stroke dash
    bridge.setStrokeDash("4, 2, 8, 2")
    assert obj.stroke_dash == [4.0, 2.0, 8.0, 2.0]

    # Rotation & shear
    bridge.setProp("rotation", "45")
    assert obj.rotation == 45.0

    bridge.setProp("shear", "10")
    assert obj.shear == 10.0


def test_swatches_management() -> None:
    bridge = Bridge()
    initial_swatches = cast(list[str], bridge.swatches)
    assert len(initial_swatches) >= 10

    bridge.addSwatch("#123456")
    updated = cast(list[str], bridge.swatches)
    assert "#123456" in updated

    # Duplicate add ignored
    bridge.addSwatch("#123456")
    assert cast(list[str], bridge.swatches).count("#123456") == 1

    bridge.removeSwatch("#123456")
    assert "#123456" not in cast(list[str], bridge.swatches)


def test_slices_and_export_slices(tmp_path: Path) -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 200, 200)

    sid = bridge.createSlice("CardSlice", 10.0, 10.0, 180.0, 180.0, "PNG", 1.5)
    assert sid != ""
    assert len(cast(list[dict[str, Any]], bridge.slices)) == 1

    slice_data = cast(list[dict[str, Any]], bridge.slices)[0]
    assert slice_data["name"] == "CardSlice"
    assert slice_data["width"] == 180.0
    assert slice_data["scale"] == 1.5

    # Modify slice prop
    bridge.setSliceProp(sid, "width", 200.0)
    slice_data = cast(list[dict[str, Any]], bridge.slices)[0]
    assert slice_data["width"] == 200.0

    # Export individual slice
    out_file = tmp_path / "exported_card.png"
    bridge.exportSlice(sid, str(out_file))
    assert out_file.exists()
    assert out_file.stat().st_size > 0

    # Export all slices to directory
    out_dir = tmp_path / "all_slices"
    bridge.exportAllSlices(str(out_dir))
    assert (out_dir / "CardSlice.png").exists()

    # Delete slice
    bridge.deleteSlice(sid)
    assert len(cast(list[dict[str, Any]], bridge.slices)) == 0


def test_flood_fill_and_lock() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100, 100)
    obj = bridge.document.objects[0]
    obj.x = 50.0
    obj.y = 50.0
    obj.width = 100.0
    obj.height = 100.0

    bridge.setFillColor("#AABBCC")
    bridge.floodFill(75.0, 75.0)
    assert obj.fill == "#AABBCC"

    # Toggle lock
    assert obj.id not in bridge._locked_ids
    bridge.toggleLock(obj.id)
    assert obj.id in bridge._locked_ids
    bridge.toggleLock(obj.id)
    assert obj.id not in bridge._locked_ids
