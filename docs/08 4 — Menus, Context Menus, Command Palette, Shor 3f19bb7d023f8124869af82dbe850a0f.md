# 08.4 — Menus, Context Menus, Command Palette, Shortcuts & Action Search

# Single action authority

Menus, toolbars, shortcuts, command palette, context menus, plugins e MCP invocam a mesma ActionId. QAction é adapter de presentation.

# Menus

File, Edit, Text, Layer, Select, Arrange, Filters/Effects, View, Window/Studio, Help. Persona pode adicionar grupos sem duplicar actions.

# Command palette

Search fuzzy por title, aliases, tags e help keywords. Resultado mostra icon, title, shortcut, category e disabled reason. Enter executa; Right/secondary pode abrir "Show in menu", "Assign shortcut", "Help".

# Shortcut model

KeyChord semântico, profile por plataforma, user override e conflict resolver. Shortcuts de tool podem alternar subtools. Press-and-hold temporary tools (Space Hand, modifier) devem restaurar previous tool.

# Context menus

Pequenos e contextuais. Canvas vazio: Paste, Select All, Place, View/Grid. Object: Cut/Copy/Duplicate, Arrange, Group, Convert, Mask, Export. Text selection recebe edição/text. Layer row recebe layer-specific operations.

# Disabled actions

Action permanece visível quando discoverability importa, com disabled reason acessível. Não esconder silenciosamente operações comuns por seleção inválida.

# Search

Help/action search cruza Actions, settings, panels, tools e help topics sem permitir mutation por label parsing.

# Shortcut editor

Table: command/category/current/default/conflicts. Record field captura chords com cancel/reset. Export/import profiles.

[08.4.1 — Default Shortcut Map, Platform Profiles & Conflict Rules](08%204%201%20%E2%80%94%20Default%20Shortcut%20Map,%20Platform%20Profiles%20&%203f19bb7d023f8116818ad74cb44be6be.md)

[08.4.2 — Command Palette Search Ranking, Aliases, Parameters & Disabled Reasons](08%204%202%20%E2%80%94%20Command%20Palette%20Search%20Ranking,%20Aliases,%20%203f19bb7d023f8109a4b4ed82c2ae323f.md)

[08.4.3 — Temporary Tool Overrides, Modifier Priority & Input Context Stack](08%204%203%20%E2%80%94%20Temporary%20Tool%20Overrides,%20Modifier%20Priori%203f19bb7d023f81618770d3a57b2651c7.md)