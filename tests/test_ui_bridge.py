"""Bridge interaction tests simulating QML slot calls (G028/G029 UI flow).

State assertions use the typed Python API (bridge.document); the QML property
view is exercised through cast helpers because pyright resolves PySide6
@Property accessors as Property objects.
"""

from __future__ import annotations

from typing import Any, cast

from petunia_app.bridge import Bridge


def qobjects(bridge: Bridge) -> list[dict[str, Any]]:
    return cast(list[dict[str, Any]], bridge.objects)


def qlayers(bridge: Bridge) -> list[dict[str, Any]]:
    return cast(list[dict[str, Any]], bridge.layers)


def qnode(bridge: Bridge) -> dict[str, Any]:
    return cast(dict[str, Any], bridge.selectedNode)


def qpreview(bridge: Bridge) -> list[dict[str, Any]]:
    return cast(list[dict[str, Any]], bridge.penPreview)


def test_shape_creation_and_inspector_flow() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100.0, 100.0)
    assert len(bridge.document.objects) == 1
    assert bridge.selectedId != ""
    obj = qobjects(bridge)[0]
    assert obj["type"] == "rectangle"
    assert abs(obj["x"] - 50.0) < 1e-9  # centered

    # inspector setProp calls from QML pass strings
    bridge.setProp("x", "60")
    bridge.setProp("y", "70")
    bridge.setProp("width", "200")
    bridge.setProp("height", "120")
    bridge.setProp("fill", "#ff00ff")
    bridge.setProp("strokeWidth", "3.5")
    bridge.setProp("visible", "false")
    scene = bridge.document.objects[0]
    assert scene.x == 60.0 and scene.y == 70.0
    assert scene.width == 200.0 and scene.height == 120.0
    assert scene.fill == "#ff00ff"
    assert scene.stroke_width == 3.5
    assert scene.visible is False

    # selection + multi-select + group + undo/redo
    bridge.create_shape("ellipse", 300.0, 300.0)
    bridge.select(bridge.document.objects[0].id)
    first_id = cast(str, bridge.selectedId)
    bridge.toggleSelect(bridge.document.objects[1].id)
    assert bridge.selectedIds == [first_id, bridge.document.objects[1].id]
    bridge.groupSelected()
    assert len(bridge.document.objects) == 1
    assert bridge.document.objects[0].type == "group"
    bridge.history.undo(bridge.document)
    assert len(bridge.document.objects) == 2
    bridge.history.redo(bridge.document)
    assert len(bridge.document.objects) == 1
    bridge.ungroupSelected()
    assert len(bridge.document.objects) == 2

    # move + delete
    bridge.select(first_id)
    bridge.moveSelected(10.0, 5.0)
    assert bridge.document.objects[0].x == 70.0
    bridge.deleteSelected()
    assert len(bridge.document.objects) == 1

    # layers model is flattened with depth
    assert [layer["type"] for layer in qlayers(bridge)] == ["ellipse"]


def test_hit_testing_and_zoom() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100.0, 100.0)
    obj_id = bridge.document.objects[0].id
    assert bridge.objectAt(100.0, 100.0) == obj_id
    assert bridge.objectAt(0.0, 0.0) == ""
    bridge.setZoom(2.0)
    assert bridge.zoom == 2.0
    bridge.zoomOut()
    assert cast(float, bridge.zoom) < 2.0
    bridge.zoomFit()
    assert bridge.zoom == 1.0


