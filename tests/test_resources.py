from __future__ import annotations

import json
from pathlib import Path
from typing import Any, cast

import jsonschema

from petunia_app.bridge import Bridge
from petunia_app.exporters import export_svg
from petunia_app.model import (
    AddObject,
    ApplyStyleCommand,
    AssetLibrary,
    CreateStyleCommand,
    CreateSymbolCommand,
    DetachStyleCommand,
    DetachSymbolCommand,
    Document,
    History,
    ObjectStyle,
    PlaceSymbolCommand,
    SceneObject,
    SetOverrideCommand,
    SymbolDefinition,
    TextStyle,
    UpdateStyleCommand,
    UpdateSymbolCommand,
    document_library,
)
from petunia_app.ptnd import read_package
from petunia_app.ptnd_scene import open_ptnd, save_ptnd
from petunia_app.raster import export_png
from petunia_app.resources import (
    find_style,
    find_symbol,
    library_conflicts,
    missing_references,
    resolve_object,
)


def test_symbol_lifecycle_and_undo() -> None:
    doc = Document(name="SymbolTest")
    history = History()

    rect = SceneObject(
        type="rectangle",
        name="Card",
        x=10.0,
        y=20.0,
        width=200.0,
        height=150.0,
        fill="#112233",
        stroke="#445566",
        stroke_width=3.0,
    )
    history.execute(AddObject(rect), doc)
    assert len(doc.objects) == 1

    # 1. Create symbol from object
    cmd_create = CreateSymbolCommand(rect.id, "CardSymbol")
    history.execute(cmd_create, doc)
    assert rect.symbol_id is not None
    sym_id = rect.symbol_id
    lib = document_library(doc)
    assert sym_id in lib.symbols
    assert lib.symbols[sym_id].name == "CardSymbol"
    assert lib.symbols[sym_id].source.fill == "#112233"

    # 2. Place new symbol instance
    cmd_place = PlaceSymbolCommand(sym_id, x=300.0, y=400.0)
    history.execute(cmd_place, doc)
    assert len(doc.objects) == 2
    inst = doc.objects[1]
    assert inst.symbol_id == sym_id
    assert inst.x == 300.0
    assert inst.y == 400.0

    # Resolved views
    res_inst = resolve_object(doc, inst)
    assert res_inst.fill == "#112233"
    assert res_inst.width == 200.0

    # 3. Local overrides on instance
    cmd_override = SetOverrideCommand(inst.id, "fill", "#ff0099")
    history.execute(cmd_override, doc)
    assert inst.overrides.get("fill") == "#ff0099"
    assert resolve_object(doc, inst).fill == "#ff0099"
    # Master remains unchanged
    assert lib.symbols[sym_id].source.fill == "#112233"

    # 4. Update symbol from an instance with local overrides
    cmd_override_rect = SetOverrideCommand(rect.id, "fill", "#00aa22")
    history.execute(cmd_override_rect, doc)
    cmd_update = UpdateSymbolCommand(rect.id)
    history.execute(cmd_update, doc)
    assert lib.symbols[sym_id].source.fill == "#00aa22"
    # Instance with override retains its local fill
    assert resolve_object(doc, inst).fill == "#ff0099"

    # 5. Detach symbol
    cmd_detach = DetachSymbolCommand(inst.id)
    history.execute(cmd_detach, doc)
    assert inst.symbol_id is None
    assert inst.fill == "#ff0099"

    # Undo detach
    history.undo(doc)
    assert inst.symbol_id == sym_id

    # Undo update
    history.undo(doc)
    assert lib.symbols[sym_id].source.fill == "#112233"

    # Undo rect override
    history.undo(doc)
    assert "fill" not in rect.overrides

    # Undo inst override
    history.undo(doc)
    assert "fill" not in inst.overrides

    # Undo place
    history.undo(doc)
    assert len(doc.objects) == 1

    # Undo create symbol
    history.undo(doc)
    assert rect.symbol_id is None
    assert sym_id not in lib.symbols


