# 20.6 — PDF/X Targets, Output Intent, Conformance Validation & Transparency Flattening

# Principle

Do not label output PDF/X until external/embedded validator evidence proves selected standard requirements.

# Candidate targets

PDF/X-1a, PDF/X-3, PDF/X-4 prioritized according writer capability. Each preset pins PDF version, allowed color spaces, transparency behavior, font embedding, output intent and metadata.

# Output intent

ICC OutputIntent embedded with identifier/condition metadata. Selected print profile becomes explicit export configuration, not merely current monitor/document profile.

# PDF/X-1a

Requires CMYK/spot-oriented output and transparency flattening/no live transparency under target rules. RGB content converted according preset.

# PDF/X-4

Can preserve live transparency and ICC-managed content within standard constraints.

# Flattening

If required: divide artwork into vector/raster regions while preserving appearance, overprint and text/vector where possible. Flatten resolution and gradient/complexity settings explicit; degradation report lists rasterized regions.

# Fonts

Embed/subset required fonts subject license. Missing/unembeddable fonts block or outline only with explicit user policy.

# Validation

Export temp PDF -> external/library validator -> parse conformance result -> only then mark success “PDF/X valid.” Validator version recorded in ExportReport/evidence.

# Tests

Known valid/invalid fixtures, spots, overprint, transparency, missing font, output intent and validator roundtrip.