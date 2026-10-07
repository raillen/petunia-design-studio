# 09.10.5 — Soft Proof, Gamut Warning, Ink/Paper Simulation & Color UI/Preflight Contract

# Soft proof preset

Target ICC profile, intent, BPC, simulate paper, simulate black ink, gamut warning color and optional display comparison mode.

# View only

Proof state is view/workspace state unless saved as document proof preset metadata; it never rewrites object channels.

# Gamut warning

Compute whether source/document colors map outside target gamut according selected criterion/engine. Overlay color is accessibility-configurable and distinct from artwork.

# Paper simulation

Absolute/soft-proof display can map paper white through monitor profile. UI clearly signals proof mode to avoid user compensating artwork unknowingly.

# Separation preview

Panel can solo/toggle process/spot separations and show ink coverage heatmap. These are derived proof views.

# Color panel

Always shows model/profile or linked swatch/spot identity. Switching model can either reinterpret UI representation of same semantic color or trigger Convert Color action—distinction explicit.

# Picker

Composite display sample vs source/document semantic sample are separate modes. Info panel names mode/profile.

# Preflight

Missing/invalid profile, RGB object in CMYK-only target policy, unsupported spot, overprint risk, TAC, registration misuse, transparency flattening and output intent.

# Tests

Proof toggle no Document revision, gamut overlay golden, paper simulation, separation toggles, picker source-vs-display and preflight navigation.