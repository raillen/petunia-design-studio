# 21 — Effects, Adjustments & Filter Specifications

# Authority

Cada efeito/adjustment tem schema de parâmetros, domínio matemático, ROI, color-space behavior, GPU/CPU strategy, preview/final rules, serialization, UI and tests.

# Common contract

EffectId/AdjustmentId; input model/bit depth; params with units/ranges/defaults; alpha/color behavior; ROI expansion; edge mode; deterministic math; GPU capability; CPU oracle; mask/blend support; GUI schema; Command; serialization; export degradation; benchmarks/goldens.

[21.01 — Common Effect Node Contract, ROI, Quality, Color & Parameter Schema](21%2001%20%E2%80%94%20Common%20Effect%20Node%20Contract,%20ROI,%20Quality,%203f19bb7d023f8126b6d3ebc63ec12003.md)

[21.02 — Levels Adjustment](21%2002%20%E2%80%94%20Levels%20Adjustment%203f19bb7d023f81a5a1f3eca46aceb7e0.md)

[21.03 — Curves Adjustment](21%2003%20%E2%80%94%20Curves%20Adjustment%203f19bb7d023f81d7a150e94deb36fcb2.md)

[21.04 — Exposure, Brightness / Contrast Adjustments](21%2004%20%E2%80%94%20Exposure,%20Brightness%20Contrast%20Adjustments%203f19bb7d023f811b8464c7a095391a6b.md)

[21.05 — White Balance Adjustment](21%2005%20%E2%80%94%20White%20Balance%20Adjustment%203f19bb7d023f811b9978c1dff0a3f20d.md)

[21.06 — HSL & Vibrance Adjustments](21%2006%20%E2%80%94%20HSL%20&%20Vibrance%20Adjustments%203f19bb7d023f81b2b172e03b527571bc.md)

[21.07 — Black & White and Channel Mixer Adjustments](21%2007%20%E2%80%94%20Black%20&%20White%20and%20Channel%20Mixer%20Adjustment%203f19bb7d023f81dd8791fcea3d03ae2d.md)

[21.08 — Gradient Map, Selective Color, Threshold & Posterize](21%2008%20%E2%80%94%20Gradient%20Map,%20Selective%20Color,%20Threshold%20&%203f19bb7d023f81d8b320e7dc74cf6de3.md)

[21.09 — Gaussian Blur](21%2009%20%E2%80%94%20Gaussian%20Blur%203f19bb7d023f812d92cdf925110e9a1d.md)

[21.10 — Motion Blur](21%2010%20%E2%80%94%20Motion%20Blur%203f19bb7d023f81359dc9ed52f39fb3c2.md)

[21.11 — Sharpen, Unsharp Mask & High Pass](21%2011%20%E2%80%94%20Sharpen,%20Unsharp%20Mask%20&%20High%20Pass%203f19bb7d023f814b950ed2ba4e981fb3.md)

[21.12 — Noise, Denoise & Grain](21%2012%20%E2%80%94%20Noise,%20Denoise%20&%20Grain%203f19bb7d023f81c48fa9d13503707ebb.md)

[21.13 — Vignette](21%2013%20%E2%80%94%20Vignette%203f19bb7d023f817abd97c8359b99f95b.md)

[21.14 — Shadows, Glows & Outline Layer Effects](21%2014%20%E2%80%94%20Shadows,%20Glows%20&%20Outline%20Layer%20Effects%203f19bb7d023f81c99144dacf433409da.md)

[21.15 — Distortion, Perspective & Warp Live Filters](21%2015%20%E2%80%94%20Distortion,%20Perspective%20&%20Warp%20Live%20Filter%203f19bb7d023f81dab743e5423df18358.md)

[21.16 — Effect Presets, Parameter Migration, Plugin Effects & Conformance](21%2016%20%E2%80%94%20Effect%20Presets,%20Parameter%20Migration,%20Plugi%203f19bb7d023f81bd8927ea257bee9c93.md)