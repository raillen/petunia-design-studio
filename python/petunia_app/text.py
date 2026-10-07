"""G033 — Canonical Text Model & Editing Command Kernel.

Provides the headless, renderer-independent text data structures:
- TextStory: Unicode codepoint storage, paragraph spans, character runs, inline fields
- Unicode grapheme cluster segmentation & navigation (UAX #29 conforming)
- Run splitting, merging and splice maintenance
- Style inheritance: Document default -> ParagraphStyle -> CharacterStyle -> local overrides
- Editing commands with exact roundtrip undo/redo
"""

from __future__ import annotations

import unicodedata
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

from .model import Change, Command, TextStyle, new_id

if TYPE_CHECKING:
    from .model import Document


# ---------------------------------------------------------------------------
# Default Styles
# ---------------------------------------------------------------------------

DEFAULT_CHARACTER_STYLE: dict[str, Any] = {
    "fontFamily": "Inter",
    "fontSize": 14.0,
    "fontWeight": "normal",
    "fontStyle": "normal",
    "fill": "#cccccc",
    "letterSpacing": 0.0,
}

DEFAULT_PARAGRAPH_STYLE: dict[str, Any] = {
    "align": "left",
    "lineHeight": 1.2,
    "spaceBefore": 0.0,
    "spaceAfter": 0.0,
}


# ---------------------------------------------------------------------------
# Unicode Grapheme Segmentation (UAX #29 core rules)
# ---------------------------------------------------------------------------


def is_combining_mark(ch: str) -> bool:
    cat = unicodedata.category(ch)
    return cat in {"Mn", "Mc", "Me"}


def is_variation_selector(ch: str) -> bool:
    cp = ord(ch)
    return (0xFE00 <= cp <= 0xFE0F) or (0xE0100 <= cp <= 0xE01EF)


def is_skin_tone_modifier(ch: str) -> bool:
    return 0x1F3FB <= ord(ch) <= 0x1F3FF


def is_regional_indicator(ch: str) -> bool:
    return 0x1F1E6 <= ord(ch) <= 0x1F1FF


def is_zwj(ch: str) -> bool:
    return ord(ch) == 0x200D


def is_extend_char(ch: str) -> bool:
    return is_combining_mark(ch) or is_variation_selector(ch) or is_skin_tone_modifier(ch)


def grapheme_spans(text: str) -> list[tuple[int, int]]:
    """Return slice boundaries (start, end) for each grapheme cluster."""
    if not text:
        return []
    spans: list[tuple[int, int]] = []
    i = 0
    n = len(text)
    while i < n:
        start = i
        ch = text[i]
        i += 1

        # CRLF pair is a single cluster
        if ch == "\r" and i < n and text[i] == "\n":
            spans.append((start, i + 1))
            i += 1
            continue

        # Regional Indicator pairs (flags)
        if is_regional_indicator(ch):
            if i < n and is_regional_indicator(text[i]):
                i += 1
            spans.append((start, i))
            continue

        # Standard character followed by extensions or ZWJ sequences
        while i < n:
            next_ch = text[i]
            if is_extend_char(next_ch):
                i += 1
            elif is_zwj(next_ch):
                i += 1
                if i < n:
                    # Glue with following character and its extensions
                    i += 1
            else:
                break

        spans.append((start, i))
    return spans


def grapheme_count(text: str) -> int:
    return len(grapheme_spans(text))


def next_grapheme_offset(text: str, offset: int) -> int:
    """Next boundary position strictly greater than offset."""
    for start, end in grapheme_spans(text):
        if start <= offset < end:
            return end
        if start > offset:
            return start
    return len(text)


def prev_grapheme_offset(text: str, offset: int) -> int:
    """Previous boundary position strictly less than offset."""
    last = 0
    for start, end in grapheme_spans(text):
        if end >= offset:
            return start
        last = end
    return last


def codepoint_to_grapheme_index(text: str, cp_offset: int) -> int:
    spans = grapheme_spans(text)
    for idx, (start, end) in enumerate(spans):
        if start <= cp_offset < end:
            return idx
    return len(spans)


def grapheme_to_codepoint_offset(text: str, gr_index: int) -> int:
    spans = grapheme_spans(text)
    if gr_index <= 0 or not spans:
        return 0
    if gr_index >= len(spans):
        return len(text)
    return spans[gr_index][0]


# ---------------------------------------------------------------------------
# Data Models
# ---------------------------------------------------------------------------


@dataclass
class CaretPosition:
    offset: int = 0
    affinity: str = "downstream"  # "upstream" | "downstream"