def test_style_lifecycle_and_undo() -> None:
    doc = Document(name="StyleTest")
    history = History()

    obj1 = SceneObject(
        type="rectangle",
        name="Button1",
        fill="#ff8800",
        stroke="#000000",
        stroke_width=2.5,
    )
    obj2 = SceneObject(
        type="ellipse",
        name="Button2",
        fill="#cccccc",
        stroke="#ffffff",
        stroke_width=1.0,
    )
    doc.objects.extend([obj1, obj2])

    # 1. Create object style from obj1
    cmd_style = CreateStyleCommand("BrandPrimary", "object", obj1.id)
    history.execute(cmd_style, doc)
    assert cmd_style.style is not None
    style_id = cmd_style.style.id
    lib = document_library(doc)
    assert style_id in lib.styles
    style = cast(ObjectStyle, lib.styles[style_id])
    assert style.fill == "#ff8800"

    # 2. Apply style to obj2
    cmd_apply = ApplyStyleCommand([obj2.id], style_id)
    history.execute(cmd_apply, doc)
    assert obj2.style_id == style_id
    res_obj2 = resolve_object(doc, obj2)
    assert res_obj2.fill == "#ff8800"
    assert res_obj2.stroke_width == 2.5

    # 3. Update style appearance from obj2 after modifying obj2
    obj2.fill = "#336699"
    cmd_update = UpdateStyleCommand(style_id, obj2.id)
    history.execute(cmd_update, doc)
    assert style.fill == "#336699"
    # obj1 resolves with updated style if applied
    obj1.style_id = style_id
    assert resolve_object(doc, obj1).fill == "#336699"

    # 4. Detach style
    cmd_detach = DetachStyleCommand(obj2.id)
    history.execute(cmd_detach, doc)
    assert obj2.style_id is None
    assert obj2.fill == "#336699"

    # 5. TextStyle creation (character & paragraph)
    cmd_text_char = CreateStyleCommand(
        "HeadlineChar", "character", properties={"fontFamily": "Inter", "fontSize": 24}
    )
    history.execute(cmd_text_char, doc)
    assert cmd_text_char.style is not None
    assert isinstance(cmd_text_char.style, TextStyle)
    assert cmd_text_char.style.kind == "character"
    assert cmd_text_char.style.properties["fontSize"] == 24

    # Undo flow
    history.undo(doc)  # undo text style
    assert cmd_text_char.style.id not in lib.styles

    history.undo(doc)  # undo detach
    assert obj2.style_id == style_id


def test_resolution_hierarchy() -> None:
    """Resolution precedence: symbol source -> style -> overrides."""
    doc = Document()
    lib = document_library(doc)

    # Master symbol definition
    source = SceneObject(type="rectangle", fill="#111111", stroke="#aaaaaa", stroke_width=1.0)
    sym = lib.symbols["sym1"] = SymbolDefinition(id="sym1", name="Master", source=source)

    # Style
    style = lib.styles["style1"] = ObjectStyle(
        id="style1", name="Primary", fill="#222222", stroke="#bbbbbb"
    )

    # Instance with both symbol and style, plus an override
    instance = SceneObject(
        type="rectangle",
        id="inst1",
        name="Inst",
        symbol_id=sym.id,
        style_id=style.id,
        overrides={"fill": "#333333"},  # override wins over style and symbol
    )
    doc.objects.append(instance)

    resolved = resolve_object(doc, instance)
    assert resolved.fill == "#333333"  # override won
    assert resolved.stroke == "#bbbbbb"  # style won over symbol
    assert resolved.stroke_width == 2.0  # style won over symbol source (1.0)


def test_missing_references_and_conflicts() -> None:
    doc = Document()
    doc.objects.append(SceneObject(symbol_id="nonexistent-sym", style_id="nonexistent-style"))
    missing = missing_references(doc)
    assert "nonexistent-sym" in missing["symbols"]
    assert "nonexistent-style" in missing["styles"]

    # Library conflicts
    doc_lib = document_library(doc)
    doc_lib.symbols["s1"] = SymbolDefinition(
        id="s1", name="SharedIcon", source=SceneObject(type="rectangle")
    )

    user_lib = AssetLibrary(id="user-lib", name="User Assets", scope="user")
    user_lib.symbols["u1"] = SymbolDefinition(
        id="u1", name="SharedIcon", source=SceneObject(type="rectangle")
    )
    user_lib.symbols["u2"] = SymbolDefinition(
        id="u2", name="UniqueIcon", source=SceneObject(type="rectangle")
    )

    conflicts = library_conflicts(doc, user_lib)
    assert conflicts == ["SharedIcon"]
    assert find_symbol(doc, "s1", user_lib) is not None
    assert find_style(doc, "missing", user_lib) is None


