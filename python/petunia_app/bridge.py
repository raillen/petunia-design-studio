from __future__ import annotations

import math
from copy import deepcopy
from pathlib import Path
from typing import Any

from PySide6.QtCore import Property, QObject, Signal, Slot

from .boolean import (
    BooleanError,
    ShapeFace,
    build_shape_faces,
    face_at_point,
)
from .exporters import export_svg
from .model import (
    AddObject,
    ApplyStyleCommand,
    BakeLiveCommand,
    BooleanCommand,
    BreakContourCommand,
    CompoundPathCommand,
    CreateStyleCommand,
    CreateSymbolCommand,
    DeleteNodeCommand,
    DetachStyleCommand,
    DetachSymbolCommand,
    Document,
    GroupObjects,
    History,
    InsertNodeCommand,
    JoinContoursCommand,
    LiveBooleanCommand,
    MoveObjects,
    PlaceSymbolCommand,
    RemoveObjects,
    ReorderObjectCommand,
    ReverseContourCommand,
    SceneObject,
    SetNodePositionCommand,
    SetObjectBoundsCommand,
    SetOverrideCommand,
    SetPathFrameCommand,
    SetProperty,
    ShapeBuilderCommitCommand,
    Snap,
    Ungroup,
    UpdateStyleCommand,
    UpdateSymbolCommand,
    evaluate_live,
)
from .paths import Contour, NodeKind, PathNode, VectorPath
from .paths import new_id as new_path_id
from .resources import (
    OVERRIDE_FIELDS,
    library_conflicts,
    missing_references,
    resolve_object,
)
from .text import (
    ApplyCharacterStyleCommand,
    ApplyParagraphStyleCommand,
    DeleteRangeCommand,
    InsertTextCommand,
    ReplaceRangeCommand,
    TextStory,
)


