"""G031 — Boolean engine, compound paths, live booleans, Shape Builder.

Differential corpus (known areas), holes, coincident edges, self
intersections, multi-operand ordering, divide piece count, provenance,
style policy, undo/redo atomicity, fuzz invariants and a 10k-segment
benchmark per the G031 evidence list.
"""

from __future__ import annotations

import random
import time
from typing import Any, cast

from petunia_app.boolean import (
    BooleanError,
    BooleanOp,
    boolean_paths,
    build_shape_faces,
    compound_paths,
    face_at_point,
    faces_to_path,
    path_area,
)
from petunia_app.bridge import Bridge
from petunia_app.model import AddObject, Document, SceneObject
from petunia_app.paths import Contour, FillRule, PathNode, VectorPath, new_id


def rect_path(x: float, y: float, w: float, h: float) -> VectorPath:
    nodes = [
        PathNode(id=new_id(), x=x, y=y),
        PathNode(id=new_id(), x=x + w, y=y),
        PathNode(id=new_id(), x=x + w, y=y + h),
        PathNode(id=new_id(), x=x, y=y + h),
    ]
    return VectorPath(
        contours=[Contour(id=new_id(), nodes=nodes, closed=True)],
        fill_rule=FillRule.NONZERO,
    )


def operands(*rects: tuple[float, float, float, float]) -> list[tuple[VectorPath, str]]:
    return [(rect_path(*r), str(i)) for i, r in enumerate(rects)]


def area_of(result: Any) -> float:
    return path_area(result.path)


def test_union_subtract_intersect_xor_differential() -> None:
    ops = operands((0, 0, 100, 100), (50, 0, 100, 100))
    union = boolean_paths(ops, BooleanOp.UNION)[0]
    assert abs(area_of(union) - 15000.0) < 1e-6
    sub = boolean_paths(ops, BooleanOp.SUBTRACT)[0]
    assert abs(area_of(sub) - 5000.0) < 1e-6
    inter = boolean_paths(ops, BooleanOp.INTERSECT)[0]
    assert abs(area_of(inter) - 5000.0) < 1e-6
    xor = boolean_paths(ops, BooleanOp.XOR)[0]
    assert abs(area_of(xor) - 10000.0) < 1e-6


def test_union_disjoint_is_sum() -> None:
    ops = operands((0, 0, 100, 100), (200, 200, 50, 50))
    union = boolean_paths(ops, BooleanOp.UNION)[0]
    assert abs(area_of(union) - 12500.0) < 1e-6
    assert len(union.path.contours) == 2


def test_holes() -> None:
    ops = operands((0, 0, 100, 100), (40, 40, 20, 20))
    sub = boolean_paths(ops, BooleanOp.SUBTRACT)[0]
    assert abs(area_of(sub) - 9600.0) < 1e-6
    assert len(sub.path.contours) == 2  # outer + hole


def test_nested_shapes() -> None:
    ops = operands((0, 0, 200, 200), (20, 20, 160, 160), (50, 50, 100, 100))
    sub = boolean_paths(ops, BooleanOp.SUBTRACT)[0]
    # C is inside B, so (A - B) - C == A - B
    assert abs(area_of(sub) - (200 * 200 - 160 * 160)) < 1e-6


def test_coincident_edges() -> None:
    # identical rectangles
    ops = operands((0, 0, 100, 100), (0, 0, 100, 100))
    assert abs(area_of(boolean_paths(ops, BooleanOp.UNION)[0]) - 10000.0) < 1e-6
    assert len(boolean_paths(ops, BooleanOp.UNION)[0].path.contours) == 1
    # shared edge
    ops = operands((0, 0, 100, 100), (100, 0, 100, 100))
    assert abs(area_of(boolean_paths(ops, BooleanOp.UNION)[0]) - 20000.0) < 1e-6
    # kissing corner
    ops = operands((0, 0, 100, 100), (100, 100, 100, 100))
    assert abs(area_of(boolean_paths(ops, BooleanOp.UNION)[0]) - 20000.0) < 1e-6
    # identical subtract -> empty
    ops = operands((0, 0, 100, 100), (0, 0, 100, 100))
    assert area_of(boolean_paths(ops, BooleanOp.SUBTRACT)[0]) < 1e-9


