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
use petunia_core::{
    Fill, ObjectId, PageId, Rect, SceneItem, SceneNode, Size2, Tolerance, VectorPath,
};
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
    let snapshot = RenderSnapshot {
        revision: SnapshotRevision(revision.0),
        pages: vec![petunia_render_model::RenderPage {
            page: PageId::new_v4(),
            size: Size2::new(
                document.setup.width.max(1.0),
                document.setup.height.max(1.0),
            )
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
            SceneItem::Path(path) => {
                out.push(self.compile_path(node, path));
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

    fn compile_path(&mut self, node: &SceneNode, path: &VectorPath) -> RenderPrimitive {
        let mut geometry = RenderPath::new();
        for contour in &path.contours {
            let flat = flatten_contour(contour, self.tolerance);
            geometry.push_contour(
                flat.iter().map(|point| (point.x, point.y)).collect(),
                contour.closed,
            );
        }
        let fill = node.fill.as_ref().map(|fill| match fill {
            Fill::Solid(color) => RenderPaint::Solid(srgb_to_linear(color)),
        });
        let bounds = self.node_bounds_opt(node, path);
        RenderPrimitive::Vector(VectorPrimitive {
            source: node.id,
            geometry,
            appearance: RenderAppearance {
                fill,
                stroke: node
                    .stroke
                    .as_ref()
                    .map(|stroke| petunia_render_model::RenderStroke {
                        paint: RenderPaint::Solid(srgb_to_linear_stroke(stroke)),
                        width: stroke.width.max(0.0),
                    }),
                opacity: node.opacity.clamp(0.0, 1.0),
            },
            transform: node.transform,
            bounds,
        })
    }

    fn node_bounds(&self, document: &petunia_core::Document, node: &SceneNode) -> Rect {
        match &node.item {
            SceneItem::Path(path) => self
                .path_bounds(path)
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

    fn node_bounds_opt(&self, node: &SceneNode, path: &VectorPath) -> Rect {
        let mut bounds: Option<Rect> = None;
        for contour in &path.contours {
            for point in &contour.nodes {
                let slot = Rect::new(point.point.x, point.point.y, 0.0, 0.0);
                bounds = Some(match bounds {
                    Some(existing) => union_rect(existing, slot),
                    None => slot,
                });
            }
        }
        let mut bounds = bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        // Conservative pad for stroke width.
        if let Some(stroke) = &node.stroke {
            let pad = (stroke.width / 2.0).max(0.0);
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

fn srgb_to_linear(color: &petunia_core::ColorRgba) -> RenderColor {
    RenderColor {
        r: encoded_to_linear(color.r),
        g: encoded_to_linear(color.g),
        b: encoded_to_linear(color.b),
        a: color.a.clamp(0.0, 1.0),
    }
}

fn srgb_to_linear_stroke(stroke: &petunia_core::scene::Stroke) -> RenderColor {
    srgb_to_linear(&petunia_core::ColorRgba::new(
        stroke.color.r,
        stroke.color.g,
        stroke.color.b,
        stroke.color.a,
    ))
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
            SceneItem::Path(path) => {
                let mut polygon = Vec::new();
                for contour in &path.contours {
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
            fill: None,
            stroke: None,
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
