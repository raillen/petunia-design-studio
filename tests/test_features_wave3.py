from pathlib import Path
from petunia_app.bridge import Bridge
from petunia_app.model import Document, SceneObject
from petunia_app.raster import export_png


def test_triangle_star_polygon_creation() -> None:
    bridge = Bridge()

    # 1. Triangle bounds
    tri_id = bridge.createShapeBounds("triangle", 100.0, 100.0, 150.0, 120.0, fill="#ff0055", stroke="#000000")
    assert tri_id != ""
    assert len(bridge.document.objects) == 1
    tri = bridge.document.objects[0]
    assert tri.type == "path"
    assert tri.path is not None
    assert len(tri.path.contours) == 1
    assert len(tri.path.contours[0].nodes) == 3
    assert tri.path.contours[0].closed is True

    # 2. Star bounds
    star_id = bridge.createShapeBounds("star", 300.0, 100.0, 100.0, 100.0, fill="#ffd700")
    assert star_id != ""
    assert len(bridge.document.objects) == 2
    star = bridge.document.objects[1]
    assert star.type == "path"
    assert star.path is not None
    assert len(star.path.contours[0].nodes) == 10  # 5-pointed star: 10 vertices
    assert star.path.contours[0].closed is True

    # 3. Polygon bounds
    poly_id = bridge.createShapeBounds("polygon", 500.0, 100.0, 120.0, 120.0, fill="#00ccff")
    assert poly_id != ""
    assert len(bridge.document.objects) == 3
    poly = bridge.document.objects[2]
    assert poly.type == "path"
    assert poly.path is not None
    assert len(poly.path.contours[0].nodes) == 6  # hexagon: 6 vertices
    assert poly.path.contours[0].closed is True

    # 4. create_shape helper
    bridge.create_shape("star", 200.0, 200.0)
    assert len(bridge.document.objects) == 4
    s4 = bridge.document.objects[3]
    assert s4.type == "path"


def test_boolean_with_primitives_auto_convert() -> None:
    bridge = Bridge()

    # Create a rectangle and an ellipse (both initially type != path)
    r_id = bridge.createShapeBounds("rectangle", 50.0, 50.0, 100.0, 100.0, fill="#ff0000")
    e_id = bridge.createShapeBounds("ellipse", 100.0, 50.0, 100.0, 100.0, fill="#00ff00")

    r = bridge.document.objects[0]
    e = bridge.document.objects[1]
    assert r.path is None
    assert e.path is None

    # Select both and perform union
    bridge.select(r_id)
    bridge.toggleSelect(e_id)
    assert len(bridge.selectedIds) == 2

    bridge.boolean("union")
    assert "aplicado" in bridge.message
    # Resulting object should be a unified path
    assert len(bridge.document.objects) == 1
    res = bridge.document.objects[0]
    assert res.type == "path"
    assert res.path is not None


def test_typography_styling_and_raster() -> None:
    bridge = Bridge()

    t_id = bridge.createText(50.0, 50.0, "Hello Affinity")
    assert t_id != ""
    bridge.select(t_id)

    # Defaults
    assert bridge.selectedFontSize == 14.0
    assert bridge.selectedFontFamily == "Inter"
    assert bridge.selectedFontBold is False
    assert bridge.selectedFontItalic is False
    assert bridge.selectedTextAlign == "left"

    # Set styles
    bridge.setTextFontSize(28.0)
    assert bridge.selectedFontSize == 28.0

    bridge.setTextFontFamily("Roboto")
    assert bridge.selectedFontFamily == "Roboto"

    bridge.setTextBold(True)
    assert bridge.selectedFontBold is True

    bridge.setTextItalic(True)
    assert bridge.selectedFontItalic is True

    bridge.setTextAlign("center")
    assert bridge.selectedTextAlign == "center"

    # Check that objects and renderObjects expose these
    objs = bridge.objects
    assert objs[0]["fontSize"] == 28.0
    assert objs[0]["fontFamily"] == "Roboto"
    assert objs[0]["fontWeight"] == "bold"
    assert objs[0]["fontStyle"] == "italic"
    assert objs[0]["textAlign"] == "center"

    # Test setProp reflection
    bridge.setProp("fontSize", 36.0)
    assert bridge.selectedFontSize == 36.0


def test_rotation_shear_and_flip() -> None:
    bridge = Bridge()

    r_id = bridge.createShapeBounds("rectangle", 100.0, 100.0, 200.0, 100.0)
    bridge.select(r_id)

    assert bridge.document.objects[0].rotation == 0.0
    bridge.rotateSelected(45.0)
    assert bridge.document.objects[0].rotation == 45.0

    bridge.rotateSelected(45.0)
    assert bridge.document.objects[0].rotation == 90.0

    bridge.setProp("rotation", 180.0)
    assert bridge.document.objects[0].rotation == 180.0

    bridge.setProp("shear", 15.0)
    assert bridge.document.objects[0].shear == 15.0
