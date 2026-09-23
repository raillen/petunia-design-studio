# 08.26 — Typography, Text Editing, Layout & Precision UX Contract

<aside>
🔤

**Typography is a direct-manipulation subsystem, not a form full of font fields.** Text editing, shaping, layout, styles and frame flow use one coherent interaction model across canvas, Properties and dedicated panels.

</aside>

# Affinity baseline

Affinity separates Artistic Text, Frame Text and Text on a Path, while Character/Paragraph/Typography panels provide fine formatting. The Character panel exposes family/style/size, variable axes, decorations, kerning, tracking, baseline, scaling, OpenType and language controls.[[1]](https://affinity.help/designer2/English.lproj/pages/Text/text_general.html)[[2]](https://affinity.help/designer2/English.lproj/pages/Panels/characterPanel.html)

The 2026 Affinity learning path still teaches frame + artistic text, kerning/tracking/leading, variable fonts and reusable text styles.[[3]](https://www.canva.com/design-school/resources/master-typography-in-affinity/)

# Aubrieta text objects

- **Artistic Text** — content-driven bounds for display text;
- **Frame Text** — frame-driven layout and overflow;
- **Path Text** — baseline attached to a curve/path;
- **Text Flow** — links between frames when 10.6 enables it;
- Shape Text only when Functional Atlas scope explicitly includes it.

The object type is document semantics, not merely the active tool.

# On-canvas editing

Entering text edit preserves structural context while giving platform-correct caret, selection, clipboard and IME behavior.

Canvas may display:

- caret and range selection;
- frame edge;
- overflow indicator;
- baseline/path handles;
- text-flow in/out ports;
- local-style override indicator.

Escape is staged: close transient UI → leave range editing to object selection → leave tool. It never destroys text.

# Context toolbar

Only high-frequency controls live here: family/style, size, essential emphasis, paragraph alignment, fill, quick style and overflow/flow status. High-dimensional options stay in dedicated panels.

# Character panel

Canonical groups:

1. typeface and variable axes;
2. size/colour/background;
3. decorations;
4. spacing and positioning;
5. OpenType features;
6. language/script.

Unsupported font features are omitted or disabled with explanation rather than presented as meaningless switches.

# Paragraph panel

V1 fields follow 10.6: alignment/justification, leading, indents, paragraph spacing, tabs and supported frame/column options. Long-publication controls remain outside UI unless scope promotes them.

# Text Styles

Expose paragraph versus character styles, inheritance, local overrides, create from selection, apply, redefine, duplicate, rename and clear overrides. Redefine may show affected-object/text count before committing.

# Variable fonts

Each axis has human-readable name, numeric input, slider, default marker and reset. Registered/advanced axes may live in progressive disclosure.

# Missing fonts

Missing fonts are recoverable state:

- preserve original family/style identity;
- render with explicit fallback;
- badge object/layer/Character panel;
- Replace / Locate / map-all occurrences;
- do not overwrite stored identity merely because fallback rendered.

# Text on path

Affinity provides text-on-path workflows.[[4]](https://affinity.help/designer2/English.lproj/pages/Text/pathText.html)

Aubrieta requires precise start/end offset handles, side/orientation controls and numeric equivalents. Convert to Curves is explicit and destructive.

# Layout relationship

The 2026 Affinity combines layout work with pages, guides and print preparation.[[5]](https://www.canva.com/design-school/resources/design-layouts-in-affinity/) Aubrieta intentionally keeps light-publishing scope: Surfaces, margins, columns, guides, baseline grid, frame flow, bleed and SurfaceTemplate/Symbol composition.

# Tables

Current Affinity training includes tables, but Aubrieta does not inherit table UI by imitation. Tables require their own Product/Functional scope before UI exists.

# Performance and precision

- typing feedback must remain perceptually immediate;
- shaping/reflow may complete asynchronously only when visual consistency is preserved;
- font menus are virtualized;
- recent/favorite fonts supported;
- search-first keyboard font selection;
- preview samples must not stall list scrolling.

# Accessibility and international text

Text-edit control must provide normal platform semantics for caret, selection and IME. Test pt-BR, Latin diacritics, CJK, Arabic/RTL, Devanagari or another complex script, emoji/variation selectors, mixed scripts and long unbroken text.

# Canonical fixtures

Missing fonts, variable axes, path text, threaded frames, overflow, mixed styles, complex scripts, 10k+ glyph text, rich/plain paste, save/reopen and export fidelity.