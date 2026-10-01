# ADR-007: Desktop file workflow adapter

- **Status:** Accepted contract; implementation evidence is limited to the checks recorded below.
- **Date:** 2026-10-01
- **Scope:** Milestone Required (MVP)


**Current extension:** [ADR-009](/developers/adr/ADR-009-persistent-raster-and-native-workflows) supersedes schema/resource, uniform text style/overflow, native file/clipboard and blanket ICC RGB image rejection pending statements. This original wave record remains historical; current code is unvalidated.

## Context

The desktop dispatched New before configuration, so Cancel left a new tab. Open/Save As bare menu tokens had no destination UI. The export dialog omitted its selected DPI and offered options the engine did not consume. Dirty close offered no save path, and callback-time tab indices are insufficient to identify a save target.

## Decision

File lifecycle actions are intercepted by the desktop adapter and resolved through one router for menu, palette and keyboard entry points. Configuration prompts do not mutate documents. Destinations are chosen in a modal with an asynchronous native picker and manual-path fallback; only explicit confirmation dispatches. Save/close requests bind to `DocumentSession::identity()`. Save execution uses the existing action lane and restores the active session. Save-before-close never force-closes after a failed save, and Save All closes no tab until every dirty tab has saved.

Export options and overwrite checks resolve through the existing typed `ExportRequest` before dispatch. Unsupported choices are replaced by capability explanations. New visible copy belongs to the EN/PT resource catalog. File popups suppress canvas shortcuts while open. No native file/schema, domain mutation, ICC or PDF production contract is changed by this adapter.

## Consequences

The adapter adds ephemeral prompt/feedback/close-target state and a direct dependency on the existing rfd version already used by Freya. File-system I/O remains owned by the domain services. Native-picker results do not publish documents. Save/export work remains synchronous after confirmation; window termination, complete focus/IME and external accessibility validation remain open.

Implementation, Atlas mapping and bounded evidence: [desktop file workflows](/developers/uiux-file-workflows) and [generated reference](/implementation/uiux-file-workflows.json). These checks do not certify the entire MVP or claim real Linux portal/assistive-technology validation.