@dataclass
class TextSelection:
    start: int = 0
    end: int = 0

    @property
    def is_empty(self) -> bool:
        return self.start == self.end

    @property
    def normalized(self) -> tuple[int, int]:
        return (min(self.start, self.end), max(self.start, self.end))


@dataclass
class InlineField:
    start: int
    end: int
    kind: str  # "page_number" | "data_merge" | "date"
    value: str
    fallback: str = ""

    def to_json(self) -> dict[str, Any]:
        return {
            "start": self.start,
            "end": self.end,
            "kind": self.kind,
            "value": self.value,
            "fallback": self.fallback,
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> InlineField:
        return cls(
            start=int(d["start"]),
            end=int(d["end"]),
            kind=str(d["kind"]),
            value=str(d.get("value", "")),
            fallback=str(d.get("fallback", "")),
        )


@dataclass
class CharacterRun:
    start: int
    end: int
    style_id: str | None = None
    overrides: dict[str, Any] = field(default_factory=lambda: {})

    def to_json(self) -> dict[str, Any]:
        res: dict[str, Any] = {"start": self.start, "end": self.end}
        if self.style_id is not None:
            res["styleId"] = self.style_id
        if self.overrides:
            res["overrides"] = dict(self.overrides)
        return res

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> CharacterRun:
        return cls(
            start=int(d["start"]),
            end=int(d["end"]),
            style_id=str(d["styleId"]) if d.get("styleId") is not None else None,
            overrides=dict(d.get("overrides", {})),
        )


@dataclass
class ParagraphSpan:
    start: int
    end: int
    style_id: str | None = None
    overrides: dict[str, Any] = field(default_factory=lambda: {})

    def to_json(self) -> dict[str, Any]:
        res: dict[str, Any] = {"start": self.start, "end": self.end}
        if self.style_id is not None:
            res["styleId"] = self.style_id
        if self.overrides:
            res["overrides"] = dict(self.overrides)
        return res

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> ParagraphSpan:
        return cls(
            start=int(d["start"]),
            end=int(d["end"]),
            style_id=str(d["styleId"]) if d.get("styleId") is not None else None,
            overrides=dict(d.get("overrides", {})),
        )


@dataclass
class TextStory:
    """Canonical text story storage independent of renderers and Qt."""

    id: str = field(default_factory=new_id)
    text: str = ""
    paragraphs: list[ParagraphSpan] = field(default_factory=lambda: [])
    runs: list[CharacterRun] = field(default_factory=lambda: [])
    fields: list[InlineField] = field(default_factory=lambda: [])
    revision: int = 0

    def __post_init__(self) -> None:
        if not self.paragraphs and not self.text:
            self.paragraphs = [ParagraphSpan(start=0, end=0)]
        elif not self.paragraphs:
            self.rebuild_paragraphs()
        if not self.runs:
            self.runs = [CharacterRun(start=0, end=len(self.text))]

    def rebuild_paragraphs(self) -> None:
        """Recompute paragraph boundaries from text, keeping styles where possible."""
        old_paras = list(self.paragraphs)
        new_paras: list[ParagraphSpan] = []
        n = len(self.text)
        if n == 0:
            style_id = old_paras[0].style_id if old_paras else None
            overrides = dict(old_paras[0].overrides) if old_paras else {}
            self.paragraphs = [ParagraphSpan(0, 0, style_id=style_id, overrides=overrides)]
            return

        start = 0
        idx = 0
        while start < n:
            pos = self.text.find("\n", start)
            end = pos + 1 if pos != -1 else n
            prev = old_paras[idx] if idx < len(old_paras) else None
            new_paras.append(
                ParagraphSpan(
                    start=start,
                    end=end,
                    style_id=prev.style_id if prev else None,
                    overrides=dict(prev.overrides) if prev else {},
                )
            )
            idx += 1
            if pos == -1:
                break
            start = end

        # If text ends with newline, trailing empty paragraph exists
        if self.text.endswith("\n"):
            prev = old_paras[idx] if idx < len(old_paras) else None
            new_paras.append(
                ParagraphSpan(
                    start=n,
                    end=n,
                    style_id=prev.style_id if prev else None,
                    overrides=dict(prev.overrides) if prev else {},
                )
            )

        self.paragraphs = new_paras

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "text": self.text,
            "revision": self.revision,
            "paragraphs": [p.to_json() for p in self.paragraphs],
            "runs": [r.to_json() for r in self.runs],
            "fields": [f.to_json() for f in self.fields],
        }

    @classmethod
    def from_json(cls, d: dict[str, Any]) -> TextStory:
        return cls(
            id=str(d.get("id", new_id())),
            text=str(d.get("text", "")),
            revision=int(d.get("revision", 0)),
            paragraphs=[ParagraphSpan.from_json(p) for p in d.get("paragraphs", [])],
            runs=[CharacterRun.from_json(r) for r in d.get("runs", [])],
            fields=[InlineField.from_json(f) for f in d.get("fields", [])],
        )


