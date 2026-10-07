"""G030 commercial-mini workflow: builds a real brand/social composition
and runs the audit evidence checks for the integrated gate.

Composition (social card, 1080x1080):
- surface: document canvas
- parametric shapes: brand wash (rectangle + gradient), accent ellipses
- Pen path: four-petal "bloom" vector mark with smooth bezier handles
- fills/gradients/strokes on every visible object
- alignment/snapping: petal tips snapped to the 8-unit grid
- layers: rename + group
- save/reopen: PTND roundtrip with semantic-equality check (data-loss gate)
- exports: PNG (raster) and SVG (vector) into benchmarks/output/

Audit evidence produced into ``benchmarks/output/``:
- ``commercial-mini.ptnd``  saved document package
- ``commercial-mini.png``   rasterized card
- ``commercial-mini.svg``   vector card
- ``audit-evidence.json``   machine-readable check results
"""

from __future__ import annotations

import json
import math
import os
import subprocess
import sys
import tempfile
from pathlib import Path

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

from petunia_app.exporters import export_svg
from petunia_app.model import (
    AddObject,
    Document,
    GroupObjects,
    History,
    SceneObject,
    SetProperty,
    Snap,
)
from petunia_app.paths import (
    Contour,
    FillRule,
    NodeKind,
    PathNode,
    VectorPath,
    new_id,
)
from petunia_app.ptnd_scene import open_ptnd, save_ptnd
from petunia_app.raster import export_png

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "benchmarks" / "output"
GRID = 8.0


def petal(cx: float, cy: float, length: float, width: float, angle_deg: float) -> Contour:
    """One bezier petal pointing along ``angle_deg`` from (cx, cy)."""
    a = math.radians(angle_deg)
    dx, dy = math.cos(a), math.sin(a)
    px, py = -dy, dx  # perpendicular
    tip = (cx + dx * length, cy + dy * length)
    left = (cx + px * width / 2, cy + py * width / 2)
    right = (cx - px * width / 2, cy - py * width / 2)
    # smooth cubic through base-left -> tip -> base-right
    c1 = (left[0] + dx * length * 0.6, left[1] + dy * length * 0.6)
    c2 = (tip[0] + px * width * 0.55, tip[1] + py * width * 0.55)
    c3 = (tip[0] - px * width * 0.55, tip[1] - py * width * 0.55)
    c4 = (right[0] + dx * length * 0.6, right[1] + dy * length * 0.6)
    n = lambda x, y, kind=NodeKind.SMOOTH: PathNode(  # noqa: E731
        id=new_id(), x=x, y=y, kind=kind
    )
    base = n(cx, cy)
    base.out_handle = (c1[0] - cx, c1[1] - cy)
    tip_node = n(tip[0], tip[1])
    tip_node.in_handle = (c2[0] - tip[0], c2[1] - tip[1])
    tip_node.out_handle = (c3[0] - tip[0], c3[1] - tip[1])
    end = n(right[0], right[1])
    end.in_handle = (c4[0] - right[0], c4[1] - right[1])
    return Contour(id=new_id(), nodes=[base, tip_node, end], closed=False)


def bloom_mark(cx: float, cy: float, radius: float) -> VectorPath:
    """Four-petal bloom: one open contour per petal, snapped tips."""
    snap = Snap(grid=GRID)
    contours = []
    for k in range(4):
        c = petal(cx, cy, radius, radius * 0.72, k * 90.0 + 45.0)
        tip = c.nodes[1]
        tip.x = snap.snap(tip.x)
        tip.y = snap.snap(tip.y)
        contours.append(c)
    return VectorPath(contours=contours, fill_rule=FillRule.NONZERO)


