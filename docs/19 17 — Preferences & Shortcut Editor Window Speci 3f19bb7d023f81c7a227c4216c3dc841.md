# 19.17 — Preferences & Shortcut Editor Window Specification

# Window

Searchable category navigation. Prefer nonmodal settings window with Apply-immediate for safe preferences; operations requiring restart clearly marked.

# Categories

General, UI, Tools, Performance, Color, Files, Shortcuts, Plugins, MCP, Updates, Accessibility.

# Scope

Each setting descriptor states User/Workspace/Document and restart need. UI shows scope where confusion possible.

# Search

Indexes labels/help/aliases and navigates/highlights matching control.

# Shortcut editor

Tree/table by ActionId category with current/default/conflict. Record control captures chord, displays platform glyphs, detects exact/prefix conflicts and supports Reset/Clear/Profile import/export.

# Transaction

Preferences use settings store, not Document history except document-scoped settings that are semantic document commands.

# Tests

Search, restart flag, conflict, imported profile with unknown plugin ActionId, keyboard navigation and corrupt settings fallback.