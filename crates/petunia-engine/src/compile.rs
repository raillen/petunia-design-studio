//! Snapshot compiler: authorial document to immutable render model.
//!
//! Evaluation runs in two conceptual stages: authorial evaluation
//! (flattened geometry, resolved paint, group semantics) and render
//! compilation (primitives in stable paint order). Anything without
//! a provider — shaped text runs, decoded images, expanded symbols,
//! parametric shapes — degrades with a [`CompileWarning`] instead of
//! failing the frame or inventing pixels.

use crate::geometry::bezier::flatten_contour;
use crate::transaction::DocumentRevision;
use petunia_core::{ObjectId, PageId, Rect, SceneItem, SceneNode, Size2, Tolerance, VectorPath};
use petunia_render_model::{
    CompileWarning, RenderAppearance, RenderClip, RenderColor, RenderFrame, RenderGroup,
    RenderPaint, RenderPath, RenderPrimitive, RenderQuality, RenderResourceTable, RenderSnapshot,
    RenderTarget, SnapshotRevision, VectorPrimitive, ViewTransform,
};
use std::collections::HashMap;

/// Flatten tolerance per quality, in document units.
#[must_use]
pub fn flatten_tolerance(quality: RenderQuality) -> f64 {
    match quality {
        RenderQuality::InteractivePreview => 0.5,
        RenderQuality::Authoring => 0.15,
        RenderQuality::Export => 0.05,
    }
}

/// Compile one document into a snapshot plus warnings. Single page
/// for now: page and spread modeling lands with the document
/// structure wave.
#[must_use]
pub fn compile_document(
    document: &petunia_core::Document,
    revision: DocumentRevision,
    quality: RenderQuality,
) -> (RenderSnapshot, Vec<CompileWarning>) {
    let tolerance = Tolerance::new(flatten_tolerance(quality)).expect("constant tolerance");
    let mut compiler = Compiler {
        tolerance,
        warnings: Vec::new(),
    };
    let mut primitives = Vec::new();
    for id in document.scene.root_order() {
        compiler.compile_node(document, *id, &mut primitives);
    }
    let canvas = document.default_page_size();
    let snapshot = RenderSnapshot {
        revision: SnapshotRevision(revision.0),
        pages: vec![petunia_render_model::RenderPage {
            page: PageId::new_v4(),
            size: Size2::new(canvas.width.max(1.0), canvas.height.max(1.0))
                .unwrap_or(Size2::new(8.0, 8.0).expect("constant size")),
            primitives,
        }],
        resources: RenderResourceTable::new(),
    };
    (snapshot, compiler.warnings)
}

struct Compiler {
    tolerance: Tolerance,
    warnings: Vec<CompileWarning>,
}

impl Compiler {
    fn warn(&mut self, source: ObjectId, message: impl Into<String>) {
        self.warnings.push(CompileWarning {
            source,
            message: message.into(),
        });
    }

    fn compile_node(
        &mut self,
        document: &petunia_core::Document,
        id: ObjectId,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(node) = document.scene.get_node(id) else {
            self.warn(id, "dangling scene reference skipped");
            return;
        };
        match &node.item {
            SceneItem::Path(object) => {
                out.push(self.compile_path(document, node, object));
            }
            SceneItem::Group(children) => {
                let mut compiled = Vec::new();
                for child in children {
                    self.compile_node(document, *child, &mut compiled);
                }
                // Opacity-only groups flatten when provably identical;
                // anything else keeps explicit group semantics.
                if node.opacity >= 1.0
                    && node.clip.is_none()
                    && compiled
                        .iter()
                        .all(|primitive| matches!(primitive, RenderPrimitive::Vector(_)))
                {
                    out.extend(compiled);
                } else {
                    // Clips resolve in the attach_clips post-pass,
                    // which sees the whole document at once.
                    out.push(RenderPrimitive::Group(RenderGroup {
                        source: id,
                        children: compiled,
                        opacity: node.opacity,
                        blend_mode: petunia_core::appearance::BlendMode::Normal,
                        mask: None,
                        clip: None,
                        effects: Vec::new(),
                        isolation: petunia_render_model::IsolationMode::Flattened,
                        bounds: self.node_bounds(document, node),
                    }));
                }
            }
            SceneItem::Shape(_)
            | SceneItem::Text(_)
            | SceneItem::Image(_)
            | SceneItem::PixelLayer(_)
            | SceneItem::Trace(_)
            | SceneItem::GeneratedVector(_)
            | SceneItem::SymbolInstance(_) => {
                self.warn(
                    id,
                    "item needs an evaluation provider not present in v0.1; skipped with warning",
                );
            }
        }
    }

