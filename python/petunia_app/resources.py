"""G032 — symbol/style/resource resolution and dependency reporting.

The resource dataclasses (``SymbolDefinition``, ``ObjectStyle``,
``TextStyle``, ``AssetLibrary``) live in :mod:`petunia_app.model`
next to the objects that reference them; this module holds the
resolution and audit logic that needs the full document context.

Resolution order for an object: symbol source (if any) -> style
appearance (if any) -> local overrides. Instance ``x``/``y`` always
win over the symbol source (instances carry their own transform).

Missing references are explicit: :func:`missing_references`
enumerates symbol/style ids that do not resolve in any loaded
library, and instances keep their last-known stored state as
fallback so a missing library never renders as blank geometry.
"""

from __future__ import annotations

from copy import deepcopy

from .model import (
    AssetLibrary,
    Document,
    ObjectStyle,
    SceneObject,
    Style,
    SymbolDefinition,
)

OVERRIDE_FIELDS = {
    "fill",
    "gradient",
    "stroke",
    "strokeWidth",
    "stroke_width",
    "cornerRadius",
    "corner_radius",
    "opacity",
    "visible",
    "width",
    "height",
    "name",
}


# ------------------------------------------------------------------
# lookup


def find_symbol(
    doc: Document, symbol_id: str, user_library: AssetLibrary | None = None
) -> SymbolDefinition | None:
    for library in doc.libraries:
        if symbol_id in library.symbols:
            return library.symbols[symbol_id]
    if user_library is not None and symbol_id in user_library.symbols:
        return user_library.symbols[symbol_id]
    return None


def find_style(
    doc: Document, style_id: str, user_library: AssetLibrary | None = None
) -> Style | None:
    for library in doc.libraries:
        if style_id in library.styles:
            return library.styles[style_id]
    if user_library is not None and style_id in user_library.styles:
        return user_library.styles[style_id]
    return None


# ------------------------------------------------------------------
# resolution


def resolve_object(
    doc: Document,
    obj: SceneObject,
    user_library: AssetLibrary | None = None,
) -> SceneObject:
    """Resolved view of an object: symbol source -> style -> overrides.

    Returns the object itself when it references no resources.
    """
    if obj.symbol_id is None and obj.style_id is None and not obj.overrides:
        return obj
    current = obj
    if obj.symbol_id is not None:
        definition = find_symbol(doc, obj.symbol_id, user_library)
        if definition is not None:
            base = deepcopy(definition.source)
            base.id = obj.id
            base.name = obj.name
            # instances carry their own transform
            dx = obj.x - definition.source.x
            dy = obj.y - definition.source.y
            base.x, base.y = obj.x, obj.y
            if base.path is not None and (dx != 0.0 or dy != 0.0):
                for contour in base.path.contours:
                    for node in contour.nodes:
                        node.x += dx
                        node.y += dy
            base.symbol_id = obj.symbol_id
            base.style_id = obj.style_id
            base.overrides = dict(obj.overrides)
            current = base
    if current.style_id is not None:
        style = find_style(doc, current.style_id, user_library)
        if isinstance(style, ObjectStyle):
            current.fill = style.fill
            current.gradient = list(style.gradient)
            current.stroke = style.stroke
            current.stroke_width = style.stroke_width
            current.visible = style.visible
    if current.overrides:
        for key, value in current.overrides.items():
            if key in OVERRIDE_FIELDS:
                attr = "stroke_width" if key == "strokeWidth" else key
                if hasattr(current, attr):
                    setattr(current, attr, value)
    return current


# ------------------------------------------------------------------
# explicit dependency reporting


def missing_references(
    doc: Document, user_library: AssetLibrary | None = None
) -> dict[str, list[str]]:
    """Symbol/style ids referenced by objects but unresolvable in any
    loaded library — the explicit missing-library signal."""
    missing_symbols: set[str] = set()
    missing_styles: set[str] = set()
    for obj in doc.objects:
        if obj.symbol_id is not None and find_symbol(
            doc, obj.symbol_id, user_library
        ) is None:
            missing_symbols.add(obj.symbol_id)
        if obj.style_id is not None and find_style(
            doc, obj.style_id, user_library
        ) is None:
            missing_styles.add(obj.style_id)
    return {
        "symbols": sorted(missing_symbols),
        "styles": sorted(missing_styles),
    }


def library_conflicts(
    doc: Document, user_library: AssetLibrary | None = None
) -> list[str]:
    """Asset names defined in both document and user scope. Document
    scope wins on lookup; the overlap is reported explicitly."""
    if user_library is None:
        return []
    document_names: set[str] = set()
    for library in doc.libraries:
        if library.scope == "document":
            document_names |= library.asset_names()
    return sorted(document_names & user_library.asset_names())
