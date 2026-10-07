# 09.10.3 — ICC Transform Pipeline, Rendering Intents, BPC, Cache & Display Profiles

# Transform descriptor

Source profile, destination profile, source/dest pixel format, rendering intent, black-point compensation, proof profile/intent flags and adaptation options.

# Intents

Perceptual, Relative Colorimetric, Absolute Colorimetric, Saturation where profile/engine supports. UI defaults by workflow but stores explicit choice in export/proof settings.

# BPC

Black Point Compensation applies only where transform engine defines; UI exposes in conversion/proof/export advanced settings.

# LittleCMS

CPU authoritative transform candidate. Transform creation failures return typed diagnostic; malformed profiles never crash document.

# Cache

TransformCache key is complete descriptor + profile fingerprints. Thread-safe immutable transform handles. Eviction bounded but transform objects small relative to images.

# Display profile

Platform adapter identifies monitor profile/EDID context. Per-window DisplayTransform changes when majority/current screen changes according Qt platform events. Rebuild async; previous valid transform used until replacement ready.

# Proof chain

source/document -> proof target simulation -> display. Absolute intent may simulate paper white/black if supported. Gamut warning computed from proof transform mask/criterion.

# Calibration changes

Display profile update invalidates only view transform/cache, not document.

# Tests

Known ICC test charts, profile missing/malformed, monitor move, intent/BPC variants and transform-cache key correctness.