def test_pen_and_node_flow() -> None:
    bridge = Bridge()
    bridge.setTool("pen")
    assert bridge.tool == "pen"
    bridge.penClick(0.0, 0.0)
    bridge.penClick(100.0, 0.0)
    bridge.penClick(100.0, 100.0)
    assert bridge.penActive
    preview = qpreview(bridge)
    assert len(preview) == 1
    assert len(preview[0]["nodes"]) == 3
    bridge.penFinish()
    assert not bridge.penActive
    assert len(bridge.document.objects) == 1
    obj = bridge.document.objects[0]
    assert obj.type == "path"
    assert obj.path is not None
    assert obj.path.fill_rule.value == "nonzero"

    # node tool: select first node (snapped to 0,0), move it, insert, reverse, break, delete
    bridge.setTool("node")
    hit = bridge.nodeAt(1.0, 1.0)
    assert hit["objId"] == obj.id
    assert hit["nodeIndex"] == 0
    bridge.moveNode(hit["objId"], hit["contourIndex"], hit["nodeIndex"], 24.0, 24.0)
    moved = obj.path.contours[0].nodes[0]
    assert moved.x == 24.0 and moved.y == 24.0

    n = qnode(bridge)
    bridge.insertNode(n["objId"], n["contourIndex"], 0, 0.5)
    assert len(obj.path.contours[0].nodes) == 4

    bridge.reverseContour(n["objId"], n["contourIndex"])
    bridge.breakContour(n["objId"], n["contourIndex"], 1)
    assert len(obj.path.contours) == 2

    n2 = qnode(bridge)
    bridge.deleteNode(n2["objId"], n2["contourIndex"], n2["nodeIndex"])

    # tool switch cancels pen
    bridge.setTool("pen")
    bridge.penClick(0.0, 0.0)
    bridge.setTool("select")
    assert not bridge.penActive

    # new document resets everything
    bridge.newDocument()
    assert len(bridge.document.objects) == 0
    assert bridge.selectedId == ""
    assert qlayers(bridge) == []


def test_export_and_serialization_from_bridge() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100.0, 100.0)
    svg = bridge.exportSvg()
    assert "<svg" in svg and "<rect" in svg
    bridge.save("/tmp/bridge-test.ptnd")
    bridge.create_shape("ellipse", 200.0, 200.0)
    bridge.open("/tmp/bridge-test.ptnd")
    assert len(bridge.document.objects) == 1
    assert bridge.document.objects[0].type == "rectangle"


def test_affinity_ui_features_and_demo_doc() -> None:
    bridge = Bridge()
    bridge.loadDemoDocument()
    assert bridge.document.name == "Inioluwa Abiri"
    assert bridge.document.width == 1200.0
    assert bridge.document.height == 800.0
    assert len(bridge.document.objects) >= 1
    layers = qlayers(bridge)
    assert any(l["name"] == "Artboard1" for l in layers)
    assert any(l["name"] == "Face (makeup)" and l["colorTag"] == "#f97316" for l in layers)
    assert any(l["name"] == "Stars" and l["colorTag"] == "#a855f7" for l in layers)

    # persona, color & alignment tests
    bridge.setPersona("pixel")
    assert cast(str, bridge.persona) == "pixel"
    bridge.setPersona("export")
    assert cast(str, bridge.persona) == "export"
    bridge.setPersona("vector")
    assert cast(str, bridge.persona) == "vector"

    # renderObjects returns flattened drawable objects
    render_objs = cast(list[dict[str, Any]], bridge.renderObjects)
    assert len(render_objs) > 0

    bridge.setFillColor("#E120A5")
    assert cast(str, bridge.currentFill) == "#E120A5"
    bridge.setStrokeColor("#FFFFFF")
    assert cast(str, bridge.currentStroke) == "#FFFFFF"
    bridge.swapColors()
    assert cast(str, bridge.currentFill) == "#FFFFFF"
    assert cast(str, bridge.currentStroke) == "#E120A5"

    # snapping
    assert bridge.snappingEnabled is True
    bridge.toggleSnapping()
    assert bridge.snappingEnabled is False
    bridge.toggleSnapping()
    assert bridge.snappingEnabled is True

    # duplicate
    bridge.create_shape("rectangle", 50.0, 50.0)
    initial_count = len(bridge.document.objects)
    bridge.duplicateSelected()
    assert len(bridge.document.objects) == initial_count + 1

    # alignment
    bridge.alignSelected("center")

    # history inspection
    items = cast(list[str], bridge.historyItems)
    assert len(items) > 0


