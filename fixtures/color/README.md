# Synthetic ICC fixture

`synthetic-cmyk.icc` is project-owned ICC v4 CMYK/XYZ output-profile data with
bounded two-point A2B/B2A LUTs. It exercises profile loading, channel units,
proof transforms, persistence and embedding. Its deliberately coarse inverse
is not a printer characterization, separation policy or colorimetric reference.
Never ship it as a production/FOGRA/SWOP profile.
