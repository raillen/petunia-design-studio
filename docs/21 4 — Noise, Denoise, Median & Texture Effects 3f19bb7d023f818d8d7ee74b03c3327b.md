# 21.4 — Noise, Denoise, Median & Texture Effects

# Noise

Parameters amount, distribution Uniform/Gaussian as supported, monochromatic/per-channel, seed. Seed is canonical for deterministic live effect.

# Median

Radius/kernel shape and channel policy. Tiled halo exact.

# Denoise

If V1 includes classical denoise, schema names algorithm version/preset/strength and preserves deterministic parameters. ML denoise belongs provider/plugin architecture unless explicitly built in.

# Texture/grain

Amount, size, roughness/seed and color mode where supported. Physical/perceptual behavior documented; avoid backend-random shader state.

# Randomness

All procedural effects use explicit deterministic PRNG algorithm/version or store generated resource. Same document/revision must render predictably within declared tolerance.

# Alpha

Noise can affect color only, alpha only or both only through explicit parameter; default protects alpha unless effect semantics say otherwise.

# Tests

Seed determinism, tile boundaries, constant fields, alpha, 8/16/float and GPU/CPU parity.