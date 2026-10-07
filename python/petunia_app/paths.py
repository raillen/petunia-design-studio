"""Canonical vector path model, curve math and topology operations (G029).

Handles are stored as offsets from the node anchor. The cubic segment between
node A and B uses P1 = A.anchor + A.out_handle and P2 = B.anchor + B.in_handle.
A segment with both handles disabled is a straight line.
"""

from __future__ import annotations

import math
import uuid
from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any

Point = tuple[float, float]


def new_id() -> str:
    return str(uuid.uuid4())


class NodeKind(StrEnum):
    CUSP = "cusp"
    SMOOTH = "smooth"
    SYMMETRIC = "symmetric"


class FillRule(StrEnum):
    NONZERO = "nonzero"
    EVENODD = "evenodd"


@dataclass
class PathNode:
    id: str
    x: float
    y: float
    kind: NodeKind = NodeKind.CUSP
    in_handle: Point | None = None
    out_handle: Point | None = None

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "x": self.x,
            "y": self.y,
            "kind": self.kind.value,
            "inHandle": list(self.in_handle) if self.in_handle is not None else None,
            "outHandle": list(self.out_handle) if self.out_handle is not None else None,
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> PathNode:
        in_h = d.get("inHandle")
        out_h = d.get("outHandle")
        return cls(
            id=str(d["id"]),
            x=float(d["x"]),
            y=float(d["y"]),
            kind=NodeKind(str(d["kind"])),
            in_handle=(float(in_h[0]), float(in_h[1])) if in_h is not None else None,
            out_handle=(float(out_h[0]), float(out_h[1])) if out_h is not None else None,
        )


@dataclass
class Contour:
    id: str
    nodes: list[PathNode] = field(default_factory=lambda: [])
    closed: bool = False

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "closed": self.closed,
            "nodes": [n.to_json() for n in self.nodes],
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> Contour:
        return cls(
            id=str(d["id"]),
            closed=bool(d.get("closed", False)),
            nodes=[PathNode.from_json(n) for n in d.get("nodes", [])],
        )