# ---------------------------------------------------------------------------
# Run Normalization and Splicing
# ---------------------------------------------------------------------------


def normalize_runs(runs: list[CharacterRun], total_length: int) -> list[CharacterRun]:
    """Merge adjacent runs with identical formatting and eliminate empty runs."""
    cleaned: list[CharacterRun] = []
    for r in runs:
        if r.end <= r.start:
            continue
        if cleaned:
            prev = cleaned[-1]
            if (
                prev.end == r.start
                and prev.style_id == r.style_id
                and prev.overrides == r.overrides
            ):
                prev.end = r.end
                continue
        cleaned.append(CharacterRun(r.start, r.end, r.style_id, dict(r.overrides)))

    if not cleaned:
        return [CharacterRun(0, total_length)]

    # Guarantee full coverage up to total_length
    if cleaned[-1].end < total_length:
        cleaned[-1].end = total_length
    return cleaned


def splice_runs(
    runs: list[CharacterRun],
    start: int,
    delete_count: int,
    insert_len: int,
    style_id: str | None = None,
    overrides: dict[str, Any] | None = None,
) -> list[CharacterRun]:
    """Splice runs when deleting `delete_count` characters at `start` and inserting `insert_len`."""
    del_end = start + delete_count
    delta = insert_len - delete_count
    new_runs: list[CharacterRun] = []

    inherited_style_id: str | None = style_id
    inherited_overrides: dict[str, Any] = dict(overrides or {})

    # Determine inherited style from insertion point if not given
    if inherited_style_id is None and not inherited_overrides:
        for r in runs:
            if r.start <= start <= r.end:
                inherited_style_id = r.style_id
                inherited_overrides = dict(r.overrides)
                break

    for r in runs:
        if r.end <= start:
            # Completely before deletion
            new_runs.append(CharacterRun(r.start, r.end, r.style_id, dict(r.overrides)))
        elif r.start >= del_end:
            # Completely after deletion
            new_runs.append(
                CharacterRun(r.start + delta, r.end + delta, r.style_id, dict(r.overrides))
            )
        else:
            # Overlaps deletion range
            if r.start < start:
                new_runs.append(CharacterRun(r.start, start, r.style_id, dict(r.overrides)))
            if r.end > del_end:
                new_runs.append(
                    CharacterRun(
                        start + insert_len,
                        r.end + delta,
                        r.style_id,
                        dict(r.overrides),
                    )
                )

    if insert_len > 0:
        new_runs.append(
            CharacterRun(
                start,
                start + insert_len,
                style_id=inherited_style_id,
                overrides=inherited_overrides,
            )
        )

    new_runs.sort(key=lambda r: r.start)
    total_len = (runs[-1].end if runs else 0) + delta
    return normalize_runs(new_runs, max(0, total_len))


def splice_fields(
    fields: list[InlineField],
    start: int,
    delete_count: int,
    insert_len: int,
) -> list[InlineField]:
    del_end = start + delete_count
    delta = insert_len - delete_count
    kept: list[InlineField] = []
    for f in fields:
        if f.end <= start:
            kept.append(f)
        elif f.start >= del_end:
            kept.append(InlineField(f.start + delta, f.end + delta, f.kind, f.value, f.fallback))
        # Fields overlapping deletion are dropped
    return kept


# ---------------------------------------------------------------------------
# Style Inheritance
# ---------------------------------------------------------------------------


def find_text_style(doc: Document | None, style_id: str | None) -> TextStyle | None:
    if doc is None or style_id is None:
        return None
    for lib in doc.libraries:
        st = lib.styles.get(style_id)
        if isinstance(st, TextStyle):
            return st
    return None


