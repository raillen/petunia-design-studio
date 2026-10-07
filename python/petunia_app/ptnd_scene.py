from __future__ import annotations

from pathlib import Path
from typing import Any, cast

from .model import AssetLibrary, Document, SceneObject, new_id
from .ptnd import PtndError, read_package, write_package


def save_ptnd(path: Path, doc: Document) -> None:
    surface_id = new_id()

    def entry_for(o: SceneObject) -> dict[str, Any]:
        entry = o.to_json()
        bx, by, bw, bh = o.bounds()
        entry["frame"] = {"x": bx, "y": by, "width": bw, "height": bh}
        entry["children"] = [entry_for(c) for c in o.children]
        return entry

    objects = [entry_for(o) for o in doc.objects]
    document: dict[str, Any] = {
        "version": 1,
        "documentId": doc.document_id,
        "name": doc.name,
        "libraries": [lib.to_json() for lib in doc.libraries],
        "stories": [s.to_json() for s in doc.stories.values()],
        "surfaces": [
            {
                "id": surface_id,
                "name": "Page 1",
                "canvas": {"width": doc.width, "height": doc.height},
                "objects": objects,
            }
        ],
    }
    manifest: dict[str, Any] = {
        "formatId": "ptnd.document",
        "containerVersion": 1,
        "schemaVersion": 1,
        "minimumReaderVersion": "1.0",
        "writer": {"appVersion": "0.1.0"},
        "documentId": doc.document_id,
        "requiredCapabilities": [],
        "resources": [],
        "extensions": [],
        "interchange": [],
    }
    if len(doc.document_id) != 36:
        raise PtndError("documentId must be a uuid")
    write_package(path, manifest, document)


def open_ptnd(path: Path) -> Document:
    manifest, document = read_package(path)
    surfaces = cast(list[dict[str, Any]], document.get("surfaces"))
    if len(surfaces) != 1:
        raise PtndError("expected exactly one surface")
    surface = surfaces[0]
    canvas = cast(dict[str, Any], surface.get("canvas", {}))
    objects_raw = cast(list[dict[str, Any]], surface.get("objects", []))
    objs: list[SceneObject] = []
    for raw in objects_raw:
        frame = raw.get("frame", {})
        merged = dict(raw)
        merged["x"] = float(frame.get("x", 0.0))
        merged["y"] = float(frame.get("y", 0.0))
        merged["width"] = float(frame.get("width", 100.0))
        merged["height"] = float(frame.get("height", 100.0))
        objs.append(SceneObject.from_json(merged))
    stories_raw = cast(list[dict[str, Any]], document.get("stories", []))
    from .text import TextStory

    stories_dict = {
        st.id: st for st in (TextStory.from_json(s) for s in stories_raw)
    }
    return Document(
        name=str(document.get("name", "Untitled")),
        width=float(canvas.get("width", 1280.0)),
        height=float(canvas.get("height", 800.0)),
        document_id=str(manifest.get("documentId", new_id())),
        objects=objs,
        libraries=[
            AssetLibrary.from_json(lib)
            for lib in cast(list[dict[str, Any]], document.get("libraries", []))
        ],
        stories=stories_dict,
    )
