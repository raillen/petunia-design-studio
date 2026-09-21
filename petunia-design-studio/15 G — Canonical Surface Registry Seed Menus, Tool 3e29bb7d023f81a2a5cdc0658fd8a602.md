# 15.G — Canonical Surface Registry Seed: Menus, Tools, Panels, Dialogs & Commands

<aside>
🧾

**Purpose:** seed the machine-readable Interaction/Surface Manifest so implementation cannot collapse “the interface works” into a vague claim. This registry names public interaction surfaces; Functional Atlas scope still decides whether a capability is V1, milestone, post-V1 or unavailable.

</aside>

# Naming grammar

- actions: `ptnd.action.<domain>.<verb>`;
- tools: `ptnd.tool.<persona>.<tool>`;
- panels: `ptnd.panel.<name>`;
- dialogs/windows: `ptnd.dialog.<name>` / `ptnd.window.<name>`;
- controls: `ptnd.surface.<area>.<control>`;
- menus: `ptnd.menu.<family>.<item>`;
- settings: `ptnd.setting.<domain>.<key>`.

# Global shell surfaces

- `ptnd.surface.shell.brand` — Petunia mark/About entry when configured;
- `ptnd.surface.shell.persona.design`;
- `ptnd.surface.shell.persona.photo`;
- `ptnd.surface.shell.workspace_profile`;
- `ptnd.surface.shell.undo`;
- `ptnd.surface.shell.redo`;
- `ptnd.surface.shell.command_palette`;
- `ptnd.surface.shell.export`;
- `ptnd.surface.shell.overflow`;
- `ptnd.surface.tabs.document_strip`;
- `ptnd.surface.tabs.document_close`;
- `ptnd.surface.tabs.document_dirty_indicator`;
- `ptnd.surface.tabs.document_overflow`;
- `ptnd.surface.context_toolbar`;
- `ptnd.surface.tool_rail`;
- `ptnd.surface.status_bar`;
- `ptnd.surface.dock.left`;
- `ptnd.surface.dock.right`;
- `ptnd.surface.dock.bottom`;
- `ptnd.surface.canvas.viewport`.

# File actions

- `ptnd.action.file.new`;
- `ptnd.action.file.open`;
- `ptnd.action.file.open_recent`;
- `ptnd.action.file.close`;
- `ptnd.action.file.close_all` when supported;
- `ptnd.action.file.save`;
- `ptnd.action.file.save_as`;
- `ptnd.action.file.save_copy` only if semantically implemented;
- `ptnd.action.file.place`;
- `ptnd.action.file.import`;
- `ptnd.action.file.export`;
- `ptnd.action.file.print` when supported;
- `ptnd.action.file.document_setup`;
- `ptnd.action.file.quit`.

# Edit actions

- `ptnd.action.edit.undo`;
- `ptnd.action.edit.redo`;
- `ptnd.action.edit.cut`;
- `ptnd.action.edit.copy`;
- `ptnd.action.edit.paste`;
- `ptnd.action.edit.paste_in_place` when supported;
- `ptnd.action.edit.duplicate`;
- `ptnd.action.edit.delete`;
- `ptnd.action.edit.preferences` according to platform convention.

# Selection actions

- `ptnd.action.select.all`;
- `ptnd.action.select.none`;
- `ptnd.action.select.invert` when valid;
- select-same/by-attribute only when functional scope promotes them.

# Object actions

- group/ungroup;
- arrange front/back/forward/backward;
- transform/reset/flip as engine supports;
- align/distribute through semantic alignment target;
- lock/unlock;
- hide/show;
- convert/expand/bake/rasterize only when real;
- boolean/path operations from Functional Atlas.

# View actions

- zoom in/out/100/fit document/fit selection/fit surface;
- toggle rulers/guides/grid/snapping/proof/pixel preview according to capability;
- toggle panels/chrome;
- focus canvas;
- full screen;
- reset workspace.

# Design tool IDs

- `ptnd.tool.design.move`;
- `ptnd.tool.design.node`;
- `ptnd.tool.design.surface`;
- `ptnd.tool.design.pen`;
- `ptnd.tool.design.pencil`;
- `ptnd.tool.design.rectangle`;
- `ptnd.tool.design.ellipse`;
- `ptnd.tool.design.polygon`;
- `ptnd.tool.design.star`;
- `ptnd.tool.design.line`;
- custom shape family if V1;
- `ptnd.tool.design.artistic_text`;
- `ptnd.tool.design.frame_text`;
- `ptnd.tool.design.place_image`;
- `ptnd.tool.design.gradient`;
- `ptnd.tool.design.transparency`;
- `ptnd.tool.design.eyedropper`;
- `ptnd.tool.design.knife`;
- `ptnd.tool.design.corner`;
- `ptnd.tool.design.contour`;
- `ptnd.tool.design.hand`;
- `ptnd.tool.design.zoom`;
- Stroke Width / Point Transform / Shape Builder / Vector Brush only according to 10.x scope.

# Photo tool IDs

