from __future__ import annotations

from typing import Any

from .model import SceneObject
from .paths import contour_segments, segment_controls
from .resources import resolve_object


def _path_data(path: Any) -> str:
    chunks: list[str] = []
    for c in path.contours:
        if not c.nodes:
            continue
        chunks.append(f"M {c.nodes[0].x} {c.nodes[0].y}")
        for i, j in contour_segments(c):
            a = c.nodes[i]
            b = c.nodes[j]
            ctrl = segment_controls(a, b)
            if ctrl is None:
                chunks.append(f"L {b.x} {b.y}")
            else:
                chunks.append(
                    f"C {ctrl[1][0]} {ctrl[1][1]} {ctrl[2][0]} {ctrl[2][1]} {b.x} {b.y}"
                )
        if c.closed:
            chunks.append("Z")
    return " ".join(chunks)


def _gradient_def(o: SceneObject, defs: list[str]) -> str | None:
    if not o.gradient:
        return None
    gid = f"grad{len(defs)}"
    defs.append(
        f'<linearGradient id="{gid}" x1="0" y1="0" x2="1" y2="0">'
        + "".join(
            f'<stop offset="{i / max(len(o.gradient) - 1, 1):.2f}" stop-color="{c}"/>'
            for i, c in enumerate(o.gradient)
        )
        + "</linearGradient>"
    )
    return f"url(#{gid})"


def _emit(doc: Any, o: SceneObject, parts: list[str], defs: list[str]) -> None:
    if not o.visible:
        return
    if o.type == "group":
        parts.append("<g>")
        for c in o.children:
            _emit(doc, resolve_object(doc, c), parts, defs)
        parts.append("</g>")
        return
    opacity_attr = f' opacity="{o.opacity:.2f}"' if o.opacity < 1.0 else ""
    if o.type == "path" and o.path is not None:
        fill = _gradient_def(o, defs) or o.fill
        rule = o.path.fill_rule.value
        style = (
            f'fill="{fill}" fill-rule="{rule}" '
            f'stroke="{o.stroke}" stroke-width="{o.stroke_width}"'
        )
        parts.append(f'<path d="{_path_data(o.path)}" {style}{opacity_attr}/>')
        return
    if o.type == "text":
        story = doc.stories.get(o.story_id) if o.story_id else None
        text_content = story.text if story else ""
        escaped = (
            text_content.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
        )
        parts.append(
            f'<text x="{o.x}" y="{o.y + o.height * 0.75}" fill="{o.fill}" '
            f'font-family="Inter" font-size="14"{opacity_attr}>{escaped}</text>'
        )
        return
    fill = _gradient_def(o, defs) or o.fill
    style = f'fill="{fill}" stroke="{o.stroke}" stroke-width="{o.stroke_width}"'
    if o.type == "ellipse":
        parts.append(
            f'<ellipse cx="{o.x + o.width / 2}" cy="{o.y + o.height / 2}" '
            f'rx="{o.width / 2}" ry="{o.height / 2}" {style}{opacity_attr}/>'
        )
    else:
        rx_ry = (
            f' rx="{o.corner_radius}" ry="{o.corner_radius}"'
            if o.corner_radius > 0
            else ""
        )
        parts.append(
            f'<rect x="{o.x}" y="{o.y}" width="{o.width}" height="{o.height}" {style}{rx_ry}{opacity_attr}/>'
        )


def export_svg(doc: Any) -> str:
    header = (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{doc.width}" '
        f'height="{doc.height}" viewBox="0 0 {doc.width} {doc.height}">'
    )
    parts = [header]
    defs: list[str] = []
    for o in doc.objects:
        _emit(doc, resolve_object(doc, o), parts, defs)
    if defs:
        parts.insert(1, f"<defs>{''.join(defs)}</defs>")
    parts.append("</svg>")
    return "\n".join(parts) + "\n"
