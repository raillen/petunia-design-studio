import math
import random
from pathlib import Path

from petunia_app.model import (
    AddObject,
    BreakContourCommand,
    DeleteNodeCommand,
    Document,
    History,
    JoinContoursCommand,
    ReverseContourCommand,
    SceneObject,
    SetNodePositionCommand,
)
from petunia_app.paths import (
    Contour,
    FillRule,
    PathNode,
    Point,
    VectorPath,
    break_contour,
    contour_bounds,
    cubic_at,
    cubic_bounds,
    cubic_flatten,
    cubic_split,
    cubic_tangent,
    delete_node,
    flatten_path,
    insert_node,
    join_contours,
    nearest_point,
    new_id,
    path_bounds,
    reverse_contour,
    segment_controls,
    set_node_position,
)


def make_line_contour() -> Contour:
    c = Contour(id=new_id())
    c.nodes = [
        PathNode(id=new_id(), x=0.0, y=0.0),
        PathNode(id=new_id(), x=100.0, y=0.0),
        PathNode(id=new_id(), x=100.0, y=100.0),
    ]
    return c


def make_cubic_contour() -> Contour:
    c = Contour(id=new_id())
    a = PathNode(id=new_id(), x=0.0, y=0.0, out_handle=(40.0, 60.0))
    b = PathNode(id=new_id(), x=100.0, y=0.0, in_handle=(-40.0, 60.0))
    c.nodes = [a, b]
    return c


def test_cubic_evaluation_and_split() -> None:
    p0, p1, p2, p3 = (0.0, 0.0), (0.0, 100.0), (100.0, 100.0), (100.0, 0.0)
    mid = cubic_at(p0, p1, p2, p3, 0.5)
    assert abs(mid[0] - 50.0) < 1e-9 and abs(mid[1] - 75.0) < 1e-9
    left, right = cubic_split(p0, p1, p2, p3, 0.5)
    assert left[3] == right[0]
    assert left[0] == p0 and right[3] == p3
    joined = cubic_at(*left, 1.0)
    assert abs(joined[0] - 50.0) < 1e-9


def test_cubic_bounds_exact() -> None:
    # control points at y=200: the curve itself peaks at y=150 (t=0.5)
    p0, p1, p2, p3 = (0.0, 0.0), (50.0, 200.0), (150.0, 200.0), (200.0, 0.0)
    x0, _y0, x1, y1 = cubic_bounds(p0, p1, p2, p3)
    assert x0 <= 0.0 and x1 >= 200.0
    assert abs(y1 - 150.0) < 1e-9  # exact bulge from derivative root
    poly = cubic_flatten(p0, p1, p2, p3, tolerance=0.01)
    assert all(0.0 - 1e-9 <= p[1] <= y1 + 1e-9 for p in poly)


def test_flatten_invariants() -> None:
    p0, p1, p2, p3 = (0.0, 0.0), (0.0, 100.0), (100.0, 100.0), (100.0, 0.0)
    poly = cubic_flatten(p0, p1, p2, p3, tolerance=0.05)
    assert len(poly) >= 2
    assert poly[0] == p0 and poly[-1] == p3
    for p in poly:
        assert all(math.isfinite(v) for v in p)
    # monotone in x for this curve
    xs = [p[0] for p in poly]
    assert all(xs[i] <= xs[i + 1] + 1e-9 for i in range(len(xs) - 1))


def test_insert_node_preserves_shape() -> None:
    c = make_cubic_contour()
    original_ctrl = segment_controls(c.nodes[0], c.nodes[1])
    assert original_ctrl is not None
    original_mid = cubic_at(*original_ctrl, 0.5)
    node = insert_node(c, 0, 0.5)
    assert len(c.nodes) == 3
    assert node.id != c.nodes[0].id and node.id != c.nodes[2].id
    assert c.nodes[1].id == node.id  # inserted right after segment start
    # new node anchor sits exactly on the original curve at t=0.5
    assert abs(node.x - original_mid[0]) < 1e-9
    assert abs(node.y - original_mid[1]) < 1e-9
    # left half midpoint equals original curve at t=0.25
    left_ctrl = segment_controls(c.nodes[0], c.nodes[1])
    assert left_ctrl is not None
    quarter = cubic_at(*original_ctrl, 0.25)
    left_mid = cubic_at(*left_ctrl, 0.5)
    assert abs(left_mid[0] - quarter[0]) < 1e-9
    assert abs(left_mid[1] - quarter[1]) < 1e-9