def test_ptnd_resources_schema_and_roundtrip(tmp_path: Path) -> None:
    pkg = tmp_path / "resources_test.ptnd"
    doc = Document(name="ResourcesCard", width=1080.0, height=1080.0)

    # Add object, make symbol, add style, place instance
    rect = SceneObject(
        type="rectangle",
        name="Badge",
        x=50.0,
        y=50.0,
        width=100.0,
        height=100.0,
        fill="#4488ff",
    )
    doc.objects.append(rect)
    cmd_sym = CreateSymbolCommand(rect.id, "BadgeSymbol")
    cmd_sym.apply(doc)

    cmd_style = CreateStyleCommand("BrandStyle", "object", rect.id)
    cmd_style.apply(doc)

    cmd_place = PlaceSymbolCommand(
        cmd_sym.definition.id if cmd_sym.definition else "", x=250.0, y=250.0
    )
    cmd_place.apply(doc)
    assert cmd_place.instance is not None
    cmd_place.instance.overrides["fill"] = "#ff4488"

    # Save to PTND
    save_ptnd(pkg, doc)

    # 1. Validate schema conformance of document.json inside the package
    _, doc_json = read_package(pkg)
    schema_path = Path("schemas/ptnd/v1/document.schema.json")
    schema = json.loads(schema_path.read_text())
    jsonschema.validate(doc_json, schema)

    # 2. Reopen and verify data parity
    reopened = open_ptnd(pkg)
    assert reopened.name == "ResourcesCard"
    assert len(reopened.libraries) >= 1
    re_lib = reopened.libraries[0]
    assert len(re_lib.symbols) == 1
    assert len(re_lib.styles) == 1
    assert len(reopened.objects) == 2

    re_inst = reopened.objects[1]
    assert re_inst.symbol_id == (cmd_sym.definition.id if cmd_sym.definition else "")
    assert re_inst.overrides.get("fill") == "#ff4488"
    assert resolve_object(reopened, re_inst).fill == "#ff4488"


def test_svg_and_png_export_with_resources(tmp_path: Path) -> None:
    doc = Document(name="ExportResources", width=400.0, height=300.0)
    rect = SceneObject(
        type="rectangle",
        name="Base",
        x=10.0,
        y=10.0,
        width=80.0,
        height=80.0,
        fill="#123456",
    )
    doc.objects.append(rect)
    cmd_sym = CreateSymbolCommand(rect.id, "Tile")
    cmd_sym.apply(doc)
    cmd_place = PlaceSymbolCommand(
        cmd_sym.definition.id if cmd_sym.definition else "", x=120.0, y=10.0
    )
    cmd_place.apply(doc)
    if cmd_place.instance:
        cmd_place.instance.overrides["fill"] = "#654321"

    # SVG export
    svg_data = export_svg(doc)
    assert "<svg" in svg_data
    assert "#123456" in svg_data
    assert "#654321" in svg_data

    # PNG export
    png_path = tmp_path / "out.png"
    export_png(doc, png_path)
    assert png_path.exists()
    assert png_path.stat().st_size > 0


def test_bridge_resources_flow() -> None:
    bridge = Bridge()
    bridge.create_shape("rectangle", 100.0, 100.0)
    obj_id = cast(str, bridge.selectedId)
    assert obj_id != ""

    # Create symbol via bridge
    sym_id = bridge.createSymbol(obj_id, "Widget")
    assert sym_id != ""
    assert cast(bool, bridge.selectedIsSymbol) is True
    assert cast(str, bridge.selectedSymbolId) == sym_id

    # Place symbol instance via bridge
    inst_id = bridge.placeSymbol(sym_id, 200.0, 200.0)
    assert inst_id != ""
    assert cast(str, bridge.selectedId) == inst_id
    assert cast(bool, bridge.selectedIsSymbol) is True

    # Override property via setProp on instance
    bridge.setProp("fill", "#abcdef")
    assert cast(dict[str, Any], bridge.selectedOverrides).get("fill") == "#abcdef"

    # Create and apply style via bridge
    style_id = bridge.createStyle("Neon", "object", inst_id)
    assert style_id != ""
    bridge.applyStyle(style_id)
    assert cast(str, bridge.selectedStyleId) == style_id

    # Detach style
    bridge.detachStyle(inst_id)
    assert cast(str, bridge.selectedStyleId) == ""

    # Detach symbol
    bridge.detachSymbol(inst_id)
    assert cast(bool, bridge.selectedIsSymbol) is False

    # Check properties query
    symbols_view = cast(list[dict[str, Any]], bridge.symbols)
    assert len(symbols_view) == 1
    assert symbols_view[0]["name"] == "Widget"

    styles_view = cast(list[dict[str, Any]], bridge.styles)
    assert len(styles_view) == 1
    assert styles_view[0]["name"] == "Neon"
