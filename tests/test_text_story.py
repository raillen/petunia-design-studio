from __future__ import annotations

import json
from pathlib import Path
from typing import cast

import jsonschema

from petunia_app.bridge import Bridge
from petunia_app.exporters import export_svg
from petunia_app.model import (
    CreateStyleCommand,
    Document,
    History,
    SceneObject,
)
from petunia_app.ptnd import read_package
from petunia_app.ptnd_scene import open_ptnd, save_ptnd
from petunia_app.raster import export_png
from petunia_app.text import (
    ApplyCharacterStyleCommand,
    ApplyParagraphStyleCommand,
    DeleteRangeCommand,
    InsertFieldCommand,
    InsertTextCommand,
    ReplaceRangeCommand,
    TextStory,
    codepoint_to_grapheme_index,
    computed_character_style,
    computed_paragraph_style,
    grapheme_count,
    grapheme_spans,
    grapheme_to_codepoint_offset,
    next_grapheme_offset,
    prev_grapheme_offset,
)


def test_unicode_grapheme_segmentation() -> None:
    # 1. Standard text
    s1 = "Hello"
    assert grapheme_count(s1) == 5
    assert grapheme_spans(s1) == [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]

    # 2. Combining marks: e + acute combining (\u0301)
    s2 = "e\u0301cole"
    # "e\u0301" is 1 grapheme cluster (2 codepoints), total 5 graphemes
    assert len(s2) == 6
    assert grapheme_count(s2) == 5
    spans2 = grapheme_spans(s2)
    assert spans2[0] == (0, 2)
    assert next_grapheme_offset(s2, 0) == 2
    assert prev_grapheme_offset(s2, 2) == 0

    # 3. Emoji skin tone modifier: 👍 + skin tone \U0001F3FD
    s3 = "👍🏽"
    assert len(s3) == 2  # 2 codepoints
    assert grapheme_count(s3) == 1
    assert grapheme_spans(s3) == [(0, 2)]

    # 4. Emoji ZWJ sequence: family 👨‍👩‍👧‍👦
    # 👨 + ZWJ + 👩 + ZWJ + 👧 + ZWJ + 👦
    family = "👨\u200D👩\u200D👧\u200D👦"
    assert grapheme_count(family) == 1
    assert grapheme_spans(family) == [(0, len(family))]

    # 5. Regional indicators: Flag of Brazil 🇧 + 🇷
    flag_br = "\U0001F1E7\U0001F1F7"
    assert grapheme_count(flag_br) == 1
    assert grapheme_spans(flag_br) == [(0, 2)]

    # 6. CRLF pair
    crlf = "A\r\nB"
    assert grapheme_count(crlf) == 3
    assert grapheme_spans(crlf) == [(0, 1), (1, 3), (3, 4)]

    # 7. Navigation and conversion
    phrase = "Aé\u0301B👍🏽C"
    assert codepoint_to_grapheme_index(phrase, 0) == 0  # 'A'
    assert codepoint_to_grapheme_index(phrase, 1) == 1  # 'e\u0301' start
    assert codepoint_to_grapheme_index(phrase, 2) == 1  # 'e\u0301' inside
    assert grapheme_to_codepoint_offset(phrase, 1) == 1
    assert grapheme_to_codepoint_offset(phrase, 2) == 3  # 'B' starts at 3 (A=1, é=2)


def test_story_paragraphs_and_runs() -> None:
    story = TextStory(text="Primeiro\nSegundo\nTerceiro")
    assert len(story.paragraphs) == 3
    assert story.paragraphs[0].start == 0 and story.paragraphs[0].end == 9  # "Primeiro\n"
    assert story.paragraphs[1].start == 9 and story.paragraphs[1].end == 17  # "Segundo\n"
    assert story.paragraphs[2].start == 17 and story.paragraphs[2].end == 25  # "Terceiro"

    # Run coverage
    assert len(story.runs) == 1
    assert story.runs[0].start == 0 and story.runs[0].end == 25


