# 09.12.2 — Export Presets, Degradation Plan, PDF/SVG/Raster Semantics & Output Validation

# Preset

ExporterId + format version + options + target profile + area policy + fidelity policy + naming template. Preset is user/workspace resource, not document truth unless explicitly embedded.

# Degradation item

code, affected IDs, feature, target constraint, proposed strategy, fidelity grade, alternatives and previewability. GUI groups duplicates but preserves object navigation.

# Strategy

Preserve/native, expand, outline text, rasterize feature/object, approximate, omit only if user policy allows, reject.

# PDF

Map paths/text/images/gradients/blends/color profiles as natively as writer supports. Font embedding/subsetting license restrictions respected. Bleed/crop marks/output intent options defined. PDF/X requires validator evidence and exact conformance target before marketed.

# SVG

Keep paths/shapes/text where representable. Namespace metadata only if standards-compliant/safe. Live effects unsupported by SVG export are expanded/rasterized according preset. Multiple Surfaces become multiple SVG outputs or chosen target, not invalid custom multipage SVG.

# Raster

Final evaluated scene rendered at requested dimensions/DPI/profile/bit depth/alpha. Resampling filter selected. Metadata privacy options explicit.

# Atomic output

Encode temp -> close/flush -> structural validation/reopen where feasible -> destination conflict policy -> atomic replace/rename.

# Report

Outputs, sizes, warnings, degradations, substituted fonts/profiles, validation status, duration and revision exported.

# Determinism

Given same snapshot/options/backend-quality, semantic output should be reproducible within format metadata allowances; test normalization handles timestamps/compression variance.