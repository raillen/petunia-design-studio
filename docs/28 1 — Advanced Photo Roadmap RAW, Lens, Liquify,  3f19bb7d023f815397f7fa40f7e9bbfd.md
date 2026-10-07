# 28.1 — Advanced Photo Roadmap: RAW, Lens, Liquify, HDR, Panorama, Stacking & Denoise

# RAW Develop

Post-V1 Planned/Research. Requires demosaic pipeline, camera profiles, white balance metadata, lens shading/noise, highlight recovery and nondestructive development parameters. Prefer external mature raw engine adapter over inventing demosaic casually.

# Lens correction

Profile-based distortion, chromatic aberration, vignetting and geometry correction. Provider/profile format/licensing must be evaluated.

# Liquify

Post-V1 candidate. Dedicated deformation mesh/field workspace with push/twirl/pucker/bloat/freeze/thaw, high-res preview and nondestructive/live option if architecture supports.

# HDR merge

Input bracketed images -> alignment -> merge to high-dynamic-range working representation -> tone mapping as separate adjustment. Requires metadata/alignment/ghost handling.

# Panorama

Feature matching/alignment/projection/blending pipeline, cancellable job, staged result and editable crop/warp where possible.

# Focus stacking

Alignment + focus measure + blend/mask generation. Output can retain source layers/masks for editability.

# Denoise

Classical/ML denoise providers. ML models require size/license/hardware/privacy policy. Local-only is default expectation unless network provider explicitly chosen.

# Status

None of these are V1 blockers. Architecture must avoid making them impossible, but agents must not implement opportunistically.