def test_artboards_and_viewport_features() -> None:
    bridge = Bridge()

    # Initial empty state has fallback document artboard
    arts = cast(list[dict[str, Any]], bridge.artboards)
    assert len(arts) == 1
    assert arts[0]["isDocument"] is True
    assert bridge.artboardCount == 1
    assert bridge.activeArtboardIndex == 0
    assert bridge.activeArtboardName == "Untitled"

    # Create custom artboard
    art_id1 = bridge.createArtboard("Web 1", 50.0, 50.0, 1200.0, 800.0, "#ffffff")
    assert art_id1 != ""
    assert bridge.artboardCount == 1
    assert bridge.activeArtboardName == "Web 1"
    assert bridge.activeArtboardWidth == 1200.0
    assert bridge.activeArtboardHeight == 800.0
    assert bridge.selectedId == art_id1

    # Insert preset artboard (FHD: 1920x1080)
    art_id2 = bridge.insertPresetArtboard("fhd")
    assert art_id2 != ""
    assert bridge.artboardCount == 2
    assert bridge.activeArtboardIndex == 1
    assert "FHD" in bridge.activeArtboardName
    assert bridge.activeArtboardWidth == 1920.0
    assert bridge.activeArtboardHeight == 1080.0

    # Verify smart placement: second artboard is positioned to the right of the first
    art2_obj = bridge.document.objects[1]
    assert art2_obj.x == 50.0 + 1200.0 + 120.0  # 1370.0

    # Navigation: previous, next, select by index
    bridge.prevArtboard()
    assert bridge.activeArtboardIndex == 0
    assert bridge.selectedId == art_id1
    assert bridge.activeArtboardName == "Web 1"

    bridge.nextArtboard()
    assert bridge.activeArtboardIndex == 1
    assert bridge.selectedId == art_id2

    bridge.selectArtboardIndex(0)
    assert bridge.activeArtboardIndex == 0
    assert bridge.selectedId == art_id1

    # Resize artboard
    bridge.resizeArtboard(art_id1, 1400.0, 900.0)
    art1_obj = bridge.document.objects[0]
    assert art1_obj.width == 1400.0
    assert art1_obj.height == 900.0

    # Rename & Background
    bridge.setArtboardName(art_id1, "Desktop App")
    assert bridge.activeArtboardName == "Desktop App"
    bridge.setArtboardBackground(art_id1, "#f8fafc")
    assert art1_obj.fill == "#f8fafc"

    # Fit all artboards
    bridge.fitAllArtboards()
    assert bridge.zoom > 0.0

    # Undo / Redo works for artboards
    bridge.history.undo(bridge.document)  # Undo background
    bridge.history.undo(bridge.document)  # Undo rename
    bridge.history.undo(bridge.document)  # Undo resize height
    bridge.history.undo(bridge.document)  # Undo resize width
    assert art1_obj.width == 1200.0
    assert art1_obj.height == 800.0

    # Undo artboard 2 creation
    bridge.history.undo(bridge.document)
    assert len(bridge._get_artboards()) == 1

    # Redo artboard 2 creation
    bridge.history.redo(bridge.document)
    assert len(bridge._get_artboards()) == 2


