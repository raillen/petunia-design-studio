# 23 — Effect, Adjustment & Generator Parameter Specifications

# Purpose

Each built-in adjustment/filter/generator gets a versioned parameter schema shared by UI, serialization, plugins/MCP discovery, CPU reference and GPU implementation.

# Contract

EffectKind, version, accepted color/pixel inputs, parameters/ranges/units/defaults, ROI expansion, edge behavior, alpha behavior, color-space expectation, preview/final quality, CPU/GPU support, determinism, serialization, export degradation and tests.

# Rule

A named effect is not implemented merely because a shader exists; its mathematical and UX contract must be specified and validated.

[23.1 — Adjustment Base Contract, ParameterSchema, Preview Sessions & Serialization](23%201%20%E2%80%94%20Adjustment%20Base%20Contract,%20ParameterSchema,%20%203f19bb7d023f813c8fe6f2cec4ceb759.md)

[23.2 — Levels, Curves, Exposure, Brightness/Contrast & Tonal Adjustments](23%202%20%E2%80%94%20Levels,%20Curves,%20Exposure,%20Brightness%20Contra%203f19bb7d023f8148accaf706f5e80651.md)

[23.3 — HSL, Vibrance, White Balance, Black & White & Selective Color](23%203%20%E2%80%94%20HSL,%20Vibrance,%20White%20Balance,%20Black%20&%20White%203f19bb7d023f81e188ddd73eb31d4f4e.md)

[23.4 — Gaussian/Motion Blur, Sharpen, High Pass & Convolution Edge Policies](23%204%20%E2%80%94%20Gaussian%20Motion%20Blur,%20Sharpen,%20High%20Pass%20&%20%203f19bb7d023f8100be7cf0f52d5a5dad.md)

[23.5 — Noise, Denoise, Vignette, Posterize, Threshold & Generator Effects](23%205%20%E2%80%94%20Noise,%20Denoise,%20Vignette,%20Posterize,%20Thresh%203f19bb7d023f81089bd5fad5d272f2d0.md)

[23.6 — Shadows, Glows, Outline & Appearance Effects](23%206%20%E2%80%94%20Shadows,%20Glows,%20Outline%20&%20Appearance%20Effect%203f19bb7d023f810f923bdb7daf23ef29.md)

[23.7 — Distortion, Warp, Perspective & Resampling Filter Contracts](23%207%20%E2%80%94%20Distortion,%20Warp,%20Perspective%20&%20Resampling%20%203f19bb7d023f814daac2d82cd686d782.md)

[23.8 — Effect Plugin Contract, Custom Effect Capability & Safety Limits](23%208%20%E2%80%94%20Effect%20Plugin%20Contract,%20Custom%20Effect%20Capab%203f19bb7d023f811aae70ec18f36fbd41.md)