# 09.9 — Typography, Text Editing, Shaping & Layout Engine

# Stack

HarfBuzz shaping + FreeType font metrics/raster support, with platform/Qt font discovery adapter. Core text semantics independent from QTextDocument.

# Text model

Unicode text + runs/styles + paragraph attributes + frame geometry + flow links. Store logical text indexes using stable Unicode-aware convention; UI mapping handles grapheme clusters.

# Shaping

Script/language/direction segmentation, font fallback, OpenType features, variable axes. Bidi and complex scripts are baseline architecture, not patch later.

# Layout

Line breaking, hyphenation provider, justification, tabs, indents, columns, vertical metrics, text-on-path and linked frames.

# Editing

Caret, selections, IME composition and clipboard handled presentation-side but translated to semantic text edit Commands. Typing coalesces history sensibly.

# Overflow

Frame reports overflow/overset derived state and port indicators. Linking frames edits flow graph with cycle prevention.

# Font resources

Document references family/style/variation + optional embedded/subset resource policy. Missing font yields explicit substitution state without destroying original request metadata.

# Text to curves

Explicit command evaluates glyph outlines into vector geometry; original text lost only via undoable destructive action.

# Performance

Incremental re-layout by affected paragraphs/frames; glyph caches derived. Benchmarks for long stories, variable fonts, multilingual and thousands of labels.

[09.9.1 — Text Canonical Model: Stories, Paragraphs, Runs, Ranges, Indices & Style Inheritance](09%209%201%20%E2%80%94%20Text%20Canonical%20Model%20Stories,%20Paragraphs,%203f19bb7d023f81dba791f8d59a09bb3e.md)

[09.9.2 — Unicode Segmentation, Bidi, Script Runs, Font Fallback & HarfBuzz Shaping Contract](09%209%202%20%E2%80%94%20Unicode%20Segmentation,%20Bidi,%20Script%20Runs,%20%203f19bb7d023f811fb5e4c06afac348ab.md)

[09.9.3 — Line Breaking, Hyphenation, Justification, Tabs, Leading, Columns & Frame Flow](09%209%203%20%E2%80%94%20Line%20Breaking,%20Hyphenation,%20Justification%203f19bb7d023f8131873bf7e767f782f6.md)

[09.9.4 — Caret, Selection, Hit Testing, IME, Clipboard, Bidi Editing & Text Commands](09%209%204%20%E2%80%94%20Caret,%20Selection,%20Hit%20Testing,%20IME,%20Clipb%203f19bb7d023f81c089b1ffa57ca16180.md)

[09.9.5 — Font Resources, Variable Fonts, OpenType, Embedding, Subsetting & Missing-Font Recovery](09%209%205%20%E2%80%94%20Font%20Resources,%20Variable%20Fonts,%20OpenType,%203f19bb7d023f81fea08adb846d1211fe.md)

[09.9.1 — Text Storage Model, Unicode Indices, Stories, Runs & Style Inheritance](09%209%201%20%E2%80%94%20Text%20Storage%20Model,%20Unicode%20Indices,%20Stor%203f19bb7d023f81b19633f520d9790a07.md)

[09.9.2 — Unicode Segmentation, Bidi, Script Runs, Font Fallback & HarfBuzz Shaping Contract](09%209%202%20%E2%80%94%20Unicode%20Segmentation,%20Bidi,%20Script%20Runs,%20%203f19bb7d023f81eb8d70c9bcc0fa93e4.md)

[09.9.3 — Line Breaking, Hyphenation, Justification, Tabs, Columns & Frame Flow](09%209%203%20%E2%80%94%20Line%20Breaking,%20Hyphenation,%20Justification%203f19bb7d023f817e9100c82e41ab6ac1.md)

[09.9.4 — Caret, Selection, Hit Testing, IME, Clipboard & Text Editing Commands](09%209%204%20%E2%80%94%20Caret,%20Selection,%20Hit%20Testing,%20IME,%20Clipb%203f19bb7d023f8125a27fe50536d0da4e.md)

[09.9.5 — Font Resources, Variable Fonts, Missing Fonts, Embedding, Subsetting & Text-to-Curves](09%209%205%20%E2%80%94%20Font%20Resources,%20Variable%20Fonts,%20Missing%20F%203f19bb7d023f813d80b9cdc050057335.md)