def computed_character_style(
    doc: Document | None, story: TextStory, offset: int
) -> dict[str, Any]:
    """Compute resolved character style:
    default -> paragraph style -> character style -> run overrides.
    """
    result = dict(DEFAULT_CHARACTER_STYLE)

    # Paragraph style font properties
    for para in story.paragraphs:
        if para.start <= offset < para.end or (para.start == para.end == offset):
            if para.style_id:
                pst = find_text_style(doc, para.style_id)
                if pst and pst.properties:
                    for k in DEFAULT_CHARACTER_STYLE:
                        if k in pst.properties:
                            result[k] = pst.properties[k]
            break

    # Character run
    for run in story.runs:
        if run.start <= offset < run.end or (run.start == run.end == offset):
            if run.style_id:
                cst = find_text_style(doc, run.style_id)
                if cst and cst.properties:
                    result.update(cst.properties)
            if run.overrides:
                result.update(run.overrides)
            break

    return result


def computed_paragraph_style(
    doc: Document | None, story: TextStory, offset: int
) -> dict[str, Any]:
    """Compute resolved paragraph style: default -> paragraph style -> overrides."""
    result = dict(DEFAULT_PARAGRAPH_STYLE)
    for para in story.paragraphs:
        if para.start <= offset < para.end or (para.start == para.end == offset):
            if para.style_id:
                pst = find_text_style(doc, para.style_id)
                if pst and pst.properties:
                    result.update(pst.properties)
            if para.overrides:
                result.update(para.overrides)
            break
    return result


# ---------------------------------------------------------------------------
# Text Editing Commands
# ---------------------------------------------------------------------------


def _find_story(doc: Document, story_id: str) -> TextStory | None:
    return getattr(doc, "stories", {}).get(story_id)


class InsertTextCommand(Command):
    """Insert text at codepoint offset with optional style/overrides."""

    def __init__(
        self,
        story_id: str,
        offset: int,
        text: str,
        style_id: str | None = None,
        overrides: dict[str, Any] | None = None,
    ) -> None:
        self.story_id = story_id
        self.offset = offset
        self.text = text
        self.style_id = style_id
        self.overrides = dict(overrides or {})
        self.snapshot_before: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None or not self.text:
            return Change()
        self.snapshot_before = story.to_json()
        insert_len = len(self.text)
        story.text = story.text[: self.offset] + self.text + story.text[self.offset :]
        story.runs = splice_runs(
            story.runs,
            self.offset,
            0,
            insert_len,
            style_id=self.style_id,
            overrides=self.overrides,
        )
        story.fields = splice_fields(story.fields, self.offset, 0, insert_len)
        story.rebuild_paragraphs()
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.snapshot_before:
            return
        restored = TextStory.from_json(self.snapshot_before)
        story.text = restored.text
        story.runs = restored.runs
        story.paragraphs = restored.paragraphs
        story.fields = restored.fields
        story.revision = restored.revision


class DeleteRangeCommand(Command):
    """Delete a codepoint range [start, end]."""

    def __init__(self, story_id: str, start: int, end: int) -> None:
        self.story_id = story_id
        self.start = min(start, end)
        self.end = max(start, end)
        self.snapshot_before: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None or self.start >= self.end:
            return Change()
        self.snapshot_before = story.to_json()
        delete_count = self.end - self.start
        story.text = story.text[: self.start] + story.text[self.end :]
        story.runs = splice_runs(story.runs, self.start, delete_count, 0)
        story.fields = splice_fields(story.fields, self.start, delete_count, 0)
        story.rebuild_paragraphs()
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.snapshot_before:
            return
        restored = TextStory.from_json(self.snapshot_before)
        story.text = restored.text
        story.runs = restored.runs
        story.paragraphs = restored.paragraphs
        story.fields = restored.fields
        story.revision = restored.revision


class ReplaceRangeCommand(Command):
    """Replace range [start, end] with new text."""

    def __init__(
        self,
        story_id: str,
        start: int,
        end: int,
        text: str,
        style_id: str | None = None,
        overrides: dict[str, Any] | None = None,
    ) -> None:
        self.story_id = story_id
        self.start = min(start, end)
        self.end = max(start, end)
        self.text = text
        self.style_id = style_id
        self.overrides = dict(overrides or {})
        self.snapshot_before: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None:
            return Change()
        self.snapshot_before = story.to_json()
        delete_count = self.end - self.start
        insert_len = len(self.text)
        story.text = story.text[: self.start] + self.text + story.text[self.end :]
        story.runs = splice_runs(
            story.runs,
            self.start,
            delete_count,
            insert_len,
            style_id=self.style_id,
            overrides=self.overrides,
        )
        story.fields = splice_fields(story.fields, self.start, delete_count, insert_len)
        story.rebuild_paragraphs()
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.snapshot_before:
            return
        restored = TextStory.from_json(self.snapshot_before)
        story.text = restored.text
        story.runs = restored.runs
        story.paragraphs = restored.paragraphs
        story.fields = restored.fields
        story.revision = restored.revision