    fn compile_path(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        object: &petunia_core::PathObject,
    ) -> RenderPrimitive {
        let mut geometry = RenderPath::new();
        for contour in &object.path.contours {
            let flat = flatten_contour(contour, self.tolerance);
            geometry.push_contour(
                flat.iter().map(|point| (point.x, point.y)).collect(),
                contour.closed,
            );
        }
        // First enabled fill and stroke win; the stack order beyond
        // that belongs to a multi-pass compositor, not this compiler.
        let mut fill = None;
        let mut stroke = None;
        for item in object.appearance.items.iter().filter(|item| item.enabled) {
            match &item.kind {
                petunia_core::AppearanceKind::Fill(paint) => {
                    if fill.is_none() {
                        fill = self.compile_paint(document, node, node.id, paint);
                    }
                }
                petunia_core::AppearanceKind::Stroke(style) => {
                    if stroke.is_none() {
                        let width = style.width.max(0.0);
                        if let Some(paint) = self.compile_color(document, node.id, &style.paint) {
                            stroke = Some(petunia_render_model::RenderStroke { paint, width });
                        }
                    }
                }
            }
        }
        let bounds = self.node_bounds_opt(object);
        RenderPrimitive::Vector(VectorPrimitive {
            source: node.id,
            geometry,
            appearance: RenderAppearance {
                fill,
                stroke,
                opacity: node.opacity.clamp(0.0, 1.0),
            },
            transform: node.transform,
            bounds,
        })
    }

    /// Resolve one authorial paint to a render paint. Solid colors
    /// resolve; gradients, patterns and unimplemented spaces degrade
    /// with a warning instead of a silent fallback.
    fn compile_paint(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        id: ObjectId,
        paint: &petunia_core::Paint,
    ) -> Option<RenderPaint> {
        match paint {
            petunia_core::Paint::Solid(source) => self.compile_color(document, id, source),
            petunia_core::Paint::LinearGradient(gradient)
            | petunia_core::Paint::RadialGradient(gradient)
            | petunia_core::Paint::ConicalGradient(gradient) => {
                self.compile_gradient(document, node, id, paint, gradient)
            }
            petunia_core::Paint::Pattern(pattern) => match &pattern.source {
                petunia_core::PatternSource::Raster(res_id) => {
                    let transform = if pattern.space == petunia_core::PaintSpace::Object {
                        node.transform.concat(pattern.transform)
                    } else {
                        pattern.transform
                    };
                    Some(RenderPaint::Pattern(petunia_render_model::RenderPattern {
                        resource: *res_id,
                        width: 64,
                        height: 64,
                        repeat_x: pattern.repeat_x,
                        repeat_y: pattern.repeat_y,
                        transform,
                    }))
                }
                petunia_core::PatternSource::Vector(_) => {
                    self.warn(id, "vector pattern paint needs a rasterizer pass; skipped");
                    None
                }
            },
        }
    }