def test_text_commands_lifecycle_and_undo() -> None:
    doc = Document(name="DocText")
    history = History()

    story = TextStory(text="Hello World")
    doc.stories[story.id] = story

    # 1. Insert text
    cmd_insert = InsertTextCommand(story.id, 5, " Beautiful")
    history.execute(cmd_insert, doc)
    assert story.text == "Hello Beautiful World"
    assert story.runs[0].end == len("Hello Beautiful World")

    # 2. Apply character style to "Beautiful" (offset 6 to 15)
    cmd_style = ApplyCharacterStyleCommand(
        story.id,
        6,
        15,
        style_id="brand-bold",
        overrides={"fontWeight": "bold", "fill": "#ff0000"},
    )
    history.execute(cmd_style, doc)
    assert len(story.runs) == 3
    assert story.runs[0].end == 6
    assert story.runs[1].style_id == "brand-bold"
    assert story.runs[1].overrides["fontWeight"] == "bold"
    assert story.runs[1].start == 6 and story.runs[1].end == 15
    assert story.runs[2].start == 15 and story.runs[2].end == len(story.text)

    # 3. Replace range
    cmd_replace = ReplaceRangeCommand(story.id, 6, 15, "Awesome")
    history.execute(cmd_replace, doc)
    assert story.text == "Hello Awesome World"

    # 4. Paragraph style
    cmd_para = ApplyParagraphStyleCommand(
        story.id, 0, len(story.text), style_id="lead-p", overrides={"align": "center"}
    )
    history.execute(cmd_para, doc)
    assert story.paragraphs[0].style_id == "lead-p"
    assert story.paragraphs[0].overrides["align"] == "center"

    # 5. Insert inline field
    cmd_field = InsertFieldCommand(story.id, 0, kind="date", value="today", fallback="2026-10-06 ")
    history.execute(cmd_field, doc)
    assert story.text.startswith("2026-10-06 ")
    assert len(story.fields) == 1
    assert story.fields[0].kind == "date"

    # 6. Delete range
    cmd_delete = DeleteRangeCommand(story.id, 0, len("2026-10-06 "))
    history.execute(cmd_delete, doc)
    assert story.text == "Hello Awesome World"
    assert len(story.fields) == 0

    # Test Undo stack
    history.undo(doc)  # undo delete
    assert story.text.startswith("2026-10-06 ")
    assert len(story.fields) == 1

    history.undo(doc)  # undo field
    assert story.text == "Hello Awesome World"
    assert len(story.fields) == 0

    history.undo(doc)  # undo paragraph style
    assert story.paragraphs[0].style_id is None

    history.undo(doc)  # undo replace
    assert story.text == "Hello Beautiful World"

    history.undo(doc)  # undo character style
    assert len(story.runs) == 1

    history.undo(doc)  # undo insert
    assert story.text == "Hello World"

    # Test Redo stack
    history.redo(doc)
    assert story.text == "Hello Beautiful World"


def test_style_inheritance_resolution() -> None:
    doc = Document()
    story = TextStory(text="Inherited Text")
    doc.stories[story.id] = story

    # 1. Document default
    style_def = computed_character_style(doc, story, 0)
    assert style_def["fontFamily"] == "Inter"
    assert style_def["fontSize"] == 14.0

    # 2. Add TextStyle to document library
    cmd_pstyle = CreateStyleCommand(
        "LeadPara", "paragraph", properties={"fontSize": 18.0, "align": "center"}
    )
    cmd_pstyle.apply(doc)
    assert cmd_pstyle.style is not None
    p_id = cmd_pstyle.style.id

    cmd_cstyle = CreateStyleCommand(
        "BoldRed", "character", properties={"fontWeight": "bold", "fill": "#ff1122"}
    )
    cmd_cstyle.apply(doc)
    assert cmd_cstyle.style is not None
    c_id = cmd_cstyle.style.id

    # Apply paragraph style to paragraph
    ApplyParagraphStyleCommand(story.id, 0, len(story.text), p_id).apply(doc)
    # Paragraph fontSize takes effect
    assert computed_character_style(doc, story, 0)["fontSize"] == 18.0
    assert computed_paragraph_style(doc, story, 0)["align"] == "center"

    # Apply character style to "Text" (offset 10 to 14)
    ApplyCharacterStyleCommand(story.id, 10, 14, c_id, overrides={"letterSpacing": 1.5}).apply(doc)
    styled = computed_character_style(doc, story, 11)
    assert styled["fontSize"] == 18.0  # inherited from paragraph style
    assert styled["fontWeight"] == "bold"  # inherited from character style
    assert styled["fill"] == "#ff1122"  # inherited from character style
    assert styled["letterSpacing"] == 1.5  # local override


