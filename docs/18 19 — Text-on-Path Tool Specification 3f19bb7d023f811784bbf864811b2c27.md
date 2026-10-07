# 18.19 — Text-on-Path Tool Specification

# Creation

Select path + invoke Text on Path action/tool or create with Text tool over eligible path hotspot.

# Canonical relation

TextObject references Path ObjectId and start/end offsets plus baseline/side/orientation settings; path remains editable unless user duplicates internal path by explicit conversion.

# Handles

Start/end markers drag along path arc length. Baseline offset handle moves normal. Flip/Reverse controls change side/direction without rewriting text.

# Editing

Caret/text selection works along shaped visual glyph positions while logical text remains story order.

# Path edits

Changing path invalidates text layout non-destructively. Deleting referenced path follows dependency policy: detach to path snapshot, delete text too, or block/prompt; choose explicit command semantics.

# Commands

SetTextPath, SetTextPathOffsets, FlipTextPath, DetachTextFromPath.

# Tests

Open/closed path, reverse path, bidi, tight curve, path deletion, save/export SVG/PDF.