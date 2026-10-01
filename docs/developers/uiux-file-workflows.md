# Desktop file workflows — UI/UX implementation

Date: 2026-10-01. Scope: **Milestone Required (MVP)**. This is the first implementation wave of the UI/UX audit, not completion of the redesign or MVP.

The desktop adapter now routes file actions from menus, the command palette and shortcuts through one workflow. `file.new` opens configuration without creating a tab; confirmation creates one document, validates dimensions/name/bleed/margins before publication, and preserves the old session if configuration fails. Escape, Cancel and the backdrop dismiss the modal.

Open, Save and Save As use a modal path field with an asynchronous native system file picker. The picker only fills the field; confirmation performs the domain action. Cancellation leaves existing documents and the dirty state intact. Save targets a process-local `SessionIdentity`, rather than the active tab at callback time, and restores the user's active tab on success or failure. Stale requests fail instead of saving a replacement document. The destination shows the native `.PTND` suffix. Replacing another file requires a second explicit confirmation after suffix normalization.

Closing a dirty tab offers Cancel, Discard and close, and Save and close. The target identity is captured at the initiating click, and the close icon stops propagation to the tab. Closing all with Save saves every dirty document before closing any tab; a failed save or cancelled destination keeps all tabs open. Earlier successful saves remain saved. Quit currently closes the document sessions; terminating the OS window/process remains outside this wave.

Ctrl/Cmd+O, S, Shift+S and W join the existing New/Export shortcuts. File/configuration popups suppress the canvas shortcut handler while typing. This does not claim complete focus arbitration for inspector fields, text/IME editing or palette navigation. Popup uses Freya's Dialog accessibility role; real screen-reader focus trapping and AT-SPI behavior still need hardware/session validation.

Export uses the same typed `ExportRequest` for DPI, normalized destination, overwrite checks and dispatch. PNG shows the active-artboard scope and predicted pixel dimensions, passes the selected DPI to the exporter and preserves transparency. SVG/PDF show document scope; PDF observes export-enabled artboards. Errors remain visible with the entered options preserved, and successful saves/exports report their effective destination in a dismissible status message. Unsupported white-background and document Display P3/CMYK choices have been removed and replaced by explicit capability explanations. The UI describes PDF as basic and explicitly states ICC, CMYK and PDF/X production output are unavailable. There is no new color-management or professional-PDF implementation here.

New-document, file-destination, close and export workflow copy is canonical EN with a synchronized PT mirror in `file_workflow_strings.rs`. A separate catalog avoids changing the raster tool catalog while the MVP implementation continues.

## Verification and remaining work

See the generated [source and check evidence](/implementation/uiux-file-workflows.json). Verification uses a separate source snapshot and Cargo target directory so these checks do not consume the renderer task's build artifacts. Compile prerequisites found during verification received minimal type fixes: explicit RGBA array length, Arc/Vec tile conversion, an optional raster-object bounds argument, and desktop metadata for the new PixelFill variant. These fixes do not establish raster-tool correctness.

Focused tests cover new/cancel/confirm, invalid configuration, inactive-tab saves, failed saves, stale identities, sequential save-before-close, target preservation after indices change, visible errors, modal shortcut protection, request normalization and actual PNG output dimensions. Chrome interaction and localization checks are recorded separately. Test execution in this side conversation does not change the main MVP task's deferred-validation policy or certify its entire evolving workspace.

Native portals/Zenity, Wayland/X11, dialog parenting, large-document save/export responsiveness, tablet use, assistive technology, screenshot corpus and whole-MVP release gates remain unverified. Save/export execution is still synchronous after destination confirmation. Native-picker failure cannot be distinguished from cancellation by the current rfd API; manual path entry stays available. Export degradation reporting, export-native picker, autosave/recovery, print workflows, complete light/dark token work and the rest of the audit remain open.

## Atlas mapping

| Contract | Implementation |
| --- | --- |
| 08.2 shell and tabs | Shared file router, close target identity, keyboard entry points |
| 08.18 dialogs | Configuration, file destinations and close/export modals |
| 08.10 / 08.29 export and workflow | Effective destination, PNG DPI/scope, capability explanations |
| 08.16 UI gauntlet | Focused Freya headless interaction and PNG artifact tests |
| 09.16 localization | Shared canonical EN and PT workflow catalog |

Decision: [ADR-007](/developers/adr/ADR-007-desktop-file-workflows).

Next UI wave / próxima etapa de UI: [Stable object edit drafts](/developers/uiux-object-edits).
