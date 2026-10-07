"""G030 benchmark bundle: measures the documented performance budgets
for the commercial-mini gate and writes ``benchmarks/results.json``.

Budgets (provisional, G030 baseline — to be tightened at G059):
- boolean union on two 10k-segment polygons: <= 2.0 s
- SVG export of the commercial-mini card:    <= 0.5 s
- PNG export of the commercial-mini card:    <= 2.0 s
- PTND save/reopen roundtrip:                <= 1.0 s
- 100k-node path flatten:                    <= 1.0 s
"""

from __future__ import annotations

import json
import math
import os
import tempfile
import time
from pathlib import Path

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

from petunia_app.boolean import BooleanOp, Polygon, boolean_paths
from petunia_app.exporters import export_svg
from petunia_app.model import Document, SceneObject
from petunia_app.paths import (
    Contour,
    FillRule,
    PathNode,
    VectorPath,
    flatten_path,
    new_id,
)
from petunia_app.ptnd_scene import open_ptnd, save_ptnd
from petunia_app.raster import export_png

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "output"
BUDGETS = {
    "boolean_10k_union": 2.0,
    "svg_export": 0.5,
    "png_export": 2.0,
    "ptnd_roundtrip": 1.0,
    "flatten_100k": 1.0,
}


def star_polygon(cx: float, cy: float, spikes: int, r_out: float, r_in: float) -> Polygon:
    pts = []
    for k in range(spikes * 2):
        r = r_out if k % 2 == 0 else r_in
        a = 2.0 * math.pi * k / (spikes * 2) - math.pi / 2
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def bench_boolean_10k() -> dict[str, object]:
    # two 10k-segment star polygons (5000 spikes each), overlapping
    a = star_polygon(0.0, 0.0, 5000, 500.0, 480.0)
    b = star_polygon(60.0, 40.0, 5000, 500.0, 480.0)
    paths = [
        (VectorPath(contours=[Contour(id=new_id(), nodes=[
            PathNode(id=new_id(), x=p[0], y=p[1]) for p in a
        ], closed=True)]), "A"),
        (VectorPath(contours=[Contour(id=new_id(), nodes=[
            PathNode(id=new_id(), x=p[0], y=p[1]) for p in b
        ], closed=True)]), "B"),
    ]
    t0 = time.perf_counter()
    result = boolean_paths(paths, BooleanOp.UNION, FillRule.NONZERO)
    dt = time.perf_counter() - t0
    return {
        "seconds": round(dt, 4),
        "budget_s": BUDGETS["boolean_10k_union"],
        "within_budget": dt <= BUDGETS["boolean_10k_union"],
        "result_contours": len(result[0].path.contours) if result else 0,
        "segments_per_operand": 10000,
    }


def bench_exports(doc: Document) -> dict[str, object]:
    t0 = time.perf_counter()
    export_svg(doc)
    svg_dt = time.perf_counter() - t0

    with tempfile.TemporaryDirectory() as tmp:
        png = Path(tmp) / "card.png"
        t0 = time.perf_counter()
        export_png(doc, png)
        png_dt = time.perf_counter() - t0

        t0 = time.perf_counter()
        save_ptnd(Path(tmp) / "card.ptnd", doc)
        back = open_ptnd(Path(tmp) / "card.ptnd")
        rt_dt = time.perf_counter() - t0

    return {
        "svg": {
            "seconds": round(svg_dt, 4),
            "budget_s": BUDGETS["svg_export"],
            "within_budget": svg_dt <= BUDGETS["svg_export"],
        },
        "png": {
            "seconds": round(png_dt, 4),
            "budget_s": BUDGETS["png_export"],
            "within_budget": png_dt <= BUDGETS["png_export"],
        },
        "ptnd_roundtrip": {
            "seconds": round(rt_dt, 4),
            "budget_s": BUDGETS["ptnd_roundtrip"],
            "within_budget": rt_dt <= BUDGETS["ptnd_roundtrip"],
            "objects_reopened": len(back.objects),
        },
    }


def bench_flatten_100k() -> dict[str, object]:
    nodes = [
        PathNode(
            id=new_id(),
            x=540.0 + 500.0 * math.cos(2.0 * math.pi * k / 100_000),
            y=540.0 + 500.0 * math.sin(2.0 * math.pi * k / 100_000),
        )
        for k in range(100_000)
    ]
    path = VectorPath(contours=[Contour(id=new_id(), nodes=nodes, closed=True)])
    t0 = time.perf_counter()
    polylines = flatten_path(path, tolerance=0.25)
    dt = time.perf_counter() - t0
    return {
        "seconds": round(dt, 4),
        "budget_s": BUDGETS["flatten_100k"],
        "within_budget": dt <= BUDGETS["flatten_100k"],
        "polyline_points": sum(len(p) for p in polylines),
    }


def main() -> int:
    doc = Document(name="Benchmark card", width=1080, height=1080)
    doc.objects = [
        SceneObject(
            type="rectangle", name="wash", x=0.0, y=0.0,
            width=1080.0, height=1080.0, fill="#0b1026",
            gradient=["#0b1026", "#1b2a52"],
        ),
        SceneObject(
            type="ellipse", name="halo", x=340.0, y=300.0,
            width=400.0, height=400.0, fill="#7c5cff",
        ),
    ]

    results: dict[str, object] = {
        "environment": {
            "python": os.environ.get("QT_QPA_PLATFORM", "default"),
            "note": "offscreen Linux container, single OS",
        },
        "budgets": BUDGETS,
        "boolean_10k_union": bench_boolean_10k(),
        "flatten_100k": bench_flatten_100k(),
    }
    results.update(bench_exports(doc))  # type: ignore[arg-type]

    OUT.mkdir(parents=True, exist_ok=True)
    (ROOT / "results.json").write_text(
        json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(json.dumps(results, indent=2, ensure_ascii=False))
    all_ok = all(
        v.get("within_budget", True)
        for v in results.values()
        if isinstance(v, dict) and "within_budget" in v
    )
    print("BUDGETS:", "ALL WITHIN" if all_ok else "OVER BUDGET")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
