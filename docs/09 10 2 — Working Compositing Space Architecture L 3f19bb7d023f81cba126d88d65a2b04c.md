# 09.10.2 — Working/Compositing Space Architecture: Linear Light, CMYK Documents & Conversion Boundaries

# Distinguish

**Canonical object color** may be RGB/CMYK/Lab/Spot.

**Render working space** is where raster/vector composition math occurs.

**Display space** is monitor output.

**Export space** target format/profile.

# V1 decision framework

Preferred interactive compositor uses high-precision linear-light RGB working representation for predictable effects/blends where semantics allow, while canonical CMYK/spot values remain preserved. However PDF-compatible blend semantics and CMYK proof require validated conversion strategy. Final choice is ADR backed by oracle tests.

# Conversion boundary

Canonical paint -> ColorEngine transform into render working space -> compositor -> proof/display. Never overwrite canonical components for mere viewing.

# Raster CMYK

CMYK RasterLayer can remain canonical CMYK tiles. Renderer converts visible tiles to working representation through cached transform. Pixel tools operating native CMYK vs rendered RGB must declare semantic mode; V1 may restrict some filters with explicit conversion requirement.

# Assignment vs conversion

Assign Profile changes interpretation metadata without component remap.

Convert Profile transforms component values to preserve appearance.

UI/actions separate both and warn appropriately.

# Blend modes

Blend reference specifies color representation. Component blend modes Hue/Saturation/Color/Luminosity need defined conversion model and oracle.

# Precision

Float16 GPU candidate, float32 CPU oracle. Error budgets set by perceptual/numeric test corpus.

# Tests

Same canonical colors under different display profiles, CMYK proof, assign vs convert, blend modes and export consistency.