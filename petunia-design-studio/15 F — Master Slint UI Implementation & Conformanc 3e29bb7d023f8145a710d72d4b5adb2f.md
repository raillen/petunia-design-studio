# 15.F — Master Slint UI Implementation & Conformance Contract

<aside>
🧭

**Purpose:** canonical implementation prompt/contract for the Slint shell. It is normative where it restates accepted Petunia contracts; it does not promote Post-V1 features.

</aside>

# 0 — Mission

Implement or refactor the complete Petunia Design Studio desktop interface in Slint so the application behaves like a professional creative tool rather than a generic dashboard. Conform to sections 08, 10, 14 and 15; do not treat screenshots as the source of domain truth.

# 1 — Non-negotiable product identity

- Product name: Petunia Design Studio.
- Native project extension: .PTND.
- Personas: Design and Photo.
- Core: Rust, UI-agnostic.
- Primary UI adapter: Slint.
- GUI bridge: PetuniaDesignGuiBridge.
- Resource/action namespace: ptnd.* for new public IDs.
- Aubrieta/GPUI names are legacy/historical unless migration explicitly needs them.

# 2 — No fake UI

No visible control may exist without a real semantic behavior. Every button, icon, menu item, submenu, checkbox, slider, dropdown, field, splitter, panel tab, tree affordance and shortcut is either wired to a real Action/Tool/PropertyEdit, intentionally disabled with a reason, or absent.

# 3 — Command lane

Slint callback → semantic ActionRequest/ToolRequest/PropertyEdit → Application → Command/Transaction → canonical state → ChangeSet → presentation delta → Slint. Never mutate document storage directly from .slint or UI adapter code.

# 4 — App shell geometry

- Menu bar: 28 logical px.
- Brand/Persona bar: 40 px.
- Context toolbar: 34 px.
- Document tabs: 30 px.
- Status bar: 26 px.
- Left tool rail: 44 px Comfortable, 40 Compact, 48 Spacious.
- Right dock default: 304 px; min 240; preferred 280–360; max 520.
- Bottom dock: hidden by default in Design; when visible preferred 180–280 px.
- Splitter: 1 px visible, 6 px hit target.
- Canvas remains the dominant flexible region.

# 5 — Default Design workspace

Right Stack A: Color | Swatches | Stroke | Appearance.

Right Stack B: Layers | Properties.

Right Stack C: Transform | Align | Navigator | History.

Left rail: Selection, Node, Surface, drawing, content, editing, navigation groups.

Optional bottom: Assets, History, Data Merge, Background Tasks.

# 6 — Default Photo workspace

Right Stack A: Color | Swatches | Brushes | Brush Settings.

Right Stack B: Layers | Properties | Masks.

Right Stack C: Adjustments | Histogram | Channels | Navigator | History.

The document, selection and canonical layer hierarchy persist across Persona switches. No implicit rasterization.

# 7 — Visual language

Quiet professional graphite UI, low chroma, flat persistent docks, subtle separators, elevated temporary surfaces, restrained Bloom-purple global accent, Design cyan and Photo coral only for Persona identity. No glassmorphism, neon gamer styling or dashboard cardification.

# 8 — Core tokens

surface.workspace #202124; [surface.chrome](http://surface.chrome) #27282B; surface.chromeStrong #222326; surface.panel #303236; surface.panelRaised #373A3F; surface.control #3A3D42; controlHover #464A50; controlPressed #50545B; border.subtle #41444A; border.strong #555A63; text.primary #F2F3F5; text.secondary #C2C6CC; text.tertiary #8E949D; text.disabled #666C75; accent.bloom #B77AFF; accent.bloomHover #C48FFF; accent.bloomPressed #9F5FEA; [studio.design](http://studio.design) #35C7D4; [studio.photo](http://studio.photo) #F06C8D; success #55C58A; warning #E3AE52; error #E76573.

# 9 — Typography

Caption 11/14; Small 12/16; Body 13/18; Body Medium 13/18 weight 500; Panel title 13/18 weight 600; Dialog section 15/20 weight 600; Dialog title 18/24 weight 600. Numeric values use tabular figures.

# 10 — Spacing/radius

4 px spacing grid. Common: 4, 8, 12, 16, 20, 24, 32, 40. Radius: 3 micro, 5 control, 7 popover, 10 modal/floating palette. Dock boundaries normally square.

# 11 — Icon system

Use semantic IconId. General icons are sourced from Lucide/Tabler through an adapter/build step; custom Petunia SVGs own ambiguous domain tools. Standard inline glyph 16 px, toolbar 18 px, rail 20 px. Normalize optical size/stroke. Do not bind product meaning to a raw file path.

Required semantic families include file/new/open/save/export/print; edit/undo/redo/cut/copy/paste; selection; node/path; shape; pen/pencil; corner/contour/knife; boolean operations; fill/gradient/transparency; text; place image; surface; hand/zoom; layer/group/mask; visibility/lock; align/distribute; color/stroke; history; navigator; asset; data merge; photo brush/eraser/crop/clone/heal; adjustment/filter/channel; warning/error/help/settings.

# 12 — Menu bar

Canonical top-level Design order: File, Edit, Object, Layer, Select, Text, View, Window, Help. Photo may contribute Image and Filters at documented positions. Menu row 28 px; icon column 20; 8 px horizontal padding; shortcut right-aligned; submenu chevron 12 px; semantic separators only.

File must cover New, Open, Open Recent, Close, Save, Save As, Place/Import as scope permits, Export, Print when supported, Document Setup, Exit/Quit. Edit covers Undo/Redo, Cut/Copy/Paste, Duplicate, Delete, Select/Preferences where platform convention places them. Object/Layer/Select/Text expose V1 actions from registries rather than duplicated UI logic.

# 13 — Menu interaction

Mouse hover opens submenu after roughly 180 ms. Keyboard navigation must support arrows/Enter/Escape and mnemonic/platform conventions where available. Outside click closes. Disabled standard actions can remain visible for discoverability but expose a disabled reason. No essential action exists only in context menu.

# 14 — Persona/brand bar

