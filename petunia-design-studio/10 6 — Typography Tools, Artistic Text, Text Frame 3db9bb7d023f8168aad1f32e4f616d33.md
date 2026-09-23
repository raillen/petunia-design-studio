# 10.6 — Typography Tools, Artistic Text, Text Frames, Text-on-Path & Layout Editing

# Tool creation

Text tool click creates Artistic Text; drag creates Text Frame. Existing text click enters editing. Alt/shortcut variants can be remapped; UI indicates mode before commit.

# Artistic Text

Auto-size to content with transform scaling semantics defined. Convert to frame preserves styling/content.

# Text Frame

Fixed geometry, insets, columns, vertical alignment, overflow indicator and linked-flow ports. Resize can change frame vs text scale depending active transform mode.

# Editing model

Caret/selection grapheme-safe, word/paragraph navigation, shift extension, double/triple click, IME, BiDi. Object selection and text editing are distinct focus modes with predictable Esc ladder.

# Styles

Character and paragraph styles plus local overrides. UI shows Mixed/override indicators and commands Clear Overrides/Redefine Style/Detach.

# Typography controls

Family/style/favorites/recent, variable axes, size, leading, tracking, kerning, baseline shift, horizontal/vertical scale only if supported, OpenType feature browser, language/script.

# Paragraph

Alignment, indents, spacing before/after and explicit tab stops are **V1_REQUIRED** paragraph controls. Advanced list/numbering systems and automatic hyphenation dictionaries are **POST_V1_CANDIDATE** unless promoted by an accepted scope decision. Baseline line-breaking remains a required Text Engine behavior.

# Text-on-path

Attach selected text/path, start/end offsets, side/reverse/alignment. Path remains independently editable; detach restores normal text position using defined transform.

# Flow/wrap

Frame links are directed chain without cycles. Exclusion shapes/text wrap feed layout exclusions. Overflow state and preflight diagnostics exposed.

# Convert to Curves

Explicit destructive command creates grouped/path glyph outlines and loses editability; warning only when necessary, fully undoable.

# Tests

IME/BiDi/CJK/Arabic/emoji, missing font, variable font axes, linked-frame edits, overflow, wrap around transformed objects, style updates, text-on-path direction.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** Artistic Text, Text Frames, linked text flow, text-on-path, grapheme-safe editing, IME, BiDi, variable fonts, missing-font preservation/substitution, Character/Paragraph styles, core OpenType controls, alignment/indents/spacing/tabs, overset detection and Convert to Curves.

**POST_V1_CANDIDATE:** advanced automatic hyphenation dictionaries, complex list/numbering systems and long-document publishing features unless promoted by a later accepted scope decision.

## Text creation state

The Text tool has semantic creation modes rather than relying on physical shortcut literals: `ArtisticTextCreate`, `TextFrameCreate`, `TextOnPathAttach`, `EditExisting`. Cursor/overlay communicates the active mode before mutation. Creation is transactional; cancel removes any provisional empty object.

## Focus ladder

Text editing focus is explicit: `ObjectSelected → TextEditing → RangeSelection/IMEComposition`. `Esc`/Finish follows a predictable ladder defined by the Action system, never an ad hoc GPUI handler. Tool switching during IME must first safely commit/cancel composition according to platform text-input contract.

## Story/frame model

Text content belongs to a canonical `TextStory`/equivalent semantic story object. One or more frames reference ranges/flow positions in that story. Linking frames changes flow relationships, not by copying text. Directed frame-flow cycles are invalid.

Overset is explicit derived state with a semantic diagnostic and UI affordance. Export/preflight must report overset text rather than silently clipping unnoticed.

## Artistic text semantics

Artistic Text auto-sizes its layout container from content but still uses normal transforms. Scaling an Artistic Text object via object transform does not silently rewrite font sizes unless the user invokes an explicit “scale text attributes” command/mode. Convert to Text Frame preserves story/style and computes a frame that reproduces current layout as closely as defined by the engine.

## Text Frame semantics

Frame geometry, inset, columns and vertical alignment are document semantics. Resizing a frame defaults to reflowing text, not scaling typography; a separate transform mode may scale the entire object. Frame links are edited through semantic ports and are undoable as one command.

## Typography property behavior

Font family request, face/style, variable axes, size, leading, tracking, kerning, baseline shift, OpenType features and language/script are typed properties. Missing font substitution preserves the requested font descriptor separately from temporary resolved fallback. User choice to replace a missing font is a document mutation; temporary display fallback is not.

## Paragraph baseline V1

Support left/center/right/justified alignment as applicable, first-line/left/right indents, spacing before/after and explicit tab stops. These are V1 typed paragraph properties. Lists/advanced hyphenation remain `POST_V1_CANDIDATE` and must not leak half-implemented controls into the UI.

## Text-on-path

Attachment stores path reference, side/orientation, start/end offset and alignment. Path edits trigger reflow. Detach computes a deterministic normal transform and preserves text story/styles. Deleting the path while text depends on it requires a defined detach/orphan-recovery command, never a dangling reference.

## Selection/edit ranges

All user-visible movement and deletion operates on Unicode grapheme/word/paragraph semantics appropriate to locale and platform expectations while internal canonical ranges use the Text Engine contract from 09.8. Bidirectional visual navigation must not corrupt logical text order.

## Styles and overrides

Character/Paragraph style application, local overrides, clear overrides, redefine and detach use stable style IDs and typed properties. Multi-range selection reports Mixed values without normalizing unrelated spans.

## Convert to Curves

Conversion evaluates the exact shaped glyph runs currently represented by text, creates vector outline objects/resources and records a single undo transaction. The original text must be recoverable by undo but is not kept as hidden duplicate canonical content after commit unless an explicit command variant says so.

## Automation/plugins

MCP and Lua expose story/frame creation, text replacement/range queries, typed typography/paragraph properties, flow linking and text-on-path through semantic APIs. They must not need knowledge of shaping-engine glyph IDs or GPUI editor internals.

## Required tests

Add IME composition interrupted by tool switch, BiDi visual/logical navigation, missing-font fallback vs replacement, Artistic Text scaling, frame resize without font-size mutation, linked-flow edits/overset, text-on-path source deletion, style mixed-state edits, Convert-to-Curves undo and UI↔MCP/Lua parity.