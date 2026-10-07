from __future__ import annotations

import uuid
from copy import deepcopy
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Literal, cast

if TYPE_CHECKING:
    from .text import TextStory

from .boolean import (
    BooleanError,
    BooleanOp,
    Polygon,
    boolean_paths,
    compound_paths,
    rebuild_path,
)
from .paths import (
    Contour,
    FillRule,
    VectorPath,
    break_contour,
    delete_node,
    insert_node,
    join_contours,
    path_bounds,
    reverse_contour,
    set_node_position,
)

ShapeType = Literal["rectangle", "ellipse", "group", "path", "text", "artboard"]


def new_id() -> str:
    return str(uuid.uuid4())


@dataclass
class LiveBoolean:
    """Live boolean node: operation plus ordered operand references (09.6.3)."""

    op: str = "union"
    source_ids: list[str] = field(default_factory=lambda: [])


@dataclass
class SceneObject:
    id: str = field(default_factory=new_id)
    type: ShapeType = "rectangle"
    name: str = "Object"
    x: float = 0.0
    y: float = 0.0
    width: float = 100.0
    height: float = 100.0
    fill: str = "#9bb8ff"
    gradient: list[str] = field(default_factory=lambda: [])
    stroke: str = "#1f2937"
    stroke_width: float = 2.0
    stroke_cap: str = "round"
    stroke_join: str = "round"
    stroke_dash: list[float] = field(default_factory=lambda: [])
    corner_radius: float = 0.0
    opacity: float = 1.0
    blend_mode: str = "normal"
    rotation: float = 0.0
    shear: float = 0.0
    visible: bool = True
    children: list[SceneObject] = field(default_factory=lambda: [])
    path: VectorPath | None = None
    live: LiveBoolean | None = None
    symbol_id: str | None = None
    overrides: dict[str, Any] = field(default_factory=lambda: {})
    style_id: str | None = None
    story_id: str | None = None

    def bounds(self) -> tuple[float, float, float, float]:
        if self.type == "artboard":
            return (self.x, self.y, self.width, self.height)
        if self.path is not None:
            return path_bounds(self.path)
        if self.children:
            x0 = min(c.bounds()[0] for c in self.children)
            y0 = min(c.bounds()[1] for c in self.children)
            x1 = max(c.bounds()[0] + c.bounds()[2] for c in self.children)
            y1 = max(c.bounds()[1] + c.bounds()[3] for c in self.children)
            return (x0, y0, x1 - x0, y1 - y0)
        return (self.x, self.y, self.width, self.height)

    def to_json(self) -> dict[str, Any]:
        if self.type in {"group", "path"}:
            bx, by, bw, bh = self.bounds()
        else:
            bx, by, bw, bh = self.x, self.y, self.width, self.height
        result: dict[str, Any] = {
            "id": self.id,
            "type": self.type,
            "name": self.name,
            "x": bx,
            "y": by,
            "width": bw,
            "height": bh,
            "fill": self.fill,
            "gradient": list(self.gradient),
            "stroke": self.stroke,
            "strokeWidth": self.stroke_width,
            "strokeCap": self.stroke_cap,
            "strokeJoin": self.stroke_join,
            "strokeDash": list(self.stroke_dash),
            "cornerRadius": self.corner_radius,
            "opacity": self.opacity,
            "blendMode": self.blend_mode,
            "rotation": self.rotation,
            "shear": self.shear,
            "visible": self.visible,
            "children": [c.to_json() for c in self.children],
        }
        if self.path is not None:
            result["path"] = self.path.to_json()
        if self.live is not None:
            result["live"] = {"op": self.live.op, "sourceIds": list(self.live.source_ids)}
        if self.symbol_id is not None:
            result["symbolId"] = self.symbol_id
        if self.style_id is not None:
            result["styleId"] = self.style_id
        if self.story_id is not None:
            result["storyId"] = self.story_id
        if self.overrides:
            result["overrides"] = dict(self.overrides)
        return result

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> SceneObject:
        raw_type = str(d["type"])
        if raw_type == "ellipse":
            kind: ShapeType = "ellipse"
        elif raw_type == "group":
            kind = "group"
        elif raw_type == "path":
            kind = "path"
        elif raw_type == "text":
            kind = "text"
        elif raw_type == "artboard":
            kind = "artboard"
        else:
            kind = "rectangle"
        raw_path = d.get("path")
        raw_live = d.get("live")
        live: LiveBoolean | None = None
        if raw_live is not None:
            live = LiveBoolean(
                op=str(raw_live.get("op", "union")),
                source_ids=[str(s) for s in raw_live.get("sourceIds", [])],
            )
        frame = d.get("frame")
        if isinstance(frame, dict):
            f_dict = cast(dict[str, Any], frame)
            x = float(f_dict.get("x", d.get("x", 0.0)))
            y = float(f_dict.get("y", d.get("y", 0.0)))
            w = float(f_dict.get("width", d.get("width", 100.0)))
            h = float(f_dict.get("height", d.get("height", 100.0)))
        else:
            x = float(d.get("x", 0.0))
            y = float(d.get("y", 0.0))
            w = float(d.get("width", 100.0))
            h = float(d.get("height", 100.0))
        return cls(
            id=str(d["id"]),
            type=kind,
            name=str(d.get("name", "Object")),
            x=x,
            y=y,
            width=w,
            height=h,
            fill=str(d.get("fill", "#9bb8ff")),
            gradient=[str(c) for c in d.get("gradient", [])],
            stroke=str(d.get("stroke", "#1f2937")),
            stroke_width=float(d.get("strokeWidth", 2.0)),
            stroke_cap=str(d.get("strokeCap", "round")),
            stroke_join=str(d.get("strokeJoin", "round")),
            stroke_dash=[float(x) for x in d.get("strokeDash", [])],
            corner_radius=float(d.get("cornerRadius", 0.0)),
            opacity=float(d.get("opacity", 1.0)),
            blend_mode=str(d.get("blendMode", "normal")),
            rotation=float(d.get("rotation", 0.0)),
            shear=float(d.get("shear", 0.0)),
            visible=bool(d.get("visible", True)),
            children=[SceneObject.from_json(c) for c in d.get("children", [])],
            path=VectorPath.from_json(raw_path) if raw_path is not None else None,
            live=live,
            symbol_id=(
                str(d["symbolId"]) if d.get("symbolId") is not None else None
            ),
            style_id=(
                str(d["styleId"]) if d.get("styleId") is not None else None
            ),
            story_id=(
                str(d["storyId"]) if d.get("storyId") is not None else None
            ),
            overrides=dict(d.get("overrides", {})),
        )


