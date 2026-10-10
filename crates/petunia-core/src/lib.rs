//! # petunia-core
//!
//! Core domain entities, vector geometry, scene graph, and serialization format
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Zero external GUI/GPU dependencies.
//! - Deterministic mathematical logic and serializable schemas.

#![forbid(unsafe_code)]

pub mod appearance;
pub mod color;
pub mod color_management;
pub mod crop;
pub mod document;
pub mod effects;
pub mod error;
pub mod generated;
pub mod guides;
pub mod id;
pub mod math;
pub mod paint;
pub mod path;
pub mod raster;
pub mod resources;
pub mod scene;
pub mod shape;
pub mod styles;
pub mod symbols;
pub mod text;
pub mod units;

pub use appearance::{
    Appearance, AppearanceItem, AppearanceKind, AppearanceOverride, AppearanceOverrideValue,
    AppearanceOverrides, AppearanceSource, BlendMode, DashPattern, Paint, PatternPaint,
    PatternRepeat, PatternSource, StrokeAlignment, StrokeCap, StrokeJoin, StrokeStyle,
    VariableWidthProfile, WidthPoint,
};
pub use color::{
    BuiltinColorSpace, Cmyka, ColorRgba, ColorSpace, ColorSpaceRef, ColorValue, DocumentColorSpec,
    EncodedRgba, Graya, Laba, LinearRgba, ProcessColor, ProcessColorValue, RenderingIntent, Rgba,
    SpotColor, SpotColorRef,
};
pub use color_management::ColorTransform;
pub use crop::{BindingSourceUse, ClipBinding, ImageSourceRect, NormalizedPoint};
pub use document::{Document, DocumentSetup};
pub use effects::{
    BlurParams, BooleanOperation, CornerParams, EffectInstance, GeometryEffect,
    GeometryEffectInstance, GeometryEffectStack, GlowParams, LiveBooleanParams, OffsetParams,
    PostPaintEffect, PostPaintEffectInstance, PostPaintEffectStack, ShadowParams,
};
pub use error::{CoreError, Result};
pub use generated::{
    AnalysisAlphaPolicy, BarcodeSpec, BarcodeSymbology, GeneratedVectorObject, GeneratorSpec,
    ImageTraceSpec, QrCodeSpec, QrErrorCorrection, QrMaskPolicy, QrPayload, QrVersionPolicy,
    TraceMode, TraceObject,
};
pub use guides::{
    AffineGridKind, AffineGridSpec, BaselineGridSpec, ExportColorOptions, ExportFormat,
    ExportSlice, GridDefinition, GridScope, GridSpec, Guide, GuideAxis, GuideScope,
    PerspectiveGridSpec, ProjectiveGridTransform, SliceExportPreset, SliceSource,
};
pub use id::{
    AppearanceItemId, ContourId, DocumentId, EffectId, GridId, GuideId, NodeId, ObjectId, PageId,
    ResourceId, SliceId, SpotColorId, StyleId, SwatchId, SymbolId,
};
pub use math::{Angle, Point, Rect, Size2, Tolerance, Transform2D, Vec2};
pub use paint::{
    ColorSource, Gradient, GradientGeometry, GradientInterpolation, GradientSpread, GradientStop,
    PaintSpace, Swatch, SwatchValue,
};
pub use path::{Contour, FillRule, NodeKind, PathNode, SegmentKind, VectorPath};
pub use raster::{
    ImageObject, ImageSamplingPolicy, PixelFormat, PixelLayer, PixelSize, PixelSurfaceDescriptor,
    PixelSurfaceRef, TileCoord,
};
pub use resources::{
    ContentHash, FontEmbedPolicy, ResourceKind, ResourceMetadata, ResourceRecord, ResourceRegistry,
    ResourceSource,
};
pub use scene::{Fill, MaskBinding, MaskMode, ParentRef, SceneGraph, SceneItem, SceneNode, Stroke};
pub use shape::{
    CornerRadii, EllipseArc, EllipseSpec, ParametricShape, PolygonSpec, RectangleSpec, StarSpec,
};
pub use styles::{
    AppearanceStyle, CharacterStyle, FontSlant, ParagraphStyle, StyleDefinition, StyleKind,
    StyleRegistry, TextAlignment,
};
pub use symbols::{
    has_symbol_cycle, SymbolDefinition, SymbolInstance, SymbolOverride, SymbolOverrideValue,
    SymbolRegistry,
};
pub use text::{
    validate_flow_links, CharacterStyleRef, FontAxis, FontRef, ParagraphRun, ParagraphStyleRef,
    TextContainer, TextFlow, TextFrameSpec, TextObject, TextOverflow, TextPathRef, TextRange,
    TextRun,
};
pub use units::{Unit, UnitValue};