- `ptnd.tool.photo.move`;
- `ptnd.tool.photo.brush`;
- `ptnd.tool.photo.eraser`;
- `ptnd.tool.photo.selection` family as scoped;
- `ptnd.tool.photo.crop`;
- `ptnd.tool.photo.gradient`;
- `ptnd.tool.photo.eyedropper`;
- clone/heal/inpainting tools only if V1;
- hand/zoom;
- mask/refine interactions as semantic tools only when the engine owns them.

# Shared canonical panel IDs

- `ptnd.panel.layers`;
- `ptnd.panel.properties`;
- `ptnd.panel.color`;
- `ptnd.panel.swatches`;
- `ptnd.panel.assets`;
- `ptnd.panel.history`;
- `ptnd.panel.navigator`;
- `ptnd.panel.transform`;
- `ptnd.panel.export`;
- `ptnd.panel.resources_links`;
- `ptnd.panel.background_tasks`;
- `ptnd.panel.plugins`.

# Design panel IDs

- `ptnd.panel.stroke`;
- `ptnd.panel.appearance`;
- `ptnd.panel.align`;
- `ptnd.panel.pathfinder`;
- `ptnd.panel.typography`;
- `ptnd.panel.paragraph`;
- `ptnd.panel.text_styles`;
- `ptnd.panel.surfaces`;
- `ptnd.panel.symbols`;
- `ptnd.panel.styles`;
- `ptnd.panel.data_merge`.

# Photo panel IDs

- `ptnd.panel.histogram`;
- `ptnd.panel.adjustments`;
- `ptnd.panel.channels`;
- `ptnd.panel.brushes`;
- `ptnd.panel.brush_settings`;
- `ptnd.panel.masks`;
- `ptnd.panel.info`;
- presets panel if promoted.

# Layers child surfaces

- tree expand/collapse;
- row select;
- row multi/range select;
- inline rename;
- visibility;
- lock;
- badges/effect/mask/symbol state;
- drag reorder;
- drag reparent;
- drag clip/mask target;
- row context menu;
- panel search/filter when enabled;
- add/new action;
- delete action;
- panel overflow.

Every child receives its own concrete SurfaceId when the manifest is generated.

# Properties child surfaces

- each PropertyDescriptor creates stable `ptnd.surface.properties.<property-id>`;
- section disclosure;
- reset/revert;
- mixed-state indicator;
- unit selector where real;
- binding/automation indicator only if capability exists;
- validation/error affordance.

# Color child surfaces

- active target Fill/Stroke;
- swap;
- default;
- none/transparent;
- model selector;
- channels;
- wheel/plane when enabled;
- alpha/opacity;
- RGB hex only in RGB-relevant context;
- recent colors;
- document swatches jump;
- eyedropper.

# Stroke child surfaces

- style;
- width;
- alignment;
- cap;
- join;
- miter;
- dash editor;
- start marker;
- end marker;
- pressure/width profile;
- preview.

# Transform child surfaces

- X, Y, W, H;
- rotation;
- skew;
- aspect lock;
- 3×3 origin;
- relative/absolute mode;
- flip H/V;
- unit context.

# Docking surfaces

- panel tab;
- tab close;
- tab drag;
- tab overflow;
- stack splitter;
- dock splitter;
- collapse/reveal;
- float;
- redock;
- target overlay left/right/top/bottom/center as legal;
- floating palette pin/dock/close.

# Document tab context surfaces

- Close;
- Close Others;
- Close Right;
- Duplicate View;
- Reveal in File Manager;
- Move to New Window;
- Pin only if implemented.

# Standard dialog/window IDs

- `ptnd.window.home`;
- `ptnd.dialog.new_document`;
- native open/save/folder requests;
- `ptnd.dialog.import_interpretation`;
- `ptnd.window.missing_links`;
- `ptnd.window.missing_fonts`;
- `ptnd.dialog.recovery`;
- `ptnd.window.export`;
- `ptnd.window.batch_export`;
- `ptnd.dialog.overwrite_conflict`;
- print/native print handoff;
- `ptnd.dialog.color_profile`;
- `ptnd.dialog.guide_grid_manager`;
- `ptnd.dialog.document_setup`;
- `ptnd.dialog.surface_setup`;
- `ptnd.window.preferences`;
- `ptnd.dialog.shortcut_capture`;
- `ptnd.window.workspace_manager`;
- `ptnd.window.plugin_manager`;
- `ptnd.dialog.about`;
- `ptnd.window.third_party_notices`;
- developer diagnostics only in appropriate builds.

# Every manifest row MUST include

stable SurfaceId; current scope; owner; visual parent; TextId; IconId; ActionId/ToolId/PropertyId; shortcut/context; enabled predicate; disabled reason; pointer behavior; keyboard behavior; focus; accessibility role/name/state; tooltip; mutation/side effects; transient state; commit/cancel; Undo/Redo; persistence; error/failure; performance class; security class; automated evidence; manual proof; current status.

# Reconciliation gate

At RC, registry enumerations + live Slint inspection + documentation inventory must reconcile. Required Untracked=0, Dead=0, Unknown Behavior=0, Untested=0 and Stale Critical Evidence=0.