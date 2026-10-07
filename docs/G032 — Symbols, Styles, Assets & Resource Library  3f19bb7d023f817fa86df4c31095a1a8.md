# G032 — Symbols, Styles, Assets & Resource Library Baseline

# Goal

Implement reusable design resources.

# Depends

Document/resources, Layers/Properties, 22.x.

# Primary

editor-engineer + ui-component-engineer.

# Deliverables

SymbolDefinition/Instance/overrides; Object/Character/Paragraph Styles; AssetLibrary; document/user library scope; panels; place/update/detach commands; PTND persistence.

# Acceptance

Reusable identity survives save/reopen; symbol edits propagate; overrides remain local; style dependencies and missing libraries are explicit.

# Tests

ID remap, override propagation, style inheritance, 10k assets virtualization, library conflict.