def test_insert_node_line_segment() -> None:
    c = make_line_contour()
    node = insert_node(c, 0, 0.25)
    assert abs(node.x - 25.0) < 1e-9 and abs(node.y) < 1e-9
    assert node.in_handle is None and node.out_handle is None


def test_insert_at_endpoints() -> None:
    for t in (0.0, 1.0):
        c = make_cubic_contour()
        insert_node(c, 0, t)
        assert len(c.nodes) == 3


def test_reverse_twice_is_identity() -> None:
    c = make_cubic_contour()
    before = c.to_json()
    reverse_contour(c)
    reverse_contour(c)
    assert c.to_json() == before


def test_join_and_break() -> None:
    a = make_line_contour()
    b = Contour(id=new_id())
    last = a.nodes[-1]
    b.nodes = [
        PathNode(id=new_id(), x=last.x, y=last.y),
        PathNode(id=new_id(), x=last.x + 50, y=last.y),
    ]
    assert join_contours(a, b)
    assert len(a.nodes) == 4
    assert len(a.nodes[3 - 1].id) > 0

    c = make_cubic_contour()
    c.closed = True
    c.nodes.append(PathNode(id=new_id(), x=0.0, y=100.0))
    extra = break_contour(c, 1)
    assert extra is None
    assert not c.closed
    assert len(c.nodes) == 4  # rotated + duplicated endpoint

    d = make_line_contour()
    second = break_contour(d, 1)
    assert second is not None
    assert len(d.nodes) == 2
    assert len(second.nodes) == 2
    assert d.nodes[-1].x == second.nodes[0].x
    assert d.nodes[-1].id != second.nodes[0].id


def test_delete_node_constraints() -> None:
    c = make_line_contour()
    assert delete_node(c, 1)
    assert len(c.nodes) == 2
    assert not delete_node(c, 0)  # needs >= 2 nodes
    closed = make_cubic_contour()
    closed.closed = True
    closed.nodes.append(PathNode(id=new_id(), x=50.0, y=100.0))
    assert delete_node(closed, 0)
    assert not delete_node(closed, 0)  # needs >= 2 nodes


def test_set_node_position() -> None:
    c = make_line_contour()
    assert set_node_position(c, 0, 5.0, 6.0)
    assert (c.nodes[0].x, c.nodes[0].y) == (5.0, 6.0)
    assert not set_node_position(c, 99, 0.0, 0.0)


def test_path_bounds_and_nearest() -> None:
    path = VectorPath(contours=[make_cubic_contour()])
    x0, _y0, x1, _y1 = path_bounds(path)
    assert x0 <= 0.0 and x1 >= 100.0
    near = nearest_point(path, 50.0, 30.0)
    assert near is not None
    assert near.distance_squared >= 0.0
    assert all(math.isfinite(v) for v in near.tangent)
    assert near.contour_index == 0 and near.segment_index == 0


def test_pen_workflow_one_undo_step() -> None:
    from petunia_app.bridge import Bridge

    bridge = Bridge()
    bridge.setTool("pen")
    bridge.penClick(0.0, 0.0)
    bridge.penClick(100.0, 0.0)
    bridge.penClick(100.0, 100.0)
    assert bridge.penActive
    assert len(bridge.document.objects) == 0
    bridge.penFinish()
    assert not bridge.penActive
    assert len(bridge.document.objects) == 1
    obj = bridge.document.objects[0]
    assert obj.type == "path"
    assert obj.path is not None
    assert len(obj.path.contours) == 1
    assert len(obj.path.contours[0].nodes) == 3
    assert not obj.path.contours[0].closed
    bridge.history.undo(bridge.document)
    assert len(bridge.document.objects) == 0
    bridge.history.redo(bridge.document)
    assert len(bridge.document.objects) == 1


