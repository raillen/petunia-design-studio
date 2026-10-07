# 20.4 — PDF/X Profiles, Output Intent, Font Rules & External Validation

# Targets

Candidates: PDF/X-1a, PDF/X-3, PDF/X-4. Each target defines allowed PDF version, color spaces, transparency, output intent, fonts and metadata requirements.

# Certification rule

UI exposes target as conforming only after test corpus validates with at least one trusted external PDF/X validator plus internal structural checks.

# Output intent

PrintPreset selects ICC output profile and intent metadata. Required profile embedded according standard/license.

# Fonts

All fonts embedded/subset unless standard permits/exception. Missing or non-embeddable fonts block strict profile or require explicit outline strategy with fidelity warning.

# Color

X-1a typically requires CMYK/spot; RGB content converted according target. X-4 can preserve managed transparency/color per profile rules. Exact implementation follows selected standard revision.

# Metadata

Required identifiers/trapping/output condition metadata generated deterministically where standard demands.

# Validation

Export temp -> structural validator -> external/embedded conformance validator integration where available -> atomic commit only if strict preset says failure blocks.

# Tests

Known valid/invalid PDFs, missing output intent, nonembedded font, RGB in restricted target, live transparency and spot colors.