    fn compile_gradient(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        id: ObjectId,
        paint: &petunia_core::Paint,
        gradient: &petunia_core::Gradient,
    ) -> Option<RenderPaint> {
        if gradient.validate().is_err() {
            self.warn(id, "invalid gradient definition; skipped");
            return None;
        }
        let mut stops = Vec::with_capacity(gradient.stops.len());
        for stop in &gradient.stops {
            let Some(color) = self.resolve_color(document, id, &stop.color) else {
                self.warn(id, "gradient stop without a resolvable color; skipped");
                return None;
            };
            stops.push(petunia_render_model::RenderGradientStop {
                offset: stop.offset,
                color,
                midpoint: stop.midpoint,
            });
        }
        let (start, end, radius, start_angle) = match &gradient.geometry {
            petunia_core::GradientGeometry::Linear { start, end } => (*start, *end, 0.0, 0.0),
            petunia_core::GradientGeometry::Radial { center, radius } => {
                (*center, *center, *radius, 0.0)
            }
            petunia_core::GradientGeometry::Conical {
                center,
                start_angle,
            } => (*center, *center, 0.0, start_angle.radians()),
        };
        let place = |point: petunia_core::Point| {
            if gradient.space == petunia_core::PaintSpace::Object {
                node.transform.transform_point(point)
            } else {
                point
            }
        };
        let start = place(start);
        let end = place(end);
        let evaluated = petunia_render_model::RenderGradient {
            stops,
            interpolation: gradient.interpolation,
            spread: gradient.spread,
            space: gradient.space,
            start: (start.x, start.y),
            end: (end.x, end.y),
            radius,
            start_angle,
        };
        match paint {
            petunia_core::Paint::LinearGradient(_) => Some(RenderPaint::LinearGradient(evaluated)),
            petunia_core::Paint::RadialGradient(_) => Some(RenderPaint::RadialGradient(evaluated)),
            petunia_core::Paint::ConicalGradient(_) => {
                Some(RenderPaint::ConicalGradient(evaluated))
            }
            _ => None,
        }
    }

    fn compile_color(
        &mut self,
        document: &petunia_core::Document,
        id: ObjectId,
        source: &petunia_core::ColorSource,
    ) -> Option<RenderPaint> {
        self.resolve_color(document, id, source)
            .map(RenderPaint::Solid)
    }

    /// Resolve one color source to a render color, following swatches
    /// and spot alternates. Paint-level wrappers decide the rest.
    fn resolve_color(
        &mut self,
        document: &petunia_core::Document,
        id: ObjectId,
        source: &petunia_core::ColorSource,
    ) -> Option<petunia_render_model::RenderColor> {
        use petunia_core::{BuiltinColorSpace, ColorSpaceRef, ColorValue, ProcessColorValue};
        let color = match source {
            petunia_core::ColorSource::Value(value) => value.clone(),
            petunia_core::ColorSource::Swatch(swatch) => {
                let Some(linked) = document.swatches.get(*swatch) else {
                    self.warn(id, "paint references a missing swatch; skipped");
                    return None;
                };
                match &linked.value {
                    petunia_core::SwatchValue::Color(color) => color.clone(),
                    petunia_core::SwatchValue::Gradient(_) => {
                        self.warn(id, "swatch gradient needs a renderer pass; skipped");
                        return None;
                    }
                }
            }
        };
        match color {
            ColorValue::Process(process) => match (&process.value, &process.space) {
                (
                    ProcessColorValue::Rgb(channels),
                    ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
                ) => Some(srgb_to_linear(channels)),
                (
                    ProcessColorValue::Rgb(channels),
                    ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
                ) => Some(RenderColor {
                    r: channels.r,
                    g: channels.g,
                    b: channels.b,
                    a: channels.alpha.clamp(0.0, 1.0),
                }),
                _ => {
                    self.warn(id, "color needs a managed transform; skipped");
                    None
                }
            },
            ColorValue::Spot(reference) => {
                let Some(ink) = document.spots.get(reference.id) else {
                    self.warn(id, "spot ink without a definition; skipped");
                    return None;
                };
                if !reference.tint.is_finite() || !(0.0..=1.0).contains(&reference.tint) {
                    self.warn(id, "spot tint outside 0..=1; skipped");
                    return None;
                }
                // The alternate preview stands in for the ink, scaled
                // by the tint the use declares.
                let mut preview = self.resolve_color(
                    document,
                    id,
                    &petunia_core::ColorSource::Value(ColorValue::Process(ink.alternate.clone())),
                )?;
                preview.a = (preview.a * reference.tint).clamp(0.0, 1.0);
                Some(preview)
            }
        }
    }

