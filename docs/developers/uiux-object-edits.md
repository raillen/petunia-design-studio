# Object editors — UI/UX implementation

Date: 2026-10-01. Scope: **Milestone Required (MVP)**. This second UI/UX wave fixes concrete editor errors; it does not complete the inspector, direct text editing, or MVP.

Layer Rename now opens an editable name prompt instead of toggling an asterisk. Cancel, Escape and the backdrop dismiss without a command. Names must contain 1–256 characters after trimming and no control characters. The rename button is keyboard-focusable; layer action clicks stop propagation, preserving the previous selection.

Text editing from the inspector and canvas uses the same explicit multiline prompt. Its draft belongs to a stable session/object identity, is seeded when opened, and never writes during rendering. Empty text stays empty when reopened. Apply replaces content on a clone of the current opening text descriptor; family, size, line height, letter spacing and text-on-path remain intact. Font-size steps change only size. This is a content dialog, not a shaped in-canvas caret/editor or professional typography implementation.

Exact transformation opens X, Y, width, height and rotation fields. These edit the object's local placement relative to its parent/artboard, rather than claiming world-space bounds. Coordinates and dimensions use points; rotation uses degrees and is converted to the domain's radians. Decimal commas and explicit `pt`/`deg`/`°` suffixes work. Incorrect units, empty/non-finite values and non-positive dimensions are rejected before a command. The draft stays open with a field-specific localized error. Applying just width preserves the exact existing rotation; the former ±10 resize controls that reset rotation have been removed. Precise transformation and text editing require one selected object. A notice explains that other legacy inspector controls still affect the first selected object; multi-selection remains incomplete.

Each prompt compares the edited property against its opening value before committing. Closed objects, a different active session (including reused object IDs), or conflicting edits fail without overwriting newer work. Unrelated changes such as opacity remain intact. Apply submits one undoable bridge command; unchanged values create no history entry. The editor remains open on a failure and preserves input. Popups suppress canvas keyboard shortcuts through the existing modal gate. Stroke-width steps also retain the existing stroke token instead of forcing the default color.

Visible copy uses the shared canonical EN / synchronized PT resource catalog. [ADR-008](/developers/adr/ADR-008-object-edit-drafts) defines draft ownership and commit rules. See [bounded source/check evidence](/implementation/uiux-object-edits.json) for executed checks and exact snapshot hashes.

## Atlas and audit mapping

| Requirement | Change and boundary |
| --- | --- |
| 08.5 controls; 08.17 panels | Rename, multiline content, local numeric placement; no complete property framework |
| 08.18 dialogs; 08.19 copy/units | Explicit Apply/Cancel, retained errors and declared units, EN/PT |
| 08.14 input; 08.16 UI gauntlet | Modal shortcut isolation, focused headless interactions; real AT-SPI/IME still open |
| UX-08 / UX-10–13 | Fake rename, numeric entry, lost rotation, shared draft and clobbered typography addressed |
| UX-09 / UX-14 | Multi-selection and direct canvas text editing remain open |

## Remaining MVP and V1 work

This table records remaining acceptance, not an assertion that concurrently developed backend code is absent. Full scope stays in the [MVP–V1 roadmap](/developers/implementation-roadmap-2026-09-30).

| Release | Remaining work |
| --- | --- |
| MVP | Multi-selection/mixed values, layer tree collapse/filter, typed shape/gradient/stroke properties and live previews |
| MVP | In-canvas text caret/selection, shaping-aware editing, bidi/grafemes, real IME and font controls |
| MVP | Theme/system preference, density, contrast, target sizes and full EN/PT/token consistency |
| MVP | Complete tool workflows across vectors, persistent bitmap layers/masks, preview, undo/redo, reopen and faithful exports |
| MVP | Recovery acceptance, degradation reporting, cancellable long operations and resource budgets |
| MVP | Wayland/X11, portals, screen reader/AT-SPI, keyboard focus, HiDPI, tablet, Linux packaging and release performance gates |
| V1 | True ICC/CMYK workflows, soft proof, production PDF/X-4 and print preflight |
| V1 | Professional typography/bitmap tools, live advanced appearance, symbols/assets and scoped interoperability |

## Verification boundaries

Checks execute on a frozen source copy with a separate Cargo target. This avoids disrupting the evolving backend's artifacts. Focused tests check invalid drafts, style/path preservation, real multiline/empty input, selection propagation, revision/undo/redo, cross-session ID collisions and property conflicts. The complete desktop test target, localization, app-only strict Clippy, formatting and documentation checks are recorded when run. Headless images are actual Freya renders, not the earlier proposed HTML wireframes.

These checks do not certify the full evolving MVP. Real screen-reader labels/focus trapping, Wayland/X11, pressure/tilt, platform portals, input method composition and task-based usability still need acceptance. The content editor adds a deliberate Apply step; direct visual text editing remains in the backlog. Prumo CLI was unavailable; the goal, contracts, scope, source hashes and checks form the local microcontext.
