"""G031 — Boolean engine, compound paths and Shape Builder region engine.

Backend: pyclipper (Clipper 1.x, Angus Johnson, BSD-3-Clause), a planar
polygon-clipping engine. Spec 09.6.4 prefers curve-preserving output and
requires an explicit fidelity analysis when a candidate library flattens to
polylines. This module implements that policy:

1. Curves are flattened adaptively at ``FLATTEN_TOLERANCE`` (0.25 units)
   before clipping; output contours are rebuilt as straight segments with
   cusp nodes. Curvature fidelity loss is bounded by the flatten tolerance.
2. Coordinates are quantized to ``1/SCALE`` units for the integer backend;
   absolute quantization error is <= 0.001 units.
3. Zero-length edges are removed and adjacent collinear spans merged
   deterministically in integer space.
4. Output orientation follows the Clipper convention; the fill rule
   (nonzero) is authoritative for hole interpretation.
5. Provenance maps every output contour to its source object. Binary
   operations report object-level provenance (contour-level attribution
   requires a PolyPath-capable backend such as Clipper2); Divide reports
   exact per-contour provenance because each piece derives from one operand.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any

import pyclipper

from .paths import (
    Contour,
    FillRule,
    NodeKind,
    PathNode,
    Point,
    VectorPath,
    flatten_path,
    new_id,
)

FLATTEN_TOLERANCE = 0.25
SCALE = 1000.0

Polygon = list[Point]


class BooleanOp(StrEnum):
    UNION = "union"
    SUBTRACT = "subtract"
    INTERSECT = "intersect"
    XOR = "xor"
    DIVIDE = "divide"


class BooleanError(Exception):
    """Typed geometry failure carrying the involved object ids (09.6.3)."""

    def __init__(self, message: str, involved_ids: list[str] | None = None) -> None:
        super().__init__(message)
        self.involved_ids = list(involved_ids or [])


@dataclass
class ContourProvenance:
    source_object_id: str
    exact: bool


@dataclass
class BooleanResult:
    path: VectorPath
    provenance: list[ContourProvenance]
    style_source_id: str


@dataclass
class ShapeFace:
    polygon: Polygon
    sources: list[str] = field(default_factory=lambda: [])


def _fill_type(rule: FillRule) -> int:
    if rule == FillRule.EVENODD:
        return pyclipper.PFT_EVENODD
    return pyclipper.PFT_NONZERO


def _to_polygons(path: VectorPath, obj_id: str) -> list[Polygon]:
    """Flatten closed contours to integer polygons; reject invalid operands."""
    if not all(isfinite(v) for v in _path_coords(path)):
        raise BooleanError("non-finite geometry", [obj_id])
    polygons: list[Polygon] = []
    for contour, poly in zip(
        path.contours, flatten_path(path, FLATTEN_TOLERANCE), strict=True
    ):
        if not contour.closed:
            raise BooleanError("open contour is not a valid boolean operand", [obj_id])
        points = poly[:-1] if len(poly) > 1 and poly[0] == poly[-1] else poly
        if len(points) < 3:
            raise BooleanError("degenerate contour with fewer than 3 points", [obj_id])
        polygons.append(
            [(round(x * SCALE), round(y * SCALE)) for x, y in points]
        )
    if not polygons:
        raise BooleanError("path has no closed contours", [obj_id])
    return polygons


def _path_coords(path: VectorPath) -> list[float]:
    values: list[float] = []
    for c in path.contours:
        for n in c.nodes:
            values.extend((n.x, n.y))
            if n.in_handle is not None:
                values.extend(n.in_handle)
            if n.out_handle is not None:
                values.extend(n.out_handle)
    return values


def isfinite(v: float) -> bool:
    return v == v and v not in (float("inf"), float("-inf"))


def _chain(
    op: int, polygons: list[list[Polygon]], fill_rule: FillRule
) -> list[Polygon]:
    """Pairwise chain for INTERSECT/XOR.

    Clipper combines clip paths by union before applying
    INTERSECT/XOR, so multi-operand semantics require explicit
    left-to-right chaining: (...((P0 op P1) op P2)...) op Pn.
    """
    acc = polygons[0]
    for polys in polygons[1:]:
        acc = _clip(op, acc, polys, fill_rule)
        if not acc:
            break
    return acc


def _clip(
    op: int, subjects: list[Polygon], clips: list[Polygon], rule: FillRule
) -> list[Polygon]:
    if not subjects:
        return []
    if not clips:
        # Only DIFFERENCE (A - empty = A) and INTERSECTION
        # (A ∩ empty = empty) short-circuit; UNION must still
        # run the engine to merge overlapping subjects, and XOR
        # requires at least one clip path to be well-defined.
        if op == pyclipper.CT_DIFFERENCE:
            return [_simplify(p) for p in subjects if _simplify(p)]
        if op == pyclipper.CT_INTERSECTION:
            return []
    clipper = pyclipper.Pyclipper(
        _fill_type(rule), _fill_type(rule), True
    )
    clipper.AddPaths(subjects, pyclipper.PT_SUBJECT, True)
    if clips:
        clipper.AddPaths(clips, pyclipper.PT_CLIP, True)
    raw = clipper.Execute(op, _fill_type(rule), _fill_type(rule))
    return [_simplify(p) for p in raw if _simplify(p)]


def _signed_area(poly: Polygon) -> float:
    area = 0.0
    n = len(poly)
    for i in range(n):
        x0, y0 = poly[i]
        x1, y1 = poly[(i + 1) % n]
        area += x0 * y1 - x1 * y0
    return area / 2.0


def _simplify(poly: Polygon) -> Polygon:
    """Remove zero-length edges and collinear vertices (deterministic, integer space)."""
    if len(poly) < 4:
        return list(poly)
    points = list(poly)
    if points[0] == points[-1]:
        points.pop()
    kept: list[Point] = []
    n = len(points)
    for i in range(n):
        prev = points[(i - 1) % n]
        curr = points[i]
        nxt = points[(i + 1) % n]
        if curr in (prev, nxt):
            continue
        cross = (curr[0] - prev[0]) * (nxt[1] - curr[1]) - (
            curr[1] - prev[1]
        ) * (nxt[0] - curr[0])
        if cross == 0:
            continue
        kept.append(curr)
    return kept if len(kept) >= 3 else []


def _contour_from_polygon(poly: Polygon) -> Contour:
    points = list(poly)
    if len(points) > 1 and points[0] == points[-1]:
        points.pop()
    nodes = [
        PathNode(id=new_id(), x=float(x) / SCALE, y=float(y) / SCALE, kind=NodeKind.CUSP)
        for x, y in points
    ]
    return Contour(id=new_id(), nodes=nodes, closed=True)


def _rebuild(polygons: list[Polygon], rule: FillRule = FillRule.NONZERO) -> VectorPath:
    return VectorPath(
        contours=[_contour_from_polygon(p) for p in polygons],
        fill_rule=rule,
    )


def _operands(
    paths: list[tuple[VectorPath, str]], op: BooleanOp
) -> list[tuple[list[Polygon], str]]:
    if len(paths) < 2:
        raise BooleanError("boolean operation requires at least two operands", [])
    return [(_to_polygons(p, oid), oid) for p, oid in paths]


def boolean_paths(
    paths: list[tuple[VectorPath, str]],
    op: BooleanOp,
    fill_rule: FillRule = FillRule.NONZERO,
) -> list[BooleanResult]:
    """Run a Boolean operation over ordered operands (bottom-to-top z-order)."""
    operands = _operands(paths, op)
    polygons = [p for p, _ in operands]
    ids = [oid for _, oid in operands]
    if op == BooleanOp.UNION:
        result = _clip(pyclipper.CT_UNION, [p for polys in polygons for p in polys], [], fill_rule)
        source = ids[-1]
        provenance = [ContourProvenance(source, False) for _ in result]
    elif op == BooleanOp.SUBTRACT:
        result = _clip(
            pyclipper.CT_DIFFERENCE,
            polygons[0],
            [p for polys in polygons[1:] for p in polys],
            fill_rule,
        )
        source = ids[0]
        provenance = [ContourProvenance(source, False) for _ in result]
    elif op == BooleanOp.INTERSECT:
        result = _chain(pyclipper.CT_INTERSECTION, polygons, fill_rule)
        source = ids[-1]
        provenance = [ContourProvenance(source, False) for _ in result]
    elif op == BooleanOp.XOR:
        result = _chain(pyclipper.CT_XOR, polygons, fill_rule)
        source = ids[-1]
        provenance = [ContourProvenance(source, False) for _ in result]
    elif op == BooleanOp.DIVIDE:
        # Atomic arrangement faces: every region of the arrangement
        # becomes a piece, attributed to its contributing sources
        # (topmost source drives the inherited appearance).
        faces = build_shape_faces(paths)
        results: list[BooleanResult] = []
        for face in faces:
            results.append(
                BooleanResult(
                    path=_rebuild([face.polygon], fill_rule),
                    provenance=[
                        ContourProvenance(sid, True) for sid in face.sources
                    ],
                    style_source_id=face.sources[-1],
                )
            )
        return results
    else:
        raise BooleanError(f"unknown boolean operation: {op}", ids)
    return [
        BooleanResult(
            path=_rebuild(result, fill_rule),
            provenance=provenance,
            style_source_id=source,
        )
    ]


def compound_paths(
    paths: list[tuple[VectorPath, str]],
    fill_rule: FillRule = FillRule.NONZERO,
) -> VectorPath:
    """Merge operand contours into one compound path (no geometry change)."""
    if len(paths) < 2:
        raise BooleanError("compound path requires at least two operands", [])
    contours: list[Contour] = []
    for path, oid in paths:
        if not all(isfinite(v) for v in _path_coords(path)):
            raise BooleanError("non-finite geometry", [oid])
        contours.extend(path.contours)
    return VectorPath(contours=contours, fill_rule=fill_rule)


def build_shape_faces(paths: list[tuple[VectorPath, str]]) -> list[ShapeFace]:
    """Derive atomic arrangement faces (regions of constant operand membership)."""
    operands = _operands(paths, BooleanOp.UNION)
    faces: list[ShapeFace] = []
    for polygons, oid in operands:
        updated: list[ShapeFace] = []
        for face in faces:
            overlap = _clip(pyclipper.CT_INTERSECTION, [face.polygon], polygons, FillRule.NONZERO)
            if overlap:
                updated.extend(ShapeFace(p, [*face.sources, oid]) for p in overlap)
                rest = _clip(pyclipper.CT_DIFFERENCE, [face.polygon], polygons, FillRule.NONZERO)
                updated.extend(ShapeFace(p, list(face.sources)) for p in rest)
            else:
                updated.append(face)
        covered = [f.polygon for f in faces]
        if covered:
            remainder = _clip(
                pyclipper.CT_DIFFERENCE, polygons, covered, FillRule.NONZERO
            )
        else:
            remainder = polygons
        updated.extend(ShapeFace(p, [oid]) for p in remainder)
        faces = updated
    return faces


def point_in_polygon(poly: Polygon, x: float, y: float) -> bool:
    """Even-odd ray cast (09.6.5 hit-testing baseline)."""
    px = round(x * SCALE)
    py = round(y * SCALE)
    inside = False
    n = len(poly)
    j = n - 1
    for i in range(n):
        xi, yi = poly[i]
        xj, yj = poly[j]
        if (yi > py) != (yj > py):
            x_intersect = (xj - xi) * (py - yi) / (yj - yi) + xi
            if px < x_intersect:
                inside = not inside
        j = i
    return inside


def face_at_point(faces: list[ShapeFace], x: float, y: float) -> int:
    for index, face in enumerate(faces):
        if point_in_polygon(face.polygon, x, y):
            return index
    return -1


def faces_to_path(faces: list[ShapeFace]) -> VectorPath:
    """Reconstruct the boundary path of a face set (union of selected faces)."""
    polygons = [f.polygon for f in faces]
    if not polygons:
        raise BooleanError("no faces selected for shape builder commit", [])
    merged = _clip(pyclipper.CT_UNION, polygons, [], FillRule.NONZERO)
    return _rebuild(merged)


def polygon_area(poly: Polygon) -> float:
    return abs(_signed_area(poly)) / (SCALE * SCALE)


def path_area(path: VectorPath) -> float:
    """Signed-area sum: holes (opposite orientation) subtract."""
    total = 0.0
    for c in path.contours:
        poly = [(n.x, n.y) for n in c.nodes]
        total += _signed_area(poly)
    return total


def rebuild_path(
    polygons: list[Polygon], rule: FillRule = FillRule.NONZERO
) -> VectorPath:
    """Rebuild a VectorPath from integer-space polygons."""
    return _rebuild(polygons, rule)


def split_path_by_line(
    path: VectorPath,
    p1: tuple[float, float],
    p2: tuple[float, float],
    fill_rule: FillRule = FillRule.NONZERO,
) -> tuple[VectorPath, VectorPath] | None:
    """Split a closed VectorPath along the cutting line segment p1 -> p2.

    Returns (piece_left, piece_right) if both sides have non-empty geometry,
    or None if the line does not divide the path into two distinct pieces.
    """
    import math

    dx = p2[0] - p1[0]
    dy = p2[1] - p1[1]
    dist_sq = dx * dx + dy * dy
    if dist_sq < 1e-4:
        return None

    try:
        polys = _to_polygons(path, "knife_cut")
    except Exception:
        return None

    length = math.sqrt(dist_sq)
    ux, uy = dx / length, dy / length
    nx, ny = -uy, ux

    far = 20000.0
    p_far1 = (p1[0] - far * ux, p1[1] - far * uy)
    p_far2 = (p2[0] + far * ux, p2[1] + far * uy)

    h1 = [
        (round(p_far1[0] * SCALE), round(p_far1[1] * SCALE)),
        (round(p_far2[0] * SCALE), round(p_far2[1] * SCALE)),
        (round((p_far2[0] + far * nx) * SCALE), round((p_far2[1] + far * ny) * SCALE)),
        (round((p_far1[0] + far * nx) * SCALE), round((p_far1[1] + far * ny) * SCALE)),
    ]

    h2 = [
        (round(p_far1[0] * SCALE), round(p_far1[1] * SCALE)),
        (round(p_far2[0] * SCALE), round(p_far2[1] * SCALE)),
        (round((p_far2[0] - far * nx) * SCALE), round((p_far2[1] - far * ny) * SCALE)),
        (round((p_far1[0] - far * nx) * SCALE), round((p_far1[1] - far * ny) * SCALE)),
    ]

    res1 = _clip(pyclipper.CT_INTERSECTION, polys, [h1], fill_rule)
    res2 = _clip(pyclipper.CT_INTERSECTION, polys, [h2], fill_rule)

    if not res1 or not res2:
        return None

    vp1 = _rebuild(res1, fill_rule)
    vp2 = _rebuild(res2, fill_rule)
    return vp1, vp2


def result_json(result: BooleanResult) -> dict[str, Any]:
    return {
        "path": result.path.to_json(),
        "provenance": [
            {"sourceObjectId": p.source_object_id, "exact": p.exact} for p in result.provenance
        ],
        "styleSourceId": result.style_source_id,
    }