    fn node_bounds(&self, document: &petunia_core::Document, node: &SceneNode) -> Rect {
        match &node.item {
            SceneItem::Path(object) => self
                .path_bounds(&object.path)
                .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0)),
            SceneItem::Group(children) => {
                let mut bounds: Option<Rect> = None;
                for child in children {
                    if let Some(node) = document.scene.get_node(*child) {
                        let child_bounds = self.node_bounds(document, node);
                        bounds = Some(match bounds {
                            Some(existing) => union_rect(existing, child_bounds),
                            None => child_bounds,
                        });
                    }
                }
                bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
            }
            _ => Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    fn node_bounds_opt(&self, object: &petunia_core::PathObject) -> Rect {
        let mut bounds: Option<Rect> = None;
        for contour in &object.path.contours {
            for point in &contour.nodes {
                let slot = Rect::new(point.point.x, point.point.y, 0.0, 0.0);
                bounds = Some(match bounds {
                    Some(existing) => union_rect(existing, slot),
                    None => slot,
                });
            }
        }
        let mut bounds = bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        // Conservative pad for the widest enabled stroke.
        let mut pad = 0.0f64;
        for item in object.appearance.items.iter().filter(|item| item.enabled) {
            if let petunia_core::AppearanceKind::Stroke(style) = &item.kind {
                pad = pad.max((style.width / 2.0).max(0.0));
            }
        }
        if pad > 0.0 {
            bounds = Rect::new(
                bounds.x - pad,
                bounds.y - pad,
                bounds.width + pad * 2.0,
                bounds.height + pad * 2.0,
            );
        }
        bounds
    }

    fn path_bounds(&self, path: &VectorPath) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        for contour in &path.contours {
            for point in flatten_contour(contour, self.tolerance) {
                let slot = Rect::new(point.x, point.y, 0.0, 0.0);
                bounds = Some(match bounds {
                    Some(existing) => union_rect(existing, slot),
                    None => slot,
                });
            }
        }
        bounds
    }
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    Rect::new(
        a.x.min(b.x),
        a.y.min(b.y),
        (a.x + a.width).max(b.x + b.width) - a.x.min(b.x),
        (a.y + a.height).max(b.y + b.height) - a.y.min(b.y),
    )
}

fn srgb_to_linear(color: &petunia_core::Rgba) -> RenderColor {
    RenderColor {
        r: encoded_to_linear(color.r),
        g: encoded_to_linear(color.g),
        b: encoded_to_linear(color.b),
        a: color.alpha.clamp(0.0, 1.0),
    }
}

fn encoded_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// Convenience frame builder for headless rendering and tests.
#[must_use]
pub fn headless_frame(
    snapshot: RenderSnapshot,
    width: u32,
    height: u32,
    scale: f64,
) -> RenderFrame {
    RenderFrame {
        snapshot,
        view: ViewTransform {
            scale,
            offset_x: 0.0,
            offset_y: 0.0,
        },
        target: RenderTarget { width, height },
    }
}

/// Clip bindings resolve through their source path geometry.
pub trait ClipResolver {
    /// Flattened clip polygon in document space, if resolvable.
    fn resolve_clip(
        &self,
        document: &petunia_core::Document,
        binding: &petunia_core::ClipBinding,
    ) -> Option<Vec<(f64, f64)>>;
}

/// Default resolver: path sources flatten, anything else degrades.
pub struct PathClipResolver {
    tolerance: Tolerance,
}

impl PathClipResolver {
    /// Create with an explicit flatten tolerance.
    #[must_use]
    pub fn new(tolerance: Tolerance) -> Self {
        Self { tolerance }
    }
}