@dataclass
class VectorPath:
    contours: list[Contour] = field(default_factory=lambda: [])
    fill_rule: FillRule = FillRule.NONZERO

    def to_json(self) -> dict[str, Any]:
        return {
            "fillRule": self.fill_rule.value,
            "contours": [c.to_json() for c in self.contours],
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> VectorPath:
        return cls(
            fill_rule=FillRule(str(d.get("fillRule", "nonzero"))),
            contours=[Contour.from_json(c) for c in d.get("contours", [])],
        )


# ---------------------------------------------------------------------------
# Cubic curve math (De Casteljau / Bernstein)
# ---------------------------------------------------------------------------


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def lerp_point(a: Point, b: Point, t: float) -> Point:
    return (lerp(a[0], b[0], t), lerp(a[1], b[1], t))


def cubic_at(p0: Point, p1: Point, p2: Point, p3: Point, t: float) -> Point:
    mt = 1.0 - t
    mt2 = mt * mt
    t2 = t * t
    w0 = mt2 * mt
    w1 = 3.0 * mt2 * t
    w2 = 3.0 * mt * t2
    w3 = t2 * t
    return (
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    )


def cubic_split(
    p0: Point, p1: Point, p2: Point, p3: Point, t: float
) -> tuple[tuple[Point, Point, Point, Point], tuple[Point, Point, Point, Point]]:
    q0 = lerp_point(p0, p1, t)
    q1 = lerp_point(p1, p2, t)
    q2 = lerp_point(p2, p3, t)
    r0 = lerp_point(q0, q1, t)
    r1 = lerp_point(q1, q2, t)
    s = lerp_point(r0, r1, t)
    return (p0, q0, r0, s), (s, r1, q2, p3)


def _chord_distance(p: Point, p0: Point, p3: Point) -> float:
    dx = p3[0] - p0[0]
    dy = p3[1] - p0[1]
    length_sq = dx * dx + dy * dy
    if length_sq == 0.0:
        return math.hypot(p[0] - p0[0], p[1] - p0[1])
    u = ((p[0] - p0[0]) * dx + (p[1] - p0[1]) * dy) / length_sq
    cx = p0[0] + u * dx
    cy = p0[1] + u * dy
    return math.hypot(p[0] - cx, p[1] - cy)


def _flatness(p0: Point, p1: Point, p2: Point, p3: Point) -> float:
    return max(_chord_distance(p1, p0, p3), _chord_distance(p2, p0, p3))


def cubic_flatten(
    p0: Point, p1: Point, p2: Point, p3: Point, tolerance: float = 0.25,
    max_depth: int = 24,
) -> list[Point]:
    points: list[Point] = [p0]

    def recurse(a: Point, b: Point, c: Point, d: Point, depth: int) -> None:
        if depth >= max_depth or _flatness(a, b, c, d) <= tolerance:
            points.append(d)
            return
        left, right = cubic_split(a, b, c, d, 0.5)
        recurse(*left, depth + 1)
        recurse(*right, depth + 1)

    recurse(p0, p1, p2, p3, 0)
    return points


def _axis_bounds(c0: float, c1: float, c2: float, c3: float) -> tuple[float, float]:
    lo = min(c0, c3)
    hi = max(c0, c3)
    a = (c1 - c0) - 2.0 * (c2 - c1) + (c3 - c2)
    b = 2.0 * (c2 - c1) - 2.0 * (c1 - c0)
    c = c1 - c0
    roots: list[float] = []
    if abs(a) < 1e-12:
        if abs(b) > 1e-12:
            t = -c / b
            if 0.0 < t < 1.0:
                roots.append(t)
    else:
        disc = b * b - 4.0 * a * c
        if disc >= 0.0:
            sq = math.sqrt(disc)
            for t in ((-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)):
                if 0.0 < t < 1.0:
                    roots.append(t)
    for t in roots:
        v = cubic_at((c0, 0.0), (c1, 0.0), (c2, 0.0), (c3, 0.0), t)[0]
        lo = min(lo, v)
        hi = max(hi, v)
    return lo, hi


def cubic_bounds(
    p0: Point, p1: Point, p2: Point, p3: Point
) -> tuple[float, float, float, float]:
    xlo, xhi = _axis_bounds(p0[0], p1[0], p2[0], p3[0])
    ylo, yhi = _axis_bounds(p0[1], p1[1], p2[1], p3[1])
    return (xlo, ylo, xhi, yhi)


def cubic_tangent(
    p0: Point, p1: Point, p2: Point, p3: Point, t: float
) -> Point:
    mt = 1.0 - t
    dx = 3.0 * (
        mt * mt * (p1[0] - p0[0])
        + 2.0 * mt * t * (p2[0] - p1[0])
        + t * t * (p3[0] - p2[0])
    )
    dy = 3.0 * (
        mt * mt * (p1[1] - p0[1])
        + 2.0 * mt * t * (p2[1] - p1[1])
        + t * t * (p3[1] - p2[1])
    )
    length = math.hypot(dx, dy)
    if length == 0.0:
        return (1.0, 0.0)
    return (dx / length, dy / length)


# ---------------------------------------------------------------------------
# Segment semantics
# ---------------------------------------------------------------------------


def segment_controls(a: PathNode, b: PathNode) -> tuple[Point, Point, Point, Point] | None:
    if a.out_handle is None and b.in_handle is None:
        return None
    p0 = (a.x, a.y)
    p3 = (b.x, b.y)
    oh = a.out_handle if a.out_handle is not None else (0.0, 0.0)
    ih = b.in_handle if b.in_handle is not None else (0.0, 0.0)
    return (
        p0,
        (p0[0] + oh[0], p0[1] + oh[1]),
        (p3[0] + ih[0], p3[1] + ih[1]),
        p3,
    )


def contour_segments(c: Contour) -> list[tuple[int, int]]:
    n = len(c.nodes)
    if n < 2:
        return []
    pairs = [(i, i + 1) for i in range(n - 1)]
    if c.closed:
        pairs.append((n - 1, 0))
    return pairs


def contour_bounds(c: Contour) -> tuple[float, float, float, float] | None:
    boxes: list[tuple[float, float, float, float]] = []
    for i, j in contour_segments(c):
        ctrl = segment_controls(c.nodes[i], c.nodes[j])
        if ctrl is None:
            a = (c.nodes[i].x, c.nodes[i].y)
            b = (c.nodes[j].x, c.nodes[j].y)
            boxes.append((min(a[0], b[0]), min(a[1], b[1]), max(a[0], b[0]), max(a[1], b[1])))
        else:
            boxes.append(cubic_bounds(*ctrl))
    if not boxes:
        if c.nodes:
            n0 = c.nodes[0]
            return (n0.x, n0.y, n0.x, n0.y)
        return None
    x0 = min(b[0] for b in boxes)
    y0 = min(b[1] for b in boxes)
    x1 = max(b[2] for b in boxes)
    y1 = max(b[3] for b in boxes)
    return (x0, y0, x1, y1)


def path_bounds(p: VectorPath) -> tuple[float, float, float, float]:
    boxes = [b for b in (contour_bounds(c) for c in p.contours) if b is not None]
    if not boxes:
        return (0.0, 0.0, 0.0, 0.0)
    return (
        min(b[0] for b in boxes),
        min(b[1] for b in boxes),
        max(b[2] for b in boxes),
        max(b[3] for b in boxes),
    )


def flatten_path(p: VectorPath, tolerance: float = 0.25) -> list[list[Point]]:
    polylines: list[list[Point]] = []
    for c in p.contours:
        points: list[Point] = []
        for i, j in contour_segments(c):
            ctrl = segment_controls(c.nodes[i], c.nodes[j])
            if ctrl is None:
                seg = [(c.nodes[i].x, c.nodes[i].y), (c.nodes[j].x, c.nodes[j].y)]
            else:
                seg = cubic_flatten(*ctrl, tolerance)
            if points:
                seg = seg[1:]
            points.extend(seg)
        polylines.append(points)
    return polylines


@dataclass
class NearestPoint:
    contour_index: int
    segment_index: int
    t: float
    point: Point
    distance_squared: float
    tangent: Point


def nearest_point(
    p: VectorPath, x: float, y: float, tolerance: float = 0.25
) -> NearestPoint | None:
    best: NearestPoint | None = None
    for ci, c in enumerate(p.contours):
        for si, (i, j) in enumerate(contour_segments(c)):
            ctrl = segment_controls(c.nodes[i], c.nodes[j])
            if ctrl is None:
                poly = [(c.nodes[i].x, c.nodes[i].y), (c.nodes[j].x, c.nodes[j].y)]
            else:
                poly = cubic_flatten(*ctrl, tolerance)
            for k in range(len(poly) - 1):
                ax, ay = poly[k]
                bx, by = poly[k + 1]
                abx = bx - ax
                aby = by - ay
                length_sq = abx * abx + aby * aby
                if length_sq == 0.0:
                    u = 0.0
                else:
                    u = max(
                        0.0,
                        min(1.0, ((x - ax) * abx + (y - ay) * aby) / length_sq),
                    )
                px = ax + u * abx
                py = ay + u * aby
                d2 = (x - px) ** 2 + (y - py) ** 2
                if best is None or d2 < best.distance_squared:
                    t = (k + u) / max(len(poly) - 1, 1)
                    tangent = (
                        _normalize(abx, aby) if ctrl is None else cubic_tangent(*ctrl, t)
                    )
                    best = NearestPoint(ci, si, t, (px, py), d2, tangent)
    return best


def _normalize(dx: float, dy: float) -> Point:
    length = math.hypot(dx, dy)
    if length == 0.0:
        return (1.0, 0.0)
    return (dx / length, dy / length)


# ---------------------------------------------------------------------------
# Topology operations
# ---------------------------------------------------------------------------


def _dist(a: PathNode, b: PathNode) -> float:
    return math.hypot(a.x - b.x, a.y - b.y)


def insert_node(
    c: Contour, segment_index: int, t: float, node_id: str | None = None
) -> PathNode:
    pairs = contour_segments(c)
    if not pairs:
        raise ValueError("contour has no segments")
    i, j = pairs[segment_index]
    a = c.nodes[i]
    b = c.nodes[j]
    nid = node_id if node_id is not None else new_id()
    ctrl = segment_controls(a, b)
    if ctrl is None:
        node = PathNode(id=nid, x=lerp(a.x, b.x, t), y=lerp(a.y, b.y, t))
    else:
        left, right = cubic_split(*ctrl, t)
        s = left[3]
        a.out_handle = (left[1][0] - a.x, left[1][1] - a.y)
        b.in_handle = (right[2][0] - b.x, right[2][1] - b.y)
        node = PathNode(
            id=nid,
            x=s[0],
            y=s[1],
            in_handle=(left[2][0] - s[0], left[2][1] - s[1]),
            out_handle=(right[1][0] - s[0], right[1][1] - s[1]),
        )
    pos = i + 1 if i < j else len(c.nodes)
    c.nodes.insert(pos, node)
    return node


def reverse_contour(c: Contour) -> None:
    c.nodes = [
        PathNode(
            id=n.id,
            x=n.x,
            y=n.y,
            kind=n.kind,
            in_handle=n.out_handle,
            out_handle=n.in_handle,
        )
        for n in reversed(c.nodes)
    ]


def join_contours(a: Contour, b: Contour, tolerance: float = 1e-6) -> bool:
    if not a.nodes or not b.nodes:
        return False
    a_last = a.nodes[-1]
    if _dist(a_last, b.nodes[0]) <= tolerance:
        a.nodes.extend(b.nodes[1:])
        return True
    if _dist(a_last, b.nodes[-1]) <= tolerance:
        reverse_contour(b)
        a.nodes.extend(b.nodes[1:])
        return True
    return False


def break_contour(c: Contour, node_index: int) -> Contour | None:
    n = len(c.nodes)
    if n < 2 or node_index < 0 or node_index >= n:
        return None
    if c.closed:
        c.nodes = c.nodes[node_index:] + c.nodes[:node_index]
        first = c.nodes[0]
        c.nodes.append(
            PathNode(
                id=new_id(),
                x=first.x,
                y=first.y,
                kind=first.kind,
                in_handle=first.in_handle,
                out_handle=first.out_handle,
            )
        )
        c.closed = False
        return None
    if node_index == 0 or node_index == n - 1:
        return None
    first = c.nodes[: node_index + 1]
    src = c.nodes[node_index]
    second = Contour(
        id=new_id(),
        nodes=[
            PathNode(
                id=new_id(),
                x=src.x,
                y=src.y,
                kind=src.kind,
                in_handle=src.in_handle,
                out_handle=src.out_handle,
            ),
            *c.nodes[node_index + 1 :],
        ],
    )
    c.nodes = first
    return second


def delete_node(c: Contour, node_index: int) -> bool:
    n = len(c.nodes)
    if n <= 2 or node_index < 0 or node_index >= n:
        return False
    del c.nodes[node_index]
    return True


def set_node_position(c: Contour, node_index: int, x: float, y: float) -> bool:
    if node_index < 0 or node_index >= len(c.nodes):
        return False
    c.nodes[node_index].x = x
    c.nodes[node_index].y = y
    return True