def test_self_intersecting_bowtie() -> None:
    bowtie = VectorPath(
        contours=[
            Contour(
                id=new_id(),
                nodes=[
                    PathNode(id=new_id(), x=0, y=0),
                    PathNode(id=new_id(), x=100, y=100),
                    PathNode(id=new_id(), x=100, y=0),
                    PathNode(id=new_id(), x=0, y=100),
                ],
                closed=True,
            )
        ],
        fill_rule=FillRule.NONZERO,
    )
    ops = [(bowtie, "0"), (rect_path(-10, -10, 120, 120), "1")]
    first = boolean_paths(ops, BooleanOp.UNION)[0]
    second = boolean_paths(ops, BooleanOp.UNION)[0]
    assert area_of(first) > 0.0
    assert abs(area_of(first) - area_of(second)) < 1e-9  # deterministic
    assert all(
        n.x == n.x and n.y == n.y for c in first.path.contours for n in c.nodes
    )


def test_multi_operand_order_explicit() -> None:
    # A - B - C with ordered operands: (A-B)-C
    ops = operands((0, 0, 100, 100), (25, 0, 50, 100), (75, 0, 50, 100))
    sub = boolean_paths(ops, BooleanOp.SUBTRACT)[0]
    # (A-B) leaves left+right strips; C covers the right strip entirely
    assert abs(area_of(sub) - 2500.0) < 1e-6  # left strip only
    # reverse order: (C-B)-A
    ops = operands((75, 0, 50, 100), (25, 0, 50, 100), (0, 0, 100, 100))
    sub = boolean_paths(ops, BooleanOp.SUBTRACT)[0]
    assert abs(area_of(sub) - 2500.0) < 1e-6  # C minus overlap with B, minus A


def test_divide_pieces_provenance_and_style() -> None:
    ops = operands((0, 0, 100, 100), (50, 0, 100, 100))
    results = boolean_paths(ops, BooleanOp.DIVIDE)
    assert len(results) == 3
    areas = sorted(area_of(r) for r in results)
    assert all(abs(a - 5000.0) < 1e-6 for a in areas)
    # exact provenance: each piece attributed to one source
    for r in results:
        assert len(r.provenance) >= 1
        assert all(p.exact for p in r.provenance)
        assert r.style_source_id in {"0", "1"}
    # topmost piece (source "1") covers the overlap region
    overlap_piece = next(
        r for r in results if r.style_source_id == "1"
    )
    assert abs(area_of(overlap_piece) - 5000.0) < 1e-6


def test_style_policy() -> None:
    ops = operands((0, 0, 100, 100), (50, 0, 100, 100))
    # subtract: style from minuend (first operand)
    results = boolean_paths(ops, BooleanOp.SUBTRACT)
    assert results[0].style_source_id == "0"
    # union/intersect/xor: style from frontmost (last operand)
    for op in (BooleanOp.UNION, BooleanOp.INTERSECT, BooleanOp.XOR):
        assert boolean_paths(ops, op)[0].style_source_id == "1"


def test_evenodd_fill_rule() -> None:
    ops = operands((0, 0, 100, 100), (25, 25, 50, 50))
    union = boolean_paths(ops, BooleanOp.UNION, FillRule.EVENODD)[0]
    assert union.path.fill_rule == FillRule.EVENODD
    # nested squares under even-odd: inner region cancels out
    assert abs(area_of(union) - 7500.0) < 1e-6


def test_open_contour_rejected_without_mutation() -> None:
    open_path = VectorPath(
        contours=[
            Contour(
                id=new_id(),
                nodes=[PathNode(id=new_id(), x=0, y=0), PathNode(id=new_id(), x=10, y=0)],
                closed=False,
            )
        ],
        fill_rule=FillRule.NONZERO,
    )
    try:
        boolean_paths([(open_path, "0"), (rect_path(0, 0, 10, 10), "1")], BooleanOp.UNION)
        raise AssertionError("expected BooleanError")
    except BooleanError as err:
        assert err.involved_ids == ["0"]


def test_insufficient_operands() -> None:
    try:
        boolean_paths(operands((0, 0, 10, 10)), BooleanOp.UNION)
        raise AssertionError("expected BooleanError")
    except BooleanError:
        pass


def test_compound_path_merges_contours() -> None:
    ops = operands((0, 0, 100, 100), (200, 0, 100, 100))
    merged = compound_paths(ops, FillRule.NONZERO)
    assert len(merged.contours) == 2
    assert abs(path_area(merged) - 20000.0) < 1e-6


