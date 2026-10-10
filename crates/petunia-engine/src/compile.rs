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
use petunia_core::{ObjectId, Rect, SceneItem, SceneNode, Tolerance};
use petunia_render_model::{
    CompileWarning, RenderAppearance, RenderClip, RenderColor, RenderFrame, RenderGroup,
    RenderPaint, RenderPath, RenderPrimitive, RenderQuality, RenderResourceTable, RenderSnapshot,
    RenderTarget, SnapshotRevision, VectorPrimitive, ViewTransform,
};
use std::collections::HashMap;
#[path = "evaluation.rs"]
mod evaluation;

pub use evaluation::{shape_path as evaluate_shape, stroke_outline as evaluate_stroke};

/// Flatten tolerance per quality, in document units.
#[must_use]
pub fn flatten_tolerance(quality: RenderQuality) -> f64 {
    match quality {
        RenderQuality::InteractivePreview => 0.5,
        RenderQuality::Authoring => 0.15,
        RenderQuality::Export => 0.05,
    }
}

/// Compile every authorial page into a revisioned snapshot plus warnings.
#[must_use]
pub fn compile_document(
    document: &petunia_core::Document,
    revision: DocumentRevision,
    quality: RenderQuality,
) -> (RenderSnapshot, Vec<CompileWarning>) {
    compile_document_with_providers(document, revision, quality, &CompileProviders::default())
}

/// Explicit runtime evaluation dependencies. No filesystem/resource reads occur in compilation.
#[derive(Default)]
pub struct CompileProviders<'a> {
    pub color_profiles: HashMap<petunia_core::ColorSpaceRef, std::sync::Arc<[u8]>>,
    pub fonts: Option<&'a crate::text::FontRegistry>,
    pub default_character: Option<&'a petunia_core::CharacterStyle>,
    pub default_paragraph: Option<&'a petunia_core::ParagraphStyle>,
    pub images: HashMap<
        petunia_core::ResourceId,
        std::sync::Arc<petunia_render_model::image::ResolvedImage>,
    >,
}

/// Compile every authorial page with immutable runtime resource providers.
#[must_use]
pub fn compile_document_with_providers(
    document: &petunia_core::Document,
    revision: DocumentRevision,
    quality: RenderQuality,
    providers: &CompileProviders<'_>,
) -> (RenderSnapshot, Vec<CompileWarning>) {
    let tolerance = Tolerance(flatten_tolerance(quality));
    let mut colors = crate::color_management::ColorEngine::new();
    for (space, bytes) in &providers.color_profiles {
        let _ = colors.register_profile(space.clone(), bytes);
    }
    let mut compiler = Compiler {
        tolerance,
        warnings: Vec::new(),
        binding_source: None,
        text_layouts: HashMap::new(),
        scope: CompileScope::new(document),
        providers,
        colors,
        depth: 0,
    };
    let pages = document
        .scene
        .page_ids()
        .into_iter()
        .map(|page| {
            let mut primitives = Vec::new();
            for id in document.scene.page_roots(page).unwrap_or_default() {
                compiler.compile_node(document, *id, &mut primitives);
            }
            petunia_render_model::RenderPage {
                page,
                size: document
                    .pages
                    .get(page)
                    .map_or(document.setup.default_page.size, |page| page.spec.size),
                primitives,
            }
        })
        .collect();
    let mut resources = RenderResourceTable::new();
    resources.images = providers.images.clone();
    for (resource_id, resource) in document.resources.iter() {
        resources.insert(
            resource_id,
            petunia_render_model::ResourceEntry {
                kind: resource.kind,
                revision: 0,
            },
        );
    }
    let snapshot = RenderSnapshot {
        revision: SnapshotRevision(revision.0),
        pages,
        resources,
    };
    (snapshot, compiler.warnings)
}

#[derive(Default)]
struct CompileScope {
    binding_only: std::collections::HashSet<ObjectId>,
    flow_previous: HashMap<ObjectId, Vec<ObjectId>>,
}
impl CompileScope {
    fn new(document: &petunia_core::Document) -> Self {
        let mut scope = Self::default();
        for node in scene_nodes(document) {
            if let Some(clip) = node.clip {
                if clip.use_as == petunia_core::BindingSourceUse::BindingOnly {
                    scope.binding_only.insert(clip.source);
                }
            }
            if let SceneItem::Text(text) = &node.item {
                if let Some(next) = text.flow.next {
                    scope.flow_previous.entry(next).or_default().push(node.id);
                }
            }
        }
        scope
    }
}

