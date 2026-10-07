# 09.3.2 — Built-in Command Catalog v1 by Domain

# Object/hierarchy

CreateObject, DeleteObjects, DuplicateObjects, ReparentObjects, ReorderObjects, RenameObject, SetVisibility, SetLocked.

# Transform

SetTransform, TransformObjects, AlignObjects, DistributeObjects, FlipObjects.

# Geometry

CreateParametricShape, SetShapeParameters, CreatePath, EditPathNodes, InsertNode, DeleteNodes, JoinEndpoints, BreakPath, Close/Open/ReverseContour, ConvertShapeToPath, BooleanBake, CreateLiveBoolean, ShapeBuilderCommit, ExpandStroke.

# Appearance

Add/Remove/ReorderAppearanceEntry, SetFill, SetGradient, SetStroke, SetOpacity, SetBlendMode, Add/Set/RemoveEffect.

# Text

CreateArtisticText, CreateTextFrame, EditTextRange, SetCharacter/ParagraphStyle, Link/UnlinkTextFrames, SetTextPath, ConvertTextToCurves.

# Raster

CommitBrushStroke, ApplyFloodFill, ApplyRasterGradient, SetCrop, TrimToCrop, CommitClone/Heal/RetouchStroke, ApplyInpaintResult, ApplyFilterToPixels.

# Masks/adjustments

Attach/Remove/InvertMask, AddAdjustment, SetAdjustmentParams, AddLiveFilter, SetFilterParams.

# Surface/layout

Create/Delete/Move/ResizeSurface, SetMargins/Columns/Bleed, Add/Move/DeleteGuide, ApplySurfaceTemplate.

# Resources/data

PlaceResource, RelinkResource, Embed/UnembedResource, Add/Set/RemoveDataBinding.

# Rule

Each catalog item gets machine-readable payload schema and tests; this catalog prevents duplicate synonymous operations.