@dataclass
class SymbolDefinition:
    """Master snapshot for symbol instances (G032)."""

    id: str
    name: str
    source: SceneObject

    def to_json(self) -> dict[str, Any]:
        source_json = self.source.to_json()
        bx, by, bw, bh = self.source.bounds()
        source_json["frame"] = {"x": bx, "y": by, "width": bw, "height": bh}
        return {"id": self.id, "name": self.name, "source": source_json}

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> SymbolDefinition:
        return cls(
            id=str(d["id"]),
            name=str(d["name"]),
            source=SceneObject.from_json(d["source"]),
        )


@dataclass
class ObjectStyle:
    """Named appearance resource (G032)."""

    id: str
    name: str
    fill: str = "#9bb8ff"
    gradient: list[str] = field(default_factory=lambda: [])
    stroke: str = "#1f2937"
    stroke_width: float = 2.0
    stroke_cap: str = "round"
    stroke_join: str = "round"
    stroke_dash: list[float] = field(default_factory=lambda: [])
    visible: bool = True
    kind: str = "object"

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "fill": self.fill,
            "gradient": list(self.gradient),
            "stroke": self.stroke,
            "strokeWidth": self.stroke_width,
            "visible": self.visible,
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> ObjectStyle:
        return cls(
            id=str(d["id"]),
            name=str(d["name"]),
            fill=str(d.get("fill", "#9bb8ff")),
            gradient=[str(c) for c in d.get("gradient", [])],
            stroke=str(d.get("stroke", "#1f2937")),
            stroke_width=float(d.get("strokeWidth", 2.0)),
            stroke_cap=str(d.get("strokeCap", "round")),
            stroke_join=str(d.get("strokeJoin", "round")),
            stroke_dash=[float(x) for x in d.get("strokeDash", [])],
            visible=bool(d.get("visible", True)),
        )


@dataclass
class TextStyle:
    """Character/Paragraph style shell (G032).

    Properties are consumed by the text wave (G033+); the
    resource identity exists now so style dependencies can be
    created, referenced and persisted before text ships.
    """

    id: str
    name: str
    kind: str  # "character" | "paragraph"
    properties: dict[str, Any] = field(default_factory=lambda: {})

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "properties": dict(self.properties),
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> TextStyle:
        return cls(
            id=str(d["id"]),
            name=str(d["name"]),
            kind=str(d.get("kind", "character")),
            properties=dict(d.get("properties", {})),
        )


Style = ObjectStyle | TextStyle


@dataclass
class AssetLibrary:
    """Named container of symbols and styles with a scope:
    ``document`` (persisted inside the PTND) or ``user`` (a
    standalone JSON file shared across documents)."""

    id: str
    name: str
    scope: str = "document"
    symbols: dict[str, SymbolDefinition] = field(default_factory=lambda: {})
    styles: dict[str, Style] = field(default_factory=lambda: {})

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "scope": self.scope,
            "symbols": [s.to_json() for s in self.symbols.values()],
            "styles": [s.to_json() for s in self.styles.values()],
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> AssetLibrary:
        library = cls(
            id=str(d["id"]),
            name=str(d["name"]),
            scope="user" if d.get("scope") == "user" else "document",
        )
        for raw in d.get("symbols", []):
            symbol = SymbolDefinition.from_json(raw)
            library.symbols[symbol.id] = symbol
        for raw in d.get("styles", []):
            style: Style
            if str(raw.get("kind", "object")) == "object":
                style = ObjectStyle.from_json(raw)
            else:
                style = TextStyle.from_json(raw)
            library.styles[style.id] = style
        return library

    def asset_names(self) -> set[str]:
        return (
            {s.name for s in self.symbols.values()}
            | {s.name for s in self.styles.values()}
        )