impl ClipResolver for PathClipResolver {
    fn resolve_clip(
        &self,
        document: &petunia_core::Document,
        binding: &petunia_core::ClipBinding,
    ) -> Option<Vec<(f64, f64)>> {
        let node = document.scene.get_node(binding.source)?;
        match &node.item {
            SceneItem::Path(object) => {
                let mut polygon = Vec::new();
                for contour in &object.path.contours {
                    if !contour.closed {
                        continue;
                    }
                    polygon.extend(
                        flatten_contour(contour, self.tolerance)
                            .iter()
                            .map(|point| (point.x, point.y)),
                    );
                }
                if polygon.len() >= 3 {
                    Some(polygon)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Attach resolved clips to group primitives carrying bindings.
/// Groups built without clip data gain it here when resolvable.
pub fn attach_clips(
    document: &petunia_core::Document,
    primitives: &mut [RenderPrimitive],
    resolver: &dyn ClipResolver,
) {
    // Clip bindings live on scene nodes; match them by source id.
    let mut bindings: HashMap<ObjectId, petunia_core::ClipBinding> = HashMap::new();
    for id in document.scene.root_order() {
        if let Some(node) = document.scene.get_node(*id) {
            if let Some(binding) = node.clip {
                bindings.insert(node.id, binding);
            }
        }
    }
    for primitive in primitives {
        let source = match primitive {
            RenderPrimitive::Vector(vector) => vector.source,
            RenderPrimitive::Group(group) => group.source,
            RenderPrimitive::Text(text) => text.source,
            RenderPrimitive::Image(image) => image.source,
            RenderPrimitive::Raster(raster) => raster.source,
        };
        if let (Some(binding), RenderPrimitive::Group(group)) = (bindings.get(&source), primitive) {
            if group.clip.is_none() {
                group.clip = resolver
                    .resolve_clip(document, binding)
                    .map(RenderClip::Polygon);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{SceneNode, VectorPath};

    fn document_with_rect() -> petunia_core::Document {
        let mut document = petunia_core::Document::new("compile");
        let page = document.scene.default_page();
        document.scene.insert_node(SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            petunia_core::ParentRef::Page(page),
        ));
        document
    }

    #[test]
    fn compile_emits_paths_with_paint_and_bounds() {
        let document = document_with_rect();
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(snapshot.revision, SnapshotRevision(0));
        assert_eq!(snapshot.pages.len(), 1);
        assert_eq!(snapshot.pages[0].primitives.len(), 1);
        match &snapshot.pages[0].primitives[0] {
            RenderPrimitive::Vector(vector) => {
                assert!(vector.appearance.fill.is_some());
                assert_eq!(vector.bounds.width, 10.0);
                assert!(!vector.geometry.contours.is_empty());
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn appearance_stack_drives_fill_and_stroke() {
        use petunia_core::{
            Appearance, AppearanceItem, AppearanceItemId, AppearanceKind, BlendMode,
            BuiltinColorSpace, ColorSource, ColorSpaceRef, ColorValue, ProcessColor,
            ProcessColorValue, Rgba, StrokeCap, StrokeJoin, StrokeStyle,
        };
        fn solid(r: f32, g: f32, b: f32) -> petunia_core::Paint {
            petunia_core::Paint::Solid(ColorSource::Value(ColorValue::Process(ProcessColor {
                value: ProcessColorValue::Rgb(Rgba {
                    r,
                    g,
                    b,
                    alpha: 1.0,
                }),
                space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            })))
        }
        fn item(kind: petunia_core::AppearanceKind) -> AppearanceItem {
            AppearanceItem {
                id: AppearanceItemId::new_v4(),
                enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                kind,
            }
        }
        let mut document = document_with_rect();
        let id = document.scene.root_order()[0];
        let appearance = Appearance {
            items: vec![
                item(AppearanceKind::Fill(solid(1.0, 0.0, 0.0))),
                item(AppearanceKind::Fill(solid(0.0, 0.0, 1.0))),
                item(AppearanceKind::Stroke(
                    StrokeStyle::new(
                        4.0,
                        StrokeCap::Round,
                        StrokeJoin::Round,
                        ColorSource::Value(ColorValue::Process(ProcessColor {
                            value: ProcessColorValue::Rgb(Rgba {
                                r: 0.0,
                                g: 1.0,
                                b: 0.0,
                                alpha: 1.0,
                            }),
                            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
                        })),
                    )
                    .expect("stroke"),
                )),
            ],
        };
        if let petunia_core::SceneItem::Path(object) =
            &mut document.scene.get_node_mut(id).expect("node").item
        {
            object.appearance = appearance;
        }
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        let RenderPrimitive::Vector(vector) = &snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        // First enabled fill wins: sRGB red linearizes to 1.0.
        let RenderPaint::Solid(fill) = vector.appearance.fill.as_ref().expect("fill") else {
            panic!("expected solid fill");
        };
        assert_eq!((fill.r, fill.g, fill.b), (1.0, 0.0, 0.0));
        // The stroke resolves with its own paint and width.
        let stroke = vector.appearance.stroke.as_ref().expect("stroke");
        assert_eq!(stroke.width, 4.0);
        let RenderPaint::Solid(paint) = &stroke.paint else {
            panic!("expected solid stroke");
        };
        assert!(paint.g > 0.9, "{paint:?}");
    }

    #[test]
    fn spot_ink_resolves_through_its_alternate() {
        use petunia_core::{
            Appearance, AppearanceItem, AppearanceItemId, AppearanceKind, BlendMode, ColorSource,
            ColorValue, SpotColor, SpotColorRef,
        };
        let mut document = document_with_rect();
        let id = document.scene.root_order()[0];
        let ink = SpotColor {
            id: petunia_core::SpotColorId::new_v4(),
            name: "Brand Red".to_string(),
            alternate: petunia_core::ProcessColor {
                value: petunia_core::ProcessColorValue::Rgb(petunia_core::Rgba {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    alpha: 1.0,
                }),
                space: petunia_core::ColorSpaceRef::Builtin(petunia_core::BuiltinColorSpace::Srgb),
            },
        };
        let ink_id = ink.id;
        document.spots.insert(ink);
        let appearance = Appearance {
            items: vec![AppearanceItem {
                id: AppearanceItemId::new_v4(),
                enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                kind: AppearanceKind::Fill(petunia_core::Paint::Solid(ColorSource::Value(
                    ColorValue::Spot(SpotColorRef {
                        id: ink_id,
                        tint: 0.5,
                    }),
                ))),
            }],
        };
        if let petunia_core::SceneItem::Path(object) =
            &mut document.scene.get_node_mut(id).expect("node").item
        {
            object.appearance = appearance;
        }
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        let RenderPrimitive::Vector(vector) = &snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        // The blue alternate stands in for the ink.
        let RenderPaint::Solid(fill) = vector.appearance.fill.as_ref().expect("fill") else {
            panic!("expected solid fill");
        };
        assert_eq!((fill.r, fill.g, fill.b), (0.0, 0.0, 1.0));
    }

    #[test]
    fn linear_gradient_evaluates_with_resolved_stops() {
        use petunia_core::{
            Gradient, GradientGeometry, GradientInterpolation, GradientSpread, GradientStop,
            PaintSpace,
        };
        fn stop(offset: f32, v: f32) -> GradientStop {
            GradientStop::new(
                offset,
                petunia_core::ColorSource::Value(petunia_core::ColorValue::Process(
                    petunia_core::ProcessColor {
                        value: petunia_core::ProcessColorValue::Rgb(petunia_core::Rgba {
                            r: v,
                            g: v,
                            b: v,
                            alpha: 1.0,
                        }),
                        space: petunia_core::ColorSpaceRef::Builtin(
                            petunia_core::BuiltinColorSpace::Srgb,
                        ),
                    },
                )),
                0.5,
            )
            .expect("stop")
        }
        let mut document = document_with_rect();
        let id = document.scene.root_order()[0];
        let gradient = Gradient::new(
            vec![stop(0.0, 0.0), stop(1.0, 1.0)],
            GradientInterpolation::LinearRgb,
            GradientSpread::Pad,
            PaintSpace::Object,
            GradientGeometry::Linear {
                start: petunia_core::Point::new(10.0, 10.0),
                end: petunia_core::Point::new(30.0, 10.0),
            },
        )
        .expect("gradient");
        if let petunia_core::SceneItem::Path(object) =
            &mut document.scene.get_node_mut(id).expect("node").item
        {
            object.appearance = petunia_core::Appearance {
                items: vec![petunia_core::AppearanceItem {
                    id: petunia_core::AppearanceItemId::new_v4(),
                    enabled: true,
                    opacity: 1.0,
                    blend_mode: petunia_core::BlendMode::Normal,
                    kind: petunia_core::AppearanceKind::Fill(petunia_core::Paint::LinearGradient(
                        gradient,
                    )),
                }],
            };
        }
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        let RenderPrimitive::Vector(vector) = &snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        let RenderPaint::LinearGradient(resolved) = vector.appearance.fill.as_ref().expect("fill")
        else {
            panic!("expected a linear gradient");
        };
        assert_eq!(resolved.stops.len(), 2);
        assert_eq!(resolved.start, (10.0, 10.0));
        assert_eq!(resolved.end, (30.0, 10.0));
    }

    #[test]
    fn unresolvable_paint_degrades_with_warnings() {
        use petunia_core::{
            Appearance, AppearanceItem, AppearanceItemId, AppearanceKind, BlendMode,
        };
        let mut document = document_with_rect();
        let id = document.scene.root_order()[0];
        let appearance = Appearance {
            items: vec![AppearanceItem {
                id: AppearanceItemId::new_v4(),
                enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                kind: AppearanceKind::Fill(petunia_core::Paint::Solid(
                    petunia_core::ColorSource::Swatch(petunia_core::SwatchId::new_v4()),
                )),
            }],
        };
        if let petunia_core::SceneItem::Path(object) =
            &mut document.scene.get_node_mut(id).expect("node").item
        {
            object.appearance = appearance;
        }
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        let RenderPrimitive::Vector(vector) = &snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        assert!(
            vector.appearance.fill.is_none(),
            "missing swatch skips the fill"
        );
        assert_eq!(warnings.len(), 1, "{warnings:?}");
    }

    #[test]
    fn unsupported_items_degrade_with_warnings() {
        use petunia_core::{SceneItem, TextFlow, TextObject};
        let mut document = document_with_rect();
        let page = document.scene.default_page();
        let text = SceneNode {
            id: ObjectId::new_v4(),
            name: "text".to_string(),
            parent: petunia_core::ParentRef::Page(page),
            visible: true,
            locked: false,
            transform: petunia_core::Transform2D::IDENTITY,
            opacity: 1.0,
            geometry_effects: Default::default(),
            post_effects: Default::default(),
            clip: None,
            mask: None,
            item: SceneItem::Text(
                TextObject::new(
                    "Hi".to_string(),
                    Vec::new(),
                    Vec::new(),
                    petunia_core::TextContainer::Artistic,
                    TextFlow { next: None },
                )
                .expect("valid"),
            ),
        };
        document.scene.insert_node(text);
        let (snapshot, warnings) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        assert_eq!(snapshot.pages[0].primitives.len(), 1);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn clip_bindings_attach_as_polygons() {
        use petunia_core::{BindingSourceUse, ClipBinding};
        let mut document = document_with_rect();
        let ids: Vec<ObjectId> = document.scene.root_order().to_vec();
        let target = ids[0];
        // Wrap the box in a translucent group so the compiler emits
        // a Group primitive carrying the clip binding.
        let page = document.scene.default_page();
        let mut group = SceneNode::new_path(
            "group",
            VectorPath::new(),
            petunia_core::ParentRef::Page(page),
        );
        let group_id = group.id;
        group.item = SceneItem::Group(vec![target]);
        group.opacity = 0.5;
        let frame_rect = SceneNode::new_path(
            "frame",
            VectorPath::rect(2.0, 2.0, 6.0, 6.0),
            petunia_core::ParentRef::Page(page),
        );
        let source = frame_rect.id;
        document.scene.insert_node(frame_rect);
        document.scene.insert_node(group);
        let node = document.scene.get_node_mut(group_id).expect("group");
        node.clip =
            Some(ClipBinding::new(group_id, source, BindingSourceUse::BindingOnly).expect("valid"));
        let (mut snapshot, _) = compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        let resolver = PathClipResolver::new(Tolerance::new(0.1).expect("valid"));
        attach_clips(&document, &mut snapshot.pages[0].primitives, &resolver);
        let group = snapshot.pages[0]
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                RenderPrimitive::Group(group) => Some(group),
                _ => None,
            })
            .expect("group primitive");
        match group.clip.as_ref().expect("clip attached") {
            RenderClip::Polygon(points) => assert!(points.len() >= 3, "{points:?}"),
            RenderClip::Rect(_) => panic!("expected polygon clip"),
        }
    }
}
