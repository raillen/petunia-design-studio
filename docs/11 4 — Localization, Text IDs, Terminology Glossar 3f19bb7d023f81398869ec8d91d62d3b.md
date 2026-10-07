# 11.4 — Localization, Text IDs, Terminology Glossary & Translation Governance

# TextId

All visible product strings use stable TextId or equivalent resource key. Do not persist translated labels in Actions/Commands/workspaces.

# Canonical source language

Developer semantic IDs/names are English technical identifiers. Product documentation/UI can publish en-US and pt-BR; additional locales follow same glossary.

# Glossary

Each canonical term records definition, allowed user-facing translations, disallowed confusing synonyms and related external terminology.

# Context

Translators receive description, surface, placeholders, grammatical role and screenshot/component context where useful.

# Placeholders

Named placeholders, typed formatting and plural/select rules. No string concatenation that breaks grammar.

# QA

Pseudo-localization, long-string expansion, RTL readiness where supported, locale numeric/unit formatting and shortcut-letter collisions.

# Brand names

Petunia Design Studio and format/technical IDs are not arbitrarily translated.