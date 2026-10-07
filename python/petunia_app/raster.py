from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import QPointF, QRectF, Qt
from PySide6.QtGui import (
    QBrush,
    QColor,
    QGuiApplication,
    QImage,
    QLinearGradient,
    QPainter,
    QPainterPath,
    QPen,
)

from .model import Document, SceneObject
from .paths import VectorPath, contour_segments, segment_controls
from .resources import resolve_object


def _add_path(painter_path: QPainterPath, path: VectorPath) -> None:
    for c in path.contours:
        if not c.nodes:
            continue
        painter_path.moveTo(c.nodes[0].x, c.nodes[0].y)
        for i, j in contour_segments(c):
            a = c.nodes[i]
            b = c.nodes[j]
            ctrl = segment_controls(a, b)
            if ctrl is None:
                painter_path.lineTo(b.x, b.y)
            else:
                painter_path.cubicTo(
                    ctrl[1][0], ctrl[1][1], ctrl[2][0], ctrl[2][1], b.x, b.y
                )
        if c.closed:
            painter_path.closeSubpath()


def _pen_for_object(o: SceneObject) -> QPen:
    cap = Qt.PenCapStyle.RoundCap
    if o.stroke_cap == "butt":
        cap = Qt.PenCapStyle.FlatCap
    elif o.stroke_cap == "square":
        cap = Qt.PenCapStyle.SquareCap

    join = Qt.PenJoinStyle.RoundJoin
    if o.stroke_join == "miter":
        join = Qt.PenJoinStyle.MiterJoin
    elif o.stroke_join == "bevel":
        join = Qt.PenJoinStyle.BevelJoin

    pen = QPen(QColor(o.stroke), o.stroke_width, Qt.PenStyle.SolidLine, cap, join)
    if o.stroke_dash:
        pen.setDashPattern([float(d) for d in o.stroke_dash])
    return pen


def _draw(doc: Document, o: SceneObject, painter: QPainter) -> None:
    if not o.visible:
        return
    painter.save()
    painter.setOpacity(max(0.0, min(1.0, o.opacity)))

    bm = o.blend_mode.lower()
    if bm == "multiply":
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_Multiply)
    elif bm == "screen":
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_Screen)
    elif bm == "overlay":
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_Overlay)
    elif bm == "darken":
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_Darken)
    elif bm == "lighten":
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_Lighten)
    elif bm in ("destination-out", "eraser"):
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_DestinationOut)
    else:
        painter.setCompositionMode(QPainter.CompositionMode.CompositionMode_SourceOver)

    if o.rotation:
        cx = o.x + o.width / 2.0
        cy = o.y + o.height / 2.0
        painter.translate(cx, cy)
        painter.rotate(o.rotation)
        painter.translate(-cx, -cy)

    pen = _pen_for_object(o)

    if o.type == "group":
        for c in o.children:
            _draw(doc, resolve_object(doc, c), painter)
        painter.restore()
        return
    if o.type == "path" and o.path is not None:
        painter_path = QPainterPath()
        _add_path(painter_path, o.path)
        if o.gradient:
            grad = QLinearGradient(QPointF(o.x, o.y), QPointF(o.x + o.width, o.y))
            for i, color in enumerate(o.gradient):
                grad.setColorAt(i / max(len(o.gradient) - 1, 1), QColor(color))
            painter.setBrush(QBrush(grad))
        else:
            painter.setBrush(QBrush(QColor(o.fill)))
        painter.setPen(pen)
        painter.drawPath(painter_path)
        painter.restore()
        return
    if o.type == "text":
        story = doc.stories.get(o.story_id) if o.story_id else None
        if story and story.text:
            painter.setPen(QPen(QColor(o.fill)))
            font = painter.font()
            family = "Inter"
            pixel_size = 14
            bold = False
            italic = False
            align = Qt.AlignmentFlag.AlignLeft
            if story.runs:
                r0 = story.runs[0]
                family = str(r0.overrides.get("fontFamily", "Inter"))
                pixel_size = int(r0.overrides.get("fontSize", 14))
                bold = str(r0.overrides.get("fontWeight", "")).lower() in {"bold", "700"}
                italic = str(r0.overrides.get("fontStyle", "")).lower() == "italic"
            if story.paragraphs:
                p_align = str(story.paragraphs[0].overrides.get("align", "left")).lower()
                if p_align == "center":
                    align = Qt.AlignmentFlag.AlignHCenter
                elif p_align == "right":
                    align = Qt.AlignmentFlag.AlignRight
            font.setFamily(family)
            font.setPixelSize(max(6, pixel_size))
            font.setBold(bold)
            font.setItalic(italic)
            painter.setFont(font)
            painter.drawText(
                QRectF(o.x, o.y, o.width, o.height),
                align | Qt.AlignmentFlag.AlignVCenter,
                story.text,
            )
        painter.restore()
        return
    if o.gradient:
        grad = QLinearGradient(QPointF(o.x, o.y), QPointF(o.x + o.width, o.y))
        for i, color in enumerate(o.gradient):
            grad.setColorAt(i / max(len(o.gradient) - 1, 1), QColor(color))
        painter.setBrush(QBrush(grad))
    else:
        painter.setBrush(QBrush(QColor(o.fill)))
    painter.setPen(pen)
    if o.type == "ellipse":
        painter.drawEllipse(QRectF(o.x, o.y, o.width, o.height))
    else:
        if o.corner_radius > 0:
            painter.drawRoundedRect(
                QRectF(o.x, o.y, o.width, o.height), o.corner_radius, o.corner_radius
            )
        else:
            painter.drawRect(QRectF(o.x, o.y, o.width, o.height))
    painter.restore()


def export_png(doc: Document, path: Path) -> None:
    _ = QGuiApplication.instance() or QGuiApplication(["petunia"])
    img = QImage(int(doc.width), int(doc.height), QImage.Format.Format_ARGB32)
    img.fill(QColor("#ffffff"))
    painter = QPainter(img)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)
    for o in doc.objects:
        _draw(doc, resolve_object(doc, o), painter)
    painter.end()
    img.save(str(path))


def export_slice_png(
    doc: Document,
    path: Path,
    x: float,
    y: float,
    w: float,
    h: float,
    scale: float = 1.0,
) -> None:
    _ = QGuiApplication.instance() or QGuiApplication(["petunia"])
    out_w = max(1, round(w * scale))
    out_h = max(1, round(h * scale))
    img = QImage(out_w, out_h, QImage.Format.Format_ARGB32)
    img.fill(QColor(0, 0, 0, 0))
    painter = QPainter(img)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)
    painter.scale(scale, scale)
    painter.translate(-x, -y)
    for o in doc.objects:
        _draw(doc, resolve_object(doc, o), painter)
    painter.end()
    img.save(str(path))