@dataclass
class Document:
    name: str = "Untitled"
    width: float = 1280.0
    height: float = 800.0
    objects: list[SceneObject] = field(default_factory=lambda: [])
    document_id: str = field(default_factory=new_id)
    libraries: list[AssetLibrary] = field(default_factory=lambda: [])
    stories: dict[str, TextStory] = field(default_factory=lambda: {})

    def __post_init__(self) -> None:
        if not any(library.scope == "document" for library in self.libraries):
            self.libraries.insert(
                0, AssetLibrary(id=new_id(), name="Document", scope="document")
            )

    def to_json(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "width": self.width,
            "height": self.height,
            "documentId": self.document_id,
            "objects": [o.to_json() for o in self.objects],
            "libraries": [lib.to_json() for lib in self.libraries],
            "stories": [s.to_json() for s in self.stories.values()],
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> Document:
        raw_stories = d.get("stories", [])
        stories_dict: dict[str, TextStory] = {}
        if raw_stories:
            from .text import TextStory

            for raw_s in cast(list[dict[str, Any]], raw_stories):
                st = TextStory.from_json(raw_s)
                stories_dict[st.id] = st
        return cls(
            name=str(d.get("name", "Untitled")),
            width=float(d["width"]),
            height=float(d["height"]),
            document_id=str(d.get("documentId", new_id())),
            objects=[SceneObject.from_json(o) for o in d.get("objects", [])],
            libraries=[
                AssetLibrary.from_json(lib) for lib in d.get("libraries", [])
            ],
            stories=stories_dict,
        )


def document_library(doc: Document) -> AssetLibrary:
    """The document-scope library (created on first access)."""
    for library in doc.libraries:
        if library.scope == "document":
            return library
    library = AssetLibrary(id=new_id(), name="Document", scope="document")
    doc.libraries.insert(0, library)
    return library


def resolve_object_local(
    doc: Document, obj: SceneObject
) -> SceneObject:
    """Lazy indirection: :mod:`petunia_app.resources` imports
    this module, so commands resolve through a runtime import."""
    from .resources import resolve_object

    return resolve_object(doc, obj)


@dataclass
class Change:
    added: list[str] = field(default_factory=lambda: [])
    removed: list[str] = field(default_factory=lambda: [])
    modified: list[str] = field(default_factory=lambda: [])


class Command:
    def apply(self, doc: Document) -> Change:  # pragma: no cover - interface
        raise NotImplementedError

    def revert(self, doc: Document) -> None:  # pragma: no cover - interface
        raise NotImplementedError


class AddObject(Command):
    def __init__(self, obj: SceneObject) -> None:
        self.obj = obj

    def apply(self, doc: Document) -> Change:
        doc.objects.append(self.obj)
        return Change(added=[self.obj.id])

    def revert(self, doc: Document) -> None:
        doc.objects = [o for o in doc.objects if o.id != self.obj.id]


class RemoveObject(Command):
    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.snapshot: SceneObject | None = None

    def apply(self, doc: Document) -> Change:
        for o in doc.objects:
            if o.id == self.obj_id:
                self.snapshot = o
                doc.objects.remove(o)
                return Change(removed=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        if self.snapshot:
            doc.objects.append(self.snapshot)


class RemoveObjects(Command):
    def __init__(self, obj_ids: list[str]) -> None:
        self.obj_ids = list(obj_ids)
        self.snapshots: list[SceneObject] = []

    def apply(self, doc: Document) -> Change:
        self.snapshots = []
        for o in list(doc.objects):
            if o.id in self.obj_ids:
                self.snapshots.append(o)
                doc.objects.remove(o)
        return Change(removed=list(self.obj_ids))

    def revert(self, doc: Document) -> None:
        doc.objects.extend(self.snapshots)


def _translate(o: SceneObject, dx: float, dy: float) -> None:
    """Translate an object. Symbol instances carry their own
    transform (x/y); plain path objects move their nodes
    (x/y is derived); everything else moves its frame."""
    if o.symbol_id is not None:
        o.x += dx
        o.y += dy
    elif o.path is not None:
        for c in o.path.contours:
            for n in c.nodes:
                n.x += dx
                n.y += dy
    else:
        o.x += dx
        o.y += dy
    for c in o.children:
        _translate(c, dx, dy)


class MoveObjects(Command):
    def __init__(self, obj_ids: list[str], dx: float, dy: float) -> None:
        self.obj_ids = list(obj_ids)
        self.dx = dx
        self.dy = dy

    def apply(self, doc: Document) -> Change:
        moved = False
        for o in doc.objects:
            if o.id in self.obj_ids:
                _translate(o, self.dx, self.dy)
                moved = True
        return Change(modified=[i for i in self.obj_ids]) if moved else Change()

    def revert(self, doc: Document) -> None:
        for o in doc.objects:
            if o.id in self.obj_ids:
                _translate(o, -self.dx, -self.dy)


class MoveObject(Command):
    def __init__(self, obj_id: str, dx: float, dy: float) -> None:
        self.obj_id = obj_id
        self.dx = dx
        self.dy = dy

    def apply(self, doc: Document) -> Change:
        for o in doc.objects:
            if o.id == self.obj_id:
                _translate(o, self.dx, self.dy)
                return Change(modified=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        for o in doc.objects:
            if o.id == self.obj_id:
                _translate(o, -self.dx, -self.dy)


class SetPathFrameCommand(Command):
    """Move/scale a path object to an absolute frame.

    Path objects derive x/y/width/height from their geometry, so
    inspector edits transform the nodes (translation and non-uniform
    scale around the frame origin; handles scale with their axis).
    """

    def __init__(self, obj_id: str, x: float, y: float, width: float, height: float) -> None:
        self.obj_id = obj_id
        self.x = x
        self.y = y
        self.width = width
        self.height = height
        self.before: dict[str, Any] | None = None
        self.after: dict[str, Any] | None = None

    def _find(self, doc: Document) -> VectorPath | None:
        for o in doc.objects:
            if o.id == self.obj_id:
                return o.path
        return None

    def apply(self, doc: Document) -> Change:
        path = self._find(doc)
        if path is None:
            return Change()
        if self.after is not None:
            self._restore(path, self.after)
            return Change(modified=[self.obj_id])
        self.before = path.to_json()
        x0, y0, x1, y1 = path_bounds(path)
        bx, by, bw, bh = x0, y0, x1 - x0, y1 - y0
        sx = self.width / bw if bw else 1.0
        sy = self.height / bh if bh else 1.0
        for c in path.contours:
            for n in c.nodes:
                n.x = self.x + (n.x - bx) * sx
                n.y = self.y + (n.y - by) * sy
                if n.in_handle is not None:
                    n.in_handle = (n.in_handle[0] * sx, n.in_handle[1] * sy)
                if n.out_handle is not None:
                    n.out_handle = (n.out_handle[0] * sx, n.out_handle[1] * sy)
        self.after = path.to_json()
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        path = self._find(doc)
        if path is None or self.before is None:
            return
        self._restore(path, self.before)

    @staticmethod
    def _restore(path: VectorPath, data: dict[str, Any]) -> None:
        path.contours = [Contour.from_json(c) for c in data["contours"]]
        path.fill_rule = FillRule(str(data["fillRule"]))


class SetObjectBoundsCommand(Command):
    """Set position and dimensions of an object (path, shape, artboard, image) atomically."""

    def __init__(
        self, obj_id: str, x: float, y: float, width: float, height: float
    ) -> None:
        self.obj_id = obj_id
        self.x = x
        self.y = y
        self.width = max(1.0, width)
        self.height = max(1.0, height)
        self.before: dict[str, Any] | None = None
        self._path_cmd: SetPathFrameCommand | None = None

    def apply(self, doc: Document) -> Change:
        o = None
        for obj in doc.objects:
            if obj.id == self.obj_id:
                o = obj
                break
        if o is None:
            return Change()
        if o.path is not None and o.symbol_id is None:
            if self._path_cmd is None:
                self._path_cmd = SetPathFrameCommand(
                    self.obj_id, self.x, self.y, self.width, self.height
                )
            return self._path_cmd.apply(doc)
        if self.before is None:
            self.before = {"x": o.x, "y": o.y, "width": o.width, "height": o.height}
        o.x = self.x
        o.y = self.y
        o.width = self.width
        o.height = self.height
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        if self._path_cmd is not None:
            self._path_cmd.revert(doc)
            return
        if self.before is None:
            return
        for obj in doc.objects:
            if obj.id == self.obj_id:
                obj.x = self.before["x"]
                obj.y = self.before["y"]
                obj.width = self.before["width"]
                obj.height = self.before["height"]
                break


class GroupObjects(Command):
    def __init__(self, obj_ids: list[str], name: str = "Group") -> None:
        self.obj_ids = list(obj_ids)
        self.name = name
        self.group: SceneObject | None = None
        self.index = 0

    def apply(self, doc: Document) -> Change:
        wanted = set(self.obj_ids)
        children = [o for o in doc.objects if o.id in wanted]
        if not children:
            return Change()
        self.index = min(doc.objects.index(c) for c in children)
        if self.group is None:
            self.group = SceneObject(type="group", name=self.name, children=children)
        else:
            self.group.children = children
        group = self.group
        group.x, group.y, group.width, group.height = group.bounds()
        for c in children:
            doc.objects.remove(c)
        doc.objects.insert(self.index, group)
        return Change(added=[group.id], removed=[c.id for c in children])

    def revert(self, doc: Document) -> None:
        if self.group is None:
            return
        doc.objects = [o for o in doc.objects if o.id != self.group.id]
        doc.objects[self.index:self.index] = self.group.children


class Ungroup(Command):
    def __init__(self, group_id: str) -> None:
        self.group_id = group_id
        self.index = 0
        self.group: SceneObject | None = None

    def apply(self, doc: Document) -> Change:
        for i, o in enumerate(doc.objects):
            if o.id == self.group_id:
                self.index = i
                self.group = o
                doc.objects[i:i + 1] = list(o.children)
                return Change(added=[c.id for c in o.children], removed=[o.id])
        return Change()

    def revert(self, doc: Document) -> None:
        if self.group is None:
            return
        child_ids = {c.id for c in self.group.children}
        doc.objects = [o for o in doc.objects if o.id not in child_ids]
        doc.objects.insert(self.index, self.group)


class SetProperty(Command):
    def __init__(self, obj_id: str, name: str, value: Any) -> None:
        self.obj_id = obj_id
        self.name = name
        self.value = value
        self.previous: Any = None

    def _find(self, objects: list[SceneObject]) -> SceneObject | None:
        for o in objects:
            if o.id == self.obj_id:
                return o
            found = self._find(o.children)
            if found is not None:
                return found
        return None

    def apply(self, doc: Document) -> Change:
        target = self._find(doc.objects)
        if target is not None:
            self.previous = getattr(target, self.name)
            setattr(target, self.name, self.value)
            return Change(modified=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        target = self._find(doc.objects)
        if target is not None:
            setattr(target, self.name, self.previous)


def _find_parent_list_and_index(
    objects: list[SceneObject], obj_id: str
) -> tuple[list[SceneObject], int] | None:
    for idx, obj in enumerate(objects):
        if obj.id == obj_id:
            return objects, idx
        res = _find_parent_list_and_index(obj.children, obj_id)
        if res is not None:
            return res
    return None


class ReorderObjectCommand(Command):
    """Reorder an object within its sibling list (doc.objects or group.children).

    Supported directions: 'front', 'back', 'forward', 'backward'
    """

    def __init__(self, obj_id: str, direction: str) -> None:
        self.obj_id = obj_id
        self.direction = direction
        self.old_index: int | None = None
        self.new_index: int | None = None

    def apply(self, doc: Document) -> Change:
        res = _find_parent_list_and_index(doc.objects, self.obj_id)
        if res is None:
            return Change()
        lst, idx = res
        self.old_index = idx
        n = len(lst)
        if self.direction == "front":
            target_idx = n - 1
        elif self.direction == "back":
            target_idx = 0
        elif self.direction == "forward":
            target_idx = min(n - 1, idx + 1)
        elif self.direction == "backward":
            target_idx = max(0, idx - 1)
        else:
            return Change()

        if target_idx == idx:
            return Change()

        self.new_index = target_idx
        item = lst.pop(idx)
        lst.insert(target_idx, item)
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        if self.old_index is None or self.new_index is None:
            return
        res = _find_parent_list_and_index(doc.objects, self.obj_id)
        if res is None:
            return
        lst, current_idx = res
        item = lst.pop(current_idx)
        lst.insert(self.old_index, item)


class PathCommand(Command):
    """Snapshot-based path topology command.

    Stores before/after canonical JSON so redo reproduces identical node and
    contour IDs (stable IDs across undo/redo per 09.6.2).
    """

    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.before: dict[str, Any] | None = None
        self.after: dict[str, Any] | None = None

    def _find_path(self, doc: Document) -> VectorPath | None:
        for o in doc.objects:
            if o.id == self.obj_id:
                return o.path
        return None

    def apply(self, doc: Document) -> Change:
        path = self._find_path(doc)
        if path is None:
            return Change()
        if self.after is not None:
            self._restore(path, self.after)
            return Change(modified=[self.obj_id])
        self.before = path.to_json()
        self._mutate(path)
        self.after = path.to_json()
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        path = self._find_path(doc)
        if path is None or self.before is None:
            return
        self._restore(path, self.before)

    def _restore(self, path: VectorPath, data: dict[str, Any]) -> None:
        path.contours = [Contour.from_json(c) for c in data["contours"]]
        path.fill_rule = FillRule(str(data["fillRule"]))

    def _mutate(self, path: VectorPath) -> None:  # pragma: no cover - interface
        raise NotImplementedError


class InsertNodeCommand(PathCommand):
    def __init__(self, obj_id: str, contour_index: int, segment_index: int, t: float) -> None:
        super().__init__(obj_id)
        self.contour_index = contour_index
        self.segment_index = segment_index
        self.t = t

    def _mutate(self, path: VectorPath) -> None:
        insert_node(path.contours[self.contour_index], self.segment_index, self.t)


class DeleteNodeCommand(PathCommand):
    def __init__(self, obj_id: str, contour_index: int, node_index: int) -> None:
        super().__init__(obj_id)
        self.contour_index = contour_index
        self.node_index = node_index

    def _mutate(self, path: VectorPath) -> None:
        delete_node(path.contours[self.contour_index], self.node_index)


class ReverseContourCommand(PathCommand):
    def __init__(self, obj_id: str, contour_index: int) -> None:
        super().__init__(obj_id)
        self.contour_index = contour_index

    def _mutate(self, path: VectorPath) -> None:
        reverse_contour(path.contours[self.contour_index])


class BreakContourCommand(PathCommand):
    def __init__(self, obj_id: str, contour_index: int, node_index: int) -> None:
        super().__init__(obj_id)
        self.contour_index = contour_index
        self.node_index = node_index

    def _mutate(self, path: VectorPath) -> None:
        contour = path.contours[self.contour_index]
        extra = break_contour(contour, self.node_index)
        if extra is not None:
            path.contours.insert(self.contour_index + 1, extra)


class JoinContoursCommand(PathCommand):
    def __init__(self, obj_id: str, contour_a: int, contour_b: int) -> None:
        super().__init__(obj_id)
        self.contour_a = contour_a
        self.contour_b = contour_b

    def _mutate(self, path: VectorPath) -> None:
        a = path.contours[self.contour_a]
        b = path.contours[self.contour_b]
        if not join_contours(a, b):
            raise ValueError("contour endpoints do not match")
        path.contours.remove(b)


class SetNodePositionCommand(PathCommand):
    def __init__(
        self, obj_id: str, contour_index: int, node_index: int, x: float, y: float
    ) -> None:
        super().__init__(obj_id)
        self.contour_index = contour_index
        self.node_index = node_index
        self.x = x
        self.y = y

    def _mutate(self, path: VectorPath) -> None:
        set_node_position(path.contours[self.contour_index], self.node_index, self.x, self.y)


def _copy_appearance(target: SceneObject, source: SceneObject) -> None:
    target.name = source.name
    target.fill = source.fill
    target.gradient = list(source.gradient)
    target.stroke = source.stroke
    target.stroke_width = source.stroke_width
    target.stroke_cap = source.stroke_cap
    target.stroke_join = source.stroke_join
    target.stroke_dash = list(source.stroke_dash)
    target.opacity = source.opacity
    target.blend_mode = source.blend_mode
    target.visible = source.visible


class BooleanCommand(Command):
    """Baked boolean over ordered operands (bottom-to-top z-order).

    The result set is computed before any mutation, so a typed
    BooleanError never leaves the document partially mutated (09.6.3).
    Style policy (09.6.4): Union/Intersect/XOR from the frontmost
    operand, Subtract from the minuend.
    """

    def __init__(self, obj_ids: list[str], op: str, fill_rule: str = "nonzero") -> None:
        self.obj_ids = list(obj_ids)
        self.op = op
        self.fill_rule = fill_rule
        self.index = 0
        self.before: list[dict[str, Any]] = []
        self.after: list[dict[str, Any]] = []
        self.computed = False

    def _operands(self, doc: Document) -> list[SceneObject] | None:
        wanted = set(self.obj_ids)
        objs = [o for o in doc.objects if o.id in wanted]
        if len(objs) != len(self.obj_ids):
            return None
        for o in objs:
            if o.path is None:
                raise BooleanError("boolean operands must be path objects", self.obj_ids)
        return objs

    def apply(self, doc: Document) -> Change:
        if self.computed:
            self._restore(doc, self.after)
            return Change(
                added=[s["id"] for s in self.after], removed=list(self.obj_ids)
            )
        objs = self._operands(doc)
        if objs is None:
            return Change()
        self.index = min(doc.objects.index(o) for o in objs)
        self.before = [o.to_json() for o in objs]
        rule = FillRule(self.fill_rule)
        results = boolean_paths(
            [(o.path, o.id) for o in objs if o.path is not None],
            BooleanOp(self.op),
            rule,
        )
        style = objs[0] if self.op == "subtract" else objs[-1]
        built: list[SceneObject] = []
        for r in results:
            scene = SceneObject(type="path", name=style.name, path=r.path)
            _copy_appearance(scene, style)
            bx, by, bw, bh = scene.bounds()
            scene.x, scene.y, scene.width, scene.height = bx, by, bw, bh
            built.append(scene)
        for o in objs:
            doc.objects.remove(o)
        doc.objects[self.index:self.index] = built
        self.after = [o.to_json() for o in built]
        self.computed = True
        return Change(added=[o.id for o in built], removed=list(self.obj_ids))

    def revert(self, doc: Document) -> None:
        self._restore(doc, self.before)

    def _restore(self, doc: Document, snapshots: list[dict[str, Any]]) -> None:
        ids = {s["id"] for s in snapshots} | set(self.obj_ids)
        if self.computed:
            ids |= {s["id"] for s in self.after}
        doc.objects = [o for o in doc.objects if o.id not in ids]
        doc.objects[self.index:self.index] = [
            SceneObject.from_json(s) for s in snapshots
        ]


class CompoundPathCommand(Command):
    """Merge operand contours into one compound path (no geometry change)."""

    def __init__(self, obj_ids: list[str], fill_rule: str = "nonzero") -> None:
        self.obj_ids = list(obj_ids)
        self.fill_rule = fill_rule
        self.index = 0
        self.before: list[dict[str, Any]] = []
        self.after: list[dict[str, Any]] = []
        self.computed = False

    def apply(self, doc: Document) -> Change:
        if self.computed:
            self._restore(doc, self.after)
            return Change(
                added=[s["id"] for s in self.after], removed=list(self.obj_ids)
            )
        wanted = set(self.obj_ids)
        objs = [o for o in doc.objects if o.id in wanted]
        if len(objs) != len(self.obj_ids):
            return Change()
        for o in objs:
            if o.path is None:
                raise BooleanError("compound path requires path objects", self.obj_ids)
        self.index = min(doc.objects.index(o) for o in objs)
        self.before = [o.to_json() for o in objs]
        style = objs[-1]
        merged = compound_paths(
            [(o.path, o.id) for o in objs if o.path is not None],
            FillRule(self.fill_rule),
        )
        scene = SceneObject(type="path", name=style.name, path=merged)
        _copy_appearance(scene, style)
        bx, by, bw, bh = scene.bounds()
        scene.x, scene.y, scene.width, scene.height = bx, by, bw, bh
        for o in objs:
            doc.objects.remove(o)
        doc.objects[self.index:self.index] = [scene]
        self.after = [scene.to_json()]
        self.computed = True
        return Change(added=[scene.id], removed=list(self.obj_ids))

    def revert(self, doc: Document) -> None:
        self._restore(doc, self.before)

    def _restore(self, doc: Document, snapshots: list[dict[str, Any]]) -> None:
        ids = {s["id"] for s in snapshots} | set(self.obj_ids)
        if self.computed:
            ids |= {s["id"] for s in self.after}
        doc.objects = [o for o in doc.objects if o.id not in ids]
        doc.objects[self.index:self.index] = [
            SceneObject.from_json(s) for s in snapshots
        ]


class LiveBooleanCommand(Command):
    """Create a live boolean node over ordered operand references.

    Operands stay in the document and remain editable; the node
    evaluates the same BooleanEngine used by the baked commands.
    """

    def __init__(self, obj_ids: list[str], op: str) -> None:
        self.obj_ids = list(obj_ids)
        self.op = op
        self.live_id: str | None = None
        self.snapshot: dict[str, Any] | None = None

    def apply(self, doc: Document) -> Change:
        if self.live_id is not None and self.snapshot is not None:
            scene = SceneObject.from_json(self.snapshot)
            doc.objects.append(scene)
            return Change(added=[scene.id])
        wanted = set(self.obj_ids)
        objs = [o for o in doc.objects if o.id in wanted]
        if len(objs) != len(self.obj_ids):
            return Change()
        for o in objs:
            if o.path is None:
                raise BooleanError("live boolean requires path operands", self.obj_ids)
        rule = FillRule(objs[-1].path.fill_rule.value if objs[-1].path else "nonzero")
        results = boolean_paths(
            [(o.path, o.id) for o in objs if o.path is not None],
            BooleanOp(self.op),
            rule,
        )
        if not results:
            raise BooleanError("live boolean produced no result", self.obj_ids)
        scene = SceneObject(type="path", name=f"Live {self.op}", path=results[0].path)
        _copy_appearance(scene, objs[-1])
        scene.live = LiveBoolean(op=self.op, source_ids=list(self.obj_ids))
        bx, by, bw, bh = scene.bounds()
        scene.x, scene.y, scene.width, scene.height = bx, by, bw, bh
        doc.objects.append(scene)
        self.live_id = scene.id
        self.snapshot = scene.to_json()
        return Change(added=[scene.id])

    def revert(self, doc: Document) -> None:
        if self.live_id is None:
            return
        doc.objects = [o for o in doc.objects if o.id != self.live_id]


class BakeLiveCommand(Command):
    """Replace a live boolean node with its evaluated static path."""

    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.before: dict[str, Any] | None = None
        self.after: dict[str, Any] | None = None

    def apply(self, doc: Document) -> Change:
        if self.after is not None:
            self._restore(doc, self.after)
            return Change(modified=[self.obj_id])
        for o in doc.objects:
            if o.id == self.obj_id:
                self.before = o.to_json()
                o.live = None
                self.after = o.to_json()
                return Change(modified=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        if self.before is not None:
            self._restore(doc, self.before)

    def _restore(self, doc: Document, data: dict[str, Any]) -> None:
        for i, o in enumerate(doc.objects):
            if o.id == self.obj_id:
                doc.objects[i] = SceneObject.from_json(data)
                return


class ShapeBuilderCommitCommand(Command):
    """Commit a Shape Builder face selection: remove operands, add boundary."""

    def __init__(
        self,
        obj_ids: list[str],
        polygons: list[Polygon],
        style_source_id: str,
    ) -> None:
        self.obj_ids = list(obj_ids)
        self.polygons: list[Polygon] = [list(poly) for poly in polygons]
        self.style_source_id = style_source_id
        self.index = 0
        self.before: list[dict[str, Any]] = []
        self.after: list[dict[str, Any]] = []
        self.computed = False

    def apply(self, doc: Document) -> Change:
        if self.computed:
            self._restore(doc, self.after)
            return Change(
                added=[s["id"] for s in self.after], removed=list(self.obj_ids)
            )
        wanted = set(self.obj_ids)
        objs = [o for o in doc.objects if o.id in wanted]
        if len(objs) != len(self.obj_ids):
            return Change()
        self.index = min(doc.objects.index(o) for o in objs)
        self.before = [o.to_json() for o in objs]
        path = rebuild_path(self.polygons)
        style = next((o for o in objs if o.id == self.style_source_id), objs[-1])
        scene = SceneObject(type="path", name=style.name, path=path)
        _copy_appearance(scene, style)
        bx, by, bw, bh = scene.bounds()
        scene.x, scene.y, scene.width, scene.height = bx, by, bw, bh
        for o in objs:
            doc.objects.remove(o)
        doc.objects[self.index:self.index] = [scene]
        self.after = [scene.to_json()]
        self.computed = True
        return Change(added=[scene.id], removed=list(self.obj_ids))

    def revert(self, doc: Document) -> None:
        self._restore(doc, self.before)

    def _restore(self, doc: Document, snapshots: list[dict[str, Any]]) -> None:
        ids = {s["id"] for s in snapshots} | set(self.obj_ids)
        if self.computed:
            ids |= {s["id"] for s in self.after}
        doc.objects = [o for o in doc.objects if o.id not in ids]
        doc.objects[self.index:self.index] = [
            SceneObject.from_json(s) for s in snapshots
        ]


def evaluate_live(doc: Document) -> list[str]:
    """Re-evaluate every live boolean node from its current operands."""
    evaluated: list[str] = []
    by_id = {o.id: o for o in doc.objects}
    for o in doc.objects:
        if o.live is None:
            continue
        operands: list[tuple[VectorPath, str]] = []
        valid = True
        for sid in o.live.source_ids:
            src = by_id.get(sid)
            if src is None or src.path is None:
                valid = False
                break
            operands.append((src.path, src.id))
        if not valid or len(operands) < 2:
            continue
        rule = FillRule(o.path.fill_rule.value) if o.path is not None else FillRule.NONZERO
        try:
            results = boolean_paths(operands, BooleanOp(o.live.op), rule)
        except BooleanError:
            continue
        if results and o.path is not None:
            o.path = results[0].path
            evaluated.append(o.id)
    return evaluated


class SetOverrideCommand(Command):
    """Set a local override on a symbol instance or styled object.
    Overrides stay local: they survive definition/style updates
    and are never pushed back to the resource."""

    def __init__(self, obj_id: str, key: str, value: Any) -> None:
        self.obj_id = obj_id
        self.key = key
        self.value = value
        self.prev: Any = None
        self.had = False

    def apply(self, doc: Document) -> Change:
        for o in doc.objects:
            if o.id == self.obj_id:
                self.had = self.key in o.overrides
                self.prev = o.overrides.get(self.key)
                o.overrides[self.key] = self.value
                return Change(modified=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        for o in doc.objects:
            if o.id == self.obj_id:
                if self.had:
                    o.overrides[self.key] = self.prev
                else:
                    o.overrides.pop(self.key, None)
                return


class CreateSymbolCommand(Command):
    """Turn an object into a symbol instance; the definition is
    stored in the document library and survives save/reopen."""

    def __init__(self, obj_id: str, name: str) -> None:
        self.obj_id = obj_id
        self.name = name
        self.definition: SymbolDefinition | None = None

    def apply(self, doc: Document) -> Change:
        for o in doc.objects:
            if o.id == self.obj_id:
                if o.symbol_id is not None:
                    return Change()
                if self.definition is None:
                    source = deepcopy(o)
                    source.symbol_id = None
                    source.style_id = None
                    source.overrides = {}
                    self.definition = SymbolDefinition(
                        id=new_id(), name=self.name, source=source
                    )
                document_library(doc).symbols[self.definition.id] = (
                    self.definition
                )
                o.symbol_id = self.definition.id
                o.overrides = {}
                return Change(modified=[self.obj_id])
        return Change()

    def revert(self, doc: Document) -> None:
        if self.definition is None:
            return
        document_library(doc).symbols.pop(self.definition.id, None)
        for o in doc.objects:
            if o.id == self.obj_id:
                o.symbol_id = None
                o.overrides = {}
                return


class PlaceSymbolCommand(Command):
    """Create a new instance of a symbol definition."""

    def __init__(
        self,
        symbol_id: str,
        name: str = "",
        x: float = 0.0,
        y: float = 0.0,
    ) -> None:
        self.symbol_id = symbol_id
        self.name = name
        self.x = x
        self.y = y
        self.instance: SceneObject | None = None

    def apply(self, doc: Document) -> Change:
        if self.instance is not None:
            doc.objects.append(self.instance)
            return Change(added=[self.instance.id])
        definition = None
        for library in doc.libraries:
            if self.symbol_id in library.symbols:
                definition = library.symbols[self.symbol_id]
                break
        if definition is None:
            return Change()
        instance = deepcopy(definition.source)
        instance.id = new_id()
        instance.name = self.name or definition.name
        instance.x = self.x
        instance.y = self.y
        instance.symbol_id = self.symbol_id
        instance.overrides = {}
        self.instance = instance
        doc.objects.append(instance)
        return Change(added=[instance.id])

    def revert(self, doc: Document) -> None:
        if self.instance is None:
            return
        doc.objects = [o for o in doc.objects if o.id != self.instance.id]


class UpdateSymbolCommand(Command):
    """Push an instance's resolved state into its definition.
    Every instance picks up the new source (propagation); each
    keeps its own local overrides."""

    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.symbol_id: str | None = None
        self.before: SceneObject | None = None
        self.after: SceneObject | None = None
        self.prev_overrides: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        obj = next((o for o in doc.objects if o.id == self.obj_id), None)
        if obj is None or obj.symbol_id is None:
            return Change()
        definition = None
        for library in doc.libraries:
            if obj.symbol_id in library.symbols:
                definition = library.symbols[obj.symbol_id]
                break
        if definition is None:
            return Change()
        if self.after is not None:
            definition.source = deepcopy(self.after)
            obj.overrides.clear()
            return Change(modified=[self.obj_id])
        self.symbol_id = obj.symbol_id
        self.before = deepcopy(definition.source)
        self.prev_overrides = dict(obj.overrides)
        resolved = resolve_object_local(doc, obj)
        new_source = deepcopy(resolved)
        new_source.id = definition.source.id
        new_source.name = definition.source.name
        new_source.symbol_id = None
        new_source.overrides = {}
        new_source.style_id = obj.style_id
        # position is instance-level: the master keeps its own
        new_source.x = definition.source.x
        new_source.y = definition.source.y
        definition.source = new_source
        self.after = deepcopy(new_source)
        obj.overrides.clear()
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        if self.before is None or self.symbol_id is None:
            return
        for library in doc.libraries:
            if self.symbol_id in library.symbols:
                library.symbols[self.symbol_id].source = deepcopy(self.before)
                break
        obj = next((o for o in doc.objects if o.id == self.obj_id), None)
        if obj is not None:
            obj.overrides = dict(self.prev_overrides)


class DetachSymbolCommand(Command):
    """Bake an instance's resolved state into a plain object."""

    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.before: dict[str, Any] = {}
        self.after: dict[str, Any] = {}
        self.computed = False

    def apply(self, doc: Document) -> Change:
        obj = next((o for o in doc.objects if o.id == self.obj_id), None)
        if obj is None or obj.symbol_id is None:
            return Change()
        if self.computed:
            self._restore(doc, self.after)
            return Change(modified=[self.obj_id])
        self.before = obj.to_json()
        resolved = resolve_object_local(doc, obj)
        obj.type = resolved.type
        obj.x, obj.y = resolved.x, resolved.y
        obj.width, obj.height = resolved.width, resolved.height
        obj.fill = resolved.fill
        obj.gradient = list(resolved.gradient)
        obj.stroke = resolved.stroke
        obj.stroke_width = resolved.stroke_width
        obj.visible = resolved.visible
        obj.name = resolved.name
        obj.children = deepcopy(resolved.children)
        obj.path = deepcopy(resolved.path)
        obj.symbol_id = None
        obj.overrides = {}
        self.after = obj.to_json()
        self.computed = True
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        self._restore(doc, self.before)

    def _restore(self, doc: Document, snapshot: dict[str, Any]) -> None:
        restored = SceneObject.from_json(snapshot)
        for o in doc.objects:
            if o.id == self.obj_id:
                o.type = restored.type
                o.x, o.y = restored.x, restored.y
                o.width, o.height = restored.width, restored.height
                o.fill = restored.fill
                o.gradient = list(restored.gradient)
                o.stroke = restored.stroke
                o.stroke_width = restored.stroke_width
                o.visible = restored.visible
                o.name = restored.name
                o.children = deepcopy(restored.children)
                o.path = deepcopy(restored.path)
                o.symbol_id = restored.symbol_id
                o.style_id = restored.style_id
                o.overrides = dict(restored.overrides)
                return


class CreateStyleCommand(Command):
    """Create a style. Object styles derive from an object's
    appearance; character/paragraph styles are named property
    shells consumed by the text wave (G033+)."""

    def __init__(
        self,
        name: str,
        kind: str = "object",
        obj_id: str = "",
        properties: dict[str, Any] | None = None,
    ) -> None:
        self.name = name
        self.kind = kind
        self.obj_id = obj_id
        self.properties = dict(properties or {})
        self.style: Style | None = None

    def apply(self, doc: Document) -> Change:
        if self.style is None:
            if self.kind == "object":
                obj = next(
                    (o for o in doc.objects if o.id == self.obj_id), None
                )
                if obj is None:
                    return Change()
                self.style = ObjectStyle(
                    id=new_id(),
                    name=self.name,
                    fill=obj.fill,
                    gradient=list(obj.gradient),
                    stroke=obj.stroke,
                    stroke_width=obj.stroke_width,
                    visible=obj.visible,
                )
            else:
                self.style = TextStyle(
                    id=new_id(),
                    name=self.name,
                    kind=self.kind,
                    properties=dict(self.properties),
                )
        document_library(doc).styles[self.style.id] = self.style
        return Change()

    def revert(self, doc: Document) -> None:
        if self.style is None:
            return
        document_library(doc).styles.pop(self.style.id, None)


class ApplyStyleCommand(Command):
    """Reference a style from objects; appearance resolves
    dynamically, so style edits propagate to all objects."""

    def __init__(self, obj_ids: list[str], style_id: str) -> None:
        self.obj_ids = list(obj_ids)
        self.style_id = style_id
        self.before: dict[str, str | None] = {}
        self.computed = False

    def apply(self, doc: Document) -> Change:
        wanted = set(self.obj_ids)
        targets = [o for o in doc.objects if o.id in wanted]
        if not targets:
            return Change()
        if not self.computed:
            self.before = {o.id: o.style_id for o in targets}
            self.computed = True
        for o in targets:
            o.style_id = self.style_id
        return Change(modified=list(self.before))

    def revert(self, doc: Document) -> None:
        for o in doc.objects:
            if o.id in self.before:
                o.style_id = self.before[o.id]


class DetachStyleCommand(Command):
    """Bake an object's resolved appearance, clearing the
    style reference."""

    def __init__(self, obj_id: str) -> None:
        self.obj_id = obj_id
        self.before: dict[str, Any] = {}
        self.after: dict[str, Any] = {}
        self.computed = False

    def apply(self, doc: Document) -> Change:
        obj = next((o for o in doc.objects if o.id == self.obj_id), None)
        if obj is None or obj.style_id is None:
            return Change()
        if self.computed:
            self._restore(doc, self.after)
            return Change(modified=[self.obj_id])
        self.before = obj.to_json()
        resolved = resolve_object_local(doc, obj)
        obj.fill = resolved.fill
        obj.gradient = list(resolved.gradient)
        obj.stroke = resolved.stroke
        obj.stroke_width = resolved.stroke_width
        obj.visible = resolved.visible
        obj.style_id = None
        obj.overrides = {}
        self.after = obj.to_json()
        self.computed = True
        return Change(modified=[self.obj_id])

    def revert(self, doc: Document) -> None:
        self._restore(doc, self.before)

    def _restore(self, doc: Document, snapshot: dict[str, Any]) -> None:
        restored = SceneObject.from_json(snapshot)
        for o in doc.objects:
            if o.id == self.obj_id:
                o.type = restored.type
                o.x, o.y = restored.x, restored.y
                o.width, o.height = restored.width, restored.height
                o.fill = restored.fill
                o.gradient = list(restored.gradient)
                o.stroke = restored.stroke
                o.stroke_width = restored.stroke_width
                o.visible = restored.visible
                o.name = restored.name
                o.children = deepcopy(restored.children)
                o.path = deepcopy(restored.path)
                o.symbol_id = restored.symbol_id
                o.style_id = restored.style_id
                o.overrides = dict(restored.overrides)
                return


class UpdateStyleCommand(Command):
    """Push an object's appearance into its style; every object
    referencing the style inherits the change."""

    def __init__(self, style_id: str, obj_id: str) -> None:
        self.style_id = style_id
        self.obj_id = obj_id
        self.before: dict[str, Any] = {}
        self.computed = False

    def apply(self, doc: Document) -> Change:
        style = None
        for library in doc.libraries:
            if self.style_id in library.styles:
                style = library.styles[self.style_id]
                break
        if not isinstance(style, ObjectStyle):
            return Change()
        obj = next((o for o in doc.objects if o.id == self.obj_id), None)
        if obj is None:
            return Change()
        if not self.computed:
            self.before = {
                "fill": style.fill,
                "gradient": list(style.gradient),
                "stroke": style.stroke,
                "stroke_width": style.stroke_width,
                "visible": style.visible,
            }
            self.computed = True
        style.fill = obj.fill
        style.gradient = list(obj.gradient)
        style.stroke = obj.stroke
        style.stroke_width = obj.stroke_width
        style.visible = obj.visible
        return Change()

    def revert(self, doc: Document) -> None:
        if not self.before:
            return
        for library in doc.libraries:
            style = library.styles.get(self.style_id)
            if isinstance(style, ObjectStyle):
                style.fill = str(self.before["fill"])
                style.gradient = list(self.before["gradient"])
                style.stroke = str(self.before["stroke"])
                style.stroke_width = float(self.before["stroke_width"])
                style.visible = bool(self.before["visible"])
                return


class History:
    def __init__(self) -> None:
        self.undo_stack: list[Command] = []
        self.redo_stack: list[Command] = []

    def execute(self, cmd: Command, doc: Document) -> Change:
        change = cmd.apply(doc)
        self.undo_stack.append(cmd)
        self.redo_stack.clear()
        return change

    def undo(self, doc: Document) -> None:
        if not self.undo_stack:
            return
        cmd = self.undo_stack.pop()
        cmd.revert(doc)
        self.redo_stack.append(cmd)

    def redo(self, doc: Document) -> None:
        if not self.redo_stack:
            return
        cmd = self.redo_stack.pop()
        cmd.apply(doc)
        self.undo_stack.append(cmd)

    def can_undo(self) -> bool:
        return bool(self.undo_stack)

    def can_redo(self) -> bool:
        return bool(self.redo_stack)


class Snap:
    def __init__(self, grid: float = 8.0) -> None:
        self.grid = grid

    def snap(self, v: float) -> float:
        return round(v / self.grid) * self.grid if self.grid > 0 else v
