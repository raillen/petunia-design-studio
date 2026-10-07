# 09.9.1 — Text Canonical Model: Stories, Paragraphs, Runs, Ranges, Indices & Style Inheritance

# Canonical storage

TextObject references TextStory and one or more TextFrame placements. Story stores Unicode text plus style runs/paragraph boundaries; layout glyphs are derived.

# Index convention

Canonical text edits use Unicode scalar/codepoint indices or a documented rope position abstraction—not UTF-8 byte offsets. UI caret/selection maps to grapheme clusters; HarfBuzz maps runs to glyph clusters. Conversion tables are derived per paragraph/run.

# Data model

```
TextStory
  StoryId
  text storage
  ParagraphSpan[]
  CharacterRun[]
  inline objects/fields[]
  revision

ParagraphSpan
  start/end
  ParagraphStyleRef + overrides

CharacterRun
  start/end
  CharacterStyleRef + overrides
```

# Storage structure

For large editable stories use rope/piece-table/gap-buffer candidate selected by benchmark. Contract requires efficient insert/delete and stable-ish position mapping for selections/annotations.

# Style inheritance

Document default -> ParagraphStyle -> CharacterStyle -> local run override. Computed style derived; canonical layer records only references/overrides needed.

# Fields

Page number, Data Merge fields and future dynamic values are inline semantic atoms with fallback text and type, not ad-hoc textual tokens once parsed.

# Normalization

Do not silently normalize user Unicode on edit unless explicit policy proves interoperability need. Search/compare can normalize derived forms.

# Newlines

Canonical paragraph separator policy fixed (e.g. U+000A internally) with import/export mapping. CRLF is external encoding detail.

# Tests

Insert/delete across runs, style splitting/merging, grapheme clusters, emoji ZWJ, combining marks, huge story edits and save roundtrip.