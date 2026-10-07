# 18.15 — Text-on-Path Tool Specification

# Identity

ToolId ptnd.tool.text_on_path or an explicit mode of the text-tool family.

# Preconditions

A compatible VectorPath is selected/hovered. Non-vector targets provide disabled reason and conversion alternatives.

# Creation

Click path creates TextObject with semantic path reference, start/end offsets, side/orientation and baseline offset. Existing text can be attached through explicit command.

# State machine

Idle -> PathCandidate -> AttachText -> TextEditing | OffsetEdit -> Commit/Cancel.

# Overlay

Path highlight, start/end limit handles, baseline offset handle, direction/flip indicator and overset-on-path marker.

# Layout

Text engine shapes story normally then maps glyph advances along path arc length. Path edits trigger derived re-layout without changing story text.

# Context

Font basics, start/end offset, baseline, alignment mode, flip side, reverse direction and overflow/wrap policy on closed paths.

# Commands

SetTextPath, SetTextPathOffsets, DetachTextPath, EditTextRange.

# Edge cases

Open/closed path, reversed contour, self-intersection, path deletion, path too short, RTL/bidi and multiple contours.

# Tests

Transform/edit source path, closed-wrap policy, RTL, missing path recovery, save/export and convert-to-curves.