# 08.18 — Window & Dialog Atlas: Every Modal, Sheet, Manager & Subsystem Window

# Purpose

This inventory specifies every **catalogued** non-panel window/dialog category so UI flows are not reduced to generic modal placeholders. Presence in this inventory does **not** independently authorize a feature for V1: the Product Charter/Functional Atlas scope status controls whether the corresponding window is required, Post-V1, capability-dependent or out of scope.

| Window / dialog | Modality | Primary content | Canonical page |
| --- | --- | --- | --- |
| Welcome / Home | window state | new/open/recent/templates/recovery/help | 08.9 |
| New Document | sheet/dialog | preset categories + document inspector | 08.9 |
| Open / Save / Choose Folder | native dialog | OS file system selection | 08.9 |
| Import Interpretation | sheet/dialog | editability, layers, text, rasterization, profiles | 08.9 |
| Missing Links Manager | nonmodal manager/dialog | missing resources, locate/relink/update | 08.9 |
| Missing Fonts | nonmodal manager/dialog | font usage/replacement mappings | 08.9 |
| Recovery | dialog | autosave/crash copy identity and restore/discard | 08.9 |
| Export | large sheet/window | targets, preview, format inspector | 08.10 |
| Batch Export | subsystem window | target variants, naming, destinations, progress | 08.10 |
| Overwrite Conflict | dialog | replace/skip/rename/apply-all | 08.10 |
| Print | native/custom hybrid | printer/page range/scaling/color handoff | this page |
| Attach Data Source | dialog | CSV/JSON parsing and preview | 08.11 |
| Data Merge Manager | subsystem window | table, fields, bindings, errors, preview | 08.11 |
| Generate Data Merge | sheet/dialog | record subset, output mode, overflow policy | 08.11 |
| Preferences | owned window | searchable settings categories | 08.12 |
| Shortcut Capture/Edit | popover/dialog | binding capture and conflict resolution | 08.12 |
| Workspace Manager | dialog/window | save/duplicate/rename/import/export/reset layouts | 08.12 |
| Plugin Manager | Preferences/subsystem | plugins, capabilities, enable/disable, diagnostics | 08.12 |
| Color Settings / Profile Assignment | dialog | assign/convert profile, intent, warnings | this page |
| Guide/Grid Manager | dialog/panel | named guides, grid spacing/subdivisions/origin | 08.6 |
| Document Setup | dialog | units, dimensions, profile, bleed/default metadata | this page |
| Surface Setup | dialog/popover | size, orientation, bleed, margins, columns | 08.7 |
| About | dialog | version/build/license links | this page |
| Third-party Notices | window/view | dependency/icon/font licenses | this page |
| Developer Diagnostics | window/panels | logs, performance, semantic tree, cache/action inspector | 08.15 |

# Print window

Aubrieta may delegate printer-specific UI to OS/native print dialog, but expose preflight-like document options before handoff when necessary:

- Surface/page range;
- copies where OS integration permits;
- scale/fit/actual size;
- orientation determined by Surface/printer;
- bleed/crop marks only if supported;
- color/profile handoff summary;
- rasterization warning for unsupported printer pipeline.

Do not duplicate full printer-driver UI inside Aubrieta.

# Color profile assignment/conversion dialog

Clearly separate:

- **Assign Profile**: changes interpretation, not channel numbers;
- **Convert to Profile**: changes values to preserve appearance.

Dialog includes source/current profile, destination, rendering intent, preview/proof if available, and concise explanation. Dangerous ambiguity is unacceptable.

# Document Setup

Sections:

- document units;
- default color model/profile;
- default raster resolution metadata;
- resource/link policy summary;
- document-level bleed/default Surface behavior where semantically global.

Changes that alter existing content require explicit preview/warning.

# About

Minimal: app name/logo, version, build hash/channel, copyright/license, website and Third-party Notices. **Check for Updates is shown only when the accepted Updater capability is active for that distribution/channel**; do not ship a placeholder action. Never use About as marketing billboard.

# Third-party notices

Searchable plain content with component name, version, license and source link. Include icon-library attribution obligations. Export/copy notices optional.

# Window consistency contract

Every non-native window defines:

- minimum/preferred size;
- resizable yes/no;
- modality/owner;
- remembered geometry yes/no;
- default/cancel buttons;
- keyboard focus order;
- help topic;
- validation/error region;
- loading/progress behavior;
- Esc/Enter semantics;
- accessibility title/description.

### Implementation cross-reference — 2026-10-01

Scope: **Milestone Required (MVP)**. The desktop file-workflow adapter resolves
configuration, stable save/close targets, normalized export destinations and PNG
DPI. Native-system and assistive-technology acceptance remain open. See
[ADR-007](../docs/developers/adr/ADR-007-desktop-file-workflows.md) and
[implementation/evidence mapping](../docs/developers/uiux-file-workflows.md).
This note does not promote ICC/CMYK or basic PDF to production-ready capability.


### Object-editor implementation — 2026-10-01

Scope: **Milestone Required (MVP)**. Explicit stable session/object drafts now
cover Rename, multiline content and exact local placement with points/degrees.
Cancel/conflict/invalid input do not publish. Typography/path and exact rotation
are preserved; multiselection, direct canvas text and external a11y remain open.
See [ADR-008](../docs/developers/adr/ADR-008-object-edit-drafts.md) and
[implementation/remaining-work/evidence](../docs/developers/uiux-object-edits.md).
