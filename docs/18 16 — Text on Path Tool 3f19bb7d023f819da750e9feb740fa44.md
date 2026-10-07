# 18.16 — Text on Path Tool

# Identity

ToolId ptnd.tool.text_on_path.

# Creation

Select/create text and eligible path, then invoke action/tool. TextObject stores path relation, start/end offsets, baseline offset, side/flip orientation.

# Interaction

On-canvas start/end handles move along arc length; center/overflow indicators show usable interval. Drag baseline offset perpendicular to path.

# Editing

Text caret remains fully editable while path provides layout baseline. Editing path immediately reflows glyph positions.

# Context

start offset, end offset, baseline offset, reverse/flip, alignment along path, overflow/wrap behavior.

# Commands

AttachTextToPath, SetTextPathOffsets, FlipTextPath, DetachTextFromPath.

# Path lifecycle

Deleting referenced path invokes guarded behavior: detach preserving visual snapshot or delete relationship/object according explicit user action; never dangling reference.

# Export

SVG textPath when supported; PDF native text transforms/paths; otherwise expanded/rasterized via fidelity report.

# Tests

Open/closed paths, reversed contours, sharp corners, bidi, path mutation, detach and save.