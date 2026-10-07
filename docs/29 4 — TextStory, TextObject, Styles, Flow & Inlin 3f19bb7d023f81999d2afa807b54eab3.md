# 29.4 — TextStory, TextObject, Styles, Flow & Inline Field Schemas

# TextStory

StoryId, Unicode storage representation abstraction, paragraph spans, character runs, inline semantic atoms, revision metadata.

# ParagraphSpan

Start/end logical positions, ParagraphStyleId optional, typed overrides, language/hyphenation/keep/tab data where direct.

# CharacterRun

Start/end, CharacterStyleId optional, typed overrides: font request, variation axes, size, color, OpenType, tracking/kerning/baseline/language.

# Inline atoms

FieldId, kind PageNumber/DataMerge/etc, parameters, fallback text and atomic index behavior. Derived evaluated value is not canonical plain text replacement.

# TextObject

Mode Artistic/Frame/Path; StoryId; frame geometry/insets/columns/flow ports or Path ObjectId/contour ref + offsets; object-local typographic defaults.

# Flow

Frame link graph has previous/next semantics and cycle restrictions. Story can have ordered placement chain.

# Style records

CharacterStyle/ParagraphStyle IDs, optional parent style, property subset and version. Cycles forbidden.

# Validation

Ranges cover valid logical positions; no overlapping contradictory run representation beyond model rules; missing fonts allowed as unresolved request, not schema failure.