class ApplyCharacterStyleCommand(Command):
    """Apply a character style to a range [start, end]."""

    def __init__(
        self,
        story_id: str,
        start: int,
        end: int,
        style_id: str | None,
        overrides: dict[str, Any] | None = None,
    ) -> None:
        self.story_id = story_id
        self.start = min(start, end)
        self.end = max(start, end)
        self.style_id = style_id
        self.overrides = dict(overrides or {})
        self.snapshot_before: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None or self.start >= self.end:
            return Change()
        self.snapshot_before = story.to_json()
        # Splice range with zero deletion and zero insertion, re-styling the segment
        new_runs: list[CharacterRun] = []
        for r in story.runs:
            if r.end <= self.start or r.start >= self.end:
                new_runs.append(r)
            else:
                if r.start < self.start:
                    new_runs.append(
                        CharacterRun(r.start, self.start, r.style_id, dict(r.overrides))
                    )
                mid_start = max(r.start, self.start)
                mid_end = min(r.end, self.end)
                merged_overrides = dict(r.overrides)
                merged_overrides.update(self.overrides)
                new_runs.append(
                    CharacterRun(
                        mid_start,
                        mid_end,
                        style_id=self.style_id if self.style_id is not None else r.style_id,
                        overrides=merged_overrides,
                    )
                )
                if r.end > self.end:
                    new_runs.append(CharacterRun(self.end, r.end, r.style_id, dict(r.overrides)))
        new_runs.sort(key=lambda r: r.start)
        story.runs = normalize_runs(new_runs, len(story.text))
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.snapshot_before:
            return
        restored = TextStory.from_json(self.snapshot_before)
        story.runs = restored.runs
        story.revision = restored.revision


class ApplyParagraphStyleCommand(Command):
    """Apply a paragraph style to one or all paragraphs in a range."""

    def __init__(
        self,
        story_id: str,
        start_offset: int,
        end_offset: int,
        style_id: str | None,
        overrides: dict[str, Any] | None = None,
    ) -> None:
        self.story_id = story_id
        self.start_offset = min(start_offset, end_offset)
        self.end_offset = max(start_offset, end_offset)
        self.style_id = style_id
        self.overrides = dict(overrides or {})
        self.before_spans: list[dict[str, Any]] = []

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None:
            return Change()
        self.before_spans = [p.to_json() for p in story.paragraphs]
        for p in story.paragraphs:
            if (p.start <= self.start_offset <= p.end) or (
                p.start <= self.end_offset <= p.end
            ) or (self.start_offset <= p.start and p.end <= self.end_offset):
                if self.style_id is not None:
                    p.style_id = self.style_id
                p.overrides.update(self.overrides)
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.before_spans:
            return
        story.paragraphs = [ParagraphSpan.from_json(d) for d in self.before_spans]
        story.revision += 1


class InsertFieldCommand(Command):
    """Insert an inline semantic field token into text."""

    def __init__(
        self,
        story_id: str,
        offset: int,
        kind: str,
        value: str,
        fallback: str = "",
    ) -> None:
        self.story_id = story_id
        self.offset = offset
        self.kind = kind
        self.value = value
        self.fallback = fallback or f"{{{value}}}"
        self.snapshot_before: dict[str, Any] = {}

    def apply(self, doc: Document) -> Change:
        story = _find_story(doc, self.story_id)
        if story is None:
            return Change()
        self.snapshot_before = story.to_json()
        insert_text = self.fallback
        insert_len = len(insert_text)
        story.text = story.text[: self.offset] + insert_text + story.text[self.offset :]
        story.runs = splice_runs(story.runs, self.offset, 0, insert_len)
        story.fields = splice_fields(story.fields, self.offset, 0, insert_len)
        story.fields.append(
            InlineField(
                start=self.offset,
                end=self.offset + insert_len,
                kind=self.kind,
                value=self.value,
                fallback=self.fallback,
            )
        )
        story.fields.sort(key=lambda f: f.start)
        story.rebuild_paragraphs()
        story.revision += 1
        return Change(modified=[self.story_id])

    def revert(self, doc: Document) -> None:
        story = _find_story(doc, self.story_id)
        if story is None or not self.snapshot_before:
            return
        restored = TextStory.from_json(self.snapshot_before)
        story.text = restored.text
        story.runs = restored.runs
        story.paragraphs = restored.paragraphs
        story.fields = restored.fields
        story.revision = restored.revision