class Bridge(QObject):
    objectsChanged = Signal()
    selectionChanged = Signal()
    toolChanged = Signal()
    zoomChanged = Signal()
    actionResult = Signal(str)
    messageChanged = Signal()
    shapeFacesChanged = Signal()
    librariesChanged = Signal()
    personaChanged = Signal()
    snappingChanged = Signal()
    documentChanged = Signal()
    artboardsChanged = Signal()
    activeArtboardIndexChanged = Signal()
    swatchesChanged = Signal()
    slicesChanged = Signal()
    shapeConfigChanged = Signal()
    cloneSourceChanged = Signal()
    viewportFocusRequested = Signal(float, float, float)
    newDocumentRequested = Signal()
    documentSetupRequested = Signal()
    appSettingsRequested = Signal()
    helpRequested = Signal()
    placeImageRequested = Signal()

    def __init__(self) -> None:
        super().__init__()
        self.document = Document()
        self.history = History()
        self.snapper = Snap(8.0)
        self._tool = "select"
        self._selected: str | None = None
        self._selection: list[str] = []
        self._zoom = 1.0
        self._pen_contour: Contour | None = None
        self._selected_node: dict[str, Any] | None = None
        self._message: str = ""
        self._shape_faces: list[ShapeFace] = []
        self._shape_selected: set[int] = set()
        self._shape_order: list[str] = []
        self._shape_active: bool = False
        self._persona: str = "vector"
        self._current_fill: str = "#E120A5"
        self._current_stroke: str = "#ffffff"
        self._current_stroke_width: float = 2.0
        self._current_stroke_cap: str = "round"
        self._current_stroke_join: str = "round"
        self._current_stroke_dash: list[float] = []
        self._current_opacity: float = 1.0
        self._current_blend_mode: str = "normal"
        self._snapping_enabled: bool = True
        self._auto_select: bool = True
        self._auto_select_mode: str = "default"
        self._color_tags: dict[str, str] = {}
        self._locked_ids: set[str] = set()
        self._expanded_ids: set[str] = set()
        self._demo_loaded: bool = False
        self._active_artboard_index: int = 0
        self._swatches: list[str] = [
            "#FFFFFF", "#000000", "#1976D2", "#00B4D8",
            "#10B981", "#F59E0B", "#EF4444", "#8B5CF6",
            "#EC4899", "#64748B", "#3B82F6", "#14B8A6"
        ]
        self._slices: list[dict[str, Any]] = []
        self._selected_slice_id: str | None = None
        self._star_points: int = 5
        self._star_inner_radius: float = 0.45
        self._polygon_sides: int = 6
        self._cog_teeth: int = 8
        self._cog_tooth_depth: float = 0.30
        self._cog_hole_radius: float = 0.35
        self._clone_source_x: float = 0.0
        self._clone_source_y: float = 0.0
        self._clone_source_set: bool = False

    def _emitAll(self) -> None:
        self.objectsChanged.emit()
        self.selectionChanged.emit()
        self.librariesChanged.emit()
        self.documentChanged.emit()
        self.artboardsChanged.emit()
        self.activeArtboardIndexChanged.emit()
        self.swatchesChanged.emit()
        self.slicesChanged.emit()

    def _emitMessage(self, text: str) -> None:
        self._message = text
        self.messageChanged.emit()
        self.actionResult.emit(text)

    @Property(str, notify=messageChanged)
    def message(self) -> str:
        return self._message

    def _populate_text_info(self, o: SceneObject, d: dict[str, Any]) -> None:
        if o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            d["text"] = st.text
            if st.runs:
                r = st.runs[0]
                d["fontSize"] = r.overrides.get("fontSize", 14.0)
                d["fontFamily"] = r.overrides.get("fontFamily", "Inter")
                d["fontWeight"] = r.overrides.get("fontWeight", "normal")
                d["fontStyle"] = r.overrides.get("fontStyle", "normal")
            if st.paragraphs:
                d["textAlign"] = st.paragraphs[0].overrides.get("align", "left")

    @Property(list, notify=objectsChanged)
    def objects(self) -> list[dict[str, Any]]:
        res: list[dict[str, Any]] = []
        for o in self.document.objects:
            d = resolve_object(self.document, o).to_json()
            self._populate_text_info(o, d)
            res.append(d)
        return res

    @Property(list, notify=objectsChanged)
    def renderObjects(self) -> list[dict[str, Any]]:
        res: list[dict[str, Any]] = []

        def collect(o: SceneObject) -> None:
            if o.type in {"group", "artboard"}:
                for c in o.children:
                    collect(c)
            else:
                d = resolve_object(self.document, o).to_json()
                self._populate_text_info(o, d)
                res.append(d)

        for o in self.document.objects:
            collect(o)
        return res

    def _get_artboards(self) -> list[SceneObject]:
        return [o for o in self.document.objects if o.type == "artboard"]

    @Property(list, notify=artboardsChanged)
    def artboards(self) -> list[dict[str, Any]]:
        art_list = self._get_artboards()
        if not art_list:
            return [
                {
                    "id": "__doc__",
                    "name": self.document.name if self.document.name else "Artboard 1",
                    "x": 0.0,
                    "y": 0.0,
                    "width": self.document.width,
                    "height": self.document.height,
                    "fill": "#ffffff",
                    "stroke": "#3e3e42",
                    "selected": self._selected in (None, "", "__doc__"),
                    "childrenCount": len(self.document.objects),
                    "isDocument": True,
                }
            ]
        res: list[dict[str, Any]] = []
        for o in art_list:
            res.append(
                {
                    "id": o.id,
                    "name": o.name,
                    "x": o.x,
                    "y": o.y,
                    "width": o.width,
                    "height": o.height,
                    "fill": o.fill,
                    "stroke": o.stroke,
                    "selected": o.id in self._selection,
                    "childrenCount": len(o.children),
                    "isDocument": False,
                }
            )
        return res

    @Property(int, notify=artboardsChanged)
    def artboardCount(self) -> int:
        return max(1, len(self._get_artboards()))

    @Property(int, notify=activeArtboardIndexChanged)
    def activeArtboardIndex(self) -> int:
        arts = self._get_artboards()
        if not arts:
            return 0
        if self._selected:
            for i, a in enumerate(arts):
                if a.id == self._selected:
                    self._active_artboard_index = i
                    return i

                def has_descendant(parent: SceneObject) -> bool:
                    for c in parent.children:
                        if c.id == self._selected or has_descendant(c):
                            return True
                    return False

                if has_descendant(a):
                    self._active_artboard_index = i
                    return i
        if self._active_artboard_index >= len(arts):
            self._active_artboard_index = max(0, len(arts) - 1)
        return self._active_artboard_index

    @Property(str, notify=activeArtboardIndexChanged)
    def activeArtboardName(self) -> str:
        arts = self._get_artboards()
        if not arts:
            return self.document.name if self.document.name else "Artboard 1"
        idx = min(self.activeArtboardIndex, len(arts) - 1)
        return arts[idx].name

    @Property(float, notify=activeArtboardIndexChanged)
    def activeArtboardWidth(self) -> float:
        arts = self._get_artboards()
        if not arts:
            return self.document.width
        idx = min(self.activeArtboardIndex, len(arts) - 1)
        return arts[idx].width

    @Property(float, notify=activeArtboardIndexChanged)
    def activeArtboardHeight(self) -> float:
        arts = self._get_artboards()
        if not arts:
            return self.document.height
        idx = min(self.activeArtboardIndex, len(arts) - 1)
        return arts[idx].height

    @Property(list, notify=librariesChanged)
    def libraries(self) -> list[dict[str, Any]]:
        return [lib.to_json() for lib in self.document.libraries]

    @Property(list, notify=librariesChanged)
    def symbols(self) -> list[dict[str, Any]]:
        out: list[dict[str, Any]] = []
        for lib in self.document.libraries:
            for s in lib.symbols.values():
                d = s.to_json()
                d["libraryName"] = lib.name
                d["libraryScope"] = lib.scope
                out.append(d)
        return out

    @Property(list, notify=librariesChanged)
    def styles(self) -> list[dict[str, Any]]:
        out: list[dict[str, Any]] = []
        for lib in self.document.libraries:
            for st in lib.styles.values():
                d = st.to_json()
                d["libraryName"] = lib.name
                d["libraryScope"] = lib.scope
                out.append(d)
        return out

    @Property(dict, notify=librariesChanged)
    def missingReferences(self) -> dict[str, list[str]]:
        return missing_references(self.document)

    @Property(list, notify=librariesChanged)
    def libraryConflicts(self) -> list[str]:
        return library_conflicts(self.document)

    @Property(list, notify=selectionChanged)
    def selectedIds(self) -> list[str]:
        return list(self._selection)

    @Property(str, notify=selectionChanged)
    def selectedId(self) -> str:
        return self._selected or ""

    @Property(str, notify=selectionChanged)
    def selectedName(self) -> str:
        o = self._find(self._selected)
        return o.name if o else ""

    @Property(bool, notify=selectionChanged)
    def selectedIsSymbol(self) -> bool:
        o = self._find(self._selected)
        return o is not None and o.symbol_id is not None

    @Property(str, notify=selectionChanged)
    def selectedSymbolId(self) -> str:
        o = self._find(self._selected)
        return (o.symbol_id or "") if o is not None else ""

    @Property(str, notify=selectionChanged)
    def selectedStyleId(self) -> str:
        o = self._find(self._selected)
        return (o.style_id or "") if o is not None else ""

    @Property(dict, notify=selectionChanged)
    def selectedOverrides(self) -> dict[str, Any]:
        o = self._find(self._selected)
        return dict(o.overrides) if o is not None else {}

    @Property(dict, notify=selectionChanged)
    def selectedStory(self) -> dict[str, Any]:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            return self.document.stories[o.story_id].to_json()
        return {}

    @Property(str, notify=selectionChanged)
    def selectedText(self) -> str:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            return self.document.stories[o.story_id].text
        return ""

    @Property(float, notify=selectionChanged)
    def selectedFontSize(self) -> float:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            if st.runs and "fontSize" in st.runs[0].overrides:
                return float(st.runs[0].overrides["fontSize"])
        return 14.0

    @Property(str, notify=selectionChanged)
    def selectedFontFamily(self) -> str:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            if st.runs and "fontFamily" in st.runs[0].overrides:
                return str(st.runs[0].overrides["fontFamily"])
        return "Inter"

    @Property(bool, notify=selectionChanged)
    def selectedFontBold(self) -> bool:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            if st.runs and "fontWeight" in st.runs[0].overrides:
                return str(st.runs[0].overrides["fontWeight"]).lower() in {"bold", "700"}
        return False

    @Property(bool, notify=selectionChanged)
    def selectedFontItalic(self) -> bool:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            if st.runs and "fontStyle" in st.runs[0].overrides:
                return str(st.runs[0].overrides["fontStyle"]).lower() == "italic"
        return False

    @Property(str, notify=selectionChanged)
    def selectedTextAlign(self) -> str:
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            st = self.document.stories[o.story_id]
            if st.paragraphs and "align" in st.paragraphs[0].overrides:
                return str(st.paragraphs[0].overrides["align"])
        return "left"

    @Property(list, notify=objectsChanged)
    def layers(self) -> list[dict[str, Any]]:
        out: list[dict[str, Any]] = []

        def walk(o: SceneObject, depth: int) -> None:
            is_expanded = (not self._demo_loaded) or (o.id in self._expanded_ids) or (not o.children)
            out.append(
                {
                    "id": o.id,
                    "name": o.name,
                    "type": o.type,
                    "visible": o.visible,
                    "depth": depth,
                    "colorTag": self._color_tags.get(o.id, ""),
                    "locked": o.id in self._locked_ids,
                    "childrenCount": len(o.children),
                    "expanded": is_expanded,
                }
            )
            if is_expanded:
                for c in o.children:
                    walk(c, depth + 1)

        for o in self.document.objects:
            walk(o, 0)
        return out

    @Slot(str)
    def toggleLayerExpanded(self, obj_id: str) -> None:
        if obj_id in self._expanded_ids:
            self._expanded_ids.remove(obj_id)
        else:
            self._expanded_ids.add(obj_id)
        self.objectsChanged.emit()

    def _find(self, obj_id: str | None) -> SceneObject | None:
        def search(o: SceneObject) -> SceneObject | None:
            if o.id == obj_id:
                return o
            for c in o.children:
                found = search(c)
                if found:
                    return found
            return None

        if obj_id is None:
            return None
        for o in self.document.objects:
            found = search(o)
            if found:
                return found
        return None

    @Property(str, notify=toolChanged)
    def tool(self) -> str:
        return self._tool

    @Slot(str)
    def setTool(self, name: str) -> None:
        if self._tool != name:
            if name != "pen":
                self._cancel_pen()
            self._tool = name
            self.toolChanged.emit()

    @Property(float, notify=zoomChanged)
    def zoom(self) -> float:
        return self._zoom

    @Slot(float)
    def setZoom(self, value: float) -> None:
        self._zoom = max(0.1, min(8.0, value))
        self.zoomChanged.emit()

    @Slot()
    def zoomIn(self) -> None:
        self.setZoom(self._zoom * 1.25)

    @Slot()
    def zoomOut(self) -> None:
        self.setZoom(self._zoom / 1.25)

    @Slot()
    def zoomFit(self) -> None:
        self.setZoom(1.0)

    @Slot()
    def newDocument(self) -> None:
        self.document = Document()
        self.history = History()
        self._selected = None
        self._selection = []
        self._cancel_pen()
        self._color_tags.clear()
        self._locked_ids.clear()
        self._demo_loaded = False
        self._emitAll()

    @Property(str, notify=personaChanged)
    def persona(self) -> str:
        return self._persona

    @Slot(str)
    def setPersona(self, p: str) -> None:
        if self._persona != p:
            self._persona = p
            self.personaChanged.emit()

    @Property(str, notify=selectionChanged)
    def currentFill(self) -> str:
        o = self._find(self._selected)
        if o is not None and o.type not in ("group", "artboard"):
            return o.fill
        return self._current_fill

    @Property(str, notify=selectionChanged)
    def currentStroke(self) -> str:
        o = self._find(self._selected)
        if o is not None and o.type not in ("group", "artboard"):
            return o.stroke
        return self._current_stroke

    @Property(float, notify=selectionChanged)
    def currentStrokeWidth(self) -> float:
        o = self._find(self._selected)
        if o is not None:
            return o.stroke_width
        return self._current_stroke_width

    @Property(float, notify=selectionChanged)
    def currentOpacity(self) -> float:
        o = self._find(self._selected)
        if o is not None:
            return float(o.opacity)
        return self._current_opacity

    @Property(float, notify=selectionChanged)
    def selectedCornerRadius(self) -> float:
        o = self._find(self._selected)
        return float(o.corner_radius) if o is not None else 0.0

    @Property(float, notify=selectionChanged)
    def selectedOpacity(self) -> float:
        o = self._find(self._selected)
        return float(o.opacity) if o is not None else self._current_opacity

    @Property(list, notify=selectionChanged)
    def selectedGradient(self) -> list[str]:
        o = self._find(self._selected)
        return list(o.gradient) if o is not None else []

    @Slot(str)
    def setFillColor(self, hex_color: str) -> None:
        self._current_fill = hex_color
        if self._selected:
            self.setProp("fill", hex_color)
        else:
            self.selectionChanged.emit()

    @Slot(str)
    def setStrokeColor(self, hex_color: str) -> None:
        self._current_stroke = hex_color
        if self._selected:
            self.setProp("stroke", hex_color)
        else:
            self.selectionChanged.emit()

    @Slot(float)
    def setStrokeWidth(self, w: float) -> None:
        self._current_stroke_width = w
        if self._selected:
            self.setProp("strokeWidth", w)
        else:
            self.selectionChanged.emit()

    @Slot(float)
    def setOpacity(self, op: float) -> None:
        self._current_opacity = max(0.0, min(1.0, op))
        if self._selected:
            self.setProp("opacity", self._current_opacity)
        else:
            self.selectionChanged.emit()

    @Slot()
    def swapColors(self) -> None:
        self._current_fill, self._current_stroke = self._current_stroke, self._current_fill
        if self._selected:
            o = self._find(self._selected)
            if o is not None:
                old_f = o.fill
                old_s = o.stroke
                self.setProp("fill", old_s)
                self.setProp("stroke", old_f)
        self.selectionChanged.emit()

    @Property(str, notify=selectionChanged)
    def selectedBlendMode(self) -> str:
        o = self._find(self._selected)
        return str(o.blend_mode) if o is not None else self._current_blend_mode

    @Property(bool, notify=selectionChanged)
    def selectedIsLocked(self) -> bool:
        return self._selected in self._locked_ids if self._selected else False

    @Property(str, notify=selectionChanged)
    def selectedStrokeCap(self) -> str:
        o = self._find(self._selected)
        return str(o.stroke_cap) if o is not None else self._current_stroke_cap

    @Property(str, notify=selectionChanged)
    def selectedStrokeJoin(self) -> str:
        o = self._find(self._selected)
        return str(o.stroke_join) if o is not None else self._current_stroke_join

    @Property(list, notify=selectionChanged)
    def selectedStrokeDash(self) -> list[float]:
        o = self._find(self._selected)
        return list(o.stroke_dash) if o is not None else list(self._current_stroke_dash)

    @Slot(str)
    def setStrokeCap(self, cap: str) -> None:
        self._current_stroke_cap = cap.lower()
        if self._selected:
            self.setProp("strokeCap", self._current_stroke_cap)
        else:
            self.selectionChanged.emit()

    @Slot(str)
    def setStrokeJoin(self, join: str) -> None:
        self._current_stroke_join = join.lower()
        if self._selected:
            self.setProp("strokeJoin", self._current_stroke_join)
        else:
            self.selectionChanged.emit()

    @Slot(str)
    def setStrokeDash(self, dash_csv: str) -> None:
        try:
            parts = [float(x.strip()) for x in dash_csv.split(",") if x.strip()]
            self._current_stroke_dash = parts
            if self._selected:
                self.setProp("strokeDash", parts)
            else:
                self.selectionChanged.emit()
        except Exception:
            pass

    @Property(list, notify=swatchesChanged)
    def swatches(self) -> list[str]:
        return list(self._swatches)

    @Slot(str)
    def addSwatch(self, hex_color: str) -> None:
        c = hex_color.upper()
        if c and c not in self._swatches:
            self._swatches.append(c)
            self.swatchesChanged.emit()

    @Slot(str)
    def removeSwatch(self, hex_color: str) -> None:
        c = hex_color.upper()
        if c in self._swatches:
            self._swatches.remove(c)
            self.swatchesChanged.emit()

    @Property(bool, notify=snappingChanged)
    def snappingEnabled(self) -> bool:
        return self._snapping_enabled

    @Slot()
    def toggleSnapping(self) -> None:
        self._snapping_enabled = not self._snapping_enabled
        self.snapper.grid = 8.0 if self._snapping_enabled else 0.0
        self.snappingChanged.emit()

    @Property(bool, notify=selectionChanged)
    def autoSelectEnabled(self) -> bool:
        return self._auto_select

    @Slot(bool)
    def setAutoSelectEnabled(self, val: bool) -> None:
        self._auto_select = val
        self.selectionChanged.emit()

    @Property(str, notify=selectionChanged)
    def autoSelectMode(self) -> str:
        return self._auto_select_mode

    @Slot(str)
    def setAutoSelectMode(self, mode: str) -> None:
        self._auto_select_mode = mode
        self.selectionChanged.emit()

    @Property(str, notify=documentChanged)
    def documentName(self) -> str:
        return self.document.name

    @Slot(str)
    def setDocumentName(self, name: str) -> None:
        self.document.name = name
        self.documentChanged.emit()

    @Property(float, notify=documentChanged)
    def documentWidth(self) -> float:
        return self.document.width

    @Property(float, notify=documentChanged)
    def documentHeight(self) -> float:
        return self.document.height

    @Slot(float, float)
    def setDocumentSize(self, w: float, h: float) -> None:
        self.document.width = w
        self.document.height = h
        self.documentChanged.emit()

    @Property(bool, notify=documentChanged)
    def demoLoaded(self) -> bool:
        return self._demo_loaded

    @Slot(str)
    def alignSelected(self, mode: str) -> None:
        targets = [self._find(oid) for oid in self._selection if self._find(oid) is not None]
        if not targets:
            return
        if len(targets) == 1:
            o = targets[0]
            assert o is not None
            bx, by, bw, bh = o.bounds()
            dw, dh = self.document.width, self.document.height
            dx, dy = 0.0, 0.0
            if mode == "left": dx = -bx
            elif mode == "center": dx = (dw - bw) / 2.0 - bx
            elif mode == "right": dx = (dw - bw) - bx
            elif mode == "top": dy = -by
            elif mode == "middle": dy = (dh - bh) / 2.0 - by
            elif mode == "bottom": dy = (dh - bh) - by
            if dx != 0.0 or dy != 0.0:
                self.history.execute(MoveObjects([o.id], dx, dy), self.document)
                self._emitAll()
            return

        boxes = [o.bounds() for o in targets if o is not None]
        min_x = min(b[0] for b in boxes)
        max_r = max(b[0] + b[2] for b in boxes)
        min_y = min(b[1] for b in boxes)
        max_b = max(b[1] + b[3] for b in boxes)

        for o in targets:
            if o is None:
                continue
            bx, by, bw, bh = o.bounds()
            dx, dy = 0.0, 0.0
            if mode == "left": dx = min_x - bx
            elif mode == "center": dx = (min_x + max_r) / 2.0 - (bx + bw / 2.0)
            elif mode == "right": dx = max_r - (bx + bw)
            elif mode == "top": dy = min_y - by
            elif mode == "middle": dy = (min_y + max_b) / 2.0 - (by + bh / 2.0)
            elif mode == "bottom": dy = max_b - (by + bh)
            if dx != 0.0 or dy != 0.0:
                self.history.execute(MoveObjects([o.id], dx, dy), self.document)

        if mode == "distribute_h" and len(targets) > 2:
            targets_sorted = sorted(targets, key=lambda obj: obj.bounds()[0])
            total_w = sum(obj.bounds()[2] for obj in targets_sorted)
            span = max_r - min_x - total_w
            spacing = span / (len(targets_sorted) - 1) if len(targets_sorted) > 1 else 0.0
            curr_x = min_x
            for obj in targets_sorted:
                bx, _, bw, _ = obj.bounds()
                dx = curr_x - bx
                if dx != 0.0:
                    self.history.execute(MoveObjects([obj.id], dx, 0.0), self.document)
                curr_x += bw + spacing

        elif mode == "distribute_v" and len(targets) > 2:
            targets_sorted = sorted(targets, key=lambda obj: obj.bounds()[1])
            total_h = sum(obj.bounds()[3] for obj in targets_sorted)
            span = max_b - min_y - total_h
            spacing = span / (len(targets_sorted) - 1) if len(targets_sorted) > 1 else 0.0
            curr_y = min_y
            for obj in targets_sorted:
                _, by, _, bh = obj.bounds()
                dy = curr_y - by
                if dy != 0.0:
                    self.history.execute(MoveObjects([obj.id], 0.0, dy), self.document)
                curr_y += bh + spacing

        self._emitAll()

    @Slot()
    def duplicateSelected(self) -> None:
        if not self._selection:
            return
        new_ids: list[str] = []
        for oid in self._selection:
            o = self._find(oid)
            if o is None:
                continue
            d = deepcopy(o)
            d.id = new_path_id()
            d.name = f"{o.name} Copia"
            d.x += 20.0
            d.y += 20.0
            self.history.execute(AddObject(d), self.document)
            new_ids.append(d.id)
        if new_ids:
            self._selected = new_ids[-1]
            self._selection = new_ids
            self._emitAll()

    @Property(list, notify=objectsChanged)
    def historyItems(self) -> list[str]:
        names: list[str] = []
        for cmd in self.history.undo_stack:
            cname = cmd.__class__.__name__
            readable = {
                "AddObject": "Adicionar Objeto",
                "RemoveObject": "Excluir Objeto",
                "RemoveObjects": "Excluir Objetos",
                "MoveObject": "Mover Objeto",
                "MoveObjects": "Mover Objeto",
                "SetProperty": "Alterar Propriedade",
                "SetPathFrameCommand": "Transformar Objeto",
                "SetObjectBoundsCommand": "Redimensionar Objeto",
                "GroupObjects": "Agrupar",
                "Ungroup": "Desagrupar",
                "BooleanCommand": "Operação Booleana",
                "CompoundPathCommand": "Caminho Composto",
                "LiveBooleanCommand": "Boolean ao Vivo",
                "BakeLiveCommand": "Consolidar",
                "ShapeBuilderCommitCommand": "Shape Builder",
                "CreateSymbolCommand": "Criar Símbolo",
                "PlaceSymbolCommand": "Inserir Símbolo",
                "UpdateSymbolCommand": "Atualizar Símbolo",
                "DetachSymbolCommand": "Desanexar Símbolo",
                "CreateStyleCommand": "Criar Estilo",
                "ApplyStyleCommand": "Aplicar Estilo",
                "UpdateStyleCommand": "Atualizar Estilo",
                "DetachStyleCommand": "Desanexar Estilo",
                "InsertNodeCommand": "Inserir Nó",
                "DeleteNodeCommand": "Excluir Nó",
                "SetNodePositionCommand": "Mover Nó",
                "ReverseContourCommand": "Reverter Contorno",
                "BreakContourCommand": "Quebrar Contorno",
                "JoinContoursCommand": "Unir Contornos",
                "InsertTextCommand": "Inserir Texto",
                "ReplaceRangeCommand": "Substituir Texto",
                "DeleteRangeCommand": "Excluir Texto",
                "ApplyCharacterStyleCommand": "Estilo de Texto",
            }.get(cname, cname)
            names.append(readable)
        return names

    @Slot(str, str)
    def setLayerColorTag(self, obj_id: str, tag: str) -> None:
        self._color_tags[obj_id] = tag
        self.objectsChanged.emit()

    @Slot(str)
    def toggleLayerLock(self, obj_id: str) -> None:
        if obj_id in self._locked_ids:
            self._locked_ids.remove(obj_id)
        else:
            self._locked_ids.add(obj_id)
        self.objectsChanged.emit()

    @Slot(str)
    def toggleLayerVisibility(self, obj_id: str) -> None:
        o = self._find(obj_id)
        if o is not None:
            self.history.execute(SetProperty(obj_id, "visible", not o.visible), self.document)
            self._emitAll()

    @Slot(str, str)
    def renameLayer(self, obj_id: str, new_name: str) -> None:
        if obj_id and new_name:
            self.history.execute(SetProperty(obj_id, "name", new_name), self.document)
            self._emitAll()

    @Slot()
    def loadDemoDocument(self) -> None:
        self.document = Document()
        self.history = History()
        self.document.name = "Inioluwa Abiri"
        self.document.width = 1200.0
        self.document.height = 800.0
        self._selected = None
        self._selection = []
        self._color_tags.clear()
        self._locked_ids.clear()
        self._cancel_pen()
        self._cancel_shape_builder()

        artboard = SceneObject(
            type="artboard",
            name="Artboard1",
            x=0.0,
            y=0.0,
            width=1200.0,
            height=800.0,
            fill="#ffffff",
        )

        face_makeup = SceneObject(type="group", name="Face (makeup)", x=450.0, y=260.0, width=150.0, height=200.0)
        self._color_tags[face_makeup.id] = "#f97316"
        cheek = SceneObject(type="ellipse", name="Cheek Blush", x=470.0, y=340.0, width=50.0, height=45.0, fill="#e82040", stroke="transparent", stroke_width=0.0)
        brow1 = SceneObject(type="ellipse", name="Brow Dot 1", x=482.0, y=284.0, width=6.0, height=6.0, fill="#ff3333", stroke="transparent", stroke_width=0.0)
        brow2 = SceneObject(type="ellipse", name="Brow Dot 2", x=495.0, y=282.0, width=6.0, height=6.0, fill="#ff3333", stroke="transparent", stroke_width=0.0)
        nose = SceneObject(type="ellipse", name="Nose Accent", x=552.0, y=368.0, width=8.0, height=8.0, fill="#0088ff", stroke="transparent", stroke_width=0.0)
        face_makeup.children = [cheek, brow1, brow2, nose]

        stars = SceneObject(type="group", name="Stars", x=50.0, y=100.0, width=800.0, height=700.0)
        self._color_tags[stars.id] = "#a855f7"
        s1 = SceneObject(type="ellipse", name="Light Orb 1", x=105.0, y=300.0, width=125.0, height=125.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        s2 = SceneObject(type="ellipse", name="Light Orb 2", x=305.0, y=248.0, width=48.0, height=48.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        s3 = SceneObject(type="ellipse", name="Light Orb 3", x=715.0, y=220.0, width=72.0, height=72.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        s4 = SceneObject(type="ellipse", name="Light Orb 4", x=654.0, y=405.0, width=46.0, height=46.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        s5 = SceneObject(type="ellipse", name="Light Orb 5", x=756.0, y=720.0, width=52.0, height=52.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        s6 = SceneObject(type="ellipse", name="Light Orb 6", x=194.0, y=758.0, width=84.0, height=84.0, fill="#ffffff", stroke="transparent", stroke_width=0.0)
        stars.children = [s1, s2, s3, s4, s5, s6]

        head_piece = SceneObject(type="group", name="Flower head piece (front)", x=240.0, y=160.0, width=380.0, height=400.0)
        self._color_tags[head_piece.id] = "#22c55e"
        earring = SceneObject(type="ellipse", name="Earring Sphere", x=378.0, y=462.0, width=54.0, height=54.0, fill="#f59e0b", stroke="transparent", stroke_width=0.0)
        head_piece.children = [earring]

        flower_cloth = SceneObject(type="group", name="Flower cloth (front)", x=140.0, y=620.0, width=560.0, height=220.0)
        self._color_tags[flower_cloth.id] = "#f97316"

        portrait = SceneObject(type="group", name="Portrait", x=200.0, y=120.0, width=500.0, height=600.0)
        self._color_tags[portrait.id] = "#0ea5e9"
        pixel_base = SceneObject(type="rectangle", name="Pixel", x=220.0, y=150.0, width=460.0, height=550.0, fill="#1c1917", stroke="transparent", stroke_width=0.0)
        cb1 = SceneObject(type="rectangle", name="Colour Balance Adjustment", x=220.0, y=150.0, width=460.0, height=550.0, fill="transparent", stroke="transparent", stroke_width=0.0)
        cb2 = SceneObject(type="rectangle", name="Colour Balance Adjustment", x=220.0, y=150.0, width=460.0, height=550.0, fill="transparent", stroke="transparent", stroke_width=0.0)
        curves = SceneObject(type="rectangle", name="Curves Adjustment", x=220.0, y=150.0, width=460.0, height=550.0, fill="transparent", stroke="transparent", stroke_width=0.0)
        bw_adj = SceneObject(type="rectangle", name="Black-and-White Portrait...", x=220.0, y=150.0, width=460.0, height=550.0, fill="transparent", stroke="transparent", stroke_width=0.0)
        portrait.children = [pixel_base, cb1, cb2, curves, bw_adj]

        artboard.children = [face_makeup, stars, head_piece, flower_cloth, portrait]
        self.document.objects = [artboard]
        self._demo_loaded = True
        self._expanded_ids = {artboard.id, portrait.id}
        self._selected = artboard.id
        self._selection = [artboard.id]
        self._zoom = 0.5
        self._current_fill = "#E120A5"
        self._current_stroke = "#ffffff"
        self._emitAll()
        self.zoomChanged.emit()
        self.documentChanged.emit()

    @Slot(float, float, result=str)
    def objectAt(self, x: float, y: float) -> str:
        def hit(o: SceneObject) -> str | None:
            if not o.visible:
                return None
            bx, by, bw, bh = o.bounds()
            if not (bx <= x <= bx + bw and by <= y <= by + bh):
                return None
            for c in reversed(o.children):
                child = hit(c)
                if child:
                    return child
            return o.id

        for o in reversed(self.document.objects):
            result = hit(o)
            if result:
                return result
        return ""

    @Slot(str)
    def select(self, obj_id: str) -> None:
        self._selected = obj_id or None
        self._selection = [self._selected] if self._selected else []
        self._selected_node = None
        self.selectionChanged.emit()
        self.activeArtboardIndexChanged.emit()

    @Slot(str)
    def toggleSelect(self, obj_id: str) -> None:
        if obj_id in self._selection:
            self._selection = [i for i in self._selection if i != obj_id]
        else:
            self._selection.append(obj_id)
        self._selected = self._selection[-1] if self._selection else None
        self._selected_node = None
        self.selectionChanged.emit()
        self.activeArtboardIndexChanged.emit()

    @Slot(str, float, float, float, float, str, result=str)
    def createArtboard(
        self,
        name: str = "",
        x: float = 0.0,
        y: float = 0.0,
        width: float = 1280.0,
        height: float = 800.0,
        background: str = "#ffffff",
    ) -> str:
        arts = self._get_artboards()
        art_name = name.strip() if name.strip() else f"Artboard {len(arts) + 1}"
        w = max(20.0, width)
        h = max(20.0, height)
        artboard = SceneObject(
            type="artboard",
            name=art_name,
            x=x,
            y=y,
            width=w,
            height=h,
            fill=background,
            stroke="#3e3e42",
            stroke_width=1.0,
        )
        self.history.execute(AddObject(artboard), self.document)
        self._selected = artboard.id
        self._selection = [artboard.id]
        self._active_artboard_index = len(self._get_artboards()) - 1
        self._emitAll()
        self.focusArtboard(artboard.id)
        self._emitMessage(f"Prancheta '{art_name}' criada ({int(w)} × {int(h)})")
        return artboard.id

    @Slot(str, result=str)
    def insertPresetArtboard(self, preset: str = "doc") -> str:
        p = preset.lower().strip()
        dims: dict[str, tuple[str, float, float]] = {
            "doc": ("Doc", self.document.width, self.document.height),
            "fhd": ("FHD 1080p", 1920.0, 1080.0),
            "4k": ("4K UHD", 3840.0, 2160.0),
            "square": ("Quadrado", 1080.0, 1080.0),
            "story": ("Story", 1080.0, 1920.0),
            "a4": ("A4", 2480.0, 3508.0),
            "mobile": ("Mobile", 390.0, 844.0),
            "dribbble": ("Dribbble", 1600.0, 1200.0),
            "twitter_header": ("Banner", 1500.0, 500.0),
        }
        label, w, h = dims.get(p, ("Personalizado", 1280.0, 800.0))
        arts = self._get_artboards()
        if arts:
            max_right = max(a.x + a.width for a in arts)
            new_x = max_right + 120.0
            new_y = arts[0].y
        else:
            new_x = 0.0
            new_y = 0.0
        name = f"Artboard {len(arts) + 1} ({label})"
        return self.createArtboard(name, new_x, new_y, w, h, "#ffffff")

    @Slot(int)
    def selectArtboardIndex(self, idx: int) -> None:
        arts = self._get_artboards()
        if not arts:
            self._active_artboard_index = 0
            self.activeArtboardIndexChanged.emit()
            self.focusArtboard("__doc__")
            return
        target_idx = max(0, min(len(arts) - 1, idx))
        self._active_artboard_index = target_idx
        art = arts[target_idx]
        self._selected = art.id
        self._selection = [art.id]
        self.selectionChanged.emit()
        self.activeArtboardIndexChanged.emit()
        self.focusArtboard(art.id)

    @Slot()
    def nextArtboard(self) -> None:
        arts = self._get_artboards()
        count = len(arts)
        if count <= 1:
            return
        next_idx = (self.activeArtboardIndex + 1) % count
        self.selectArtboardIndex(next_idx)

    @Slot()
    def prevArtboard(self) -> None:
        arts = self._get_artboards()
        count = len(arts)
        if count <= 1:
            return
        prev_idx = (self.activeArtboardIndex - 1 + count) % count
        self.selectArtboardIndex(prev_idx)

    @Slot(str)
    def focusArtboard(self, artboard_id: str) -> None:
        if artboard_id == "__doc__" or not artboard_id:
            cx = self.document.width / 2.0
            cy = self.document.height / 2.0
            self.viewportFocusRequested.emit(cx, cy, self._zoom)
            return
        o = self._find(artboard_id)
        if o is not None:
            cx = o.x + o.width / 2.0
            cy = o.y + o.height / 2.0
            self.viewportFocusRequested.emit(cx, cy, self._zoom)

    @Slot()
    def fitAllArtboards(self) -> None:
        arts = self._get_artboards()
        if arts:
            min_x = min(a.x for a in arts)
            min_y = min(a.y for a in arts)
            max_r = max(a.x + a.width for a in arts)
            max_b = max(a.y + a.height for a in arts)
            cx = (min_x + max_r) / 2.0
            cy = (min_y + max_b) / 2.0
            span_w = max_r - min_x
            span_h = max_b - min_y
        else:
            cx = self.document.width / 2.0
            cy = self.document.height / 2.0
            span_w = self.document.width
            span_h = self.document.height
        fit_zoom = 1.0
        if span_w > 0 and span_h > 0:
            fit_zoom = max(0.1, min(2.0, min(1000.0 / span_w, 650.0 / span_h)))
        self._zoom = fit_zoom
        self.zoomChanged.emit()
        self.viewportFocusRequested.emit(cx, cy, fit_zoom)

    @Slot(str, float, float)
    def resizeArtboard(self, artboard_id: str, new_w: float, new_h: float) -> None:
        w = max(10.0, new_w)
        h = max(10.0, new_h)
        if artboard_id == "__doc__":
            self.setDocumentSize(w, h)
            return
        o = self._find(artboard_id)
        if o is not None and o.type == "artboard":
            self.history.execute(SetProperty(artboard_id, "width", w), self.document)
            self.history.execute(SetProperty(artboard_id, "height", h), self.document)
            self.artboardsChanged.emit()
            self.activeArtboardIndexChanged.emit()
            self._emitAll()

    @Slot(str, str)
    def setArtboardBackground(self, artboard_id: str, color: str) -> None:
        if artboard_id and artboard_id != "__doc__":
            self.history.execute(SetProperty(artboard_id, "fill", color), self.document)
            self.artboardsChanged.emit()
            self.objectsChanged.emit()

    @Slot(str, str)
    def setArtboardName(self, artboard_id: str, name: str) -> None:
        if artboard_id and name.strip():
            if artboard_id == "__doc__":
                self.setDocumentName(name.strip())
            else:
                self.renameLayer(artboard_id, name.strip())
            self.artboardsChanged.emit()
            self.activeArtboardIndexChanged.emit()

    @Property(int, notify=shapeConfigChanged)
    def starPoints(self) -> int:
        return self._star_points

    @Slot(int)
    def setStarPoints(self, points: int) -> None:
        self._star_points = max(3, min(36, points))
        self.shapeConfigChanged.emit()

    @Property(float, notify=shapeConfigChanged)
    def starInnerRadius(self) -> float:
        return self._star_inner_radius

    @Slot(float)
    def setStarInnerRadius(self, ratio: float) -> None:
        self._star_inner_radius = max(0.1, min(0.9, ratio))
        self.shapeConfigChanged.emit()

    @Property(int, notify=shapeConfigChanged)
    def polygonSides(self) -> int:
        return self._polygon_sides

    @Slot(int)
    def setPolygonSides(self, sides: int) -> None:
        self._polygon_sides = max(3, min(36, sides))
        self.shapeConfigChanged.emit()

    @Property(int, notify=shapeConfigChanged)
    def cogTeeth(self) -> int:
        return self._cog_teeth

    @Slot(int)
    def setCogTeeth(self, teeth: int) -> None:
        self._cog_teeth = max(3, min(48, teeth))
        self.shapeConfigChanged.emit()

    @Property(float, notify=shapeConfigChanged)
    def cogToothDepth(self) -> float:
        return self._cog_tooth_depth

    @Slot(float)
    def setCogToothDepth(self, depth: float) -> None:
        self._cog_tooth_depth = max(0.05, min(0.8, depth))
        self.shapeConfigChanged.emit()

    @Property(float, notify=shapeConfigChanged)
    def cogHoleRadius(self) -> float:
        return self._cog_hole_radius

    @Slot(float)
    def setCogHoleRadius(self, radius: float) -> None:
        self._cog_hole_radius = max(0.0, min(0.8, radius))
        self.shapeConfigChanged.emit()

    @Slot(str, float, float)
    def create_shape(self, shape_type: str, cx: float, cy: float) -> None:
        if shape_type in {"triangle", "star", "polygon", "diamond", "arrow", "heart", "cog"}:
            self.createShapeBounds(shape_type, cx - 50.0, cy - 50.0, 100.0, 100.0)
            return
        obj = SceneObject(type="ellipse" if shape_type == "ellipse" else "rectangle")
        obj.x = cx - obj.width / 2
        obj.y = cy - obj.height / 2
        obj.name = f"{obj.type.capitalize()} {len(self.document.objects) + 1}"
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()

    @Slot(str, float, float, float, float, result=str)
    @Slot(str, float, float, float, float, str, str, float, float, result=str)
    def createShapeBounds(
        self,
        shape_type: str,
        x: float,
        y: float,
        width: float,
        height: float,
        fill: str = "",
        stroke: str = "",
        stroke_width: float = -1.0,
        corner_radius: float = 0.0,
    ) -> str:
        w = max(2.0, abs(width))
        h = max(2.0, abs(height))
        norm_x = min(x, x + width) if width < 0 else x
        norm_y = min(y, y + height) if height < 0 else y
        f = fill if fill else self._current_fill
        s = stroke if stroke else self._current_stroke
        sw = stroke_width if stroke_width >= 0 else self._current_stroke_width

        if shape_type == "triangle":
            nodes = [
                PathNode(id=new_path_id(), x=norm_x + w / 2.0, y=norm_y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w, y=norm_y + h, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x, y=norm_y + h, kind=NodeKind.CUSP),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Triângulo {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "star":
            cx = norm_x + w / 2.0
            cy = norm_y + h / 2.0
            rx = w / 2.0
            ry = h / 2.0
            inner_rx = rx * self._star_inner_radius
            inner_ry = ry * self._star_inner_radius
            nodes = []
            pts = max(3, self._star_points)
            total_nodes = pts * 2
            for i in range(total_nodes):
                angle = -math.pi / 2.0 + i * (math.pi / pts)
                cur_rx = rx if i % 2 == 0 else inner_rx
                cur_ry = ry if i % 2 == 0 else inner_ry
                px = cx + cur_rx * math.cos(angle)
                py = cy + cur_ry * math.sin(angle)
                nodes.append(PathNode(id=new_path_id(), x=px, y=py, kind=NodeKind.CUSP))
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Estrela ({pts} Pontas) {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "polygon":
            cx = norm_x + w / 2.0
            cy = norm_y + h / 2.0
            rx = w / 2.0
            ry = h / 2.0
            nodes = []
            sides = max(3, self._polygon_sides)
            for i in range(sides):
                angle = -math.pi / 2.0 + i * (2.0 * math.pi / sides)
                px = cx + rx * math.cos(angle)
                py = cy + ry * math.sin(angle)
                nodes.append(PathNode(id=new_path_id(), x=px, y=py, kind=NodeKind.CUSP))
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Polígono ({sides} Lados) {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "diamond":
            cx = norm_x + w / 2.0
            cy = norm_y + h / 2.0
            nodes = [
                PathNode(id=new_path_id(), x=cx, y=norm_y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w, y=cy, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=cx, y=norm_y + h, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x, y=cy, kind=NodeKind.CUSP),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Losango {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "arrow":
            head_len = w * 0.4
            shaft_h = h * 0.4
            y_top = norm_y + (h - shaft_h) / 2.0
            y_bot = y_top + shaft_h
            nodes = [
                PathNode(id=new_path_id(), x=norm_x, y=y_top, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w - head_len, y=y_top, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w - head_len, y=norm_y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w, y=norm_y + h / 2.0, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w - head_len, y=norm_y + h, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x + w - head_len, y=y_bot, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=norm_x, y=y_bot, kind=NodeKind.CUSP),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Seta {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "heart":
            hcx = norm_x + w / 2.0
            nodes = [
                PathNode(
                    id=new_path_id(),
                    x=hcx,
                    y=norm_y + h * 0.25,
                    kind=NodeKind.CUSP,
                    out_handle=(-w * 0.25, -h * 0.25),
                ),
                PathNode(
                    id=new_path_id(),
                    x=norm_x,
                    y=norm_y + h * 0.35,
                    kind=NodeKind.SMOOTH,
                    in_handle=(0.0, -h * 0.15),
                    out_handle=(0.0, h * 0.25),
                ),
                PathNode(
                    id=new_path_id(),
                    x=hcx,
                    y=norm_y + h,
                    kind=NodeKind.CUSP,
                    in_handle=(-w * 0.15, -h * 0.15),
                    out_handle=(w * 0.15, -h * 0.15),
                ),
                PathNode(
                    id=new_path_id(),
                    x=norm_x + w,
                    y=norm_y + h * 0.35,
                    kind=NodeKind.SMOOTH,
                    in_handle=(0.0, h * 0.25),
                    out_handle=(0.0, -h * 0.15),
                ),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            vpath = VectorPath(contours=[contour])
            obj = SceneObject(
                type="path",
                name=f"Coração {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        elif shape_type == "cog":
            cx = norm_x + w / 2.0
            cy = norm_y + h / 2.0
            ro = min(w, h) / 2.0
            rr = ro * max(0.2, 1.0 - self._cog_tooth_depth)
            rh = ro * max(0.0, min(0.8, self._cog_hole_radius))
            teeth = max(3, self._cog_teeth)
            outer_nodes: list[PathNode] = []
            step = 2.0 * math.pi / teeth
            for i in range(teeth):
                th0 = -math.pi / 2.0 + i * step
                outer_nodes.append(PathNode(
                    id=new_path_id(),
                    x=cx + rr * math.cos(th0),
                    y=cy + rr * math.sin(th0),
                    kind=NodeKind.CUSP
                ))
                th1 = th0 + 0.25 * step
                outer_nodes.append(PathNode(
                    id=new_path_id(),
                    x=cx + ro * math.cos(th1),
                    y=cy + ro * math.sin(th1),
                    kind=NodeKind.CUSP
                ))
                th2 = th0 + 0.65 * step
                outer_nodes.append(PathNode(
                    id=new_path_id(),
                    x=cx + ro * math.cos(th2),
                    y=cy + ro * math.sin(th2),
                    kind=NodeKind.CUSP
                ))
                th3 = th0 + 0.85 * step
                outer_nodes.append(PathNode(
                    id=new_path_id(),
                    x=cx + rr * math.cos(th3),
                    y=cy + rr * math.sin(th3),
                    kind=NodeKind.CUSP
                ))
            outer_contour = Contour(id=new_path_id(), nodes=outer_nodes, closed=True)
            contours = [outer_contour]

            if rh > 2.0:
                hole_nodes: list[PathNode] = []
                hole_pts = 8
                for hi in range(hole_pts):
                    hang = 2.0 * math.pi * hi / hole_pts
                    hx = cx + rh * math.cos(-hang)
                    hy = cy + rh * math.sin(-hang)
                    hole_nodes.append(PathNode(id=new_path_id(), x=hx, y=hy, kind=NodeKind.CUSP))
                hole_contour = Contour(id=new_path_id(), nodes=hole_nodes, closed=True)
                contours.append(hole_contour)

            vpath = VectorPath(contours=contours)
            obj = SceneObject(
                type="path",
                name=f"Engrenagem ({teeth} Dentes) {len(self.document.objects) + 1}",
                path=vpath,
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                opacity=self._current_opacity,
            )
        else:
            kind = "ellipse" if shape_type == "ellipse" else "rectangle"
            obj = SceneObject(
                type=kind,
                name=f"{kind.capitalize()} {len(self.document.objects) + 1}",
                x=norm_x,
                y=norm_y,
                width=w,
                height=h,
                fill=f,
                stroke=s,
                stroke_width=sw,
                corner_radius=max(0.0, corner_radius),
                opacity=self._current_opacity,
            )
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        return obj.id

    @Slot(float, float, float, float, str, result=str)
    def createTextBounds(
        self,
        x: float,
        y: float,
        width: float = 200.0,
        height: float = 40.0,
        text: str = "Texto",
    ) -> str:
        w = max(30.0, abs(width))
        h = max(20.0, abs(height))
        norm_x = min(x, x + width) if width < 0 else x
        norm_y = min(y, y + height) if height < 0 else y
        story = TextStory(text=text)
        self.document.stories[story.id] = story
        obj = SceneObject(
            type="text",
            name=f"Text {len(self.document.objects) + 1}",
            x=norm_x,
            y=norm_y,
            width=w,
            height=h,
            fill=self._current_fill,
            story_id=story.id,
        )
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        return obj.id

    @Slot(list, str, float, str, bool, float, str, result=str)
    @Slot(list, str, float, str, bool, float, result=str)
    def createFreehandStroke(
        self,
        points: list[Any],
        stroke_color: str = "",
        stroke_width: float = -1.0,
        fill_color: str = "transparent",
        closed: bool = False,
        smoothing: float = 0.4,
        blend_mode: str = "normal",
    ) -> str:
        if not points or len(points) < 2:
            return ""
        pts: list[tuple[float, float]] = []
        for p in points:
            if isinstance(p, (list, tuple)) and len(p) >= 2:
                pts.append((float(p[0]), float(p[1])))
            elif isinstance(p, dict) and "x" in p and "y" in p:
                pts.append((float(p["x"]), float(p["y"])))
        if len(pts) < 2:
            return ""

        filtered_pts: list[tuple[float, float]] = [pts[0]]
        for p in pts[1:]:
            dx = p[0] - filtered_pts[-1][0]
            dy = p[1] - filtered_pts[-1][1]
            if (dx * dx + dy * dy) >= 4.0:
                filtered_pts.append(p)
        if len(filtered_pts) < 2:
            filtered_pts = pts[:2]

        nodes: list[PathNode] = []
        n_count = len(filtered_pts)
        for i, (px, py) in enumerate(filtered_pts):
            in_h: tuple[float, float] | None = None
            out_h: tuple[float, float] | None = None
            if smoothing > 0.0 and n_count > 2:
                if i == 0:
                    nxt = filtered_pts[1]
                    out_h = ((nxt[0] - px) * smoothing * 0.33, (nxt[1] - py) * smoothing * 0.33)
                elif i == n_count - 1:
                    prv = filtered_pts[i - 1]
                    in_h = ((prv[0] - px) * smoothing * 0.33, (prv[1] - py) * smoothing * 0.33)
                else:
                    prv = filtered_pts[i - 1]
                    nxt = filtered_pts[i + 1]
                    tan_x = (nxt[0] - prv[0]) * smoothing * 0.33
                    tan_y = (nxt[1] - prv[1]) * smoothing * 0.33
                    in_h = (-tan_x, -tan_y)
                    out_h = (tan_x, tan_y)
            nodes.append(
                PathNode(
                    id=new_path_id(),
                    x=px,
                    y=py,
                    kind=NodeKind.SMOOTH if (in_h or out_h) else NodeKind.CUSP,
                    in_handle=in_h,
                    out_handle=out_h,
                )
            )

        contour = Contour(id=new_path_id(), nodes=nodes, closed=closed)
        vpath = VectorPath(contours=[contour])
        s_col = stroke_color if stroke_color else self._current_stroke
        s_wid = stroke_width if stroke_width >= 0 else self._current_stroke_width
        obj = SceneObject(
            type="path",
            name=f"Traço {len(self.document.objects) + 1}",
            path=vpath,
            fill=fill_color,
            stroke=s_col,
            stroke_width=s_wid,
            blend_mode=blend_mode,
        )
        bx, by, bw, bh = obj.bounds()
        obj.x, obj.y, obj.width, obj.height = bx, by, bw, bh
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        return obj.id

    @Slot(float, float, float, float, result=int)
    def knifeCut(self, x1: float, y1: float, x2: float, y2: float) -> int:
        from .boolean import split_path_by_line

        cut_p1 = (float(x1), float(y1))
        cut_p2 = (float(x2), float(y2))
        dx = x2 - x1
        dy = y2 - y1
        if dx * dx + dy * dy < 4.0:
            return 0

        if self._selection:
            candidates = [o for o in self.document.objects if o.id in set(self._selection)]
        else:
            candidates = [o for o in self.document.objects if o.visible and o.type not in ("artboard", "group")]

        cut_count = 0
        new_objects: list[SceneObject] = []
        replaced_ids: set[str] = set()

        for o in candidates:
            self._ensure_path_for_object(o)
            if not o.path:
                continue
            split_result = split_path_by_line(o.path, cut_p1, cut_p2)
            if split_result is None:
                continue

            vp1, vp2 = split_result
            p1_obj = SceneObject(
                type="path",
                name=f"{o.name} (Parte 1)",
                path=vp1,
                fill=o.fill,
                stroke=o.stroke,
                stroke_width=o.stroke_width,
                opacity=o.opacity,
                blend_mode=o.blend_mode,
            )
            p2_obj = SceneObject(
                type="path",
                name=f"{o.name} (Parte 2)",
                path=vp2,
                fill=o.fill,
                stroke=o.stroke,
                stroke_width=o.stroke_width,
                opacity=o.opacity,
                blend_mode=o.blend_mode,
            )
            p1_obj.x, p1_obj.y, p1_obj.width, p1_obj.height = p1_obj.bounds()
            p2_obj.x, p2_obj.y, p2_obj.width, p2_obj.height = p2_obj.bounds()

            new_objects.extend([p1_obj, p2_obj])
            replaced_ids.add(o.id)
            cut_count += 1

        if cut_count > 0:
            updated_objs: list[SceneObject] = []
            for o in self.document.objects:
                if o.id in replaced_ids:
                    pass
                else:
                    updated_objs.append(o)
            updated_objs.extend(new_objects)
            self.document.objects = updated_objs
            self._selection = [n.id for n in new_objects]
            self._selected = new_objects[0].id if new_objects else None
            self._emitAll()
            self._emitMessage(f"Faca: {cut_count} objeto(s) cortado(s)")

        return cut_count

    @Slot(list, bool)
    @Slot(list)
    def lassoSelect(self, polygon_points: list[Any], additive: bool = False) -> None:
        """Select objects intersecting or contained in the freehand lasso polygon."""
        if not polygon_points or len(polygon_points) < 3:
            return
        pts: list[tuple[float, float]] = []
        for p in polygon_points:
            if isinstance(p, (list, tuple)) and len(p) >= 2:
                pts.append((float(p[0]), float(p[1])))
            elif isinstance(p, dict) and "x" in p and "y" in p:
                pts.append((float(p["x"]), float(p["y"])))
        if len(pts) < 3:
            return

        def point_in_poly(x: float, y: float) -> bool:
            inside = False
            n = len(pts)
            p1x, p1y = pts[0]
            for i in range(n + 1):
                p2x, p2y = pts[i % n]
                if y > min(p1y, p2y):
                    if y <= max(p1y, p2y):
                        if x <= max(p1x, p2x):
                            if p1y != p2y:
                                xinters = (y - p1y) * (p2x - p1x) / (p2y - p1y) + p1x
                            if p1x == p2x or x <= xinters:
                                inside = not inside
                p1x, p1y = p2x, p2y
            return inside

        hits = []
        for o in self.document.objects:
            if not o.visible or o.type == "artboard":
                continue
            cx = o.x + o.width / 2.0
            cy = o.y + o.height / 2.0
            if (
                point_in_poly(cx, cy)
                or point_in_poly(o.x, o.y)
                or point_in_poly(o.x + o.width, o.y + o.height)
            ):
                hits.append(o.id)

        if additive:
            new_sel = list(set(self._selection) | set(hits))
        else:
            new_sel = hits

        self._selection = new_sel
        self._selected = new_sel[0] if new_sel else None
        self.selectionChanged.emit()
        self._emitMessage(f"Laço: {len(hits)} objeto(s) selecionado(s)")

    @Property(float, notify=cloneSourceChanged)
    def cloneSourceX(self) -> float:
        return self._clone_source_x

    @Property(float, notify=cloneSourceChanged)
    def cloneSourceY(self) -> float:
        return self._clone_source_y

    @Property(bool, notify=cloneSourceChanged)
    def cloneSourceSet(self) -> bool:
        return self._clone_source_set

    @Slot(float, float)
    def setCloneSource(self, x: float, y: float) -> None:
        self._clone_source_x = float(x)
        self._clone_source_y = float(y)
        self._clone_source_set = True
        self.cloneSourceChanged.emit()
        self._emitMessage(f"Origem do Carimbo: ({int(x)}, {int(y)})")

    @Slot()
    def clearCloneSource(self) -> None:
        self._clone_source_set = False
        self.cloneSourceChanged.emit()

    @Slot(float, float, bool, result=str)
    def sampleColorAt(self, x: float, y: float, prefer_stroke: bool = False) -> str:
        hit_id = self.objectAt(x, y)
        if not hit_id:
            return ""
        o = self._find(hit_id)
        if o is None:
            return ""
        sampled = ""
        if prefer_stroke or o.fill in ("transparent", ""):
            sampled = o.stroke if o.stroke not in ("transparent", "") else o.fill
        else:
            sampled = o.fill
        if sampled:
            if prefer_stroke:
                self.setStrokeColor(sampled)
            else:
                self.setFillColor(sampled)
            self._emitMessage(f"Cor capturada: {sampled}")
            return sampled
        return ""

    @Slot(float, float, float, float, result="QVariant")
    def measureDistance(self, x1: float, y1: float, x2: float, y2: float) -> dict[str, float]:
        dx = x2 - x1
        dy = y2 - y1
        dist = math.hypot(dx, dy)
        angle = math.degrees(math.atan2(dy, dx))
        res = {"dx": dx, "dy": dy, "distance": dist, "angle": angle}
        self._emitMessage(f"Distância: {dist:.1f} px | Ângulo: {angle:.1f}° | ΔX: {dx:.1f} | ΔY: {dy:.1f}")
        return res

    @Slot(str, list)
    def setObjectGradient(self, obj_id: str, stops: list[str]) -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(SetProperty(target_id, "gradient", [str(s) for s in stops]), self.document)
        self._emitAll()
        self._emitMessage(f"Gradiente atualizado ({len(stops)} cores)")

    @Slot(str)
    def reverseObjectGradient(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.gradient:
            self.setObjectGradient(target_id, list(reversed(o.gradient)))

    @Slot(str)
    def removeObjectGradient(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(SetProperty(target_id, "gradient", []), self.document)
        self._emitAll()

    @Slot(str, float)
    def setCornerRadius(self, obj_id: str = "", radius: float = 0.0) -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(SetProperty(target_id, "corner_radius", max(0.0, radius)), self.document)
        self._emitAll()

    @Slot(str, result=str)
    def convertToCurves(self, obj_id: str = "") -> str:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o is None or o.type not in ("rectangle", "ellipse"):
            self._emitMessage("Selecione um retângulo ou elipse para converter em curvas")
            return ""
        if o.type == "rectangle":
            nodes = [
                PathNode(id=new_path_id(), x=o.x, y=o.y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x + o.width, y=o.y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x + o.width, y=o.y + o.height, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x, y=o.y + o.height, kind=NodeKind.CUSP),
            ]
        else:
            kappa = 0.5522847498307936
            cx = o.x + o.width / 2.0
            cy = o.y + o.height / 2.0
            rx = o.width / 2.0
            ry = o.height / 2.0
            kx = kappa * rx
            ky = kappa * ry
            nodes = [
                PathNode(id=new_path_id(), x=cx, y=cy - ry, kind=NodeKind.SMOOTH, in_handle=(-kx, 0.0), out_handle=(kx, 0.0)),
                PathNode(id=new_path_id(), x=cx + rx, y=cy, kind=NodeKind.SMOOTH, in_handle=(0.0, -ky), out_handle=(0.0, ky)),
                PathNode(id=new_path_id(), x=cx, y=cy + ry, kind=NodeKind.SMOOTH, in_handle=(kx, 0.0), out_handle=(-kx, 0.0)),
                PathNode(id=new_path_id(), x=cx - rx, y=cy, kind=NodeKind.SMOOTH, in_handle=(0.0, ky), out_handle=(0.0, -ky)),
            ]
        contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
        vpath = VectorPath(contours=[contour])
        self.history.execute(SetProperty(o.id, "type", "path"), self.document)
        self.history.execute(SetProperty(o.id, "path", vpath), self.document)
        self._emitAll()
        self._emitMessage(f"'{o.name}' convertido em curvas")
        return o.id

    @Slot(str, int, int, str)
    def setNodeKind(self, obj_id: str, contour_index: int, node_index: int, kind: str) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.path and 0 <= contour_index < len(o.path.contours):
            contour = o.path.contours[contour_index]
            if 0 <= node_index < len(contour.nodes):
                node = contour.nodes[node_index]
                try:
                    k = NodeKind(kind.lower())
                except ValueError:
                    k = NodeKind.CUSP
                node.kind = k
                if k in (NodeKind.SMOOTH, NodeKind.SYMMETRIC):
                    if node.in_handle is None and node.out_handle is None:
                        node.in_handle = (-15.0, 0.0)
                        node.out_handle = (15.0, 0.0)
                self.history.execute(
                    SetNodePositionCommand(target_id, contour_index, node_index, node.x, node.y),
                    self.document,
                )
                self._emitAll()

    @Slot(str, int, int, str, float, float)
    def moveNodeHandle(
        self, obj_id: str, contour_index: int, node_index: int, handle_type: str, hx: float, hy: float
    ) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.path and 0 <= contour_index < len(o.path.contours):
            contour = o.path.contours[contour_index]
            if 0 <= node_index < len(contour.nodes):
                node = contour.nodes[node_index]
                dx = hx - node.x
                dy = hy - node.y
                if handle_type == "in":
                    node.in_handle = (dx, dy)
                    if node.kind == NodeKind.SYMMETRIC:
                        node.out_handle = (-dx, -dy)
                    elif node.kind == NodeKind.SMOOTH and node.out_handle:
                        mag = math.hypot(*node.out_handle)
                        h_mag = math.hypot(dx, dy)
                        if h_mag > 0:
                            node.out_handle = (-dx / h_mag * mag, -dy / h_mag * mag)
                else:
                    node.out_handle = (dx, dy)
                    if node.kind == NodeKind.SYMMETRIC:
                        node.in_handle = (-dx, -dy)
                    elif node.kind == NodeKind.SMOOTH and node.in_handle:
                        mag = math.hypot(*node.in_handle)
                        h_mag = math.hypot(dx, dy)
                        if h_mag > 0:
                            node.in_handle = (-dx / h_mag * mag, -dy / h_mag * mag)
                self.objectsChanged.emit()

    @Slot(float, float)
    def updatePenLastNodeHandles(self, out_x: float, out_y: float) -> None:
        if self._pen_contour and self._pen_contour.nodes:
            last = self._pen_contour.nodes[-1]
            last.out_handle = (out_x - last.x, out_y - last.y)
            last.in_handle = (-(out_x - last.x), -(out_y - last.y))
            last.kind = NodeKind.SMOOTH
            self.toolChanged.emit()

    @Slot(str, float, float, float, float, result=str)
    def placeImage(
        self,
        file_path: str,
        x: float = 100.0,
        y: float = 100.0,
        w: float = 300.0,
        h: float = 200.0,
    ) -> str:
        name = Path(file_path).name if file_path else "Imagem"
        obj = SceneObject(
            type="rectangle",
            name=f"Imagem: {name}",
            x=x,
            y=y,
            width=w,
            height=h,
            fill="#27272a",
            stroke="#52525b",
            stroke_width=1.5,
        )
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        self._emitMessage(f"Imagem '{name}' inserida")
        return obj.id

    @Slot(float, float, str, result=str)
    def createText(self, x: float, y: float, text: str = "Texto") -> str:
        story = TextStory(text=text)
        self.document.stories[story.id] = story
        obj = SceneObject(
            type="text",
            name=f"Text {len(self.document.objects) + 1}",
            x=x,
            y=y,
            width=200.0,
            height=40.0,
            fill="#cccccc",
            story_id=story.id,
        )
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        return obj.id

    @Slot(str, int, str)
    def insertText(self, obj_id: str, offset: int, text: str) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.story_id and o.story_id in self.document.stories:
            self.history.execute(
                InsertTextCommand(o.story_id, offset, text), self.document
            )
            self._emitAll()

    @Slot(str, int, int)
    def deleteTextRange(self, obj_id: str, start: int, end: int) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.story_id and o.story_id in self.document.stories:
            self.history.execute(
                DeleteRangeCommand(o.story_id, start, end), self.document
            )
            self._emitAll()

    @Slot(str, int, int, str)
    def replaceTextRange(self, obj_id: str, start: int, end: int, text: str) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.story_id and o.story_id in self.document.stories:
            self.history.execute(
                ReplaceRangeCommand(o.story_id, start, end, text), self.document
            )
            self._emitAll()

    @Slot(str, int, int, str)
    def applyCharacterStyle(self, obj_id: str, start: int, end: int, style_id: str) -> None:
        target_id = obj_id or (self._selected or "")
        o = self._find(target_id)
        if o and o.story_id and o.story_id in self.document.stories:
            self.history.execute(
                ApplyCharacterStyleCommand(o.story_id, start, end, style_id), self.document
            )
            self._emitAll()

    @Slot(float)
    def setTextFontSize(self, size: float) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            story = self.document.stories[o.story_id]
            self.history.execute(
                ApplyCharacterStyleCommand(o.story_id, 0, len(story.text), None, {"fontSize": float(size)}),
                self.document,
            )
            self._emitAll()

    @Slot(str)
    def setTextFontFamily(self, family: str) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            story = self.document.stories[o.story_id]
            self.history.execute(
                ApplyCharacterStyleCommand(o.story_id, 0, len(story.text), None, {"fontFamily": str(family)}),
                self.document,
            )
            self._emitAll()

    @Slot(bool)
    def setTextBold(self, bold: bool) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            story = self.document.stories[o.story_id]
            weight = "bold" if bold else "normal"
            self.history.execute(
                ApplyCharacterStyleCommand(o.story_id, 0, len(story.text), None, {"fontWeight": weight}),
                self.document,
            )
            self._emitAll()

    @Slot(bool)
    def setTextItalic(self, italic: bool) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            story = self.document.stories[o.story_id]
            style = "italic" if italic else "normal"
            self.history.execute(
                ApplyCharacterStyleCommand(o.story_id, 0, len(story.text), None, {"fontStyle": style}),
                self.document,
            )
            self._emitAll()

    @Slot(str)
    def setTextAlign(self, align: str) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o and o.story_id and o.story_id in self.document.stories:
            story = self.document.stories[o.story_id]
            self.history.execute(
                ApplyParagraphStyleCommand(o.story_id, 0, len(story.text), None, {"align": str(align)}),
                self.document,
            )
            self._emitAll()

    @Slot(float, float)
    def moveSelected(self, dx: float, dy: float) -> None:
        if self._selection:
            self.history.execute(MoveObjects(self._selection, dx, dy), self.document)
            self._emitAll()

    @Slot()
    def deleteSelected(self) -> None:
        if self._selection:
            self.history.execute(RemoveObjects(self._selection), self.document)
            self._selected = None
            self._selection = []
            self._emitAll()

    @Slot()
    def groupSelected(self) -> None:
        if not self._selection:
            return
        cmd = GroupObjects(self._selection, f"Group {len(self.document.objects) + 1}")
        self.history.execute(cmd, self.document)
        if cmd.group is not None:
            self._selected = cmd.group.id
            self._selection = [cmd.group.id]
        self._emitAll()

    @Slot()
    def ungroupSelected(self) -> None:
        if self._selected:
            self.history.execute(Ungroup(self._selected), self.document)
            self._selected = None
            self._selection = []
            self._emitAll()

    @Slot(str)
    def bringToFront(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(ReorderObjectCommand(target_id, "front"), self.document)
        self._emitAll()

    @Slot(str)
    def sendToBack(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(ReorderObjectCommand(target_id, "back"), self.document)
        self._emitAll()

    @Slot(str)
    def moveForward(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(ReorderObjectCommand(target_id, "forward"), self.document)
        self._emitAll()

    @Slot(str)
    def moveBackward(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(ReorderObjectCommand(target_id, "backward"), self.document)
        self._emitAll()

    @Slot(str, str)
    def setBlendMode(self, obj_id: str, mode: str) -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        self.history.execute(SetProperty(target_id, "blend_mode", mode.lower()), self.document)
        self._emitAll()

    @Slot(float, float)
    def floodFill(self, x: float, y: float) -> None:
        hit_id = None
        for o in reversed(self.document.objects):
            bx, by, bw, bh = o.bounds()
            if bx <= x <= bx + bw and by <= y <= by + bh:
                hit_id = o.id
                break
        if hit_id:
            self.history.execute(SetProperty(hit_id, "fill", self._current_fill), self.document)
            self._selected = hit_id
            self._selection = [hit_id]
            self._emitAll()
            self._emitMessage(f"Preenchimento aplicado em {hit_id}")
        else:
            self.create_shape("rectangle", x - 50, y - 50)
            self._emitMessage("Preenchimento rápido inserido")

    @Slot(str)
    def toggleLock(self, obj_id: str) -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        if target_id in self._locked_ids:
            self._locked_ids.remove(target_id)
        else:
            self._locked_ids.add(target_id)
        self.objectsChanged.emit()

    # -- Export Persona Slices ------------------------------------------------

    @Property(list, notify=slicesChanged)
    def slices(self) -> list[dict[str, Any]]:
        return list(self._slices)

    @Property(str, notify=slicesChanged)
    def selectedSliceId(self) -> str:
        return self._selected_slice_id or ""

    @Slot(str, float, float, float, float, str, float, result=str)
    def createSlice(
        self,
        name: str = "",
        x: float = 0.0,
        y: float = 0.0,
        width: float = 100.0,
        height: float = 100.0,
        fmt: str = "PNG",
        scale: float = 1.0,
    ) -> str:
        sid = f"slice_{len(self._slices) + 1}_{int(x)}_{int(y)}"
        sname = name or f"Slice {len(self._slices) + 1}"
        item = {
            "id": sid,
            "name": sname,
            "x": float(x),
            "y": float(y),
            "width": max(10.0, float(width)),
            "height": max(10.0, float(height)),
            "format": fmt.upper(),
            "scale": float(scale) if scale > 0 else 1.0,
        }
        self._slices.append(item)
        self._selected_slice_id = sid
        self.slicesChanged.emit()
        self._emitMessage(f"Fatia criada: {sname} ({int(width)}x{int(height)})")
        return sid

    @Slot(str)
    def deleteSlice(self, slice_id: str) -> None:
        self._slices = [s for s in self._slices if s["id"] != slice_id]
        if self._selected_slice_id == slice_id:
            self._selected_slice_id = self._slices[0]["id"] if self._slices else None
        self.slicesChanged.emit()

    @Slot(str)
    def selectSlice(self, slice_id: str) -> None:
        self._selected_slice_id = slice_id
        self.slicesChanged.emit()

    @Slot(str, str, "QVariant")
    def setSliceProp(self, slice_id: str, prop: str, value: Any) -> None:
        for s in self._slices:
            if s["id"] == slice_id:
                if prop in ("x", "y", "width", "height", "scale"):
                    s[prop] = float(value)
                else:
                    s[prop] = str(value)
                self.slicesChanged.emit()
                break

    @Slot(str, str)
    def exportSlice(self, slice_id: str, out_path: str = "") -> None:
        from .raster import export_slice_png

        target = None
        for s in self._slices:
            if s["id"] == slice_id:
                target = s
                break
        if not target:
            return
        dest = Path(out_path) if out_path else Path(f"/tmp/{target['name'].replace(' ', '_')}.png")
        export_slice_png(
            self.document,
            dest,
            target["x"],
            target["y"],
            target["width"],
            target["height"],
            target.get("scale", 1.0),
        )
        self._emitMessage(f"Fatia exportada: {dest}")

    @Slot(str)
    def exportAllSlices(self, out_dir: str = "/tmp") -> None:
        from .raster import export_slice_png

        d = Path(out_dir)
        d.mkdir(parents=True, exist_ok=True)
        count = 0
        for s in self._slices:
            dest = d / f"{s['name'].replace(' ', '_')}.png"
            export_slice_png(
                self.document,
                dest,
                s["x"],
                s["y"],
                s["width"],
                s["height"],
                s.get("scale", 1.0),
            )
            count += 1
        self._emitMessage(f"{count} fatias exportadas em {out_dir}")

    @Slot(str, "QVariant")
    def setProp(self, prop: str, value: Any) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o is None:
            return
        try:
            coerced: Any = value
            numeric_props = {
                "x", "y", "width", "height", "strokeWidth", "stroke_width",
                "cornerRadius", "corner_radius", "opacity", "rotation", "shear",
            }
            if prop in numeric_props:
                coerced = float(value)
            if prop == "visible":
                coerced = str(value).lower() in {"true", "1", "yes"}
            if prop == "strokeWidth":
                prop = "stroke_width"
            if prop == "cornerRadius":
                prop = "corner_radius"
            if prop in {"blendMode", "blend_mode"}:
                prop = "blend_mode"
                coerced = str(value).lower()
            if prop in {"strokeCap", "stroke_cap"}:
                prop = "stroke_cap"
                coerced = str(value).lower()
            if prop in {"strokeJoin", "stroke_join"}:
                prop = "stroke_join"
                coerced = str(value).lower()
            if prop in {"strokeDash", "stroke_dash"}:
                prop = "stroke_dash"
                if isinstance(value, str):
                    coerced = [float(v.strip()) for v in value.split(",") if v.strip()]
                elif isinstance(value, (list, tuple)):
                    coerced = [float(v) for v in value]
            if prop in {"fontSize", "font_size"}:
                self.setTextFontSize(float(value))
                return
            if prop in {"fontFamily", "font_family"}:
                self.setTextFontFamily(str(value))
                return
            if prop in {"bold", "fontWeight"}:
                self.setTextBold(str(value).lower() in {"bold", "700", "true", "1"})
                return
            if prop in {"italic", "fontStyle"}:
                self.setTextItalic(str(value).lower() in {"italic", "true", "1"})
                return
            if prop in {"textAlign", "text_align", "align"}:
                self.setTextAlign(str(value).lower())
                return
            if prop == "text" and o.story_id and o.story_id in self.document.stories:
                story = self.document.stories[o.story_id]
                self.history.execute(
                    ReplaceRangeCommand(o.story_id, 0, len(story.text), str(value)),
                    self.document,
                )
                self._emitAll()
                return
            if prop in {"x", "y", "width", "height"} and o.path is not None and o.symbol_id is None:
                # Path objects derive their frame from geometry.
                bx, by, bw, bh = o.bounds()
                target_x = coerced if prop == "x" else bx
                target_y = coerced if prop == "y" else by
                target_w = coerced if prop == "width" else bw
                target_h = coerced if prop == "height" else bh
                self.history.execute(
                    SetPathFrameCommand(
                        self._selected, target_x, target_y, target_w, target_h
                    ),
                    self.document,
                )
                self._emitAll()
                return
            if o.symbol_id is not None and prop in OVERRIDE_FIELDS:
                self.history.execute(
                    SetOverrideCommand(self._selected, prop, coerced), self.document
                )
            else:
                self.history.execute(SetProperty(self._selected, prop, coerced), self.document)
            self._emitAll()
        except (ValueError, TypeError, AttributeError) as exc:
            self._emitMessage(f"Erro: {exc}")

    @Slot(str, float, float, float, float)
    @Slot(float, float, float, float)
    def setObjectBounds(self, *args) -> None:
        if len(args) == 5:
            obj_id, x, y, w, h = str(args[0]), float(args[1]), float(args[2]), float(args[3]), float(args[4])
        elif len(args) == 4:
            obj_id = self._selected or ""
            x, y, w, h = float(args[0]), float(args[1]), float(args[2]), float(args[3])
        else:
            return
        target_id = obj_id or self._selected
        if not target_id:
            return
        sx = self.snapper.snap(x)
        sy = self.snapper.snap(y)
        sw = max(2.0, w)
        sh = max(2.0, h)
        self.history.execute(
            SetObjectBoundsCommand(target_id, sx, sy, sw, sh),
            self.document,
        )
        self._emitAll()

    @Slot(str, str, result=str)
    def createSymbol(self, obj_id: str = "", name: str = "") -> str:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return ""
        o = self._find(target_id)
        if o is None:
            return ""
        sym_name = name or f"{o.name} Symbol"
        cmd = CreateSymbolCommand(target_id, sym_name)
        self.history.execute(cmd, self.document)
        self._emitAll()
        return cmd.definition.id if cmd.definition else ""

    @Slot(str, float, float, result=str)
    def placeSymbol(self, symbol_id: str, x: float = 0.0, y: float = 0.0) -> str:
        cmd = PlaceSymbolCommand(symbol_id, x=x, y=y)
        self.history.execute(cmd, self.document)
        if cmd.instance:
            self._selected = cmd.instance.id
            self._selection = [cmd.instance.id]
        self._emitAll()
        return cmd.instance.id if cmd.instance else ""

    @Slot(str)
    def updateSymbol(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        cmd = UpdateSymbolCommand(target_id)
        self.history.execute(cmd, self.document)
        self._emitAll()

    @Slot(str)
    def detachSymbol(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        cmd = DetachSymbolCommand(target_id)
        self.history.execute(cmd, self.document)
        self._emitAll()

    @Slot(str, str, str, result=str)
    def createStyle(self, name: str, kind: str = "object", obj_id: str = "") -> str:
        target_id = obj_id or (self._selected or "")
        cmd = CreateStyleCommand(name=name, kind=kind, obj_id=target_id)
        self.history.execute(cmd, self.document)
        self._emitAll()
        return cmd.style.id if cmd.style else ""

    @Slot(str)
    def applyStyle(self, style_id: str) -> None:
        targets = (
            list(self._selection)
            if self._selection
            else ([self._selected] if self._selected else [])
        )
        if not targets:
            return
        cmd = ApplyStyleCommand(targets, style_id)
        self.history.execute(cmd, self.document)
        self._emitAll()

    @Slot(str)
    def detachStyle(self, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        cmd = DetachStyleCommand(target_id)
        self.history.execute(cmd, self.document)
        self._emitAll()

    @Slot(str, str)
    def updateStyle(self, style_id: str, obj_id: str = "") -> None:
        target_id = obj_id or (self._selected or "")
        if not target_id or not style_id:
            return
        cmd = UpdateStyleCommand(style_id, target_id)
        self.history.execute(cmd, self.document)
        self._emitAll()

    @Slot()
    def snapSelected(self) -> None:
        for oid in list(self._selection):
            o = self._find(oid)
            if o is None:
                continue
            if o.path is not None:
                bx, by, _, _ = o.bounds()
            else:
                bx, by = o.x, o.y
            dx = self.snapper.snap(bx) - bx
            dy = self.snapper.snap(by) - by
            if dx != 0.0 or dy != 0.0:
                self.history.execute(MoveObjects([oid], dx, dy), self.document)
        self._emitAll()

    @Property(bool, notify=documentChanged)
    def canUndo(self) -> bool:
        return self.history.can_undo()

    @Property(bool, notify=documentChanged)
    def canRedo(self) -> bool:
        return self.history.can_redo()

    @Slot()
    def undo(self) -> None:
        self.history.undo(self.document)
        self._selected = None
        self._selection = []
        self._selected_node = None
        self._emitAll()

    @Slot()
    def redo(self) -> None:
        self.history.redo(self.document)
        self._selected = None
        self._selection = []
        self._selected_node = None
        self._emitAll()

    @Slot(int)
    def jumpHistory(self, target_index: int) -> None:
        current_len = len(self.history.undo_stack)
        if target_index < 0 or target_index >= current_len:
            return
        steps_to_undo = current_len - 1 - target_index
        for _ in range(steps_to_undo):
            self.history.undo(self.document)
        self._selected = None
        self._selection = []
        self._selected_node = None
        self._emitAll()

    @Slot(str)
    def flipSelected(self, direction: str) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o is None:
            return
        bx, by, bw, bh = o.bounds()
        cx = bx + bw / 2.0
        cy = by + bh / 2.0
        if o.path is not None:
            new_contours = []
            for c in o.path.contours:
                new_nodes = []
                for n in c.nodes:
                    if direction == "horizontal":
                        nx = 2.0 * cx - n.x
                        ny = n.y
                        in_h = (-n.in_handle[0], n.in_handle[1]) if n.in_handle else None
                        out_h = (-n.out_handle[0], n.out_handle[1]) if n.out_handle else None
                    else:
                        nx = n.x
                        ny = 2.0 * cy - n.y
                        in_h = (n.in_handle[0], -n.in_handle[1]) if n.in_handle else None
                        out_h = (n.out_handle[0], -n.out_handle[1]) if n.out_handle else None
                    new_nodes.append(PathNode(id=n.id, x=nx, y=ny, kind=n.kind, in_handle=in_h, out_handle=out_h))
                new_contours.append(Contour(id=c.id, nodes=new_nodes, closed=c.closed))
            new_path = VectorPath(contours=new_contours, fill_rule=o.path.fill_rule)
            self.history.execute(SetProperty(self._selected, "path", new_path), self.document)
            self._emitAll()
        elif o.type in {"rectangle", "ellipse"}:
            self._emitMessage(f"Objeto invertido ({direction})")

    @Slot(float)
    def rotateSelected(self, angle: float) -> None:
        if not self._selected:
            return
        o = self._find(self._selected)
        if o is None:
            return
        cur_rot = getattr(o, "rotation", 0.0)
        new_rot = round((cur_rot + angle) % 360.0, 1)
        self.history.execute(SetProperty(self._selected, "rotation", new_rot), self.document)
        self._emitAll()
        self._emitMessage(f"Rotacionado para {new_rot:.0f}°")

    @Slot(str, float)
    @Slot(float)
    def setRotation(self, *args) -> None:
        if len(args) == 2:
            obj_id, angle = str(args[0]), float(args[1])
        elif len(args) == 1:
            obj_id, angle = (self._selected or ""), float(args[0])
        else:
            return
        target_id = obj_id or (self._selected or "")
        if not target_id:
            return
        norm_angle = round(float(angle) % 360.0, 1)
        self.history.execute(SetProperty(target_id, "rotation", norm_angle), self.document)
        self._emitAll()

    @Slot(result=str)
    def exportSvg(self) -> str:
        return export_svg(self.document)

    @Slot(str)
    def exportPng(self, path: str) -> None:
        from .raster import export_png

        export_png(self.document, Path(path))
        self._emitMessage(f"PNG salvo em {path}")

    @Slot(str)
    def save(self, path: str) -> None:
        from .ptnd_scene import save_ptnd

        try:
            save_ptnd(Path(path), self.document)
            self._emitMessage(f"Salvo em {path}")
        except Exception as exc:
            self._emitMessage(f"Erro ao salvar: {exc}")

    @Slot(str)
    def open(self, path: str) -> None:
        from .ptnd_scene import open_ptnd

        try:
            self.document = open_ptnd(Path(path))
            self.history = History()
            self._selected = None
            self._selection = []
            self._cancel_pen()
            self._emitAll()
        except Exception as exc:
            self._emitMessage(f"Erro ao abrir: {exc}")

    @Slot(float, result=float)
    def snapValue(self, v: float) -> float:
        return self.snapper.snap(v)

    # -- Pen state machine (G029) -------------------------------------------

    @Property(bool, notify=toolChanged)
    def penActive(self) -> bool:
        return self._pen_contour is not None

    @Property(list, notify=toolChanged)
    def penPreview(self) -> list[dict[str, Any]]:
        if self._pen_contour is None:
            return []
        return [
            {
                "closed": self._pen_contour.closed,
                "nodes": [
                    {
                        "x": n.x,
                        "y": n.y,
                        "inHandle": list(n.in_handle) if n.in_handle else None,
                        "outHandle": list(n.out_handle) if n.out_handle else None,
                    }
                    for n in self._pen_contour.nodes
                ],
            }
        ]

    @Slot(float, float)
    def penClick(self, x: float, y: float) -> None:
        sx = self.snapper.snap(x)
        sy = self.snapper.snap(y)
        if self._pen_contour is None:
            self._pen_contour = Contour(id=new_path_id())
            self._pen_contour.nodes.append(
                PathNode(id=new_path_id(), x=sx, y=sy, kind=NodeKind.CUSP)
            )
            self.toolChanged.emit()
            return
        contour = self._pen_contour
        first = contour.nodes[0]
        if len(contour.nodes) >= 2 and abs(first.x - sx) < 8.0 and abs(first.y - sy) < 8.0:
            contour.closed = True
            self._commit_pen()
            return
        contour.nodes.append(PathNode(id=new_path_id(), x=sx, y=sy, kind=NodeKind.CUSP))
        self.toolChanged.emit()

    @Slot()
    def penFinish(self) -> None:
        if self._pen_contour is not None:
            self._commit_pen()

    @Slot()
    def penCancel(self) -> None:
        self._cancel_pen()
        self.toolChanged.emit()

    def _commit_pen(self) -> None:
        contour = self._pen_contour
        self._pen_contour = None
        if contour is None or len(contour.nodes) < 2:
            return
        obj = SceneObject(
            type="path",
            name=f"Path {len(self.document.objects) + 1}",
            path=VectorPath(contours=[contour]),
        )
        bx, by, bw, bh = obj.bounds()
        obj.x, obj.y, obj.width, obj.height = bx, by, bw, bh
        self.history.execute(AddObject(obj), self.document)
        self._selected = obj.id
        self._selection = [obj.id]
        self._emitAll()
        self.toolChanged.emit()

    def _cancel_pen(self) -> None:
        self._pen_contour = None

    # -- Node tool (G029) ----------------------------------------------------

    @Property(dict, notify=selectionChanged)
    def selectedNode(self) -> dict[str, Any]:
        return self._selected_node or {}

    @Slot(float, float, result=dict)
    def nodeAt(self, x: float, y: float, threshold: float = 10.0) -> dict[str, Any]:
        o = self._find(self._selected)
        if o is None or o.path is None:
            return {}
        best: dict[str, Any] | None = None
        best_d2 = threshold * threshold

        # Hit test Bézier handles of selected node first
        if self._selected_node and self._selected_node.get("objId") == o.id:
            ci = self._selected_node.get("contourIndex", -1)
            ni = self._selected_node.get("nodeIndex", -1)
            if 0 <= ci < len(o.path.contours) and 0 <= ni < len(o.path.contours[ci].nodes):
                sn = o.path.contours[ci].nodes[ni]
                if sn.in_handle:
                    ihx = sn.x + sn.in_handle[0]
                    ihy = sn.y + sn.in_handle[1]
                    if (ihx - x) ** 2 + (ihy - y) ** 2 <= best_d2:
                        return {
                            "objId": o.id,
                            "contourIndex": ci,
                            "nodeIndex": ni,
                            "handle": "in",
                            "x": ihx,
                            "y": ihy,
                        }
                if sn.out_handle:
                    ohx = sn.x + sn.out_handle[0]
                    ohy = sn.y + sn.out_handle[1]
                    if (ohx - x) ** 2 + (ohy - y) ** 2 <= best_d2:
                        return {
                            "objId": o.id,
                            "contourIndex": ci,
                            "nodeIndex": ni,
                            "handle": "out",
                            "x": ohx,
                            "y": ohy,
                        }

        for ci, contour in enumerate(o.path.contours):
            for ni, node in enumerate(contour.nodes):
                d2 = (node.x - x) ** 2 + (node.y - y) ** 2
                if d2 <= best_d2:
                    best_d2 = d2
                    best = {
                        "objId": o.id,
                        "contourIndex": ci,
                        "nodeIndex": ni,
                        "x": node.x,
                        "y": node.y,
                    }
        self._selected_node = best
        if best is not None:
            self.selectionChanged.emit()
        return best or {}

    @Slot(str, int, int, float, float)
    def moveNode(
        self, obj_id: str, contour_index: int, node_index: int, x: float, y: float
    ) -> None:
        sx = self.snapper.snap(x)
        sy = self.snapper.snap(y)
        self.history.execute(
            SetNodePositionCommand(obj_id, contour_index, node_index, sx, sy),
            self.document,
        )
        self._emitAll()

    @Slot(str, int, int, float)
    def insertNode(self, obj_id: str, contour_index: int, segment_index: int, t: float) -> None:
        self.history.execute(
            InsertNodeCommand(obj_id, contour_index, segment_index, t),
            self.document,
        )
        self._emitAll()

    @Slot(str, int, int)
    def deleteNode(self, obj_id: str, contour_index: int, node_index: int) -> None:
        self.history.execute(
            DeleteNodeCommand(obj_id, contour_index, node_index),
            self.document,
        )
        self._selected_node = None
        self._emitAll()

    @Slot(str, int)
    def reverseContour(self, obj_id: str, contour_index: int) -> None:
        self.history.execute(
            ReverseContourCommand(obj_id, contour_index), self.document
        )
        self._emitAll()

    @Slot(str, int, int)
    def breakContour(self, obj_id: str, contour_index: int, node_index: int) -> None:
        self.history.execute(
            BreakContourCommand(obj_id, contour_index, node_index), self.document
        )
        self._emitAll()

    @Slot(str, int, int, result=bool)
    def joinContours(self, obj_id: str, contour_a: int, contour_b: int) -> bool:
        cmd = JoinContoursCommand(obj_id, contour_a, contour_b)
        try:
            self.history.execute(cmd, self.document)
        except ValueError:
            self._emitMessage("Erro: endpoints não coincidem")
            return False
        self._emitAll()
        return True

    # ------------------------------------------------------------------
    # G031 — Boolean, compound paths, live booleans, Shape Builder
    # ------------------------------------------------------------------

    def _ensure_path_for_object(self, o: SceneObject) -> None:
        if o.path is not None:
            return
        if o.type == "rectangle":
            nodes = [
                PathNode(id=new_path_id(), x=o.x, y=o.y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x + o.width, y=o.y, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x + o.width, y=o.y + o.height, kind=NodeKind.CUSP),
                PathNode(id=new_path_id(), x=o.x, y=o.y + o.height, kind=NodeKind.CUSP),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            o.path = VectorPath(contours=[contour])
            o.type = "path"
        elif o.type == "ellipse":
            kappa = 0.5522847498307936
            cx = o.x + o.width / 2.0
            cy = o.y + o.height / 2.0
            rx = o.width / 2.0
            ry = o.height / 2.0
            kx = kappa * rx
            ky = kappa * ry
            nodes = [
                PathNode(id=new_path_id(), x=cx, y=cy - ry, kind=NodeKind.SMOOTH, in_handle=(-kx, 0.0), out_handle=(kx, 0.0)),
                PathNode(id=new_path_id(), x=cx + rx, y=cy, kind=NodeKind.SMOOTH, in_handle=(0.0, -ky), out_handle=(0.0, ky)),
                PathNode(id=new_path_id(), x=cx, y=cy + ry, kind=NodeKind.SMOOTH, in_handle=(kx, 0.0), out_handle=(-kx, 0.0)),
                PathNode(id=new_path_id(), x=cx - rx, y=cy, kind=NodeKind.SMOOTH, in_handle=(0.0, ky), out_handle=(0.0, -ky)),
            ]
            contour = Contour(id=new_path_id(), nodes=nodes, closed=True)
            o.path = VectorPath(contours=[contour])
            o.type = "path"

    def _selected_path_objects(self) -> list[SceneObject]:
        wanted = set(self._selection)
        return [o for o in self.document.objects if o.id in wanted]

    @Slot(str)
    def boolean(self, op: str) -> None:
        objs = self._selected_path_objects()
        if len(objs) < 2:
            self._emitMessage("Selecione ao menos dois caminhos")
            return
        for o in objs:
            self._ensure_path_for_object(o)
            if o.path is None:
                self._emitMessage("Boolean opera apenas sobre caminhos")
                return
        rule = objs[-1].path.fill_rule.value if objs[-1].path else "nonzero"
        cmd = BooleanCommand([o.id for o in objs], op, rule)
        try:
            self.history.execute(cmd, self.document)
        except BooleanError as err:
            self._emitMessage(f"Erro de geometria: {err}")
            return
        self._selected = cmd.after[0]["id"] if cmd.after else None
        self._selection = [s["id"] for s in cmd.after]
        self._cancel_shape_builder()
        self._emitAll()
        self._emitMessage(f"Boolean {op} aplicado")

    @Slot(str)
    def compoundPath(self, fill_rule: str = "nonzero") -> None:
        objs = self._selected_path_objects()
        if len(objs) < 2:
            self._emitMessage("Selecione ao menos dois caminhos")
            return
        for o in objs:
            self._ensure_path_for_object(o)
            if o.path is None:
                self._emitMessage("Caminho composto requer caminhos")
                return
        cmd = CompoundPathCommand([o.id for o in objs], fill_rule)
        try:
            self.history.execute(cmd, self.document)
        except BooleanError as err:
            self._emitMessage(f"Erro de geometria: {err}")
            return
        self._selected = cmd.after[0]["id"] if cmd.after else None
        self._selection = [s["id"] for s in cmd.after]
        self._cancel_shape_builder()
        self._emitAll()
        self._emitMessage("Caminho composto criado")

    @Slot(str)
    def makeLive(self, op: str) -> None:
        objs = self._selected_path_objects()
        if len(objs) < 2:
            self._emitMessage("Selecione ao menos dois caminhos")
            return
        for o in objs:
            self._ensure_path_for_object(o)
            if o.path is None:
                self._emitMessage("Boolean ao vivo requer caminhos")
                return
        cmd = LiveBooleanCommand([o.id for o in objs], op)
        try:
            self.history.execute(cmd, self.document)
        except BooleanError as err:
            self._emitMessage(f"Erro de geometria: {err}")
            return
        self._selected = cmd.live_id
        self._selection = [cmd.live_id] if cmd.live_id else []
        self._emitAll()
        self._emitMessage(f"Boolean ao vivo '{op}' criado")

    @Slot()
    def bakeLive(self) -> None:
        obj = self._find(self._selected)
        if obj is None or obj.live is None:
            self._emitMessage("Selecione um boolean ao vivo")
            return
        self.history.execute(BakeLiveCommand(obj.id), self.document)
        self._emitAll()
        self._emitMessage("Boolean ao vivo consolidado")

    @Slot()
    def refreshLive(self) -> None:
        evaluated = evaluate_live(self.document)
        self._emitAll()
        if evaluated:
            self._emitMessage(f"{len(evaluated)} boolean(s) ao vivo atualizado(s)")

    @Slot()
    def shapeBuilderStart(self) -> None:
        objs = self._selected_path_objects()
        if len(objs) < 2:
            self._emitMessage("Shape Builder requer ao menos dois caminhos")
            return
        for o in objs:
            self._ensure_path_for_object(o)
            if o.path is None:
                self._emitMessage("Shape Builder requer caminhos")
                return
        self._shape_faces = build_shape_faces(
            [(o.path, o.id) for o in objs if o.path is not None]
        )
        self._shape_order = [o.id for o in objs]
        self._shape_selected = set()
        self._shape_active = True
        self.shapeFacesChanged.emit()
        self._emitMessage("Shape Builder: clique nas regiões para somar/subtrair")

    @Slot(float, float, result=int)
    def shapeBuilderHit(self, x: float, y: float) -> int:
        if not self._shape_active:
            return -1
        return face_at_point(self._shape_faces, x, y)

    @Slot(int)
    def shapeBuilderToggle(self, index: int) -> None:
        if not self._shape_active or index < 0 or index >= len(self._shape_faces):
            return
        if index in self._shape_selected:
            self._shape_selected.discard(index)
        else:
            self._shape_selected.add(index)
        self.shapeFacesChanged.emit()

    @Slot()
    def shapeBuilderCommit(self) -> None:
        if not self._shape_active:
            return
        if not self._shape_selected:
            self._emitMessage("Selecione ao menos uma região")
            return
        faces = [self._shape_faces[i] for i in sorted(self._shape_selected)]
        polygons = [f.polygon for f in faces]
        style_source = self._shape_order[-1]
        for oid in reversed(self._shape_order):
            if any(oid in f.sources for f in faces):
                style_source = oid
                break
        cmd = ShapeBuilderCommitCommand(
            list(self._shape_order), polygons, style_source
        )
        self.history.execute(cmd, self.document)
        self._selected = cmd.after[0]["id"] if cmd.after else None
        self._selection = [s["id"] for s in cmd.after]
        self._cancel_shape_builder()
        self._emitAll()
        self._emitMessage("Shape Builder aplicado")

    @Slot()
    def shapeBuilderCancel(self) -> None:
        self._cancel_shape_builder()
        self._emitAll()
        self._emitMessage("Shape Builder cancelado")

    def _cancel_shape_builder(self) -> None:
        self._shape_faces = []
        self._shape_selected = set()
        self._shape_order = []
        self._shape_active = False

    @Property(bool, notify=shapeFacesChanged)
    def shapeBuilderActive(self) -> bool:
        return self._shape_active

    @Property(list, notify=shapeFacesChanged)
    def shapeFaces(self) -> list[dict[str, Any]]:
        result: list[dict[str, Any]] = []
        for i, face in enumerate(self._shape_faces):
            pts = [[x / 1000.0, y / 1000.0] for x, y in face.polygon]
            result.append(
                {
                    "points": pts,
                    "sources": list(face.sources),
                    "selected": i in self._shape_selected,
                }
            )
        return result

    @Slot()
    def requestNewDocument(self) -> None:
        self.newDocumentRequested.emit()

    @Slot()
    def requestDocumentSetup(self) -> None:
        self.documentSetupRequested.emit()

    @Slot()
    def requestAppSettings(self) -> None:
        self.appSettingsRequested.emit()

    @Slot()
    def requestHelp(self) -> None:
        self.helpRequested.emit()

    @Slot()
    def requestPlaceImage(self) -> None:
        self.placeImageRequested.emit()
