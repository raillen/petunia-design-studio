# 19 — Individual Panel & Dialog Specifications

# Purpose

Specify each panel/dialog as a semantic application component rather than a widget screenshot.

# Panel contract

PanelId, provider/module, default dock, allowed multiplicity, model/view source, selection dependencies, actions, property schemas, drag/drop, search/filter, empty/loading/error states, keyboard focus/navigation, accessibility roles, persistence scope, performance/virtualization and tests.

# Dialog contract

DialogId, invocation ActionId, modality, input schema, validation, destructive decisions, async jobs, cancel behavior, focus/default action, accessibility, persistence of last options and tests.

# Rule

Panels never mutate DocumentStore directly; all edits use Actions/Commands/property editor transactions.

[19.01 — Layers Panel Specification](19%2001%20%E2%80%94%20Layers%20Panel%20Specification%203f19bb7d023f8198a8e8e886ff43774a.md)

[19.02 — Properties Panel Specification](19%2002%20%E2%80%94%20Properties%20Panel%20Specification%203f19bb7d023f811e85fddaa56e732fd9.md)

[19.03 — Appearance Panel Specification](19%2003%20%E2%80%94%20Appearance%20Panel%20Specification%203f19bb7d023f81f8b475dccd6123c220.md)

[19.04 — Color Panel Specification](19%2004%20%E2%80%94%20Color%20Panel%20Specification%203f19bb7d023f818d8fefd519130ffb64.md)

[19.05 — Swatches Panel Specification](19%2005%20%E2%80%94%20Swatches%20Panel%20Specification%203f19bb7d023f811e9888edf1cfe295b2.md)

[19.06 — Stroke Panel Specification](19%2006%20%E2%80%94%20Stroke%20Panel%20Specification%203f19bb7d023f815b9a48d0c17368a73b.md)

[19.07 — Transform & Align Panels Specification](19%2007%20%E2%80%94%20Transform%20&%20Align%20Panels%20Specification%203f19bb7d023f81d39932e922f5203bf0.md)

[19.08 — Typography & Paragraph Panels Specification](19%2008%20%E2%80%94%20Typography%20&%20Paragraph%20Panels%20Specificatio%203f19bb7d023f81a19ac9f093fc930d15.md)

[19.09 — Assets / Symbols / Styles Panels Specification](19%2009%20%E2%80%94%20Assets%20Symbols%20Styles%20Panels%20Specification%203f19bb7d023f8176bd10e9cf98b934ef.md)

[19.10 — History Panel Specification](19%2010%20%E2%80%94%20History%20Panel%20Specification%203f19bb7d023f8141a584e3baa9e7a89e.md)

[19.11 — Navigator & View Controls Panel Specification](19%2011%20%E2%80%94%20Navigator%20&%20View%20Controls%20Panel%20Specificat%203f19bb7d023f81a4b2bdd701e66219a2.md)

[19.12 — Brushes & Brush Settings Panels Specification](19%2012%20%E2%80%94%20Brushes%20&%20Brush%20Settings%20Panels%20Specificat%203f19bb7d023f8143bb71c784657033b3.md)

[19.13 — Adjustments / Live Filters Panel Specification](19%2013%20%E2%80%94%20Adjustments%20Live%20Filters%20Panel%20Specificati%203f19bb7d023f818fa965e0a6ac213e08.md)

[19.14 — Channels / Masks / Histogram / Info Panels Specification](19%2014%20%E2%80%94%20Channels%20Masks%20Histogram%20Info%20Panels%20Speci%203f19bb7d023f818697aaf90af5486571.md)

[19.15 — Export / Preflight / Data Merge Panels Specification](19%2015%20%E2%80%94%20Export%20Preflight%20Data%20Merge%20Panels%20Specifi%203f19bb7d023f8119bddbd3a0991d7f0a.md)

[19.16 — New Document / Document Setup Dialog Specifications](19%2016%20%E2%80%94%20New%20Document%20Document%20Setup%20Dialog%20Specifi%203f19bb7d023f8145ba12d8237d3c1ede.md)

[19.17 — Preferences & Shortcut Editor Window Specification](19%2017%20%E2%80%94%20Preferences%20&%20Shortcut%20Editor%20Window%20Speci%203f19bb7d023f81c7a227c4216c3dc841.md)

[19.18 — Export / Batch Export Dialog Specification](19%2018%20%E2%80%94%20Export%20Batch%20Export%20Dialog%20Specification%203f19bb7d023f816db7d4fdd69eb0f167.md)

[19.19 — Resource Manager / Relink / Missing Fonts Dialog Specifications](19%2019%20%E2%80%94%20Resource%20Manager%20Relink%20Missing%20Fonts%20Dial%203f19bb7d023f81cd871bdb1e3752d980.md)

[19.20 — Plugin Manager / MCP Manager Dialog Specifications](19%2020%20%E2%80%94%20Plugin%20Manager%20MCP%20Manager%20Dialog%20Specific%203f19bb7d023f81aa9ef7d9beab052963.md)

[19.21 — Recovery, Corruption & Compatibility Dialog Specifications](19%2021%20%E2%80%94%20Recovery,%20Corruption%20&%20Compatibility%20Dialo%203f19bb7d023f81a39f1ed95e2a8b2e23.md)

[19.22 — About, Diagnostics, Job Center & Notification Specifications](19%2022%20%E2%80%94%20About,%20Diagnostics,%20Job%20Center%20&%20Notificat%203f19bb7d023f81b7b8a5dd251009d548.md)