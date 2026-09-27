# 08.9 — Welcome, New/Open, Place/Import, Relink, Missing Assets & File Workflows

# Welcome / no-document window

The no-document experience provides:

- New Document;
- Open;
- Recent documents with thumbnails/metadata;
- templates/presets;
- recovery/autosave items when available;
- learning/help entry points;
- optional What's New after update.

It must remain useful offline and avoid marketing clutter.

# New Document dialog

Layout:

- left preset/category sidebar: Print, Web/Screen, Photo, Social, Custom, Recent;
- center preset grid/list;
- right settings inspector.

Settings:

- name optional;
- width/height + units;
- orientation;
- DPI/PPI metadata;
- color model;
- color profile;
- bit depth where relevant;
- background transparent/white/custom;
- initial Surface count/layout if appropriate;
- bleed/margins advanced section;
- save custom preset.

Primary action `Create`; secondary `Cancel`. Invalid combinations explain reason inline.

# Open file

Use platform-native file dialog by default when it satisfies requirements. Supported-file filter groups: Aubrieta, Vector, Raster, PDF, All Supported, All Files. Opening unsupported/partially supported file routes into Import Interpretation when needed.

# Import interpretation dialog

Used only when imported format has choices that affect editability/appearance.

Potential options:

- import text as text vs curves when applicable;
- preserve layers/groups;
- rasterize unsupported effects;
- page/surface selection;
- embedded vs linked images;
- color-profile policy;
- DPI for unavoidable rasterization;
- warnings summary.

Provide safe recommended defaults and a concise `Compatibility` section. Advanced options collapsed by default.

# Place dialog/workflow

`Place…` inserts external resource into current document. After selecting file(s):

- pointer becomes place cursor;
- click places at intrinsic/default size;
- click-drag defines bounds;
- multiple resources become a place queue navigable by Esc/arrow or thumbnail HUD;
- linked vs embedded policy follows user preference or placement choice.

# Linked resources

Resources panel/manager shows:

- thumbnail;
- file name/path summary;
- linked/embedded state;
- modified externally state;
- missing state;
- profile/metadata where useful.

Actions:

- Relink;
- Update;
- Embed;
- Reveal in File Manager;
- Replace;
- Collect/Package — **Post-V1 Candidate** unless promoted by a print-production milestone; V1 linked-resource management still supports Relink/Update/Embed/Replace.

# Missing assets

Opening document with missing links does **not** block the entire document. Show nonmodal banner/task summary and mark affected layers. Provide `Relink All…`, `Locate…`, `Ignore for Now`. Placeholder preserves bounds and last preview if available.

# Font missing flow

Nonmodal document issue indicator. Missing Fonts dialog lists font, usage count and replacement mapping. Preview substitution optional. Replace All/Per Style. Never silently change canonical font reference on open without explicit action.

# Autosave/recovery

Recovery UI distinguishes:

- saved document;
- autosave newer than saved file;
- crash recovery copy;
- unsaved temporary document.

Recovery prompt offers **Open Recovery** and **Discard** as V1 actions. A richer side-by-side/metadata comparison is a **Post-V1 Candidate**; V1 still shows enough file/time/revision identity to make the choice safe. Discard destructive confirmation includes file/time identity.

# Save As / duplicate

Save As exposes destination and format only when changing native/compatible format. Native saves preserve all capabilities. Exporting to lossy format is not disguised as Save As if it loses editability.

# Recent files

Context actions: Open, Reveal, Remove from Recents. Missing recent files show unavailable state and can be removed. Thumbnail generation is asynchronous.

# Drag and drop files

Dropping onto:

- Welcome window → Open;
- canvas → Place/import;
- Assets panel → import resource/library where supported;
- Data Merge → attach supported data source.

Drop target overlay clearly communicates action before release.