def test_pen_close_on_first_node() -> None:
    from petunia_app.bridge import Bridge

    bridge = Bridge()
    bridge.setTool("pen")
    bridge.penClick(8.0, 8.0)
    bridge.penClick(100.0, 8.0)
    bridge.penClick(100.0, 100.0)
    bridge.penClick(8.0, 8.0)  # near first node -> close & commit
    assert not bridge.penActive
    assert len(bridge.document.objects) == 1
    obj = bridge.document.objects[0]
    assert obj.path is not None
    assert obj.path.contours[0].closed


def test_pen_snaps_to_grid() -> None:
    from petunia_app.bridge import Bridge

    bridge = Bridge()
    bridge.setTool("pen")
    bridge.penClick(13.0, 13.0)
    bridge.penClick(100.0, 100.0)
    bridge.penFinish()
    obj = bridge.document.objects[0]
    assert obj.path is not None
    nodes = obj.path.contours[0].nodes
    assert nodes[0].x == 16.0 and nodes[0].y == 16.0


def test_node_tool_id_stability() -> None:
    from petunia_app.bridge import Bridge

    bridge = Bridge()
    path = VectorPath(contours=[make_cubic_contour()])
    obj = SceneObject(type="path", name="P", path=path)
    bridge.history.execute(AddObject(obj), bridge.document)
    assert obj.path is not None
    original_ids = [n.id for n in obj.path.contours[0].nodes]

    bridge.insertNode(obj.id, 0, 0, 0.5)
    assert obj.path is not None
    contours = obj.path.contours
    assert len(contours[0].nodes) == 3
    assert [n.id for n in contours[0].nodes][:1] == original_ids[:1]
    assert contours[0].nodes[2].id == original_ids[1]

    # undo/redo must preserve IDs exactly
    bridge.history.undo(bridge.document)
    assert obj.path is not None
    assert [n.id for n in obj.path.contours[0].nodes] == original_ids
    bridge.history.redo(bridge.document)
    assert obj.path is not None
    contours = obj.path.contours
    assert [n.id for n in contours[0].nodes][:1] == original_ids[:1]
    assert contours[0].nodes[2].id == original_ids[1]


def test_topology_commands_undo_redo() -> None:
    doc = Document()
    history = History()
    # second contour starts where the first ends so join is valid
    tail = Contour(id=new_id())
    tail.nodes = [
        PathNode(id=new_id(), x=100.0, y=0.0),
        PathNode(id=new_id(), x=100.0, y=100.0),
        PathNode(id=new_id(), x=200.0, y=100.0),
    ]
    path = VectorPath(contours=[make_cubic_contour(), tail])
    obj = SceneObject(type="path", name="P", path=path)
    history.execute(AddObject(obj), doc)

    def obj_path() -> VectorPath:
        result = doc.objects[0].path
        assert result is not None
        return result

    history.execute(ReverseContourCommand(obj.id, 0), doc)
    assert len(obj_path().contours[0].nodes) == 2
    history.undo(doc)
    assert obj_path().contours[0].nodes[0].x == 0.0

    history.execute(BreakContourCommand(obj.id, 1, 1), doc)
    assert len(obj_path().contours) == 3
    history.undo(doc)
    assert len(obj_path().contours) == 2

    history.execute(JoinContoursCommand(obj.id, 0, 1), doc)
    assert len(obj_path().contours) == 1
    history.undo(doc)
    assert len(obj_path().contours) == 2

    history.execute(SetNodePositionCommand(obj.id, 0, 0, 42.0, 43.0), doc)
    assert obj_path().contours[0].nodes[0].x == 42.0
    history.undo(doc)
    assert obj_path().contours[0].nodes[0].x == 0.0

    history.execute(DeleteNodeCommand(obj.id, 1, 1), doc)
    assert len(obj_path().contours[1].nodes) == 2
    history.undo(doc)
    assert len(obj_path().contours[1].nodes) == 3


