# ADR-008: Stable object edit drafts

- **Status:** Accepted contract; bounded checks only / contrato aceito com checks delimitados.
- **Date:** 2026-10-01
- **Scope:** Milestone Required (MVP)


**Current extension:** [ADR-009](/developers/adr/ADR-009-persistent-raster-and-native-workflows) supersedes schema/resource, uniform text style/overflow, native file/clipboard and blanket ICC RGB image rejection pending statements. This original wave record remains historical; current code is unvalidated.

## Context

Layer renaming toggled `*`; text drafts were shared across objects and reseeded on render, while commits recreated default typography and detached text-on-path. Numeric resize reset rotation and provided no exact placement.

## Decision

The desktop owns one ephemeral explicit object-edit prompt, identified by session and ObjectId. Only the edited property is compared against its opening value before a single bridge command. Cancellation and invalid/conflicting drafts never publish. Content edits clone the text descriptor; placement declares points/degrees and positive dimensions. Modal ownership gates canvas shortcuts. Unchanged values create no history. Other legacy inspector controls and direct shaped text editing retain their existing scope limitations.

## Consequences

The global text buffer is removed. Editing content is now an explicit multiline modal accessible from the canvas and inspector. Conflict errors retain the draft and require reopening with current data. Multi-selection, complete property widgets, IME and external accessibility acceptance remain open. No document schema or backend rendering contract changes.

[Stable object edit drafts](/developers/uiux-object-edits); [evidence/evidência](/implementation/uiux-object-edits.json).