def test_shape_builder_faces_hit_and_commit() -> None:
    ops = operands((0, 0, 100, 100), (50, 0, 100, 100))
    faces = build_shape_faces(ops)
    assert len(faces) == 3  # A-only, overlap, B-only
    # hit test: middle of A-only region
    a_only = face_at_point(faces, 25.0, 50.0)
    assert a_only >= 0
    assert faces[a_only].sources == ["0"]
    overlap = face_at_point(faces, 75.0, 50.0)
    assert overlap >= 0
    assert set(faces[overlap].sources) == {"0", "1"}
    b_only = face_at_point(faces, 125.0, 50.0)
    assert b_only >= 0
    assert faces[b_only].sources == ["1"]
    # commit only the overlap -> intersection
    path = faces_to_path([faces[overlap]])
    assert abs(path_area(path) - 5000.0) < 1e-6
    # commit all -> union
    path = faces_to_path(faces)
    assert abs(path_area(path) - 15000.0) < 1e-6


def test_shape_builder_non_overlapping() -> None:
    ops = operands((0, 0, 100, 100), (200, 0, 100, 100))
    faces = build_shape_faces(ops)
    assert len(faces) == 2
    assert face_at_point(faces, 250.0, 50.0) >= 0
    assert face_at_point(faces, 150.0, 50.0) == -1


def test_undo_redo_stable_ids() -> None:
    from petunia_app.model import BooleanCommand, History

    doc = Document()
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    b = SceneObject(type="path", name="B", path=rect_path(50, 0, 100, 100))
    doc.objects = [a, b]
    history = History()
    cmd = BooleanCommand([a.id, b.id], "union", "nonzero")
    history.execute(cmd, doc)
    assert len(doc.objects) == 1
    result_id = doc.objects[0].id
    history.undo(doc)
    assert len(doc.objects) == 2
    assert doc.objects[0].id == a.id and doc.objects[1].id == b.id
    history.redo(doc)
    assert len(doc.objects) == 1
    assert doc.objects[0].id == result_id  # stable id across redo


def test_bridge_boolean_flow() -> None:
    bridge = Bridge()
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    b = SceneObject(type="path", name="B", path=rect_path(50, 0, 100, 100))
    bridge.history.execute(AddObject(a), bridge.document)
    bridge.history.execute(AddObject(b), bridge.document)
    bridge.select(a.id)
    bridge.toggleSelect(b.id)
    bridge.boolean("union")
    assert len(bridge.document.objects) == 1
    obj = bridge.document.objects[0]
    assert obj.type == "path"
    assert abs(path_area(obj.path) - 15000.0) < 1e-6 if obj.path else False
    assert bridge.selectedId == obj.id
    # style inherited from frontmost operand
    assert obj.name == "B"
    # undo restores both
    bridge.undo()
    assert len(bridge.document.objects) == 2


def test_bridge_boolean_requires_paths() -> None:
    bridge = Bridge()
    r = SceneObject(type="rectangle")
    bridge.history.execute(AddObject(r), bridge.document)
    bridge.select(r.id)
    bridge.boolean("union")  # only one operand -> message, no crash
    assert len(bridge.document.objects) == 1


def test_bridge_live_boolean() -> None:
    bridge = Bridge()
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    b = SceneObject(type="path", name="B", path=rect_path(50, 0, 100, 100))
    bridge.history.execute(AddObject(a), bridge.document)
    bridge.history.execute(AddObject(b), bridge.document)
    bridge.select(a.id)
    bridge.toggleSelect(b.id)
    bridge.makeLive("union")
    assert len(bridge.document.objects) == 3
    live = bridge.document.objects[2]
    assert live.live is not None and live.live.op == "union"
    assert abs(path_area(live.path) - 15000.0) < 1e-6 if live.path else False
    # move operand B; refresh re-evaluates
    bridge.select(b.id)
    bridge.moveSelected(50.0, 0.0)
    bridge.refreshLive()
    live = bridge.document.objects[2]
    assert abs(path_area(live.path) - 20000.0) < 1e-6 if live.path else False
    # bake
    bridge.select(live.id)
    bridge.bakeLive()
    assert bridge.document.objects[2].live is None


