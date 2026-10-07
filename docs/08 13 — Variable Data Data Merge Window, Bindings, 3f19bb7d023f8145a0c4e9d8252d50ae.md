# 08.13 — Variable Data / Data Merge Window, Bindings, Preview & Generation

# Workflow

Add source -> schema -> bindings -> preview -> preflight -> generation.

# Data source

CSV/JSON/plugin tabular provider; encoding/delimiter/rows/fields/warnings; Refresh não destrói bindings compatíveis.

# Binding

Object/property + field/expression. Canvas optional binding badges. Missing field é error explícito.

# Expressions

Deterministic expression language com autocomplete; sem Python eval.

# Preview

Record navigator; derived only, never writes sample data into canonical objects.

# Image binding

contain/cover/stretch/original, missing fallback, brokered resource access.

# Generation

Surfaces in current doc, new doc, separate .PTND, direct exports. JobId/progress/cancel.

# Preflight

Missing fields, type errors, unavailable resources/fonts, text overflow, output filename conflicts, export degradation.

# GUI

Dockable panel para daily work; wizard/full window apenas para multi-output generation.