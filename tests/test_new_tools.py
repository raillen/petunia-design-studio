from __future__ import annotations

import pytest
from petunia_app.bridge import Bridge
from petunia_app.paths import VectorPath


def test_cog_shape_creation():
    bridge = Bridge()
    bridge.newDocument()

    # Set cog parameters
    bridge.setCogTeeth(12)
    bridge.setCogToothDepth(0.3)
    bridge.setCogHoleRadius(0.25)
    assert bridge.cogTeeth == 12
    assert bridge.cogToothDepth == 0.3
    assert bridge.cogHoleRadius == 0.25

    # Create cog shape
    cog_id = bridge.createShapeBounds("cog", 100, 100, 200, 200)
    assert cog_id != ""

    obj = next((o for o in bridge.document.objects if o.id == cog_id), None)
    assert obj is not None
    assert obj.path is not None
    assert isinstance(obj.path, VectorPath)
    # 12 teeth = 48 nodes on outer contour + 8 nodes on inner hole contour = 56 nodes
    total_nodes = sum(len(c.nodes) for c in obj.path.contours)
    assert total_nodes == 48 + 8

    # Test cog without hole
    bridge.setCogHoleRadius(0.0)
    cog2_id = bridge.createShapeBounds("cog", 300, 100, 200, 200)
    obj2 = next((o for o in bridge.document.objects if o.id == cog2_id), None)
    assert obj2 is not None
    assert obj2.path is not None
    assert len(obj2.path.contours) == 1
    total_nodes_2 = sum(len(c.nodes) for c in obj2.path.contours)
    assert total_nodes_2 == 48


def test_knife_cut_splits_rectangle():
    bridge = Bridge()
    bridge.newDocument()

    rect_id = bridge.createShapeBounds("rectangle", 100, 100, 200, 200)
    bridge.select(rect_id)
    assert len(bridge.document.objects) == 1

    # Vertical cut through middle (x=200, y=50 to x=200, y=350)
    slices = bridge.knifeCut(200, 50, 200, 350)
    assert slices == 1
    # Original rectangle was replaced with 2 slices
    assert len(bridge.document.objects) == 2

    # Check both slices are valid VectorPaths
    for o in bridge.document.objects:
        assert o.path is not None
        assert isinstance(o.path, VectorPath)
        total_nodes = sum(len(c.nodes) for c in o.path.contours)
        assert total_nodes >= 4
        assert o.fill == bridge.currentFill


def test_knife_cut_no_intersection():
    bridge = Bridge()
    bridge.newDocument()

    rect_id = bridge.createShapeBounds("rectangle", 100, 100, 200, 200)
    bridge.select(rect_id)

    # Cut line far away from rectangle
    slices = bridge.knifeCut(500, 50, 500, 350)
    assert slices == 0
    assert len(bridge.document.objects) == 1


def test_pixel_lasso_selection():
    bridge = Bridge()
    bridge.newDocument()

    # Create 3 shapes at different positions
    id1 = bridge.createShapeBounds("rectangle", 50, 50, 50, 50)     # Center: 75, 75
    id2 = bridge.createShapeBounds("rectangle", 200, 200, 50, 50)   # Center: 225, 225
    id3 = bridge.createShapeBounds("rectangle", 500, 500, 50, 50)   # Center: 525, 525

    # Polygon enclosing only object 1
    polygon1 = [{"x": 20, "y": 20}, {"x": 120, "y": 20}, {"x": 120, "y": 120}, {"x": 20, "y": 120}]
    bridge.lassoSelect(polygon1, additive=False)
    assert id1 in bridge.selectedIds
    assert id2 not in bridge.selectedIds
    assert id3 not in bridge.selectedIds

    # Additive polygon enclosing object 2
    polygon2 = [{"x": 180, "y": 180}, {"x": 280, "y": 180}, {"x": 280, "y": 280}, {"x": 180, "y": 280}]
    bridge.lassoSelect(polygon2, additive=True)
    assert id1 in bridge.selectedIds
    assert id2 in bridge.selectedIds
    assert id3 not in bridge.selectedIds


def test_clone_stamp_source_lifecycle():
    bridge = Bridge()
    assert not bridge.cloneSourceSet

    bridge.setCloneSource(150.5, 220.3)
    assert bridge.cloneSourceSet
    assert pytest.approx(bridge.cloneSourceX, 0.1) == 150.5
    assert pytest.approx(bridge.cloneSourceY, 0.1) == 220.3

    bridge.clearCloneSource()
    assert not bridge.cloneSourceSet


def test_freehand_stroke_with_blend_mode():
    bridge = Bridge()
    bridge.newDocument()

    points = [{"x": 10.0, "y": 10.0}, {"x": 50.0, "y": 50.0}, {"x": 100.0, "y": 100.0}]
    stroke_id = bridge.createFreehandStroke(points, stroke_color="#ff0055", stroke_width=8.0, blend_mode="multiply")
    assert stroke_id != ""

    obj = next((o for o in bridge.document.objects if o.id == stroke_id), None)
    assert obj is not None
    assert obj.blend_mode == "multiply"
    assert obj.stroke_width == 8.0
    assert obj.stroke == "#ff0055"
