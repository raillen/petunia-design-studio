# 21 — Effects & Adjustments Specifications: Parameter Schemas, Math, ROI, CPU/GPU & Fidelity

# Authority

Cada effect/adjustment recebe schema tipado e referência matemática/testável.

# Required fields

EffectId/AdjustmentId; parameter types/default/ranges/units; color-space assumptions; alpha semantics; ROI expansion; border mode; preview/final quality; CPU reference; GPU capability; deterministic behavior; mask interaction; serialization; export degradation; accessibility UI; goldens.

# Families

Blur/sharpen/noise/distortion/shadow/glow/outline; Levels/Curves/Exposure/WB/HSL/Vibrance/B&W/Channel Mixer/Gradient Map/Selective Color/Threshold/Posterize.

# Rule

Renderer-specific shader parameter layout is derived from semantic schema, never canonical API.

[21.1 — Blur Family: Gaussian, Box, Motion, Radial/Zoom & Edge Modes](21%201%20%E2%80%94%20Blur%20Family%20Gaussian,%20Box,%20Motion,%20Radial%20Z%203f19bb7d023f81ab9a51dd5931ea26e6.md)

[21.2 — Sharpen Family: Unsharp Mask, High Pass & Local Contrast](21%202%20%E2%80%94%20Sharpen%20Family%20Unsharp%20Mask,%20High%20Pass%20&%20Lo%203f19bb7d023f8147a4d0efdb45d61396.md)

[21.3 — Shadow, Glow, Outline & Appearance Effects](21%203%20%E2%80%94%20Shadow,%20Glow,%20Outline%20&%20Appearance%20Effects%203f19bb7d023f81ae96ceff6b1f691f7b.md)

[21.4 — Noise, Denoise, Median & Texture Effects](21%204%20%E2%80%94%20Noise,%20Denoise,%20Median%20&%20Texture%20Effects%203f19bb7d023f818d8d7ee74b03c3327b.md)

[21.5 — Levels, Curves, Exposure & Brightness/Contrast Adjustment Schemas](21%205%20%E2%80%94%20Levels,%20Curves,%20Exposure%20&%20Brightness%20Contr%203f19bb7d023f81709f4cf83226403ea0.md)

[21.6 — White Balance, HSL, Vibrance, Black & White & Selective Color](21%206%20%E2%80%94%20White%20Balance,%20HSL,%20Vibrance,%20Black%20&%20White%203f19bb7d023f814b9800e99d6071c210.md)

[21.7 — Channel Mixer, Gradient Map, Threshold & Posterize](21%207%20%E2%80%94%20Channel%20Mixer,%20Gradient%20Map,%20Threshold%20&%20Po%203f19bb7d023f813189fed301ce8b1bf9.md)

[21.8 — Distortion, Perspective-Compatible & Warp Filter Contract](21%208%20%E2%80%94%20Distortion,%20Perspective-Compatible%20&%20Warp%20F%203f19bb7d023f81d0975cee11101dd6c4.md)