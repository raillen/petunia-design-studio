# 09.8 — Typography, Text Editing & Layout Engine

# Canonical text model

Text content, spans, character style refs, paragraph style refs, frame geometry, flow links and path attachment are document semantics. Glyph runs are derived.

# Stack boundary

Parley/HarfRust/Skrifa/Fontique/ICU4X adapters sit behind `aubrieta_text`; external types do not leak into document serialization.

# Pipeline

Content + styles + locale/script + font resolution → shaping → line breaking/layout → glyph runs → bounds/hit maps → render/export.

# Font resolution

Resolve explicit family/style, embedded/linked fonts, variable axes, fallback chain and missing-font substitution policy. Preserve requested font identity even when fallback displays temporarily.

# Editing

Caret, grapheme-aware movement, word/line navigation, multi-range future-proofing, selection, IME composition, clipboard rich/plain policies and BiDi visual/logical navigation are explicit contracts.

# Text frames

Inset, columns, vertical alignment, overflow state, linked frame flow, exclusions/wrap and baseline grid. Layout dependency propagates through linked frames deterministically.

# Text-on-path

Stores text/path relationship and offset/alignment; editing path or content reflows without destructive conversion.

# OpenType

Feature registry, kerning, ligatures, small caps, numeral styles and variable font axes represented semantically, unsupported font feature gracefully ignored with UI state.

# Export

PDF/SVG choose text-preserving vs outlines based on adapter capability/user option; document never silently converts canonical text.

# Tests

Unicode scripts, BiDi, emoji/fallback, combining marks, IME, variable fonts, missing fonts, linked overflow, extreme tracking/leading and save/reopen fidelity.

# Canonical text entities

Aubrieta should separate **text content** from **text containers** so linked frames and multiple presentations do not duplicate strings.

Recommended V1 semantic model:

```
TextStory
├── id: TextStoryId
├── utf8_content
├── character_runs: Vec<CharacterStyleRun>
├── paragraph_runs: Vec<ParagraphStyleRun>
├── inline_objects / anchors
├── language/script metadata where explicitly authored
└── extension metadata

TextObject / TextFrame / TextOnPath
└── references TextStoryId + container-specific geometry/layout properties
```

Artistic text may own a dedicated story; linked text frames share one story and define a flow chain. Glyphs, line fragments and frame-assigned text ranges are derived.

# Text position contract

Canonical persisted text is valid Unicode UTF-8. Internal persistent ranges use a documented `TextOffset` representation that is valid only on Unicode scalar boundaries; V1 recommendation is **UTF-8 byte offsets** because they map directly to Rust strings and serialize compactly. APIs never accept arbitrary integer offsets without boundary validation.

Grapheme clusters, words, BiDi visual runs and line boundaries are **derived navigation units**. User-facing caret movement must never split a grapheme simply because the storage offset is a byte offset.

Every edit produces a range transform/map so style runs, inline anchors, selections and IME state can be rebased deterministically within the transaction.

# Style-run invariants

Character and paragraph styles use non-overlapping normalized runs over a story revision. Adjacent equivalent runs are merged. Empty runs are removed unless they carry an explicit insertion-style semantic.

A run may contain:

- reference to reusable style ID;
- local overrides as typed properties;
- language/script override;
- OpenType/variable-axis properties.

The canonical model stores requested semantic values, not shaped glyph IDs.

# Editing buffer implementation boundary

The storage contract does **not** require the canonical document to expose a specific rope/piece-table crate. The active text editor may maintain a rope/piece-table/edit buffer optimized for interactive insertion and then synchronize through text commands/transactions. Third-party buffer types never leak into the document schema.

If large-text profiling justifies a rope inside `aubrieta_text`, it remains an implementation detail behind the same story/range contracts.

# Text edit transaction

A continuous typing/composition session is an editing transaction with explicit coalescing boundaries. Semantic operations include:

- insert/replace/delete range;
- apply character property/style;
- apply paragraph property/style;
- split/join paragraph;
- insert/remove inline object;
- update frame/story relationship.

Commands validate story revision/range boundaries and emit text-specific ChangeSet invalidation. Typing history coalescing ends on selection jump, style-changing command, focus/story change or explicit transaction boundary.

# IME state ownership

IME composition text, candidate state and platform composition ranges are **session/editor state**, not canonical document content until committed. Composition preview participates in render/layout preview but is excluded from explicit save/autosave committed revision. Cancel restores the exact pre-composition story state.

Candidate-window positioning derives from current caret geometry in viewport coordinates through the GUI/platform adapter; the document engine never depends on GPUI or OS IME types.

# BiDi and caret semantics

Maintain both logical story offsets and derived visual caret positions. Arrow-key behavior follows platform/editor conventions using the derived BiDi layout; Home/End and word movement are specified/tested separately from raw storage order. Selection retains logical ranges even when painted as multiple visual rectangles.

