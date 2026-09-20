# 08.12 — Preferences, Customization, Themes, Icons, Workspaces, Onboarding & Help

# Preferences window

Preferences is a resizable owned window/dialog with left navigation and right settings content. Search at top filters both category names and settings labels.

Canonical categories:

- Appearance;
- Workspaces & Docking;
- Keyboard Shortcuts;
- Icons & Symbols;
- Tools & Canvas;
- Documents & Autosave;
- Color Management;
- Files & Resources;
- Photo;
- Data Merge;
- Plugins & Automation;
- Accessibility;
- Updates/Advanced where relevant.

# Appearance

- Theme: Dark / Light / System;
- accent color with curated presets + custom;
- density: Compact / Comfortable / Spacious;
- canvas background defaults;
- animations/reduce motion override;
- UI scale if OS scale is insufficient, with warning against compounding accessibility scaling;
- panel contrast/separator strength advanced option only if needed.

# Icons

Semantic icon architecture remains canonical. Preferences offers:

- Lucide;
- Phosphor;
- Aubrieta/Custom where complete;
- validated third-party/custom icon packs through the V1 semantic resource-pack architecture. New provider *formats* are Post-V1 only if they require a schema/runtime extension.

Preview card shows representative tools, menu icons and states before Apply. Changing icon pack never changes action semantics or shortcuts.

# Workspace settings

List built-in/user workspaces, default per Persona, autosave layout toggle, import/export layout, reset current layout.

# Keyboard Shortcuts

Searchable command table with context, bindings, conflict detection and reset. Export/import keymap profile. Recording mode has visible cancel/clear actions.

# Tools & Canvas

- tool rail customization/order;
- grouped-tool preference;
- cursor options;
- node/handle sizes;
- selection color;
- snapping defaults;
- zoom direction/sensitivity;
- pan gesture;
- guide/grid defaults;
- on-canvas HUD toggles.

# Documents & Autosave

- autosave interval/strategy;
- recovery retention;
- default units;
- default color model/profile;
- default new-document preset;
- linked-resource behavior;
- save previews/thumbnails.

# Color Management

- display profile automatic/manual;
- default RGB/CMYK profiles;
- rendering intent defaults;
- black-point compensation if engine supports;
- soft-proof defaults;
- warning when profile unavailable.

# Files & Resources

- default export/import directories;
- recent-file history policy;
- fonts/resource scanning locations where supported;
- linked file search paths;
- cache size/location advanced controls.

# Plugins & Automation

- installed plugins;
- enabled state;
- capabilities/permissions;
- update/source metadata;
- MCP/local automation settings and permission policy;
- Lua 5.5/`mlua` V1 scripting runtime status, per-plugin memory/CPU quota diagnostics and capability grants;
- Wasmtime/WASI high-isolation components shown as a distinct extension tier when installed/supported;
- developer plugin logs where appropriate.

# Preference apply model

Settings that safely apply live can preview immediately (theme/accent/density). Structural/high-risk settings require Apply or restart indicator. Cancel restores previewed values transactionally.

# Onboarding

First-run onboarding is short and skippable:

1. choose theme/density;
2. choose shortcut familiarity preset if offered;
3. show Design/Photo concept;
4. optional sample document/tutorial.

Never block opening a file behind onboarding.

# What's New

After significant update, show concise release highlights once with link to full notes. Not modal every launch.

# Help system

Entry points:

- menu Help;
- `?` button on complex panels/sections;
- tooltip secondary line `Learn more…` only where useful;
- command palette search;
- optional gpui-wry help/manual window.

Contextual help URL/topic ID is semantic, not hardcoded per button text.

# Reset flows

Granular resets:

- Reset Appearance;
- Reset Shortcuts;
- Reset Workspace;
- Reset Tool Settings;
- Reset All Preferences.

Full reset requires explicit confirmation and lists what is preserved (documents/plugins/user assets).

# Resource packs and customization boundary

Preferences must expose resource-pack selection without exposing unsafe filesystem semantics directly. Theme, icon and locale packs are independent selections but may be distributed together in a bundle. The UI must show pack name/version/author/license/source, compatibility status, validation errors and fallback behavior. Changes preview live when safe and can always revert to built-in defaults.

Built-in packs are immutable. User packs live outside the application install directory and are discovered by the resource-pack service. Import validates manifest schema, resource paths, duplicate semantic IDs, unsupported token types and license metadata before activation.