def build_composition() -> Document:
    doc = Document(name="Aurora — Social Card", width=1080, height=1080)
    history = History()

    # 1. brand wash background (parametric rectangle + gradient)
    bg = SceneObject(
        type="rectangle",
        name="Brand wash",
        x=0.0,
        y=0.0,
        width=1080.0,
        height=1080.0,
        fill="#0b1026",
        gradient=["#0b1026", "#1b2a52", "#3b2f63"],
    )
    history.execute(AddObject(bg), doc)

    # 2. accent ellipses (parametric), snapped to the grid
    halo = SceneObject(
        type="ellipse",
        name="Halo",
        x=340.0,
        y=300.0,
        width=400.0,
        height=400.0,
        fill="#7c5cff",
        stroke="#a78bfa",
        stroke_width=3.0,
        visible=True,
    )
    history.execute(AddObject(halo), doc)
    history.execute(SetProperty(halo.id, "x", Snap(grid=GRID).snap(halo.x)), doc)
    history.execute(SetProperty(halo.id, "y", Snap(grid=GRID).snap(halo.y)), doc)

    core = SceneObject(
        type="ellipse",
        name="Core",
        x=490.0,
        y=450.0,
        width=100.0,
        height=100.0,
        fill="#f0abfc",
        stroke="#ffffff",
        stroke_width=2.0,
    )
    history.execute(AddObject(core), doc)

    # 3. Pen path: vector bloom mark centered on the card
    mark = SceneObject(
        type="path",
        name="Bloom mark",
        path=bloom_mark(540.0, 540.0, 220.0),
        fill="#22d3ee",
        gradient=["#22d3ee", "#818cf8"],
        stroke="#e0f2fe",
        stroke_width=2.5,
    )
    history.execute(AddObject(mark), doc)

    # 4. layers: rename + group the accent set
    history.execute(SetProperty(halo.id, "name", "Halo · glow"), doc)
    history.execute(SetProperty(core.id, "name", "Core · hot"), doc)
    history.execute(GroupObjects([halo.id, core.id]), doc)
    group = next(o for o in doc.objects if o.type == "group")
    group.name = "Accents"

    return doc


def semantic(doc: Document) -> dict[str, object]:
    return {"name": doc.name, "objects": [o.to_json() for o in doc.objects]}


def run_audit(doc: Document) -> dict[str, object]:
    evidence: dict[str, object] = {}

    # save/reopen data-loss gate
    with tempfile.TemporaryDirectory() as tmp:
        ptnd = Path(tmp) / "commercial-mini.ptnd"
        save_ptnd(ptnd, doc)
        reopened = open_ptnd(ptnd)
        evidence["roundtrip_semantic_equal"] = semantic(doc) == semantic(reopened)
        evidence["roundtrip_object_count"] = len(reopened.objects)
        # keep artifacts for the bundle
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / "commercial-mini.ptnd").write_bytes(ptnd.read_bytes())

    # vector + raster exports
    svg = export_svg(doc)
    (OUT / "commercial-mini.svg").write_text(svg, encoding="utf-8")
    evidence["svg_bytes"] = len(svg.encode("utf-8"))
    evidence["svg_has_path_element"] = "<path" in svg
    export_png(doc, OUT / "commercial-mini.png")
    evidence["png_bytes"] = (OUT / "commercial-mini.png").stat().st_size

    # correctness gate: full python suite
    r = subprocess.run(
        [sys.executable, "-m", "pytest", "-q"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    evidence["pytest_exit"] = r.returncode
    evidence["pytest_tail"] = r.stdout.strip().splitlines()[-1] if r.stdout.strip() else ""
    return evidence


def main() -> int:
    doc = build_composition()
    evidence = run_audit(doc)
    (OUT / "audit-evidence.json").write_text(
        json.dumps(evidence, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(json.dumps(evidence, indent=2, ensure_ascii=False))
    ok = (
        evidence["roundtrip_semantic_equal"]
        and evidence["svg_has_path_element"]
        and evidence["png_bytes"] > 0
        and evidence["pytest_exit"] == 0
    )
    print("GATE:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
