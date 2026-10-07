# 19.09 — History Panel Specification

# Identity

PanelId ptnd.panel.history. Shared.

# Model

Chronological committed Transaction records with HistoryEntryId, ActionId-derived localized label, source UI/plugin/MCP, timestamp optional and current pointer.

# Navigation

Click prior/future history entry issues Undo/Redo until target pointer; it does not restore arbitrary snapshots outside HistoryManager semantics.

# Branching

New mutation after undo truncates redo by V1 linear-history policy unless future branching-history ADR changes it.

# Coalescing

Panel displays logical transaction after gesture/text coalescing, not low-level preview updates.

# Savepoint

Saved revision marker visible. Dirty state derived from history/document revision semantics, not panel row index alone.

# Details

Optional expandable affected objects/properties and source for developer mode; sensitive text/pixels not logged.

# Persistence

V1 history may be session-only unless explicit PTND persistent-history feature is added.

# Tests

Undo/redo pointer, redo truncation, coalescing, savepoint, plugin/MCP source labels and huge history virtualization.