struct Compiler<'a> {
    tolerance: Tolerance,
    warnings: Vec<CompileWarning>,
    binding_source: Option<ObjectId>,
    text_layouts: HashMap<ObjectId, crate::text::TextLayout>,
    scope: CompileScope,
    providers: &'a CompileProviders<'a>,
    colors: crate::color_management::ColorEngine,
    depth: usize,
}

impl Compiler<'_> {
    fn flow_layout(
        &mut self,
        document: &petunia_core::Document,
        id: ObjectId,
        fonts: &crate::text::FontRegistry,
        character: &petunia_core::CharacterStyle,
        paragraph: &petunia_core::ParagraphStyle,
    ) -> Option<Result<crate::text::TextLayout, crate::text::TextEvaluationError>> {
        if let Some(layout) = self.text_layouts.get(&id) {
            return Some(Ok(layout.clone()));
        }
        let mut root = id;
        let mut seen = std::collections::HashSet::new();
        let mut linked = document.scene.get_node(id).is_some_and(
            |node| matches!(&node.item,SceneItem::Text(text) if text.flow.next.is_some()),
        );
        loop {
            if !seen.insert(root) || seen.len() > 64 {
                return Some(Err(crate::text::TextEvaluationError::Invalid(
                    "flow cycle/depth limit".into(),
                )));
            }
            let predecessors = self
                .scope
                .flow_previous
                .get(&root)
                .map(Vec::as_slice)
                .unwrap_or_default();
            match predecessors {
                [] => break,
                [previous] => {
                    root = *previous;
                    linked = true;
                }
                _ => {
                    return Some(Err(crate::text::TextEvaluationError::Invalid(
                        "frame has multiple incoming text flows".into(),
                    )))
                }
            }
        }
        if !linked {
            return None;
        }
        let mut frames = Vec::new();
        let mut cursor = Some(root);
        seen.clear();
        while let Some(id) = cursor {
            if !seen.insert(id) || frames.len() >= 64 {
                return Some(Err(crate::text::TextEvaluationError::Invalid(
                    "flow cycle/depth limit".into(),
                )));
            }
            let Some(SceneItem::Text(text)) = document.scene.get_node(id).map(|node| &node.item)
            else {
                return Some(Err(crate::text::TextEvaluationError::Invalid(
                    "flow target is not text".into(),
                )));
            };
            frames.push((id, text));
            cursor = text.flow.next;
        }
        match crate::text::evaluate_text_flow(
            &frames,
            &document.styles,
            fonts,
            character,
            paragraph,
        ) {
            Ok(layouts) => {
                self.text_layouts.extend(layouts);
                self.text_layouts.get(&id).cloned().map(Ok)
            }
            Err(error) => Some(Err(error)),
        }
    }
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
        if self.depth >= 256 {
            self.warn(
                id,
                "scene/reference nesting exceeds evaluation limit; skipped",
            );
            return;
        }
        self.depth += 1;
        self.compile_node_inner(document, id, out);
        self.depth -= 1;
    }

    fn compile_node_inner(
        &mut self,
        document: &petunia_core::Document,
        id: ObjectId,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(node) = document.scene.get_node(id) else {
            self.warn(id, "dangling scene reference skipped");
            return;
        };
        if !node.visible
            || document.scene.ancestors(id).iter().any(|ancestor| {
                document
                    .scene
                    .get_node(*ancestor)
                    .is_some_and(|node| !node.visible)
            })
        {
            return;
        }
        if self.binding_source != Some(id) && self.scope.binding_only.contains(&id) {
            return;
        }
        let mut evaluated_node = node.clone();
        evaluated_node.transform = document.scene.world_transform(id).unwrap_or(node.transform);
        let node = &evaluated_node;
        let mut content = Vec::new();
        match &node.item {
            SceneItem::Path(object) => self.compile_path(document, node, object, &mut content),
            SceneItem::Shape(object) => {
                let path = evaluation::shape_path(object.shape, self.tolerance);
                self.compile_path(
                    document,
                    node,
                    &petunia_core::PathObject {
                        path,
                        appearance: object.appearance.clone(),
                    },
                    &mut content,
                );
            }
            SceneItem::Group(children) => {
                if node
                    .geometry_effects
                    .items
                    .iter()
                    .any(|effect| effect.enabled)
                {
                    self.warn(id,"group geometry effects require a subtree evaluator; source subtree retained");
                }
                for child in children {
                    self.compile_node(document, *child, &mut content);
                }
            }
            SceneItem::Text(object) => self.compile_text(document, node, object, &mut content),
            SceneItem::Image(object) => self.compile_image(node, object, &mut content),
            SceneItem::GeneratedVector(object) => {
                match crate::generated::evaluate_generator(&object.generator) {
                    Ok(generated) => self.compile_path(
                        document,
                        node,
                        &petunia_core::PathObject {
                            path: generated.path,
                            appearance: object.appearance.clone(),
                        },
                        &mut content,
                    ),
                    Err(error) => self.warn(id, format!("generator evaluation failed: {error}")),
                }
            }
            SceneItem::PixelLayer(layer) => self.compile_image(
                node,
                &petunia_core::ImageObject {
                    resource: layer.surface.resource,
                    source_rect: None,
                    sampling: petunia_core::ImageSamplingPolicy::Nearest,
                },
                &mut content,
            ),
            SceneItem::Trace(trace) => self.compile_trace(document, node, trace, &mut content),
            SceneItem::SymbolInstance(instance) => {
                self.compile_symbol(document, node, instance, &mut content)
            }
        }
        let effects = self.compile_effects(document, node);
        let clip = node.clip.and_then(|binding| {
            let clip = self
                .clip_geometry(document, binding.source)
                .map(RenderClip::Path);
            if clip.is_none() {
                self.warn(
                    id,
                    "clip source is unavailable or unsupported; target skipped",
                );
            }
            clip
        });
        if node.clip.is_some() && clip.is_none() {
            return;
        }
        let mask = node.mask.map(|binding| {
            let previous = self.binding_source.replace(binding.source);
            let mut children = Vec::new();
            self.compile_node(document, binding.source, &mut children);
            self.binding_source = previous;
            petunia_render_model::RenderMask::Primitives {
                children,
                luminance: binding.mode == petunia_core::MaskMode::Luminance,
            }
        });
        let bounds = effect_bounds(primitive_list_bounds(&content), &effects);
        if node.opacity < 1.0 || clip.is_some() || mask.is_some() || !effects.is_empty() {
            out.push(RenderPrimitive::Group(RenderGroup {
                source: id,
                children: content,
                opacity: node.opacity,
                blend_mode: petunia_core::BlendMode::Normal,
                mask,
                clip,
                effects,
                isolation: petunia_render_model::IsolationMode::Isolated,
                bounds,
            }));
        } else {
            out.extend(content);
        }
    }

    fn compile_path(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        object: &petunia_core::PathObject,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let mut geometry = RenderPath::new();
        geometry.fill_rule = object.path.fill_rule;
        for contour in &object.path.contours {
            let remaining = (1usize << 18)
                .saturating_sub(geometry.contours.iter().map(Vec::len).sum::<usize>());
            let flat = match crate::geometry::bezier::try_flatten_contour(
                contour,
                self.tolerance,
                remaining,
            ) {
                Ok(points) => points,
                Err(error) => {
                    self.warn(node.id, format!("geometry evaluation unavailable: {error}"));
                    return;
                }
            };
            geometry.push_contour(
                flat.iter().map(|point| (point.x, point.y)).collect(),
                contour.closed,
            );
        }
        for effect in node
            .geometry_effects
            .items
            .iter()
            .filter(|effect| effect.enabled)
        {
            self.warn(
                node.id,
                format!(
                    "geometry effect {:?} has no faithful evaluator; source geometry retained",
                    effect.operation
                ),
            );
        }
        for item in object.appearance.items.iter().filter(|item| item.enabled) {
            let (fill, stroke) = match &item.kind {
                petunia_core::AppearanceKind::Fill(paint) => {
                    (self.compile_paint(document, node, node.id, paint), None)
                }
                petunia_core::AppearanceKind::Stroke(style) => {
                    if style.start_marker.is_some() || style.end_marker.is_some() {
                        self.warn(
                            node.id,
                            "stroke markers require a vector definition provider; markers omitted",
                        );
                    }
                    let paint = self.compile_color(document, node.id, &style.paint);
                    let outline = evaluation::stroke_outline(&object.path, style, self.tolerance);
                    let primitive = RenderPrimitive::Vector(Box::new(VectorPrimitive {
                        source: node.id,
                        bounds: render_path_bounds(&outline, node.transform),
                        geometry: outline,
                        appearance: RenderAppearance {
                            fill: paint,
                            stroke: None,
                            opacity: item.opacity,
                        },
                        transform: node.transform,
                    }));
                    self.push_appearance(node, item, primitive, out);
                    continue;
                }
            };
            let primitive = RenderPrimitive::Vector(Box::new(VectorPrimitive {
                source: node.id,
                geometry: geometry.clone(),
                appearance: RenderAppearance {
                    fill,
                    stroke,
                    opacity: item.opacity,
                },
                transform: node.transform,
                bounds: render_path_bounds(&geometry, node.transform),
            }));
            self.push_appearance(node, item, primitive, out);
        }
    }

    fn push_appearance(
        &self,
        node: &SceneNode,
        item: &petunia_core::AppearanceItem,
        primitive: RenderPrimitive,
        out: &mut Vec<RenderPrimitive>,
    ) {
        if item.blend_mode == petunia_core::BlendMode::Normal {
            out.push(primitive);
        } else {
            let bounds = primitive_bounds(&primitive);
            out.push(RenderPrimitive::Group(RenderGroup {
                source: node.id,
                children: vec![primitive],
                opacity: 1.0,
                blend_mode: item.blend_mode,
                mask: None,
                clip: None,
                effects: Vec::new(),
                isolation: petunia_render_model::IsolationMode::Isolated,
                bounds,
            }));
        }
    }

    fn compile_text(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        object: &petunia_core::TextObject,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(fonts) = self.providers.fonts else {
            self.warn(
                node.id,
                "text needs an explicitly loaded font registry; skipped",
            );
            return;
        };
        let default = petunia_core::CharacterStyle {
            font: petunia_core::FontRef {
                family: "sans-serif".into(),
                style_name: None,
                resource: None,
                axes: Vec::new(),
            },
            size: 16.0,
            color: petunia_core::ColorSource::Value(petunia_core::ColorValue::Process(
                petunia_core::ProcessColor {
                    value: petunia_core::ProcessColorValue::Rgb(petunia_core::Rgba {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        alpha: 1.0,
                    }),
                    space: petunia_core::ColorSpaceRef::Builtin(
                        petunia_core::BuiltinColorSpace::Srgb,
                    ),
                },
            )),
            tracking: 0.0,
            baseline_shift: 0.0,
            language: "und".into(),
            slant: petunia_core::FontSlant::Normal,
            features: Vec::new(),
        };
        let paragraph = petunia_core::ParagraphStyle {
            alignment: petunia_core::TextAlignment::Left,
            line_height: 19.2,
            space_before: 0.0,
            space_after: 0.0,
            first_line_indent: 0.0,
            left_indent: 0.0,
            right_indent: 0.0,
            hyphenation: false,
            baseline_grid: None,
        };
        let character = self.providers.default_character.unwrap_or(&default);
        let paragraph = self.providers.default_paragraph.unwrap_or(&paragraph);
        let evaluated = if let petunia_core::TextContainer::OnPath(reference) = object.container {
            let path = document
                .scene
                .get_node(reference.target)
                .and_then(|target| match &target.item {
                    SceneItem::Path(object) => Some(object.path.clone()),
                    SceneItem::Shape(object) => Some(evaluate_shape(object.shape, self.tolerance)),
                    _ => None,
                });
            match (
                path,
                node.transform.inverse(),
                document.scene.world_transform(reference.target),
            ) {
                (Some(mut path), Some(inverse), Some(world)) => {
                    let mapping = inverse.concat(world);
                    for contour in &mut path.contours {
                        for anchor in &mut contour.nodes {
                            anchor.point = mapping.transform_point(anchor.point);
                            anchor.handle_in =
                                anchor.handle_in.map(|point| mapping.transform_point(point));
                            anchor.handle_out = anchor
                                .handle_out
                                .map(|point| mapping.transform_point(point));
                        }
                    }
                    crate::text::evaluate_text_on_path(
                        object,
                        &path,
                        &document.styles,
                        fonts,
                        character,
                        paragraph,
                    )
                }
                _ => Err(crate::text::TextEvaluationError::MissingPathProvider),
            }
        } else {
            self.flow_layout(document, node.id, fonts, character, paragraph)
                .unwrap_or_else(|| {
                    crate::text::evaluate_text(
                        object,
                        &document.styles,
                        fonts,
                        character,
                        paragraph,
                    )
                })
        };
        match evaluated {
            Ok(layout) => {
                for warning in layout.warnings {
                    self.warn(node.id, warning);
                }
                if layout.overflow {
                    self.warn(node.id, "text frame overflow");
                }
                let mut glyphs = Vec::new();
                for outline in layout.outlines {
                    let appearance = petunia_core::Appearance {
                        items: vec![petunia_core::AppearanceItem {
                            id: petunia_core::AppearanceItemId::new_v4(),
                            enabled: true,
                            opacity: 1.0,
                            blend_mode: petunia_core::BlendMode::Normal,
                            kind: petunia_core::AppearanceKind::Fill(petunia_core::Paint::Solid(
                                outline.color,
                            )),
                        }],
                    };
                    self.compile_path(
                        document,
                        node,
                        &petunia_core::PathObject {
                            path: outline.path,
                            appearance,
                        },
                        &mut glyphs,
                    );
                }
                if let petunia_core::TextContainer::Frame(frame) = object.container {
                    if frame.overflow == petunia_core::TextOverflow::Clip {
                        let points = [
                            (0.0, 0.0),
                            (frame.size.width, 0.0),
                            (frame.size.width, frame.size.height),
                            (0.0, frame.size.height),
                        ]
                        .map(|(x, y)| {
                            let point = node
                                .transform
                                .transform_point(petunia_core::Point::new(x, y));
                            (point.x, point.y)
                        })
                        .to_vec();
                        out.push(RenderPrimitive::Group(RenderGroup {
                            source: node.id,
                            bounds: primitive_list_bounds(&glyphs),
                            children: glyphs,
                            opacity: 1.0,
                            blend_mode: petunia_core::BlendMode::Normal,
                            mask: None,
                            clip: Some(RenderClip::Polygon(points)),
                            effects: Vec::new(),
                            isolation: petunia_render_model::IsolationMode::Isolated,
                        }));
                        return;
                    }
                }
                out.extend(glyphs);
            }
            Err(error) => self.warn(node.id, format!("text layout failed: {error}")),
        }
    }

    fn compile_trace(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        trace: &petunia_core::TraceObject,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(image) = self.providers.images.get(&trace.source) else {
            self.warn(node.id, "trace requires decoded image provider; skipped");
            return;
        };
        let encode = |v: f32| {
            if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        };
        let pixels: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|pixel| {
                [encode(pixel.r), encode(pixel.g), encode(pixel.b), pixel.a]
                    .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
            })
            .collect();
        let snapshot = match crate::analysis::RgbaSnapshot::new(
            image.width as usize,
            image.height as usize,
            pixels,
        ) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.warn(node.id, error.to_string());
                return;
            }
        };
        match crate::trace::evaluate_trace(
            &snapshot,
            &trace.spec,
            &crate::analysis::AnalysisLimits::default(),
            &crate::jobs::CancelToken::default(),
        ) {
            Ok(paths) => {
                for path in paths {
                    let appearance = petunia_core::Appearance {
                        items: vec![petunia_core::AppearanceItem {
                            id: petunia_core::AppearanceItemId::new_v4(),
                            enabled: true,
                            opacity: 1.0,
                            blend_mode: petunia_core::BlendMode::Normal,
                            kind: petunia_core::AppearanceKind::Fill(petunia_core::Paint::Solid(
                                petunia_core::ColorSource::Value(path.color),
                            )),
                        }],
                    };
                    self.compile_path(
                        document,
                        node,
                        &petunia_core::PathObject {
                            path: path.path,
                            appearance,
                        },
                        out,
                    );
                }
            }
            Err(error) => self.warn(node.id, format!("trace evaluation failed: {error}")),
        }
    }

    fn compile_symbol(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
        instance: &petunia_core::SymbolInstance,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(definition) = document.symbols.get(instance.definition) else {
            self.warn(node.id, "missing symbol definition; skipped");
            return;
        };
        let mut expanded = document.clone();
        expanded.scene = petunia_core::SceneGraph::new();
        let page = expanded.scene.default_page();
        let mut pending = definition.nodes.clone();
        for change in &instance.overrides {
            let Some(target) = pending.get_mut(&change.target) else {
                self.warn(node.id, "missing symbol override target");
                continue;
            };
            match &change.value {
                petunia_core::SymbolOverrideValue::Visibility(visible) => target.visible = *visible,
                petunia_core::SymbolOverrideValue::TextContent(text) => {
                    if let SceneItem::Text(object) = &mut target.item {
                        object.text = text.clone();
                        object.runs.clear();
                        object.paragraphs.clear();
                    }
                }
                petunia_core::SymbolOverrideValue::Paint(color) => {
                    if let Some(appearance) = match &mut target.item {
                        SceneItem::Path(object) => Some(&mut object.appearance),
                        SceneItem::Shape(object) => Some(&mut object.appearance),
                        SceneItem::GeneratedVector(object) => Some(&mut object.appearance),
                        _ => None,
                    } {
                        for item in &mut appearance.items {
                            match &mut item.kind {
                                petunia_core::AppearanceKind::Fill(paint) => {
                                    *paint = petunia_core::Paint::Solid(color.clone())
                                }
                                petunia_core::AppearanceKind::Stroke(style) => {
                                    style.paint = color.clone()
                                }
                            }
                        }
                    }
                }
                petunia_core::SymbolOverrideValue::Resource(resource) => match &mut target.item {
                    SceneItem::Image(image) => image.resource = *resource,
                    SceneItem::PixelLayer(layer) => layer.surface.resource = *resource,
                    SceneItem::Trace(trace) => trace.source = *resource,
                    _ => self.warn(node.id, "resource override target has no resource"),
                },
            }
        }
        let mut queue = std::collections::VecDeque::new();
        for root in &definition.roots {
            queue.push_back((*root, None));
        }
        while let Some((id, parent)) = queue.pop_front() {
            let Some(mut child) = pending.remove(&id) else {
                self.warn(node.id, "symbol contains dangling/cyclic subtree");
                continue;
            };
            let children = child.item.children().unwrap_or_default().to_vec();
            if let SceneItem::Group(list) = &mut child.item {
                list.clear();
            }
            let inserted = if let Some(parent) = parent {
                child.parent = petunia_core::ParentRef::Object(parent);
                expanded.scene.insert_child(parent, child, None)
            } else {
                child.parent = petunia_core::ParentRef::Page(page);
                child.transform = node.transform.concat(child.transform);
                expanded.scene.insert_root(page, child)
            };
            if let Err(error) = inserted {
                self.warn(node.id, format!("symbol expansion failed: {error}"));
                continue;
            }
            for child in children {
                queue.push_back((child, Some(id)));
            }
        }
        let scope = std::mem::replace(&mut self.scope, CompileScope::new(&expanded));
        let layouts = std::mem::take(&mut self.text_layouts);
        let mut children = Vec::new();
        for root in &definition.roots {
            self.compile_node(&expanded, *root, &mut children);
        }
        self.scope = scope;
        self.text_layouts = layouts;
        for child in &mut children {
            bind_instance_source(child, node.id);
        }
        out.push(RenderPrimitive::Group(RenderGroup {
            source: node.id,
            bounds: primitive_list_bounds(&children),
            children,
            opacity: 1.0,
            blend_mode: petunia_core::BlendMode::Normal,
            mask: None,
            clip: None,
            effects: Vec::new(),
            isolation: petunia_render_model::IsolationMode::Flattened,
        }));
    }

    fn compile_image(
        &mut self,
        node: &SceneNode,
        object: &petunia_core::ImageObject,
        out: &mut Vec<RenderPrimitive>,
    ) {
        let Some(image) = self.providers.images.get(&object.resource) else {
            self.warn(
                node.id,
                "image bytes need an immutable decoded resource provider; skipped",
            );
            return;
        };
        out.push(RenderPrimitive::Image(
            petunia_render_model::ImagePrimitive {
                source: node.id,
                resource: object.resource,
                source_rect: object.source_rect,
                transform: node.transform,
                sampling: object.sampling,
                opacity: 1.0,
                bounds: transformed_rect(
                    Rect::new(0.0, 0.0, image.width as f64, image.height as f64),
                    node.transform,
                ),
            },
        ));
    }

    fn clip_geometry(
        &self,
        document: &petunia_core::Document,
        source: ObjectId,
    ) -> Option<RenderPath> {
        let node = document.scene.get_node(source)?;
        let path = match &node.item {
            SceneItem::Path(object) => object.path.clone(),
            SceneItem::Shape(object) => evaluation::shape_path(object.shape, self.tolerance),
            _ => return None,
        };
        let transform = document.scene.world_transform(source)?;
        let mut geometry = RenderPath::new();
        geometry.fill_rule = path.fill_rule;
        for contour in path.contours.iter().filter(|contour| contour.closed) {
            geometry.push_contour(
                crate::geometry::bezier::try_flatten_contour(contour, self.tolerance, 1 << 18)
                    .ok()?
                    .iter()
                    .map(|point| {
                        let point = transform.transform_point(*point);
                        (point.x, point.y)
                    })
                    .collect(),
                true,
            );
        }
        if geometry.contours.is_empty() {
            None
        } else {
            Some(geometry)
        }
    }

    fn compile_effects(
        &mut self,
        document: &petunia_core::Document,
        node: &SceneNode,
    ) -> Vec<petunia_render_model::RenderEffect> {
        let mut effects = Vec::new();
        for effect in node
            .post_effects
            .items
            .iter()
            .filter(|effect| effect.enabled)
        {
            if effect.opacity != 1.0
                || effect.blend_mode != petunia_core::BlendMode::Normal
                || effect.mask.is_some()
            {
                self.warn(
                    node.id,
                    "effect opacity/blend/mask requires effect-instance compositor; effect omitted",
                );
                continue;
            }
            match &effect.operation {
                petunia_core::PostPaintEffect::GaussianBlur(blur) => {
                    effects.push(petunia_render_model::RenderEffect::Blur {
                        sigma_x: blur.sigma_x,
                        sigma_y: blur.sigma_y,
                    })
                }
                petunia_core::PostPaintEffect::DropShadow(shadow) => {
                    if shadow.spread != 0.0 {
                        self.warn(
                            node.id,
                            "nonzero shadow spread requires coverage morphology; effect omitted",
                        );
                        continue;
                    }
                    if let Some(color) = self.resolve_color(document, node.id, &shadow.color) {
                        effects.push(petunia_render_model::RenderEffect::DropShadow(Box::new(
                            petunia_render_model::ShadowEffect {
                                offset: (shadow.offset.dx, shadow.offset.dy),
                                sigma: (shadow.sigma.dx, shadow.sigma.dy),
                                color,
                                spread: shadow.spread,
                                opacity: 1.0,
                            },
                        )));
                    }
                }
                _ => self.warn(
                    node.id,
                    "post-paint effect has no faithful evaluator; omitted with diagnostic",
                ),
            }
        }
        effects
    }

    /// Resolve authorial paint using explicit color/image providers. Missing
    /// providers and unsupported vector patterns produce diagnostics.
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
                    let Some(image) = self.providers.images.get(res_id) else {
                        self.warn(id, "pattern needs a decoded image resource; skipped");
                        return None;
                    };
                    let transform = if pattern.space == petunia_core::PaintSpace::Object {
                        node.transform.concat(pattern.transform)
                    } else {
                        pattern.transform
                    };
                    if transform.inverse().is_none() {
                        self.warn(id, "pattern transform is singular; skipped");
                        return None;
                    }
                    Some(RenderPaint::Pattern(petunia_render_model::RenderPattern {
                        resource: *res_id,
                        width: image.width,
                        height: image.height,
                        repeat_x: pattern.repeat_x,
                        repeat_y: pattern.repeat_y,
                        transform,
                    }))
                }
                petunia_core::PatternSource::Vector(_) => {
                    self.warn(
                        id,
                        "vector pattern requires reusable tile provider; skipped",
                    );
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
        let evaluated = petunia_render_model::RenderGradient {
            transform: if gradient.space == petunia_core::PaintSpace::Object {
                node.transform
            } else {
                petunia_core::Transform2D::IDENTITY
            },
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
                _ => match self.colors.to_linear_srgb(&process) {
                    Ok(color) => Some(RenderColor {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: color.alpha,
                    }),
                    Err(error) => {
                        self.warn(id, format!("color conversion failed: {error}"));
                        None
                    }
                },
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
}

fn primitive_bounds(primitive: &RenderPrimitive) -> Rect {
    match primitive {
        RenderPrimitive::Vector(v) => v.bounds,
        RenderPrimitive::Group(g) => g.bounds,
        RenderPrimitive::Text(t) => t.bounds,
        RenderPrimitive::Image(i) => i.bounds,
        RenderPrimitive::Raster(r) => r.bounds,
    }
}
fn primitive_list_bounds(primitives: &[RenderPrimitive]) -> Rect {
    primitives
        .iter()
        .map(primitive_bounds)
        .reduce(union_rect)
        .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
}
fn render_path_bounds(path: &RenderPath, transform: petunia_core::Transform2D) -> Rect {
    let mut bounds: Option<Rect> = None;
    for (x, y) in path.contours.iter().flatten() {
        let point = transform.transform_point(petunia_core::Point::new(*x, *y));
        let rect = Rect::new(point.x, point.y, 0.0, 0.0);
        bounds = Some(bounds.map_or(rect, |b| union_rect(b, rect)));
    }
    bounds.unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
}
fn effect_bounds(mut bounds: Rect, effects: &[petunia_render_model::RenderEffect]) -> Rect {
    for effect in effects {
        match effect {
            petunia_render_model::RenderEffect::Blur { sigma_x, sigma_y } => {
                bounds.x -= 3.0 * sigma_x;
                bounds.y -= 3.0 * sigma_y;
                bounds.width += 6.0 * sigma_x;
                bounds.height += 6.0 * sigma_y;
            }
            petunia_render_model::RenderEffect::DropShadow(shadow) => {
                let spread = shadow.spread.max(0.0);
                bounds = union_rect(
                    bounds,
                    Rect::new(
                        bounds.x + shadow.offset.0 - 3.0 * shadow.sigma.0 - spread,
                        bounds.y + shadow.offset.1 - 3.0 * shadow.sigma.1 - spread,
                        bounds.width + 6.0 * shadow.sigma.0 + 2.0 * spread,
                        bounds.height + 6.0 * shadow.sigma.1 + 2.0 * spread,
                    ),
                );
            }
            _ => {}
        }
    }
    bounds
}

fn transformed_rect(rect: Rect, transform: petunia_core::Transform2D) -> Rect {
    let corners = [
        (rect.x, rect.y),
        (rect.x + rect.width, rect.y),
        (rect.x + rect.width, rect.y + rect.height),
        (rect.x, rect.y + rect.height),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in corners {
        let point = transform.transform_point(petunia_core::Point::new(x, y));
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn scene_nodes(document: &petunia_core::Document) -> impl Iterator<Item = &SceneNode> {
    document
        .scene
        .root_lists()
        .into_iter()
        .flat_map(|roots| roots.iter().copied())
        .flat_map(|id| std::iter::once(id).chain(document.scene.descendants(id)))
        .filter_map(|id| document.scene.get_node(id))
}

// Expanded geometry belongs to the selectable authorial instance. Definition
// identities remain in Core for typed overrides, never become scene objects.
fn bind_instance_source(primitive: &mut RenderPrimitive, source: ObjectId) {
    match primitive {
        RenderPrimitive::Vector(value) => value.source = source,
        RenderPrimitive::Image(value) => value.source = source,
        RenderPrimitive::Text(value) => value.source = source,
        RenderPrimitive::Raster(value) => value.source = source,
        RenderPrimitive::Group(group) => {
            group.source = source;
            for child in &mut group.children {
                bind_instance_source(child, source);
            }
        }
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
            rotation: 0.0,
            scale,
            offset_x: 0.0,
            offset_y: 0.0,
        },
        target: RenderTarget { width, height },
    }
}

/// Select one page explicitly for a headless frame; pages never overpaint each other.
#[must_use]
pub fn headless_page_frame(
    mut snapshot: RenderSnapshot,
    page: petunia_core::PageId,
    width: u32,
    height: u32,
    scale: f64,
) -> Option<RenderFrame> {
    let selected = snapshot
        .pages
        .iter()
        .position(|candidate| candidate.page == page)?;
    let page = snapshot.pages.remove(selected);
    snapshot.pages = vec![page];
    Some(headless_frame(snapshot, width, height, scale))
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
                            .map(|point| {
                                let point = document
                                    .scene
                                    .world_transform(binding.source)
                                    .unwrap_or(node.transform)
                                    .transform_point(*point);
                                (point.x, point.y)
                            }),
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
        let RenderPaint::Solid(fill) = vector.appearance.fill.as_ref().expect("fill") else {
            panic!("solid");
        };
        assert_eq!((fill.r, fill.g, fill.b), (1.0, 0.0, 0.0));
        assert_eq!(
            snapshot.pages[0].primitives.len(),
            3,
            "complete ordered stack"
        );
        let RenderPrimitive::Vector(stroke) = &snapshot.pages[0].primitives[2] else {
            panic!("stroke outline");
        };
        assert!(
            stroke.appearance.stroke.is_none(),
            "stroke is evaluated into a local outline"
        );
        let RenderPaint::Solid(paint) = stroke.appearance.fill.as_ref().expect("paint") else {
            panic!("solid");
        };
        assert!(paint.g > 0.9);
        assert!(
            stroke.bounds.width > 10.0,
            "stroke visual bounds include outline"
        );
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
            RenderClip::Path(path) => assert!(!path.contours.is_empty()),
            RenderClip::Rect(_) => panic!("expected polygon/path clip"),
        }
    }
}
