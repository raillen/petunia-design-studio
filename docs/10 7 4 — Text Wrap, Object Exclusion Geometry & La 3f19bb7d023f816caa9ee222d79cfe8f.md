# 10.7.4 — Text Wrap, Object Exclusion Geometry & Layout Invalidation

# Wrap descriptor

WrapMode None, BoundingBox, ObjectShape/Contour, custom vector path; offset per side/uniform; side rule both/largest/left/right future.

# Source

Drawable object can publish wrap geometry independent from visibility only if setting says so. Raster alpha-contour wrap is derived/high-cost optional.

# Layout

Text frame computes available line segments by subtracting wrap exclusion from frame column line bands. Wrap objects scoped to relevant Surface/story stacking policy.

# Invalidation

Moving/changing wrap object invalidates intersecting text frames/paragraph layout only within affected region/order. ChangeSet dependency tracks ObjectId.

# UI

Text Wrap panel/properties and canvas outline. Disabled reason when object/frame relation invalid.

# Tests

rotated shape wrap, multiple objects, offset, columns, object moved across frame and no wrap across unrelated Surface.