# Font identity model

Document text stores a semantic `FontRequest`, not a path to a system file as the only identity. It should include as applicable:

- requested family;
- style/weight/stretch;
- PostScript/full-name metadata when available;
- variable font axis values;
- embedded/linked resource reference where present;
- substitution/fallback policy metadata.

Resolved physical font face + file fingerprint is derived/session state unless embedded as a document resource.

# Font resolution order

Recommended deterministic order:

1. explicitly embedded font resource permitted for use;
2. explicitly linked document font resource;
3. matching installed/system/user font according to Fontique/platform provider;
4. Aubrieta configured fallback chain by script/language;
5. last-resort font with visible missing-font diagnostic.

Fallback display **never overwrites the requested font identity**. Reopening after the missing font becomes available can restore intended appearance automatically.

# Font licensing/embedding

Font embedding/export must respect embedding permission metadata where technically available and the export format contract. Aubrieta should not silently package a restricted system font into `.aubrieta` or PDF. When embedding is disallowed, preserve font request + diagnostic and use a documented external/reference/substitution strategy.

The format/resource index records whether an embedded font was intentionally embedded and its provenance/license metadata when available.

# Shaping contract

Shaping input is explicit: Unicode text slice, script/language, direction, font face/size/variation axes, features and cluster mapping policy. HarfRust/Skrifa outputs are adapted into Aubrieta-owned glyph-run structures containing enough mapping to support:

- caret hit testing;
- selection painting;
- text extraction;
- export;
- missing-glyph diagnostics.

Glyph IDs are never persisted as canonical text meaning.

# Line breaking and paragraph layout

Paragraph layout combines Unicode line-breaking opportunities, authored paragraph properties and frame geometry. Required semantics include:

- alignment/justification;
- indents/tabs;
- spacing before/after;
- leading/line-height policy;
- columns;
- baseline grid interaction;
- first-line behavior;
- explicit line/paragraph breaks;
- overflow indicator.

Advanced keep/widow/orphan behavior is only active if marked by Functional Atlas status; UI must not advertise it ahead of implementation.

# Linked text-frame flow

A flow chain is explicit and acyclic. V1 rule: a TextFrame has at most one incoming and one outgoing flow link. Creating a cycle or second ambiguous predecessor is rejected.

Layout proceeds deterministically from chain head through frames. A change to content/style/frame geometry invalidates the changed frame and all downstream frames until layout stabilizes. Upstream unaffected frames remain cached when their assigned range and metrics stay valid.

Frame-assigned story ranges are derived, never persisted as independent truth that can disagree with story content.

# Overflow semantics

A frame exposes derived `OverflowState`: `Fits`, `FlowsToNext`, or `Overset`. Overset text remains fully canonical in the story and is never discarded because it is not currently visible. The UI shows an overflow affordance and automation/preflight can query it.

# Text-on-path

Store `TextStoryId`, `PathReference`, start offset, side/orientation/alignment and baseline offset. Path geometry is referenced semantically; layout produces derived glyph transforms. Deleting the source path requires explicit detach/convert/reject policy rather than dangling reference.

# Text hit testing

Text layout produces a hit map from viewport/document point to nearest valid caret position, including BiDi affinity. Hit testing operates on grapheme/caret boundaries, not arbitrary glyph rectangle centers.

# Selection and clipboard

Creative-text selection can expose plain text plus Aubrieta rich semantic clipboard representation. Cross-application clipboard advertises standard rich/plain flavors where platform permits; paste negotiates the richest safe compatible representation. Pasting external rich content goes through an importer/sanitizer rather than injecting foreign markup into the document.

# Performance requirements

- ordinary typing/caret feedback must remain on the interactive latency path and must not wait for font scanning/network/filesystem work;
- font discovery/indexing occurs asynchronously and returns stable snapshots/deltas;
- large font menus are virtualized by UI but resolution/search data comes from a non-GUI service;
- linked-frame reflow should stop as soon as downstream assigned ranges/metrics converge;
- shaping/layout caches key by text/style/font/context fingerprints, not UI object identity.

# Failure behavior

Missing font, corrupt font, unsupported OpenType table or shaping/layout failure returns scoped diagnostics and fallback rendering where safe. A malformed font is a hostile-input boundary and cannot panic/overflow the app. Export preflight escalates missing/substituted fonts according to output fidelity policy.

# Additional gauntlets

- grapheme movement across emoji ZWJ sequences/combining marks;
- Arabic/Hebrew mixed BiDi with Latin/numbers;
- IME composition commit/cancel while frame reflows;
- style run rebasing under randomized edits;
- missing font appearing/disappearing during session without rewriting requested identity;
- variable-font axis save/reopen;
- linked-frame chain edits and cycle rejection;
- overset text preserved through save/export/import;
- font substitution visible in diagnostics/preflight;
- malformed/adversarial font fixture handling.