def test_new_interactive_tools_flow() -> None:
    bridge = Bridge()

    # 1. Shape Bounds Creation (Rectangle & Ellipse)
    r_id = bridge.createShapeBounds("rectangle", 50.0, 60.0, 200.0, 150.0, fill="#ff0000", corner_radius=12.0)
    assert r_id != ""
    assert len(bridge.document.objects) == 1
    rect = bridge.document.objects[0]
    assert rect.x == 50.0 and rect.y == 60.0
    assert rect.width == 200.0 and rect.height == 150.0
    assert rect.fill == "#ff0000"
    assert rect.corner_radius == 12.0
    assert bridge.selectedCornerRadius == 12.0

    e_id = bridge.createShapeBounds("ellipse", 300.0, 100.0, 80.0, 80.0, fill="#00ff00")
    assert e_id != ""
    assert len(bridge.document.objects) == 2
    ellipse = bridge.document.objects[1]
    assert ellipse.type == "ellipse"

    # 2. Eyedropper Sampling
    sampled = bridge.sampleColorAt(60.0, 70.0)
    assert sampled == "#ff0000"
    assert bridge.currentFill == "#ff0000"

    # 3. Measurement Tool
    m = bridge.measureDistance(0.0, 0.0, 300.0, 400.0)
    assert m["distance"] == 500.0
    assert m["dx"] == 300.0
    assert m["dy"] == 400.0

    # 4. Gradient Fill Tool
    bridge.select(r_id)
    bridge.setObjectGradient(r_id, ["#ff0000", "#0000ff"])
    assert rect.gradient == ["#ff0000", "#0000ff"]
    assert bridge.selectedGradient == ["#ff0000", "#0000ff"]
    bridge.reverseObjectGradient(r_id)
    assert rect.gradient == ["#0000ff", "#ff0000"]
    bridge.removeObjectGradient(r_id)
    assert rect.gradient == []

    # 5. Corner Radius Adjustment & Convert to Curves
    bridge.setCornerRadius(r_id, 24.0)
    assert rect.corner_radius == 24.0
    path_id = bridge.convertToCurves(r_id)
    assert path_id == r_id
    assert rect.type == "path"
    assert rect.path is not None
    assert len(rect.path.contours) == 1
    assert len(rect.path.contours[0].nodes) == 4

    # 6. Freehand Stroke (Pencil / Brush Tool)
    stroke_id = bridge.createFreehandStroke([[10.0, 10.0], [20.0, 25.0], [40.0, 50.0], [80.0, 90.0]], stroke_color="#ffff00", stroke_width=4.0)
    assert stroke_id != ""
    stroke_obj = bridge._find(stroke_id)
    assert stroke_obj is not None
    assert stroke_obj.type == "path"
    assert stroke_obj.stroke == "#ffff00"
    assert stroke_obj.stroke_width == 4.0
    assert stroke_obj.path is not None

    # 7. Text Bounds Creation
    t_id = bridge.createTextBounds(100.0, 200.0, 250.0, 60.0, "Design Profissional")
    assert t_id != ""
    text_obj = bridge._find(t_id)
    assert text_obj is not None
    assert text_obj.type == "text"
    assert text_obj.width == 250.0
    assert text_obj.height == 60.0

    # 8. Place Image
    img_id = bridge.placeImage("/tmp/foto.png", 50.0, 50.0, 320.0, 240.0)
    assert img_id != ""
    img_obj = bridge._find(img_id)
    assert img_obj is not None
    assert "foto.png" in img_obj.name

    # 9. Transform Bounds Tool (setObjectBounds)
    bridge.setObjectBounds(img_id, 80.0, 96.0, 400.0, 300.0)
    assert img_obj.x == 80.0
    assert img_obj.y == 96.0
    assert img_obj.width == 400.0
    assert img_obj.height == 300.0
    # Undo / Redo for SetObjectBounds
    bridge.undo()
    assert img_obj.x == 50.0
    assert img_obj.width == 320.0
    bridge.redo()
    assert img_obj.x == 80.0
    assert img_obj.width == 400.0

    # 10. Node Tool Bézier Handle Hit-Testing
    bridge.select(path_id)
    n0 = bridge.nodeAt(50.0, 60.0)
    assert n0.get("objId") == path_id
    # Set handles on node 0
    bridge.setNodeKind(path_id, 0, 0, "smooth")
    # Query handle knob location
    contour = rect.path.contours[0]
    node = contour.nodes[0]
    assert node.out_handle is not None
    hx = node.x + node.out_handle[0]
    hy = node.y + node.out_handle[1]
    hit_h = bridge.nodeAt(hx, hy)
    assert hit_h.get("handle") == "out"
    assert hit_h.get("nodeIndex") == 0

    # 12. History & Transform Operations
    assert bridge.canUndo is True
    assert bridge.canRedo is False
    hist_len = len(bridge.historyItems)
    assert hist_len > 0
    # Jump history test
    bridge.jumpHistory(hist_len - 2)
    assert len(bridge.historyItems) == hist_len - 1
    assert bridge.canRedo is True
    bridge.redo()
    assert len(bridge.historyItems) == hist_len

    # Test Rotate and Flip
    bridge.select(path_id)
    bridge.rotateSelected(90.0)
    assert bridge.canUndo is True
    bridge.undo()
    assert bridge.canRedo is True
    bridge.redo()

    bridge.flipSelected("horizontal")
    assert bridge.canUndo is True
    bridge.undo()


