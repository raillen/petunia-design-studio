# 10.7.5 — Layout Preflight: Overset, Effective DPI, Bleed, Missing Resources & Page Integrity

# Rules

Overset text; orphan/empty linked frame anomalies; missing font; missing/changed image; low effective DPI; object crossing trim without bleed; object outside all pages optional; empty page optional; invalid template ref; unsupported export feature.

# Effective DPI

For placed raster, derive source pixel density after object transform at target physical units. Nonuniform scaling reports min axes/effective range.

# Navigation

Issue links ObjectId/SurfaceId/story range; canvas selects/zooms target.

# Incremental

Rules depend on ChangeSet categories and can rerun affected page only.

# Severity

Preset/user policy defines warning/error; strict print preset may block.

# Tests

overset created/cleared, image scaled low DPI, bleed edge and missing template/resource.