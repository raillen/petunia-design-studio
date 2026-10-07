# 09.9.1 — Text Storage Model, Unicode Indices, Stories, Runs & Style Inheritance

# Canonical text model

```
TextStory
  StoryId
  text storage
  ParagraphBoundary[]
  CharacterRun[]
  ParagraphRun[]
  inline fields/objects
TextFrame/Object references StoryId + range/flow role
```

# Storage encoding

UTF-8 is recommended serialized storage; in-memory implementation may use rope/piece table optimized for edits. Public text positions are **Unicode scalar offsets plus affinity/mapping metadata** or another explicitly chosen semantic index, never UTF-8 byte offset exposed to UI.

# Grapheme

Caret/user character movement uses Unicode grapheme clusters. A mapping service converts canonical text offsets ↔ grapheme boundaries ↔ shaping cluster indices.

# Normalization

Do not silently normalize user text on every edit. Preserve entered code points unless import/command explicitly normalizes; shaping/search services handle normalization needs deliberately.

# Paragraphs

Paragraph boundaries are semantic ranges. Paragraph style references + local overrides stored separately from character runs.

# Character runs

Range + CharacterStyleId/reference + overrides. Adjacent equivalent runs normalized/merged after edits to avoid fragmentation.

# Style inheritance

Base document defaults -> referenced style -> local overrides. Clearing override reveals inherited value; serialization preserves reference and override delta.

# Inline fields

Page number/Data Merge fields are semantic tokens/ranges evaluated to derived display text; source expression remains canonical.

# Editing data structure

Piece table/rope candidate must support efficient insert/delete in long stories while producing immutable snapshot for layout jobs.

# Tests

Combining marks, emoji ZWJ, CRLF import, long multilingual story, style-run splice, undo text edits and serialized exact code-point roundtrip.