# 21.03 — Curves Adjustment

# ID

ptnd.adjustment.curves.

# Curve model

Per channel monotonic/nonmonotonic transfer curve defined by ordered control points x,y in normalized domain. Interpolation method (cubic/spline) fixed and serialized; endpoints default (0,0),(1,1).

# Editing

Add point, drag, numeric x/y, delete interior, reset channel, pencil/freehand curve optional post-V1. Points remain sorted by x; duplicate x resolved by rule.

# Evaluation

Compile control points to LUT/function at sufficient precision. Interpolate color channels in declared representation. Master/channel order fixed.

# UI

Graph grid, histogram background, channel selector, point table accessible alternative, input/output readout.

# ROI

Point effect.

# Tests

Identity, S-curve, extreme points, nonmonotonic curve if allowed, per-channel, numeric interpolation oracle and serialization.