def test_bridge_shape_builder_flow() -> None:
    bridge = Bridge()
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    b = SceneObject(type="path", name="B", path=rect_path(50, 0, 100, 100))
    bridge.history.execute(AddObject(a), bridge.document)
    bridge.history.execute(AddObject(b), bridge.document)
    bridge.select(a.id)
    bridge.toggleSelect(b.id)
    bridge.shapeBuilderStart()
    assert bridge.shapeBuilderActive
    assert len(cast(list[dict[str, Any]], bridge.shapeFaces)) == 3
    overlap = bridge.shapeBuilderHit(75.0, 50.0)
    assert overlap >= 0
    bridge.shapeBuilderToggle(overlap)
    bridge.shapeBuilderCommit()
    assert not bridge.shapeBuilderActive
    assert len(bridge.document.objects) == 1
    obj = bridge.document.objects[0]
    assert abs(path_area(obj.path) - 5000.0) < 1e-6 if obj.path else False


def test_save_reopen_with_live_and_boolean() -> None:
    import tempfile
    from pathlib import Path

    from petunia_app.ptnd_scene import open_ptnd, save_ptnd

    doc = Document(name="Bool")
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    b = SceneObject(type="path", name="B", path=rect_path(50, 0, 100, 100))
    live = SceneObject(type="path", name="Live")
    ap, bp = a.path, b.path
    assert ap is not None and bp is not None
    live.path = boolean_paths([(ap, a.id), (bp, b.id)], BooleanOp.UNION)[0].path
    live.live = None
    doc.objects = [a, b, live]
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "bool.ptnd"
        save_ptnd(path, doc)
        back = open_ptnd(path)
        assert len(back.objects) == 3
        assert back.objects[2].path is not None
        assert abs(path_area(back.objects[2].path) - 15000.0) < 1e-6


def test_setprop_moves_path_frame() -> None:
    bridge = Bridge()
    a = SceneObject(type="path", name="A", path=rect_path(0, 0, 100, 100))
    bridge.history.execute(AddObject(a), bridge.document)
    bridge.select(a.id)
    bridge.setProp("x", "50")
    ap = a.path
    assert ap is not None
    xs = [n.x for n in ap.contours[0].nodes]
    assert min(xs) == 50.0
    assert abs(path_area(ap) - 10000.0) < 1e-6
    # non-uniform scale via inspector
    bridge.setProp("width", "200")
    xs = [n.x for n in ap.contours[0].nodes]
    assert min(xs) == 50.0 and max(xs) == 250.0
    assert abs(path_area(ap) - 20000.0) < 1e-6
    # undo restores geometry
    bridge.undo()
    xs = [n.x for n in ap.contours[0].nodes]
    assert min(xs) == 50.0 and max(xs) == 150.0


def test_fuzz_random_rectangles_invariants() -> None:
    rng = random.Random(42)
    for _ in range(60):
        rects = [
            (
                rng.uniform(0, 200),
                rng.uniform(0, 200),
                rng.uniform(10, 150),
                rng.uniform(10, 150),
            )
            for _ in range(rng.randint(2, 4))
        ]
        ops = operands(*rects)
        union = boolean_paths(ops, BooleanOp.UNION)[0]
        total = sum(w * h for _, _, w, h in rects)
        max_single = max(w * h for _, _, w, h in rects)
        # 1/1000-unit quantization of the integer backend
        # bounds the area error, so invariants hold within
        # a relative tolerance.
        assert max_single * 0.999 <= area_of(union) <= total * 1.001
        # determinism
        again = boolean_paths(ops, BooleanOp.UNION)[0]
        assert abs(area_of(union) - area_of(again)) < 1e-9
        # intersect is bounded by the smallest operand
        inter = boolean_paths(ops, BooleanOp.INTERSECT)[0]
        assert -1e-6 <= area_of(inter) <= min(w * h for _, _, w, h in rects) * 1.001 + 1e-6


def test_benchmark_10k_segments() -> None:
    import math

    n = 10_000
    nodes = [
        PathNode(
            id=new_id(),
            x=400.0 + 350.0 * math.cos(2 * math.pi * i / n),
            y=400.0 + 350.0 * math.sin(2 * math.pi * i / n),
        )
        for i in range(n)
    ]
    big = VectorPath(
        contours=[Contour(id=new_id(), nodes=nodes, closed=True)],
        fill_rule=FillRule.NONZERO,
    )
    ops = [(big, "0"), (rect_path(0, 0, 200, 200), "1")]
    start = time.perf_counter()
    result = boolean_paths(ops, BooleanOp.UNION)[0]
    elapsed = time.perf_counter() - start
    assert area_of(result) > 0.0
    assert elapsed < 5.0, f"10k-segment union took {elapsed:.2f}s"