def test_ptnd_text_story_schema_and_roundtrip(tmp_path: Path) -> None:
    pkg = tmp_path / "text_doc.ptnd"
    doc = Document(name="TypographyCard", width=800.0, height=600.0)

    story = TextStory(text="Título Principal\nSubtítulo elegante")
    doc.stories[story.id] = story

    text_obj = SceneObject(
        type="text",
        name="HeaderFrame",
        x=50.0,
        y=80.0,
        width=400.0,
        height=100.0,
        fill="#ffffff",
        story_id=story.id,
    )
    doc.objects.append(text_obj)

    # Save to PTND package
    save_ptnd(pkg, doc)

    # 1. Schema validation against document.schema.json
    _, doc_json = read_package(pkg)
    schema_path = Path("schemas/ptnd/v1/document.schema.json")
    schema = json.loads(schema_path.read_text())
    jsonschema.validate(doc_json, schema)

    # 2. Reopen and verify data parity
    reopened = open_ptnd(pkg)
    assert len(reopened.stories) == 1
    assert story.id in reopened.stories
    re_story = reopened.stories[story.id]
    assert re_story.text == "Título Principal\nSubtítulo elegante"
    assert len(re_story.paragraphs) == 2
    assert len(reopened.objects) == 1
    assert reopened.objects[0].type == "text"
    assert reopened.objects[0].story_id == story.id


def test_svg_and_png_export_with_text(tmp_path: Path) -> None:
    doc = Document(name="TextExport", width=500.0, height=200.0)
    story = TextStory(text="Vector & Raster Text")
    doc.stories[story.id] = story
    text_obj = SceneObject(
        type="text",
        name="Label",
        x=20.0,
        y=30.0,
        width=300.0,
        height=50.0,
        fill="#334455",
        story_id=story.id,
    )
    doc.objects.append(text_obj)

    # SVG export
    svg_data = export_svg(doc)
    assert "<text" in svg_data
    assert "Vector &amp; Raster Text" in svg_data

    # PNG export
    png_path = tmp_path / "text.png"
    export_png(doc, png_path)
    assert png_path.exists()
    assert png_path.stat().st_size > 0


def test_bridge_text_workflow() -> None:
    bridge = Bridge()

    # 1. Create text object
    obj_id = bridge.createText(50.0, 50.0, "Texto Inicial")
    assert obj_id != ""
    assert cast(str, bridge.selectedId) == obj_id
    assert cast(str, bridge.selectedText) == "Texto Inicial"

    # 2. Insert text via bridge
    bridge.insertText(obj_id, 5, " Muito")
    assert cast(str, bridge.selectedText) == "Texto Muito Inicial"

    # 3. Replace text via setProp
    bridge.setProp("text", "Conteúdo Atualizado")
    assert cast(str, bridge.selectedText) == "Conteúdo Atualizado"

    # 4. Delete range
    bridge.deleteTextRange(obj_id, 0, 9)
    assert cast(str, bridge.selectedText) == "Atualizado"

    # 5. Undo/Redo
    bridge.undo()
    bridge.select(obj_id)
    assert cast(str, bridge.selectedText) == "Conteúdo Atualizado"
    bridge.redo()
    bridge.select(obj_id)
    assert cast(str, bridge.selectedText) == "Atualizado"


def test_large_story_editing_stress() -> None:
    story = TextStory()
    doc = Document()
    doc.stories[story.id] = story

    # Build 500 paragraphs
    paragraphs = [f"Parágrafo {i} com texto informativo e detalhado." for i in range(500)]
    full_text = "\n".join(paragraphs)
    InsertTextCommand(story.id, 0, full_text).apply(doc)

    assert len(story.text) > 20000
    assert len(story.paragraphs) == 500

    # Splice in the middle
    mid = len(story.text) // 2
    InsertTextCommand(story.id, mid, " [INSERÇÃO NO MEIO] ").apply(doc)
    assert " [INSERÇÃO NO MEIO] " in story.text

    # Apply style across 50 paragraphs
    ApplyCharacterStyleCommand(story.id, 100, 1500, style_id="highlight").apply(doc)
    assert any(r.style_id == "highlight" for r in story.runs)

    # Delete large range
    DeleteRangeCommand(story.id, 500, 3000).apply(doc)
    assert len(story.text) < len(full_text)