def test_path_save_reopen_roundtrip(tmp_path: Path) -> None:
    from petunia_app.ptnd_scene import open_ptnd, save_ptnd

    doc = Document(name="Paths", width=500, height=500)
    cubic = make_cubic_contour()
    cubic.closed = True
    cubic.nodes.append(
        PathNode(id=new_id(), x=50.0, y=100.0, in_handle=(0.0, -30.0))
    )
    obj = SceneObject(
        type="path",
        name="Curve",
        path=VectorPath(contours=[cubic, make_line_contour()], fill_rule=FillRule.EVENODD),
    )
    doc.objects.append(obj)

    pkg = tmp_path / "path.ptnd"
    save_ptnd(pkg, doc)
    back = open_ptnd(pkg)
    assert back.to_json() == doc.to_json()
    assert back.objects[0].path is not None
    assert doc.objects[0].path is not None
    assert back.objects[0].path.fill_rule == FillRule.EVENODD
    assert back.objects[0].path.contours[0].closed
    ids_before = [n.id for c in doc.objects[0].path.contours for n in c.nodes]
    ids_after = [n.id for c in back.objects[0].path.contours for n in c.nodes]
    assert ids_before == ids_after


def test_fuzz_curve_invariants() -> None:
    rng = random.Random(42)
    for _ in range(200):
        scale = rng.choice([0.001, 1.0, 1000.0])
        pts: tuple[Point, Point, Point, Point] = (
            (rng.uniform(-1, 1) * scale, rng.uniform(-1, 1) * scale),
            (rng.uniform(-1, 1) * scale, rng.uniform(-1, 1) * scale),
            (rng.uniform(-1, 1) * scale, rng.uniform(-1, 1) * scale),
            (rng.uniform(-1, 1) * scale, rng.uniform(-1, 1) * scale),
        )
        poly = cubic_flatten(pts[0], pts[1], pts[2], pts[3], tolerance=0.25)
        assert len(poly) >= 2
        for p in poly:
            assert all(math.isfinite(v) for v in p)
        assert poly[0] == pts[0] and poly[-1] == pts[3]
        left, right = cubic_split(pts[0], pts[1], pts[2], pts[3], 0.5)
        assert left[3] == right[0]
        x0, y0, x1, y1 = cubic_bounds(pts[0], pts[1], pts[2], pts[3])
        assert x0 <= x1 and y0 <= y1
        assert all(x0 - 1e-6 <= p[0] <= x1 + 1e-6 for p in poly)
        assert all(y0 - 1e-6 <= p[1] <= y1 + 1e-6 for p in poly)
        t = rng.random()
        tangent = cubic_tangent(pts[0], pts[1], pts[2], pts[3], t)
        assert all(math.isfinite(v) for v in tangent)


def test_contour_bounds_open_and_closed() -> None:
    open_c = make_line_contour()
    assert contour_bounds(open_c) == (0.0, 0.0, 100.0, 100.0)
    closed = Contour(id=new_id())
    closed.closed = True
    closed.nodes = [
        PathNode(id=new_id(), x=0.0, y=0.0),
        PathNode(id=new_id(), x=10.0, y=0.0),
    ]
    assert contour_bounds(closed) == (0.0, 0.0, 10.0, 0.0)


def test_flatten_path_multiple_contours() -> None:
    path = VectorPath(contours=[make_line_contour(), make_cubic_contour()])
    polylines = flatten_path(path, tolerance=0.1)
    assert len(polylines) == 2
    assert polylines[0][0] == (0.0, 0.0)
    assert polylines[1][0] == (0.0, 0.0)
