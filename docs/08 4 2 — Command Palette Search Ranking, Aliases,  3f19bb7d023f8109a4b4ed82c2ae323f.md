# 08.4.2 — Command Palette Search Ranking, Aliases, Parameters & Disabled Reasons

# Index

Action title, aliases, keywords, category, HelpId, Tool/Panel names and settings entries. Index localized labels plus stable English aliases for technical users if product chooses.

# Ranking

Exact prefix > token prefix > fuzzy subsequence, weighted recent/frequent optional but never hiding exact match. Disabled actions remain searchable with reason.

# Parameters

Simple Action parameter schema can render inline fields after action selection (e.g. Zoom %, Rotate Angle). Complex actions open dialog.

# Navigation

Arrow selects, Enter execute, Tab/Right opens parameter/details, shortcut shown, Esc closes and restores previous focus.

# Scope

Can search Panels (“Show Layers”), tools, preferences, help. Results have semantic category icon/type.

# Performance

Index prebuilt/incremental; <50 ms query on large plugin action catalogs.

# Accessibility/tests

Screen reader announces result count/title/shortcut/disabled reason. Test localization, typo fuzzy match, plugin removal and stale availability.