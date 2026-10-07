# 24.5 — Import/Export Fidelity Matrix Schema, Reports & Release Claim Policy

# Machine-readable matrix

FormatCapability {

exporter/importer id/version;

format/profile/version;

feature ID;

direction;

capability;

constraints;

fidelity grade;

strategy options;

tested fixture IDs;

known limitations;

}

# Feature IDs

ptnd.vector.path, text.artistic, text.frame, color.cmyk, [color.spot](http://color.spot), effect.gaussian_blur, mask.pixel, surface.multi_page etc.

# Analyze

Before export, traverse snapshot and compare used feature set to target matrix. Produce DegradationPlan grouped by objects/features.

# Import report

Parser reports source feature recognized/preserved/approximated/dropped with source location and resulting ObjectIds.

# Release claims

Marketing/help generated from matrix statuses; cannot call “CMYK PDF support” if only RGB export tests exist.

# Version

Matrix is per adapter/version and can improve without PTND schema change.

# Tests

Each Supported/Exact claim links at least one deterministic fixture; CI detects claim without fixture.