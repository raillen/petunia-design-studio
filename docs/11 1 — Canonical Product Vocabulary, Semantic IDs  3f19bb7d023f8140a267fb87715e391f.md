# 11.1 — Canonical Product Vocabulary, Semantic IDs & Forbidden Ambiguity

# Canonical terms

Petunia Design Studio, Design Persona, Photo Persona, Surface, Workspace, Studio Panel, Tool, Action, Command, Transaction, ChangeSet, Live Effect, Adjustment, Resource, Data Merge, Preflight and PTND.

# Identity vs label

Technical IDs remain stable and language-neutral: ptnd.action.*, ptnd.tool.*, ptnd.panel.*, [ptnd.property](http://ptnd.property).*, ptnd.effect.*, ptnd.format.*. Visible localized labels may change without breaking automation.

# Forbidden ambiguity

Do not use Layer to mean every drawable if Object is intended; do not call Surface “canvas” when referring to canonical page/artboard; do not call renderer output “document”; do not use plugin/action labels as persistent IDs.

# Naming quality

Prefer domain nouns/verbs over Manager, Helper, Utils, Data. Examples: HistoryService not UndoManagerGodObject; ColorTransformCache not ColorHelper.

# Synonyms

Docs can mention common external synonyms in glossary/search metadata but one canonical Petunia term owns implementation semantics.

# Migration

Legacy Aubrieta/Rust/Slint names stay historical only and must not appear in new namespaces unless compatibility reader requires them.