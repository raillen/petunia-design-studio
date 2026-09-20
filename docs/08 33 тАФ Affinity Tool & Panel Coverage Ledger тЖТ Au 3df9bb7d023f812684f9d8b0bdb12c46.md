# 08.33 — Affinity Tool & Panel Coverage Ledger → Aubrieta Scope/Authority Matrix

<aside>
🧭

**Coverage ledger:** this page enumerates the official Affinity Designer 2 and Photo 2 tool/panel families used as prior art and maps them to Aubrieta authority/scope. It exists to prevent accidental omissions, not to copy Affinity feature scope wholesale.

</aside>

# Primary sources

Official Designer 2 help exposes the full Design Tools, Shape Tools, Text Tools, selection/retouch tools and panel inventory.[[1]](https://affinity.help/designer2/English.lproj/)

Official Photo 2 help exposes Photo editing, selection, fill, paint, erase, retouch, warp, RAW, export and analysis panels.[[2]](https://affinity.help/photo2/English.lproj/)

The September 2026 unified Affinity guidance is used only for modern Studio/workspace composition and cross-discipline flow.[[3]](https://www.canva.com/design-school/courses/affinity-essentials/)

# Status vocabulary

**ADOPT** — interaction family fits Aubrieta closely.

**ADAPT** — familiar model retained but Aubrieta changes semantics/UX.

**REFERENCE** — useful prior art; feature scope is Post-V1/Open/Not Yet Adopted.

**OUTSIDE CURRENT PRODUCT** — not part of current Aubrieta charter unless a future decision changes it.

The real implementation status remains the canonical 12.8 / 10.x requirement status.

# Designer — core design tools

| Affinity tool/family | Aubrieta treatment | Canonical authority |
| --- | --- | --- |
| Move | ADAPT — selection/transform grammar, clearer Auto Select/key-object/origin state | 08.23–08.24 + 10.1 |
| Node | ADAPT — same direct curve editing family with stronger snap provenance/a11y | 08.24/08.27 + 10.2 |
| Point Transform | ADAPT — fold into common Transform HUD/property model | 08.24 + 10.1 |
| Contour | ADAPT — live by default, explicit Bake, signed radius/degenerate warnings | 08.24 + 10.3 |
| Corner | ADAPT — live parameters, explicit Bake Corner Geometry | 08.24 + 10.2/10.3 |
| Pen | ADOPT/ADAPT — Pen/Smart/Polygon/Line-style modes, visible append/close-path state | 08.24 + 10.2 |
| Pencil | ADAPT — sculpt/autoclose/smoothing/stabilizer with live fitted-path feedback | 08.24 + 10.2 |
| Stroke Width | ADOPT/ADAPT — on-curve width points + numeric/interpolation clarity | 08.24 + 10.4 |
| Knife/Scissors | ADAPT — intersection preview; distinguish Split Object vs Break Path | 08.24 + 10.2 |
| Vector Brush | REFERENCE unless promoted; interaction documented without V1 promotion | 08.24 + 10.2/10.4 + 12.8 |
| Gradient | ADAPT — common stop editor, explicit interpolation/document-colour semantics | 08.24 + 10.4 |
| Vector Flood Fill / Smart Fill | ADAPT — bounded-region preview and explicit result semantics | 08.24 + 10.3 |
| Transparency | ADOPT/ADAPT — separate mode sharing gradient-stop componentry | 08.24 + 10.4 |
| Artboard | ADAPT into broader Surface model | 08.24 + 10.7 |
| Place | ADAPT — click/drag placement plus explicit Embed/Link policy | 08.24/08.29 + 09.11/10.x |
| Vector Crop | ADOPT/ADAPT — nondestructive default | 08.24 + 10.x |
| Shape Builder | ADAPT — candidate-region model, stronger candidate/commit distinction | 08.24 + 10.3 |
| Colour Picker | ADAPT — semantic colour/profile readout | 08.24/08.25 + 09.9/10.4 |
| Style Picker | ADAPT — Property Schema groups + affected-property preview | 08.24 + 09.25 |
| Measure / Area | ADOPT as transient view tools; persistent annotations require explicit scope | 08.24/08.27 |
| View / Zoom | ADOPT creative-tool convention with Aubrieta viewport contracts | 08.6/08.23 |

# Parametric shape family

Aubrieta V1 requires Rectangle, Rounded Rectangle, Ellipse, Polygon and Star according to 10.3. These share one semantic Shape Tool family with shape-specific parameters/handles and Convert to Curves.

Affinity specialty tools — Triangle, Diamond, Trapezoid, Double Star, Square Star, Arrow, Donut, Pie, Segment, Crescent, Cog, Cloud, callouts, Tear, Heart, Spiral, QR and novelty primitives — are **REFERENCE / POST-V1 CANDIDATE** unless explicitly promoted. Their existence in Affinity is not an argument to copy them all.

When a specialty primitive is promoted, it must define its own versioned parameter schema, canvas handles, degeneracy rules, accessibility labels and fixtures before UI exposure.

# Text tools

| Affinity tool/family | Aubrieta treatment | Authority |
| --- | --- | --- |
| Artistic Text | ADOPT/ADAPT | 08.26 + 10.6 |
| Frame Text | ADOPT/ADAPT with light-publishing flow | 08.26 + 10.6–10.7 |
| Text on Path | ADOPT/ADAPT; precise start/end/orientation handles | 08.26 + 10.6 |
| Shape Text / advanced publishing | REFERENCE unless promoted by Functional Atlas | 10.6 + 12.8 |

# Photo — selection / paint / retouch

| Affinity tool/family | Aubrieta treatment | Authority |
| --- | --- | --- |
| Selection Brush | ADOPT/ADAPT — Add/Subtract, edge snap, source scope, Refine | 08.31 + 10.9 |
| Flood Select | ADOPT/ADAPT — New/Add/Subtract/Intersect, tolerance HUD | 08.31 + 10.9 |
| Marquee family | ADOPT — shared selection modes/feather grammar | 08.31 + 10.9 |
| Object/Subject ML selection | REFERENCE / POST-V1 unless explicitly promoted | 08.8 + 10.9 + 12.8 |
| Paint Brush | ADAPT — shared brush grammar plus explicit writable target/destructiveness | 08.31 + 10.9 |
| Pixel Tool | REFERENCE/ADOPT only if pixel-precision workflow is in scope | 10.9 |
| Erase / Background Erase / Flood Erase | ADAPT — distinguish destructive pixel erase from mask workflows | 08.31 + 10.9 |
| Clone | REFERENCE / Post-V1 unless promoted; UX fully documented | 08.31 + 10.9 |
| Healing / Patch / Blemish / Inpainting | REFERENCE / Post-V1 unless promoted; target/destructiveness contract retained | 08.31 + 10.9 |
| Dodge / Burn / Sponge | REFERENCE / Post-V1 unless promoted | 08.31 + 10.9 |
| Blur / Sharpen / Median / Smudge brushes | REFERENCE / Post-V1; prefer nondestructive filter path in V1 where available | 08.31 + 10.9–10.10 |
| Crop/Straighten | ADAPT — nondestructive default, explicit resample | 08.31 + 10.9 |

# Photo — nondestructive editing and analysis

Adjustment layers, Live Filters, masks, channels, Histogram, Brushes, Navigator/Info/Scopes and selection refinement are **ADOPT/ADAPT interaction families**. Actual adjustment/filter catalog exposure is generated from implemented descriptors and never promises unsupported engines. See 08.32 + 10.10.

Develop/RAW, Liquify, Panorama, HDR, astrophotography, focus merge, macros/batch-photo and other specialized Photo workspaces are **REFERENCE / OUTSIDE CURRENT PRODUCT** unless the Product Charter and Functional Atlas later promote them.

# Panels — Designer and Photo coverage

**Directly mapped:** Layers, Appearance, Colour, Swatches, Stroke, Transform, Assets, Symbols, Character, Paragraph, Typography, Text Styles, History, Navigator, Brushes, Channels, Histogram and relevant export panels.

**Conditional/reference:** Constraints, Isometric, 32-bit Preview, Scopes, Sources, Metadata, Macros, States, Stock and specialized Persona panels. A panel is introduced only when a persistent mental model/browsing workflow justifies it; Properties remains the default home for selection-local state.

# Export tools

Affinity Slice/Export Persona is prior art for multi-output regions. Aubrieta does not require a separate Persona; Quick Export, Surface/export metadata and Batch Export can expose the same capability with less mode switching. See 08.29 and 08.10.

# Coverage gate

When Affinity research introduces a new reference tool, update this ledger before citing it in an implementation dossier. If an Aubrieta feature has no mapped authority here or in 08/10, the agent must determine whether it is genuinely new Aubrieta design or undocumented scope drift.

# Freshness

This ledger records the 2026 research baseline. Affinity documentation may evolve; new upstream behavior is evaluated by evidence and user benefit, never auto-imported.