Leading: Petunia flower mark 24 px in 32 px hit area. Persona segmented control height 30 px inside 40 px row. Design active uses [studio.design](http://studio.design); Photo active uses [studio.photo](http://studio.photo); global focus ring remains Bloom accent. Next: Workspace Profile dropdown. Trailing: Undo/Redo, optional snap/display quick controls, Export, Command Palette/Search and overflow based on width.

# 15 — Context toolbar

Exactly one 34 px row. Contents are driven by active tool and selection. Order: scope/target → tool mode → primary numeric parameters → behavior toggles → local snap/constraint → destructive commit only if semantically required → overflow. Equivalent properties keep the same label, unit handling and component family across tools.

# 16 — Document tabs

30 px target. Active tab: stronger foreground and subtle raised graphite. Tab always communicates name and dirty state. Close button appears on active/hover but does not move the label. Drag reorder. Context menu: Close, Close Others, Close Right, Duplicate View, Reveal in File Manager, Move to New Window as supported. Overflow before illegibility.

# 17 — Tool rail

Icon-only. Button 34×34 Comfortable. Hover tooltip includes tool name, shortcut and one sentence. Click activates last-used member; long press/right click opens grouped tools. Active state uses background + icon cue, not color alone. Dragging from the rail is not required unless a tool explicitly supports it.

# 18 — Canvas

Canvas/pasteboard must visually dominate. Paper/Surface sits above a neutral dark workspace. Document content is never tinted by UI theme. Selection outlines, nodes, handles, guides, snapping and measurement use dedicated overlay tokens with contrast adaptation. Canvas accepts pointer, pen, keyboard and navigation gestures through normalized input.

# 19 — View navigation

Space/temporary Hand, pan, wheel zoom, pinch where platform exposes it, Fit, 100%, selection fit and Surface fit follow the command system. Temporary navigation does not dirty document or create Undo entries. Status bar and cursor reveal temporary mode.

# 20 — Status bar

26 px target. Left: document/surface/zoom/color-profile summaries where useful. Center: current tool hint, modifier alternatives, transient operation values or selection summary. Right: Snap, Guides, Grid, Proof Colors, pixel preview and background-task indicator according to capability. Critical errors do not live only here.

# 21 — Panel shell

Tab strip 32 px; inner padding 8; section header 28; property row 30–32; local search row 30. Panel header icons have tooltips and accessible names. Avoid generic ellipsis when one or two meaningful actions fit visibly.

# 22 — Docking

Support left/right/bottom zones and tab stacks. Each panel declares min/preferred size and float eligibility. Drag shows source ghost and legal target overlay; invalid target uses forbidden cursor; Esc restores original. Splitter drag is live; min sizes are hard; double-click restores preferred ratio. Workspace state is not document state.

# 23 — Layers panel

Row 28 px. Disclosure 16; thumbnail/type 20; indent 16 per level; lock/eye hit areas 28. Columns: disclosure → thumbnail/type → label → semantic badges → lock → visibility. Selection is synchronized to canonical selection state. Inline rename with F2/slow double click; Enter commit; Esc revert. Drag distinguishes reorder, reparent and clip/mask targets.

# 24 — Layers semantics

Visibility must never toggle selection. Lock prevents viewport edit/pick according to core contract while remaining manageable in Layers. Multi-selection handles Ctrl/Cmd toggle and Shift range where valid. Large documents use a virtualized/incremental model. Hover actions cannot shift label geometry.

# 25 — Properties

Contextual but structurally stable. Sections: Transform, Appearance, Fill, Stroke, Opacity/Blend, Effects, constraints/alignment and object-specific parameters. Multi-selection shows Mixed rather than fabricated values. Property edits use typed schemas and transactions. Reset/revert appears consistently.

# 26 — Transform

Fields X/Y/W/H/Rotation/Skew as supported, 3×3 origin anchor, aspect lock, relative/absolute mode, flip H/V. Numeric fields parse document units and explicit typed units. Scrubbing label is allowed. Drag/edit commits as one history transaction where appropriate.

# 27 — Color

Supports Fill/Stroke dual swatches, swap/default/none, semantic color-model selector and precision fields. RGB may expose hex; CMYK/Lab/Gray use their own channel semantics. Color wheel/triangle is an optional visual editor, never the only exact input path. Profile context remains visible when meaningful.

# 28 — Swatches

Document/library/recent groups, search/filter, grid/list when useful. Apply Fill/Stroke from context. Create/edit/duplicate/delete with stable resource IDs. Drag/drop targets are explicit. Imported libraries and unavailable resources show state rather than silently substituting.

# 29 — Stroke

Width, alignment, cap, join, miter, dash, markers, pressure/width profile according to V1 engine scope. Compact preview can visualize stroke. Unsupported selection disables relevant fields rather than performing no-op.

# 30 — Appearance

Ordered appearance/effect stack with stable item IDs. Add, reorder, duplicate, enable/disable, delete and inspect. Distinguish disable from delete. Destructive Expand/Bake is explicit and never conflated with hiding an effect.

# 31 — Transform/Align/Navigator/History stack

Transform is precision state; Align exposes engine-owned reference targets and disabled predicates; Navigator shows thumbnail + viewport rect + zoom; History shows transactional entries and current marker only to the depth supported by the actual history model.

# 32 — Assets

Browser supports categories, search, list/grid, thumbnails, import and drag/drop. Thumbnail generation must be asynchronous/incremental and cancellable. Drag indicates valid canvas/layer/property targets. No eager decode of every asset at startup.

# 33 — Typography

Font family searchable combobox, style, size, leading, tracking, kerning, baseline, OpenType access, language, color and paragraph controls. Font results must be virtualized and preview generation must not block typing/canvas. Missing fonts surface in document diagnostics.

# 34 — Paragraph/Text Styles

Paragraph: alignment/justification, indents, spacing, first-line, tabs and text-frame columns per V1. Text Styles: character/paragraph styles, create/duplicate/update/redefine/clear overrides. Style override state must be explicit.

# 35 — Pages/Surfaces

Thumbnail + name + dimensions + status. Add, reorder, duplicate, rename, resize, multi-select and export flags. Layout can list/grid. Surface is canonical document data; panel selection does not invent a parallel surface list.

# 36 — Pathfinder

Expose only V1 operations registered by the functional engine. Differentiate live/non-destructive boolean from destructive expand/bake. Each command has can_execute, tooltip and editability consequence. Do not display a larger Affinity-like set than the engine actually supports.

# 37 — Data Merge

Source selector, field catalog, record table, binding state, preview record, validation, generation options and errors. Large record tables are virtualized. No data generation action bypasses command/job contracts. Errors identify record/field/binding when possible.

# 38 — Photo panels

Histogram updates asynchronously and must not stall paint/input. Adjustments is a catalog that creates real nondestructive adjustment objects. Channels exposes process/alpha/mask semantics supported by core. Brushes is a preset browser; Brush Settings edits actual brush engine parameters. Masks owns mask-specific density/feather/invert/refine controls only when implemented.

# 39 — Resources/Links

Show linked/embedded status, path/provider, missing/outdated state and actions Locate/Relink/Update/Embed where supported. Never silently replace missing content. File paths are presentation; stable resource identity remains canonical.

# 40 — Background Tasks

Each job row has title, phase/progress, source/target summary, cancel where safe, retry only for retryable failure, result/reveal action and diagnostic details. Completed trivial jobs may disappear after a short interval; errors persist until acknowledged/resolved.

# 41 — Search/Command Palette

Default Ctrl/Cmd+K. Width about 560 px, max height around 480 px, positioned near upper center. Search Actions, Tools, Files/Recent, Settings and Help. Results display icon, title, category and shortcut. Arrow navigation, Enter execute, Esc close. Search is fuzzy but exact semantic matches rank first.

# 42 — Tooltips

Initial delay ~500 ms, subsequent nearby 100–150. Max width 320. Title 12–13 medium; description 11–12. Shortcut aligned or included on title row. Disabled tooltip explains unmet precondition when non-obvious.

# 43 — Buttons

Primary reserved for important confirmation; Secondary ordinary; Tertiary/Ghost for chrome; Destructive semantic danger; Icon-only requires tooltip/accessibility name; Split button only when default action and variants truly coexist. Busy protects non-idempotent actions from duplicate activation.

# 44 — Numeric fields

Height 32 Comfortable, 28 Compact. Click edits; double-click selects; Enter commits; Esc restores edit baseline; arrows step; Shift fine adjustment; label scrub where expert-beneficial. Units explicit. Invalid value never enters canonical state. NaN/Infinity are rejected unless a domain contract explicitly models them.

# 45 — Dropdown/combobox

28–32 px row; chevron 12; popup constrained to work area; opens above when required. Keyboard arrows/Home/End/typeahead/Enter/Esc. Long lists add search and virtualization. Selected item uses check/state, not only color.

# 46 — Slider/scrubber

Track 4 px; thumb ~12 px; visible exact value for professional properties. Keyboard adjustments. One drag is one history transaction if document mutation. Scrubber cursor changes and status hint shows fine-adjust modifier.

# 47 — Tabs

Panel tabs 32 Comfortable; document tabs 30; segmented controls are not tabs. Active state uses text plus indicator/fill. Tab overflow preserves access; drag reorder where contract allows.

# 48 — Search fields

Leading search icon, clear button when nonempty, Esc clears before closing transient search, result count/filter chips when valuable. Debounce only expensive queries; local in-memory small filters should feel immediate.

# 49 — Toast/banner/error

Toast only for ephemeral nonblocking outcomes. Actionable persistent error uses banner/panel/dialog according to severity. Error message states what happened, impact and next action. Never show raw panic/backtrace as primary user message.

# 50 — New Document

Large dialog/sheet, preferred 720×560 logical px, responsive downward. Left preset categories/templates; center preset cards/list; right document settings or lower responsive inspector. Fields: dimensions, units, orientation, surfaces/pages as scope permits, color model/profile, raster resolution metadata, bleed/margins when relevant. Create primary; Cancel secondary. Keyboard and validation required.

# 51 — Open/Save

Use native platform dialog behind adapter. Unsaved/overwrite state remains application logic. Save As defaults .PTND. A failed save keeps document dirty and prior valid project whenever safe-write strategy permits.

# 52 — Unsaved Changes

Clear document name and choices Save, Don’t Save/Discard, Cancel; never ambiguous Yes/No. For multiple documents, use multi-document close review rather than repeated surprise dialogs where supported.

# 53 — Import Interpretation

Shown only when import has meaningful choices. Explain editability, text/layer preservation, color/profile interpretation and rasterization consequences. Options the backend ignores must not be displayed.

# 54 — Missing Links/Fonts

Nonmodal manager preferred for multiple issues. Table/list with item, usage/location, status, replacement/relink action and batch operations. Preserve document editability while clearly showing degraded state.

# 55 — Export

Large sheet/window, roughly 760–960×600–720 depending preview. Left target/preset/format; center preview when meaningful; right format settings; bottom destination/name/summary/actions. Export is job-backed when expensive. Validation precedes file mutation.

# 56 — Preferences

Searchable categories: General, Appearance, Canvas, Input, Performance, Files, Color, Plugins, Shortcuts, Language, Updates only if capability exists. 760×560 minimum-ish target. Changes apply immediately only when reversible/safe; others use Apply/Restart notice.

# 57 — Workspace Manager

List built-in/user workspaces. Actions Save As, Update, Rename, Duplicate, Delete user, Reset built-in, Import, Export. Built-ins cannot be accidentally destroyed. Import validates schema and missing panels/providers.

# 58 — About/Third-party notices

About is minimal: Petunia logo/name, version/build/channel, license, website/help, notices. No fake updater control. Third-party notices searchable by component, version, license and source.

# 59 — Universal tool state machine

Inactive → Ready → Hover Target → Armed → Dragging/Drawing/Editing → Preview → Commit → Ready. Optional Candidate Selection, Numeric Entry, Suspended by Pan, Recoverable Error and Cancel/Rollback. Slint renders state; tool controller owns it.

# 60 — Move/Select

Click select, modifier add/toggle, marquee from empty canvas, selection cycling policy visible, Auto Select state explicit. Drag moves selected object(s) with snapping/constraints. Transform origin is visible when enabled. Numeric takeover edits delta/absolute values according to command contract. Esc restores baseline before commit.

# 61 — Node

Selected path exposes nodes, Bézier handles and segment roles. Node types use shape + icon/semantics, not color alone. Supports engine-approved convert/smooth/cusp/join/break/close/reverse actions. Hover shows precise target; topology-changing actions preview counts when useful. Dense-node rendering must stay responsive.

# 62 — Pen

Modes only if engine owns them: standard Pen, polygon/line/smart variants as V1 scope dictates. Click creates nodes; drag creates handles; Rubber Band preview; close-path target appears around first node; append-to-current-path state is visually obvious. Backspace/Delete and Esc semantics are documented and tested. Numeric/constraint modifiers show in status.

# 63 — Pencil/Freehand

Continuous pointer/pen samples become fitted editable geometry through the engine. UI exposes supported smoothing/stabilizer/controller parameters. Diagnostic distinction exists between sampled stroke and fitted result. Input-to-preview latency has a budget and must not be blocked by unrelated panel recomputation.

# 64 — Shape tools

Rectangle, Ellipse, Polygon, Star, Line and supported custom shapes share creation grammar: press/click begin, drag preview, modifiers for center/proportion where documented, numeric dimensions available, commit once. Context toolbar shows shape parameters without forcing conversion to curves.

# 65 — Corner

Semantic corner handles. Radius drag and numeric takeover. Corner type control only includes supported types. Bake is explicitly named **Bake Corner Geometry** and is visually separated from reversible parameter editing.

# 66 — Contour

Signed offset/radius with visible zero crossing, join/cap/fill controls according to engine. Preview must warn about degeneracy/overflow rather than silently generating invalid geometry. Bake is explicit.

# 67 — Knife/Scissors

Crosshair/knife cursor. Preview intersections before commit. Distinguish Split Object from Break Path. Invalid intersection/selection yields disabled reason or recoverable message, not no-op.

# 68 — Gradient

Canvas geometry and stop editing are separated. Stops show selection, position, opacity and semantic color. Context toolbar/panel exposes type, angle/position, reverse and interpolation settings actually supported. Stop drag is one undo transaction per logical operation.

# 69 — Transparency

Reuses gradient-stop interaction component but writes transparency semantics. UI styling distinguishes target from Fill gradient while preserving learned interactions.

# 70 — Text tools

Artistic Text and Frame Text use correct text-edit focus contexts. While editing text/IME, global single-letter tool shortcuts are suppressed. Context toolbar shows font/style/size/alignment/style summary. Text frame resize versus content editing has distinct cursor/selection state.

# 71 — Surface tool

Creates/selects/resizes Surfaces with dimensions/bleed/margins metadata as scope dictates. Canvas overlay distinguishes surface boundary, bleed, margins and selected surface. Reordering belongs to Pages/Surfaces panel rather than arbitrary z-order behavior.

# 72 — Place Image

File selection is platform request. Placement shows preview/cursor, click/drag placement depending contract, preserves link/embed policy and color-profile metadata. Decode/import errors do not create half-valid objects.

# 73 — Fill/Eyedropper

Fill control targets Fill or Stroke explicitly. Eyedropper cursor shows sample target; status/Color panel reports semantic result. Sampling behavior across color-managed canvas is documented; tool must not silently alter unrelated paint target.

# 74 — Hand/Zoom

Navigation-only state. Temporary override returns to prior tool. Zoom centers predictably on cursor/selection according to View contract. No document mutation/history entry.

# 75 — Photo Brush

Cursor displays outer size and hardness/falloff cues when supported. Pressure/tilt input passes through normalized semantic events. One stroke is one transaction. Paint updates canvas incrementally; panels/histogram use background/incremental refresh and must not stall stroke.

# 76 — Eraser

Shares brush grammar but exposes erase semantics. Must respect active pixel layer/mask target and non-destructive architecture. Disabled when target cannot accept raster edits, with explanation.

# 77 — Crop

Crop overlay uses rule-of-thirds/grid options where supported, handles with enlarged hit areas, numeric dimensions/aspect presets, rotate/straighten only if engine owns them. Commit/cancel explicit; crop should remain non-destructive if the functional contract says so.

# 78 — Clone/Heal/Inpainting family

Only expose tools marked V1 in Functional Atlas. Source-point state must be obvious, cursor shows source offset relationship, missing source cannot silently paint. Expensive evaluation uses incremental preview/jobs when necessary.

# 79 — Adjustments/live filters

Creation catalog yields actual nondestructive document objects. Parameter panels derive from typed descriptors. Preview toggle must not delete source. Bake/rasterize is explicit and separated. Long evaluation can show busy state without freezing shell.

# 80 — Undo/Redo

Top controls, Edit menu and shortcuts use one history command. Disabled state reflects actual stack. UI preview/transient tool state never contaminates committed history. Continuous drag/stroke groups events into one logical history item.

# 81 — Selection consistency

Canvas, Layers, Properties and automation consume one canonical selection owner. UI may maintain hover/local focus only. Deleting an object invalidates stale selection IDs. Persona switching preserves selection where semantically valid.

# 82 — Focus hierarchy

Priority: modal → popover/menu → text/IME → active tool/editor → focused panel → global shortcuts. One Escape only affects the highest applicable layer. Closing popover/dialog restores focus to invoker when still valid.

# 83 — Accessibility IDs

Every interactive or automation-relevant Slint element receives a stable accessible-id/test identifier derived from semantic SurfaceId, never screen coordinates or translated text. Accessible role/name/value/enabled/expanded states mirror product semantics.

# 84 — Localization

User-facing copy is TextId-based. Canonical en-US, mandatory pt-BR translation policy, pseudo-locale for QA. Test 30–50% expansion, German-like long strings, Japanese IME and Arabic/RTL sample. No fixed width chosen solely for English labels.

# 85 — DPI

Test 100/125/150/175/200%. Handles/hit targets scale with UI/device scale, not document zoom. 1 px separators stay crisp when possible. Moving window between different-scale monitors recomputes UI/canvas/cursor geometry safely.

# 86 — Light theme

Same semantic hierarchy with light neutral surfaces. Do not invert blindly. Selection/focus/disabled/error contrast must remain distinct. Document white paper must still separate from application chrome.

# 87 — Compact/Comfortable/Spacious

Density changes control metrics/padding but not typography semantics, hit accessibility, feature availability or canvas/document geometry. Comfortable is default.

# 88 — Reduced Motion

Respect platform preference. Remove decorative translation/scale; retain only state feedback necessary for comprehension. No expert command waits for animation completion.

# 89 — Responsive desktop behavior

At narrow supported width, toolbar groups overflow, document tabs scroll/overflow, docks enforce minimums then collapse according to priority, canvas remains usable. Never overlap controls. At ultrawide, do not stretch property fields arbitrarily; docks stay bounded unless user resizes.

# 90 — Window minimum

Define tested minimum after Slint implementation profiling/layout QA. Before freeze, treat 1280×720 as stress target and 1366×768 as required mainstream target unless product ADR changes it. If 1280 cannot support full docks, controlled collapse is acceptable; hidden functionality remains recoverable.

# 91 — Slint implementation rules

.slint files own composition, presentation state, visual transitions, semantic accessibility metadata and callback emission. They do not implement geometry, raster algorithms, file parsing, persistence, undo, plugin authorization or document mutation.

# 92 — Large-list strategy

Use Slint ListView/appropriate model virtualization for Layers, Assets, Fonts, History, Data Merge and other large sequences. Stable row IDs must be semantic object/resource IDs; index is presentation position, not persistent identity.

# 93 — Icon integration

Prefer lucide-slint or a build-time SVG-to-Slint path for general icons, pinned to a reviewed version. Tabler SVG may supply missing general symbols. Domain tools use owned SVG paths. Do not parse thousands of SVGs at runtime for chrome if build-time conversion is practical.

# 94 — Resource packaging

Themes/tokens/icons/text catalogs are versioned resources. Invalid custom packs fall back per-token where safe and emit diagnostics. Theme packs cannot rebind commands, grant permissions or alter document semantics.

# 95 — Performance measurement

Measure Slint migration rather than assuming it is light. Capture release-build startup, time-to-interactive, shell idle CPU/RAM, large Layers scroll, tab/panel switching, docking, text editing, menu/command palette, resize and canvas-host interaction. Record hardware, OS, GPU, renderer/backend and revision.

# 96 — UI allocation/invalidation

Hover/focus must not rebuild document-derived models. Context toolbar updates are limited to affected properties. Panel changes should consume deltas rather than whole-document clones. Avoid large Rust→Slint model copies per frame.

# 97 — Long session

Soak scenario: open representative .PTND → vector edit → text edit → panel churn → Photo adjustment/paint → undo/redo → import/place → save → export → workspace switch → close/reopen, repeated. Watch RAM, VRAM, handles, tasks, caches and latency slope.

# 98 — Security boundary

Slint input is untrusted user input. Validate numeric/text/property requests in application/domain layer. Drag/drop and clipboard paths are treated as external inputs. Resource packs, imported SVG/images/fonts/profiles and .PTND packages are validated before canonical mutation.

# 99 — Safe save

Save request invokes persistence service. UI shows Saving only while real job runs. Success clears dirty only after committed safe save. Failure preserves dirty and displays recovery action. UI never fakes success on callback dispatch.

# 100 — Migration

Legacy .aubrieta/.aubri open path, if real legacy files exist, passes through versioned importer/migration and produces explicit converted state. New Save writes .PTND. Do not silently mutate original legacy file before successful conversion/save.

# 101 — Testing

Component tests: states/focus/accessibility. Semantic adapter tests: ActionId/PropertyEdit mapping. Interaction tests: menus/panels/tools/dialogs. Visual tests: component gallery + canonical shell. Performance tests: representative flows. Security tests: malformed inputs and permission boundaries. Manual proof-of-use remains required for RC.

# 102 — Golden shell scenes

Design Default; Photo Default; Light; Compact; 1366×768 constrained; 200% DPI; command palette; Export; Preferences; missing-link manager; error banner/toast; floating palette; multi-document tabs; active Pen/Node; large Layers tree.

# 103 — Reference comparison

Compare to current Affinity only for: information hierarchy, compact density, reachability, panel mental model, tool/context relationship and flow length. Do not use pixel-diff against Affinity or copy its branding/assets. Petunia goldens are authoritative.

# 104 — Interaction evidence

For each critical surface record ActionId dispatched, pre/post semantic state, disabled predicate, focus target, accessibility ID and resulting ChangeSet/history/persistence effect where relevant. A screenshot alone is never interaction evidence.

# 105 — Failure injection

Test file dialog cancel, missing link/font/profile, corrupt import, export write failure, disk full/read-only save, background job cancel, plugin failure, unavailable panel provider, malformed workspace layout and renderer/device error paths available to the shell.

# 106 — Release gate

No dead visible control. No placeholder V1 panel. No required menu item without ActionId. No known shortcut conflict in same context. No toolkit-specific core dependency. No required a11y label missing. No clipping in canonical locale/DPI matrix. No stale critical screenshot accepted as evidence. No Save/Open/Export workflow with untested failure path.

# 107 — Code quality

No giant app_window callback hub containing domain logic. Decompose Slint files by semantic region/component. Rust adapter names are explicit; avoid generic utils. One source of truth for tokens/actions/text/icons. Keep view-model conversion testable. Do not introduce abstraction wrappers that merely rename Slint without enforcing semantic contracts.

# 108 — Completion report

Return a matrix for every visible interactive surface with status: implemented/reachable/exercised/evidenced/verified. List files changed, ActionIds bound, components created, dialogs/menus/panels completed, tests run, performance measurements, accessibility checks, visual goldens, known limitations and exact blockers.

# 109 — Absolute final rule

Do not report Petunia Design Studio Slint UI complete until the application can be launched from clean state, a user can create/open a document, perform representative Design editing, use panels and context controls, switch to Photo without document conversion, undo/redo, save .PTND, close/reopen, export, recover from a deliberately induced failure, and all required interaction surfaces have non-stale evidence.

# 110 — Action registry naming

All user actions use stable IDs under ptnd.action.*. Examples: [ptnd.action.file.new](http://ptnd.action.file.new), [file.open](http://file.open), [file.save](http://file.save), edit.undo, object.duplicate, layer.toggle_visibility, view.toggle_grid, panel.layers.focus, [persona.design](http://persona.design).activate. Tool activation uses ptnd.tool.*. IDs are immutable API identity; translated labels are not.

# 111 — Text and icon identity

Every action references TextId title/description and IconId when visual. Example: [ptnd.text.action.file.save](http://ptnd.text.action.file.save).title and [ptnd.icon.file.save](http://ptnd.icon.file.save). Do not derive accessibility text from icon filenames. Do not duplicate label strings in Slint.

# 112 — File menu baseline

New…; New From Template when real; Open…; Open Recent; Close; Close All when supported; Save; Save As…; Save a Copy… only if persistence model defines it; Place…/Import…; Document Setup…; Export…; Print… when implemented; Exit/Quit according to platform. Recent menu has bounded count and Clear Recent. Missing recent files are handled gracefully.

# 113 — Edit menu baseline

Undo, Redo; Cut, Copy, Paste, Paste in Place if implemented, Duplicate, Delete; Select All/Deselect when platform/menu ownership fits; Preferences/Settings placed according to platform convention. Clipboard actions reflect current context and never target hidden text input accidentally.

# 114 — Object menu baseline

Transform, Arrange, Align, Group/Ungroup, Lock/Unlock, Hide/Show, Convert/Expand, Boolean/Pathfinder, rasterize/bake only where functional scope permits. Submenus avoid more than two levels. Current selection type controls enabled state.

# 115 — Layer menu baseline

New Layer/group/mask/adjustment depending persona/scope; duplicate; rename; arrange; visibility/lock; masking/clipping commands; merge/raster commands only where supported. Layer menu actions and Layers-row context menu share ActionId.

# 116 — Select menu baseline

Select All, Deselect, Invert where valid, Select Same/By Attribute only if implemented, selection expansion/refinement only in Photo scope where supported. Do not expose Affinity-like selection features ahead of engine scope.

# 117 — Text menu baseline

Character/Paragraph panels, text style actions, convert text to curves only where supported, insert special characters if implemented, spell/language options only with actual service. While text input is active, menus act on text context appropriately.

# 118 — View menu baseline

Zoom In/Out, Actual/100%, Fit Document/Surface/Selection, rulers, guides, grid, snapping display, proof/pixel preview according to capability, show/hide UI zones, Focus Canvas, Full Screen, panel visibility entry point. View-only actions never dirty document.

# 119 — Window menu baseline

Panels submenu generated from PanelRegistry; workspace presets; reset workspace; document/window management; Background Tasks; Diagnostics only in appropriate builds. No hard-coded panel list that drifts from registry.

# 120 — Help menu baseline

Search Help, Quick Reference/Keyboard Help, Documentation, Report Issue when channel exists, About and Third-party Notices. URLs are HelpTopicId/ResourceId, not literals spread through UI.

# 121 — Context menu rules

Canvas object, Layers row, panel tab, swatch/resource and document tab each have short context menus. Context menu is accelerator only; required actions have another discoverable route. Labels adapt semantically when useful, but ActionId stays stable.

# 122 — Shortcut philosophy

One coherent default keymap, user-editable. Contexts: Global, Canvas, TextEditing, NodeEditing, PhotoBrush, Dialog, LayersPanel, DataMergeTable and other justified scopes. Conflict checker distinguishes same-context conflicts from mutually exclusive contexts.

# 123 — Suggested default navigation/edit shortcuts

Ctrl/Cmd+N New; Ctrl/Cmd+O Open; Ctrl/Cmd+S Save; Ctrl/Cmd+Shift+S Save As; Ctrl/Cmd+Z Undo; Ctrl/Cmd+Shift+Z or platform Redo convention; Ctrl/Cmd+X/C/V Cut/Copy/Paste; Delete/Backspace Delete selection; Ctrl/Cmd+D Duplicate when not conflicting with documented text behavior; Ctrl/Cmd+K Command Palette; Tab Toggle UI/Focus Canvas only if keymap contract accepts it; Space temporary Hand; Z Zoom; Esc cancel/dismiss highest-priority context.

# 124 — Suggested Design tool shortcuts

V Move/Select; N Node only if Pencil receives another binding; P Pen; M shape cycle/group only if conflict-free; T Text; G Gradient; H Hand; Z Zoom; I Eyedropper; A Surface/Artboard only if accepted. Exact V1 defaults must be generated from KeymapRegistry and documented in-app. Never copy legacy Affinity shortcuts blindly when Petunia tool inventory differs.

# 125 — Shortcut test

For every binding: press in expected context, verify ActionId; press while text/number/IME field focused, verify it does not leak; verify user remap; verify conflict warning; verify reset default; verify displayed shortcut updates in menus/tooltips immediately.

# 126 — Drag-and-drop contract

All drag interactions have threshold, source identity, preview/ghost, valid-target highlight, invalid-target cursor, auto-scroll near edge, cancellation, drop transaction and error path. Dragging does not mutate canonical state until the documented commit boundary.

# 127 — Dock drag

Panel drag shows title/icon ghost. Edge/center targets are large enough for precise and low-precision pointing. Split preview reflects actual resulting geometry. Drop outside creates floating palette only if allowed. Esc restores exact previous tree/geometry.

# 128 — Layer drag

Insertion line means reorder sibling; container highlight means reparent; specialized clip/mask target has distinct affordance. Hover-expansion may open collapsed groups after delay. Drop result is a single command transaction and has Undo.

# 129 — Asset drag

Asset ghost shows thumbnail/name. Valid target can be canvas, Layers insertion position, property/resource slot or library category according to type. Invalid target never creates partial resources. Expensive decode begins only after validated intent or through cancellable preload.

# 130 — Color drag

Dragging swatch/color onto Fill/Stroke/object/property slot must clearly indicate target semantic. Do not guess Fill versus Stroke from pointer proximity without visible affordance.

# 131 — Modal focus policy

Opening modal traps logical focus inside. Initial focus chooses the first safe primary input, not destructive action. Tab/Shift+Tab cycle. Enter activates only a safe and valid default. Esc cancels only when cancellation is supported. Closing restores focus to invoker.

# 132 — Popover focus policy

Transient popover is not a modal. It may keep canvas context but routes keys appropriately while focused. Outside click/Esc closes. Pinning converts to persistent palette only where supported by component contract.

# 133 — Number field transaction

Focus/edit begins a transaction baseline. Keystrokes update local parse state; live preview may dispatch transient preview edits if tool/property supports it. Enter/focus commit dispatches one semantic edit transaction. Esc restores baseline. Invalid parse never leaves local UI state.

# 134 — Color transaction

Color picker may preview continuously but one drag gesture or explicit edit should coalesce into a logical history transaction when it mutates document appearance. Cancel returns baseline if the picker is transactional. Recent colors update only after committed user selection according to policy.

# 135 — Search ranking

Command Palette exact prefix/exact token match outranks fuzzy match. Context-valid actions rank above invalid ones but invalid can remain discoverable with reason. Recent frequency may break ties without hiding deterministic exact matches.

# 136 — Empty states

No document: meaningful Home/New/Open state. No selection: Properties explains selection dependency without giant dead panel. No assets: import/create CTA. No swatches: document defaults plus create. No results: search clear/filter guidance. Empty state is not decorative whitespace.

# 137 — Loading states

Long panel data uses skeleton/progress only when work actually exists. Never show fake spinner for instantaneous state. Existing usable data may remain visible with subtle refresh state instead of blanking the panel.

# 138 — Stale state

Presentation models can mark stale while a background result recomputes. UI must not present stale derived values as committed fresh data without indication when that distinction matters. New result is discarded if its revision no longer matches canonical state.

# 139 — Error state

Inline errors belong near the failing control when local. Panel-level errors show retry/recovery. Document integrity/security errors use persistent banner/dialog. Every error has stable diagnostic code behind localized copy.

# 140 — Warning state

Warnings do not block safe operation unless contract says so. They explain consequence and let user proceed intentionally. Repeated noncritical warnings should not spam toasts per frame/operation.

# 141 — Destructive confirmation

Confirm only destructive/non-reversible/high-impact actions; do not modal-fatigue routine reversible edits. Dialog names the object/count and consequence. Default focus is Cancel for severe destructive actions. If Undo fully restores operation, prefer nonmodal execution plus Undo unless policy says otherwise.

# 142 — Progress semantics

Progress values must come from real job state. Indeterminate only when denominator is unknown. Cancel button appears only if job can honor cancellation safely. Retry appears only for retryable outcomes and must be idempotent or explicitly de-duplicate side effects.

# 143 — Notifications

Success toast may offer Reveal/Open. Warning/error is persistent enough to read. Duplicate identical diagnostics are coalesced. Background task completion should not steal focus.

# 144 — Home/Welcome state

No-document window presents New Document, Open, recent projects, recovery entries and Help. Recent cards show name, path summary, modified timestamp where available and missing-file state. Do not require account/login. No marketing carousel in the core workflow.

# 145 — Recent projects

Bounded list; pin optional; context: Open, Reveal, Remove from Recent. Missing files stay distinguishable until removed. Selecting a recent item never mutates project. Paths displayed with privacy-aware truncation in screenshots/diagnostics.

# 146 — Document lifecycle UI

Opening creates session/tab only after sufficient validation to avoid ghost documents. Save state transitions: Clean → Dirty → Saving → Clean on success or Dirty+Error on failure. Closing dirty document invokes unsaved flow. Duplicate view shares document identity but owns view/session state.

# 147 — Dirty state negative guarantees

Workspace layout change, panel size, zoom, pan, selection, hover, active tool, focused field, theme, density, command-palette history and window geometry MUST NOT mark the document dirty.

# 148 — Multi-window

Detached windows share or own document sessions according to 09.24. Closing one view does not prompt if another view still owns the dirty document and policy says document remains open. Window geometry is clamped to visible monitors on restore.

# 149 — Workspace persistence

Persist dock tree, panel order, dimensions, floating geometry, panel collapsed state, density/theme override if user-scoped, Persona association and compatible monitor metadata. Corrupt workspace state falls back to a known-good preset without corrupting documents.

# 150 — Panel absence

If plugin/module panel is unavailable, restore neighboring layout, preserve unavailable PanelId placement metadata where practical, show diagnostic in Workspace/Plugin management and reinstate when provider returns. Never panic or leave a zero-size invisible splitter.

# 151 — Design system component gallery

Ship a developer-only gallery showing every primitive/state/theme/density/DPI combination. Gallery becomes visual-regression target. It must include long text, RTL sample, error/mixed/loading, focus, disabled and accessibility labels.

# 152 — Token enforcement

Add repository checks that flag literal production UI colors, unapproved font sizes, raw icon paths and user-visible strings outside resource files. Allow documented exceptions for canvas/document colors, exported data and test fixtures.

# 153 — Slint file ownership

app_window.slint composes high-level regions only. shell/ *owns chrome, tab strip, docking host, status and workspace. primitives/* owns low-level controls. components/ *owns semantic composites. panels/* and dialogs/* consume presentation models. Tool/domain behavior never lives in .slint.

# 154 — Rust adapter ownership

Rust UI adapter owns Slint model adaptation, callback binding, platform requests, event normalization, renderer host lifecycle and subscription to presentation deltas. It does not bypass application services or mutate DocumentStore.

# 155 — Bridge API

PetuniaDesignGuiBridge should expose coarse-grained session snapshot, action-state map, panel presentation models, property schemas/values, selection summary, task summaries, semantic notifications/dialog requests, viewport handles and resource/help IDs. It must not mirror every internal struct.

# 156 — Presentation model revisioning

Every sizable presentation model carries revision/version metadata or equivalent invalidation identity. Async results include source revision. UI discards stale deltas/results that target older document/session state.

# 157 — Threading

Slint main/UI thread owns presentation state. Background tasks communicate through Jobs/ChangeSets/message channels. No background worker writes Slint properties directly without safe dispatch. Domain must not assume Slint executor identity.

# 158 — Shutdown

On application exit: resolve dirty documents, cancel/finish jobs according to policy, flush safe preferences/workspace state, release renderer resources and close plugin workers. Shutdown timeout/error behavior is documented; do not block forever on stuck background job.

# 159 — Crash-safe UI preferences

Workspace/settings writes use safe persistence strategy where practical. Corrupt preferences are recoverable to defaults without affecting .PTND documents. A malformed theme/icon pack cannot prevent startup; start in canonical safe theme.

# 160 — Renderer host failure

If renderer/device cannot initialize, shell shows actionable fatal/recovery UI rather than empty canvas. Where fallback backend is supported, offer/attempt according to policy and record diagnostic. Never continue accepting edits whose visual result cannot be trusted without warning.

# 161 — Canvas resize

Window/panel resize recomputes viewport and surface without changing document coordinates. Swapchain/surface failures are recoverable. Resize events are coalesced where useful but visual response remains fluid.

# 162 — Input capture

Pointer capture during drag/stroke is released on commit, cancel, focus loss, modal interruption and tool switch according to state machine. Lost release events must not leave app in perpetual dragging/painting state.

# 163 — IME

Slint text controls and creative text editor must preserve preedit, commit/cancel, candidate positioning and composition range. Global shortcuts are suppressed during composition. Composition text is not committed to document until editor contract says so.

# 164 — Clipboard

Clipboard operations support semantic Petunia payload plus standard interoperable formats where available. Untrusted external clipboard data is validated before import. Paste into text field stays text-field context; Paste into canvas resolves through document command.

# 165 — Drag/drop from OS

Files dropped onto window are classified by importer capability. Hover indicates intended operation; multiple files use deterministic batching policy. Unsupported or malformed file produces diagnostics without creating partial document nodes.

# 166 — Font UI

Font list is asynchronous/virtualized, searchable and keyboard-friendly. Preview text rendering must not block the UI on large font collections. Missing/unavailable font state is explicit; selecting a font validates availability before canonical change.

# 167 — ICC/color profile UI

Profile lists can be large and platform-dependent; searchable/virtualized. Assign vs Convert are separate commands with explanatory copy. Missing/invalid profiles trigger diagnostics, never silent fallback that changes document appearance unexpectedly.

# 168 — Export presets

Presets have stable IDs and indicate built-in/user. Editing a built-in creates override/copy rather than silently mutating shipped default if policy says immutable. Invalid old preset is migrated or marked incompatible with reason.

# 169 — File format choice

Save/Open uses native PTND identity. Export formats are never presented as Save formats unless application semantics truly support round-trip editability. Import/open distinction is clear for lossy/external formats.

# 170 — Recovery UI

After crash/autosave recovery availability, show original document identity, recovery timestamp, source revision/path and actions Open Recovery, Compare/Inspect if supported, Discard, Reveal. Never overwrite original before user explicitly saves recovered state.

# 171 — Accessibility reading order

Logical order follows menu/top controls → document tabs/context → left tools → central task area → relevant dock according to focus path, not raw construction order. Panels maintain local predictable tab sequence. Hidden/collapsed elements are removed from accessibility navigation.

# 172 — Screen-reader naming

Examples: "Move Tool, selected, shortcut V"; "Flower Illustration, vector group, selected, visible, unlocked"; "Opacity, 82 percent"; "Stroke width, 2 millimeters". Avoid technical IDs/file names as spoken labels.

# 173 — Non-color semantics

Active tool, selected row, error, warning, snap result and lock/visibility cannot rely only on hue. Combine fill, border, icon shape, text weight, pattern or accessible state.

# 174 — Pointer target

Visible glyph can be 16–20 px but effective hit target aims 32 px or more. Tiny node/handle visuals get larger invisible hit areas scaled for DPI/pen. Crowded overlapping handles use deterministic hit priority.

# 175 — Pen/tablet

Normalize pressure, tilt, eraser identity and barrel buttons where platform exposes them. Pressure is clamped/validated. Pen hover should not commit paint. Palm/touch interactions defer to platform policy and must not create duplicate mouse+pen strokes.

# 176 — Trackpad

Smooth scroll/pan and pinch zoom should use platform semantics. Gesture recognition must not steal vertical scroll from panels or numeric fields under pointer. Inertial canvas pan can be preference-controlled and never affect document state.

# 177 — Touch

Desktop touch is optional unless platform profile requires it. If supported, hit areas and gestures follow same semantic actions. Do not ship half-working touch that conflicts with pen/mouse without declaring experimental status.

# 178 — High-frequency input

Pointer/pen motion paths avoid allocations, document-wide invalidation, synchronous file I/O, font enumeration, histogram recompute or layout rebuild. Record input latency traces on representative hardware.

# 179 — Performance budgets policy

Budgets are established from measured baselines on named low, recommended and high hardware tiers. Hard budgets apply only after methodology is stable. Until then, regressions are still flagged by before/after evidence and user-perceived stalls.

# 180 — Security/reliability UI rule

UI cannot suppress or downgrade a domain security/data-integrity error merely to preserve visual flow. If operation fails validation, show accurate state and recovery. Presentation never converts "failed" into apparent success.

# 181 — Interaction manifest completeness

Before release, enumerate every visible interactive element generated by Slint and registries. No item may remain untracked. Manifest count must reconcile with menu/action/panel/tool registries and dialog inventories.

# 182 — Surface IDs

IDs are semantic and stable: [ptnd.surface.menu.file.save](http://ptnd.surface.menu.file.save), ptnd.surface.panel.layers.visibility, ptnd.surface.dialog.export.format, ptnd.surface.tool.pen.mode. Do not use row index, screen position or translated label as ID.

# 183 — UI test tiers

Tier 1: pure component/state tests. Tier 2: adapter semantic tests. Tier 3: headless application tests. Tier 4: Slint interaction tests. Tier 5: full app proof-of-use. Tier 6: visual/accessibility/performance/security specialty runs.

# 184 — Component tests

For each primitive: render/instantiate; default/hover/pressed/focus/disabled/selected/mixed/error/loading; keyboard; pointer; accessibility metadata; localization expansion; light/dark; density; DPI. A component without state coverage cannot be declared canonical.

# 185 — Adapter tests

Verify Slint callbacks dispatch exact ActionId/PropertyEdit/ToolRequest; enabled/checked/mixed states round-trip; presentation deltas update only intended properties; dialog requests map correctly; focus context changes do not alter domain state.

# 186 — Headless parity

Any domain operation available through UI should be exercisable through semantic ports/headless harness where meaningful. If a feature only works because Slint directly owns business logic, architecture gate fails.

# 187 — Full workflow A: vector design

New PTND → create Surface → draw shape → Pen path → select nodes → edit fill/stroke → transform → align → boolean/path operation in V1 → text → save → close → reopen → export SVG/PDF/raster according to implemented adapters.

# 188 — Full workflow B: Photo

Open/place image → switch Photo → select pixel layer → adjustment → mask where supported → brush/erase/retouch V1 tool → undo/redo → save PTND → close/reopen → export raster. Switching back Design preserves editability.

# 189 — Full workflow C: mixed

Create vector composition → place raster → apply Photo nondestructive edit → add text → group/mask → duplicate Surface → modify → export variants. Validate one shared document/history/layer hierarchy.

# 190 — Full workflow D: production

Multi-Surface document → guides/bleed → styles/assets/symbols as supported → Data Merge if V1 → preflight-like diagnostics → export/print handoff → reopen. Validate resource links and color profile metadata.

# 191 — Failure workflow A

Save to unwritable destination → error → document remains dirty → choose new destination → save succeeds → old project preserved. Evidence includes state transitions and filesystem result.

# 192 — Failure workflow B

Open corrupt/truncated PTND → parser rejects safely → no half-open document → diagnostic with recovery/help. Existing open documents remain untouched.

# 193 — Failure workflow C

Import malformed/oversized asset → validation/resource limits → controlled error/cancel → no partial object/resource leak → app remains responsive.

# 194 — Failure workflow D

Cancel long export/import/background task → job acknowledges cancellation → transient resources cleaned → no duplicate output/half-committed document mutation.

# 195 — Failure workflow E

Renderer/device error where injectable → shell surfaces clear state → preserve canonical document → recover/recreate/fail-safe according to platform contract.

# 196 — Workspace torture

Randomized/reproducible sequences of panel open/close, split, resize, reorder, float/redock, Persona switch, restart/restore. Assert valid dock tree, visible center region, no negative sizes, no lost registered panels and no document dirty mutation.

# 197 — Menu torture

Enumerate all registered actions. Open each owning menu/context route. Compare enabled state to ActionQueryPort. Execute safe test variants. Verify disabled actions do not dispatch mutation. Verify shortcut label equals KeymapRegistry.

# 198 — Shortcut torture

Automated matrix across focus contexts. Especially single-letter tools versus TextEditing/IME/NumericField, Delete versus text caret, Space versus input fields, Ctrl/Cmd shortcuts inside dialogs, Esc hierarchy and Persona-specific reused keys.

# 199 — Panel torture

Populate large and pathological models: deep Layers tree, 10k/100k synthetic rows when harness supports, thousands of fonts/assets/swatches/history entries. Scroll, select, rename, drag, filter, update deltas. Measure memory and frame/interaction latency.

# 200 — Localization torture

Pseudo-locale expands labels 40–60%, adds diacritics and marker brackets. Test pt-BR, German-length strings, Japanese/CJK, Arabic/RTL. Menu, tabs, dialogs, tooltips and property rows must remain reachable. Truncation uses tooltip only for secondary data, not primary actions.

# 201 — DPI/window torture

Run canonical shell at 100/125/150/175/200%, 1366×768 through 4K/ultrawide. Continuously resize. Check border crispness, icon scale, text clipping, popover clamp, modal fit, splitter hit areas, canvas handle size and restored geometry.

# 202 — Theme torture

Dark, Light, System, accent variations and high-contrast/accessible overrides where supported. Validate text/border/focus/error/selection contrast and canvas overlay legibility over light/dark/colorful artwork.

# 203 — Input torture

Rapid mouse move, lost button-up/focus loss, repeated click, wheel over nested scrolls, pen pressure extremes, pinch/trackpad, drag cancel, modal interruption, window deactivation mid-stroke/drag. No stuck capture or duplicate action.

# 204 — Undo torture

Execute long mixed command sequence, undo all, redo all, branch after undo, cancel previews, switch Persona, save/reopen between logical points where supported. Verify IDs/references remain valid and memory stays within measured expectations.

# 205 — Persistence torture

Round-trip complex document containing vector, text, raster, masks/effects, resources, profiles, Surfaces, styles/symbols/Data Merge according to scope. Semantic equivalence after save/open; derived caches may differ.

# 206 — Atomic-save torture

Inject write failure before temp complete, before replace and after temp complete where harness supports. Original valid project remains recoverable. Dirty state and recovery messaging are correct.

# 207 — Legacy migration torture

Real legacy Aubrieta fixtures only: open, migrate, compare semantic state, save PTND, reopen PTND. Unsupported future/unknown legacy versions fail explicitly. Migration never relies solely on filename suffix.

# 208 — Export torture

Every supported export adapter gets empty/small/large, unsupported-feature, unwritable path, existing-file conflict, cancellation and color/profile cases. Export reads canonical/evaluated state, never screenshot of viewport.

# 209 — Security corpus

PTND malformed manifest, zip/path traversal, duplicate entries, huge declared resource, invalid JSON/encoding, oversized raster dimensions, malformed SVG/PDF/profile/font metadata, broken external paths and plugin/resource-pack malformed manifests according to actual supported parsers.

# 210 — Performance corpus

Version representative small/typical/large design/photo/mixed projects. Record expected object/node/layer/raster dimensions and why each workload exists. Keep heavy scheduled fixtures separate from fast CI fixtures.

# 211 — Low-spec hardware gate

Define a low-spec tier from project goals and actual machines, then measure launch, normal canvas interaction, panel scroll, basic vector edit, text edit, Photo basic adjustment and save/open. If budget cannot be met, document degradation strategy rather than hiding result.

# 212 — Memory gate

Measure clean idle, empty doc, typical project, large project, import peak, export peak and soak retained memory. Attribute major categories when profiling allows: document/history, raster tiles, renderer/GPU mirrors, thumbnails/caches, UI models.

# 213 — VRAM gate

Track render targets, textures, glyph/font atlases, raster tiles, thumbnails, buffers and temporary uploads. Closing project/workspace must release project-specific GPU resources after expected deferred lifetime.

# 214 — CPU gate

Idle CPU near quiescent; hover should not trigger busy loops. Profile vector editing, text layout, raster paint, panel update and export. Background jobs use bounded parallelism so UI remains responsive.

# 215 — Frame pacing gate

Use frame-time distribution/spikes, not average FPS alone. Record interaction traces for pan/zoom, transform drag, node drag, paint stroke, docking resize and large panel scroll.

# 216 — Startup gate

Measure cold/warm first-run and normal start. Time-to-window and time-to-interactive separated. Defer noncritical font/library/thumbnail/plugin scanning to background where architecture allows without surprising missing state.

# 217 — Cache governance

Every cache declares key, owner, size/budget, invalidation, eviction and recoverability. Unbounded thumbnail/font/preview/history-adjacent caches are release findings.

# 218 — Log governance

No per-frame/per-pointer-move info logs in release. Errors have context and stable codes. Paths/user content are minimized/redacted in support bundles. Debug tracing can be enabled explicitly.

# 219 — Documentation gate

Every implemented visible feature updates canonical docs, user-facing help when needed, action/shortcut registry docs and implementation evidence. Old Aubrieta/GPUI screenshots or names are removed from active docs or marked historical.

# 220 — RC declaration

The agent may state **Petunia Design Studio Slint UI — Release Candidate** only after all required hard gates pass, surface manifest has zero required unverified items, clean-state workflows succeed, PTND round-trip succeeds, known unwaived defects are zero for the accepted scope, and performance/security/reliability evidence is current for the final revision.