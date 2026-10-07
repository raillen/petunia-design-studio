# 19 — Individual Panel Specifications: Models, Commands, Keyboard, Scale & Accessibility

# Authority

Cada Studio Panel recebe contrato independente de dados, interação, escala e acessibilidade.

# Mandatory template

PanelId; Persona visibility; default dock; data source/view model; row/control schema; selection synchronization; Actions; drag/drop; context menu; search/filter; empty/loading/error; keyboard navigation; accessibility tree; persistence scope; plugin extension points; virtualization; performance; tests.

# Rule

Painel nunca é fonte canônica de documento. Ele projeta snapshot/selection e envia Actions/Commands.

# Scale

Tree/table/grid panels devem declarar comportamento em 10, 1k, 10k e 100k entries quando plausível. QWidget-per-row é proibido em coleções grandes.

[19.01 — Layers Panel Specification](19%2001%20%E2%80%94%20Layers%20Panel%20Specification%203f19bb7d023f81f2ac69c20fa11accfa.md)

[19.02 — Properties Panel Specification](19%2002%20%E2%80%94%20Properties%20Panel%20Specification%203f19bb7d023f8166b2ebe6061fd0f6a0.md)

[19.03 — Appearance Panel Specification](19%2003%20%E2%80%94%20Appearance%20Panel%20Specification%203f19bb7d023f8199a022cd67bf0b49e2.md)

[19.04 — Color & Swatches Panels Specification](19%2004%20%E2%80%94%20Color%20&%20Swatches%20Panels%20Specification%203f19bb7d023f81239039f558981a70fb.md)

[19.05 — Stroke Panel Specification](19%2005%20%E2%80%94%20Stroke%20Panel%20Specification%203f19bb7d023f8113a495faefc3fd5c68.md)

[19.06 — Transform & Align Panels Specification](19%2006%20%E2%80%94%20Transform%20&%20Align%20Panels%20Specification%203f19bb7d023f81ce9dbafd7ac5318027.md)

[19.07 — Typography & Paragraph Panels Specification](19%2007%20%E2%80%94%20Typography%20&%20Paragraph%20Panels%20Specificatio%203f19bb7d023f8105ad9de293179eeeb6.md)

[19.08 — Assets, Symbols & Styles Panels Specification](19%2008%20%E2%80%94%20Assets,%20Symbols%20&%20Styles%20Panels%20Specificat%203f19bb7d023f81a3b07ee25d6d26f119.md)

[19.09 — History Panel Specification](19%2009%20%E2%80%94%20History%20Panel%20Specification%203f19bb7d023f81aca5e0fa2f2e0b2075.md)

[19.10 — Navigator & Export Panels Specification](19%2010%20%E2%80%94%20Navigator%20&%20Export%20Panels%20Specification%203f19bb7d023f8181acced91479acaf94.md)

[19.11 — Histogram, Channels & Adjustments Panels Specification](19%2011%20%E2%80%94%20Histogram,%20Channels%20&%20Adjustments%20Panels%20S%203f19bb7d023f811abd26c7deaa0de1a3.md)

[19.12 — Brushes, Masks & Info Panels Specification](19%2012%20%E2%80%94%20Brushes,%20Masks%20&%20Info%20Panels%20Specification%203f19bb7d023f81aa9667d4a6d39c96cc.md)