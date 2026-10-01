//! Immutable render snapshots. Geometry is evaluated once in an explicit frame.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use petunia_design_document::{Document, DocumentObject, EffectKind, Surface};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{GAffine, GPath, GRect};
use petunia_design_text::{prepare_text, PreparedText, TextFrameSpec, TextRenderError};

#[derive(Clone, Debug)]
pub(crate) enum RenderGeometry {
    Path(Arc<GPath>),
    Text(Arc<PreparedText>),
}
impl std::ops::Deref for RenderGeometry {
    type Target = GPath;
    fn deref(&self) -> &GPath {
        match self {
            Self::Path(path) => path,
            Self::Text(text) => text.outline(),
        }
    }
}

/// A render operation fails explicitly instead of substituting bounding boxes.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Cooperative cancellation before publication of a partial render.
    #[error("render cancelled")]
    Cancelled,
    /// Invalid scene, frame, viewport or appearance data.
    #[error("invalid render input: {0}")]
    Invalid(String),
    /// The requested work exceeds the caller's resource budget.
    #[error("render resource limit: {0}")]
    Limit(&'static str),
    /// Content requires a capability which this backend cannot faithfully render.
    #[error("object {object}: unavailable render capability: {feature}")]
    Unsupported {
        /// Stable identity of the affected object.
        object: ObjectId,
        /// Missing capability; suitable for an export error/disabled reason.
        feature: &'static str,
    },
}

/// One immutable node; fields are private so consumers cannot mutate a snapshot.
#[derive(Clone, Debug)]
pub struct RenderNode {
    pub(crate) source: DocumentObject,
    pub(crate) geometry: RenderGeometry,
    pub(crate) world: GAffine,
    pub(crate) visual_bounds: Option<GRect>,
}

impl RenderNode {
    /// Read-only canonical descriptor retained by this immutable scene.
    pub fn source(&self) -> &DocumentObject {
        &self.source
    }
    pub fn prepared_text(&self) -> Option<&PreparedText> {
        match &self.geometry {
            RenderGeometry::Text(text) => Some(text),
            _ => None,
        }
    }
    /// Shared text footprint used by selection, independent of the stored box
    /// for artistic overflow. Frames keep their editable clipping rectangle.
    pub fn text_bounds(&self) -> Option<GRect> {
        let text = self.prepared_text()?;
        if self.source.text_style.flow == petunia_design_document::TextFlow::Frame {
            let [_, _, w, h] = self.source.bounds?;
            Some(GRect::new(0., 0., w, h))
        } else {
            match (text.logical_bounds(), text.ink_bounds()) {
                (Some(a), Some(b)) => a.union(b),
                (Some(a), None) | (None, Some(a)) => Some(a),
                _ => None,
            }
        }
    }
    pub fn text_hit(&self, world_point: petunia_design_geometry::GPoint) -> Option<bool> {
        let _ = self.prepared_text()?;
        let point = self.world.inverse()?.apply(world_point);
        Some(self.text_bounds().is_some_and(|b| {
            point.x >= b.x0 && point.x <= b.x1 && point.y >= b.y0 && point.y <= b.y1
        }))
    }
    /// Canonical object identity.
    pub fn id(&self) -> ObjectId {
        self.source.id
    }
    /// Evaluated geometry in object-local coordinates; no placement baked in.
    pub fn geometry(&self) -> &GPath {
        &self.geometry
    }
    /// Resolves local points to pasteboard coordinates, including every ancestor.
    pub fn local_to_world(&self) -> GAffine {
        self.world
    }
    /// Bounds including descendants, strokes and ordered effect reach.
    pub fn visual_bounds(&self) -> Option<GRect> {
        self.visual_bounds
    }
    /// Children in canonical back-to-front order.
    pub fn children(&self) -> &[ObjectId] {
        &self.source.children
    }
}

/// Renderable surface independent of a live document, toolkit or display server.
#[derive(Clone, Debug)]
pub struct RenderSurface {
    pub(crate) id: SurfaceId,
    pub(crate) bounds: [f64; 4],
    pub(crate) roots: Vec<ObjectId>,
    pub(crate) nodes: HashMap<ObjectId, RenderNode>,
}

impl RenderSurface {
    /// Stable surface identity.
    pub fn id(&self) -> SurfaceId {
        self.id
    }
    /// Pasteboard origin and dimensions in document points.
    pub fn bounds(&self) -> [f64; 4] {
        self.bounds
    }
    /// Access a read-only node by identity.
    pub fn node(&self, id: ObjectId) -> Option<&RenderNode> {
        self.nodes.get(&id)
    }
    pub fn text_nodes(&self) -> impl Iterator<Item = &RenderNode> {
        self.nodes
            .values()
            .filter(|node| node.prepared_text().is_some())
    }
    /// Deterministic bounded diagnostics; source family and bytes stay editable.
    pub fn text_warnings(&self) -> Vec<String> {
        let mut nodes: Vec<_> = self.text_nodes().collect();
        nodes.sort_by_key(|node| node.id());
        nodes
            .into_iter()
            .filter_map(|node| {
                let text = node.prepared_text()?;
                if text.missing_family() {
                    if let Some(petunia_design_document::ShapeKind::Text { font_family, .. }) =
                        &node.source.shape
                    {
                        return Some(format!(
                            "{}: missing font family ‘{}’; fallback displayed",
                            node.source.name, font_family
                        ));
                    }
                }
                if node.source.text_style.flow == petunia_design_document::TextFlow::Frame
                    && node
                        .source
                        .bounds
                        .is_some_and(|b| text.flow_height() > b[3])
                {
                    return Some(format!("{}: text exceeds its frame", node.source.name));
                }
                None
            })
            .take(64)
            .collect()
    }
    /// Root identities in back-to-front order.
    pub fn roots(&self) -> &[ObjectId] {
        &self.roots
    }

    /// Replaces pixels only in a derived preview scene; canonical descriptors
    /// and ownership remain in the captured document snapshot.
    pub fn with_raster_preview(
        &self,
        id: ObjectId,
        layer: Arc<petunia_design_raster::RasterLayer>,
    ) -> Result<Self, RenderError> {
        layer
            .validate()
            .map_err(|e| RenderError::Invalid(e.to_string()))?;
        let mut next = self.clone();
        let node = next
            .nodes
            .get_mut(&id)
            .ok_or_else(|| RenderError::Invalid("missing raster preview target".into()))?;
        if !matches!(
            node.source.shape,
            Some(petunia_design_document::ShapeKind::Raster { .. })
        ) {
            return Err(RenderError::Invalid(
                "preview target is not a raster layer".into(),
            ));
        }
        node.source.shape = Some(petunia_design_document::ShapeKind::Raster { layer });
        Ok(next)
    }

    /// Conservative damage in world coordinates, retaining the previous scene
    /// to clear deleted/moved artwork and propagating mask/group dependencies.
    pub fn damage_to(&self, next: &Self) -> Result<Option<GRect>, RenderError> {
        if self.id != next.id {
            return Err(RenderError::Invalid(
                "damage requires the same surface identity".into(),
            ));
        }
        let mut changed = HashSet::new();
        let mut dependants: HashMap<ObjectId, Vec<ObjectId>> = HashMap::new();
        for snapshot in [self, next] {
            for (id, node) in &snapshot.nodes {
                if self
                    .nodes
                    .get(id)
                    .zip(next.nodes.get(id))
                    .is_none_or(|(a, b)| a.source != b.source || a.world != b.world)
                {
                    changed.insert(*id);
                }
                if let Some(parent) = node.source.parent {
                    dependants.entry(*id).or_default().push(parent);
                }
                if let Some(mask) = node.source.clip_mask_id {
                    dependants.entry(mask).or_default().push(*id);
                }
            }
        }
        if self.roots != next.roots {
            changed.extend(self.roots.iter().chain(&next.roots).copied());
        }
        let mut queue: Vec<_> = changed.iter().copied().collect();
        while let Some(id) = queue.pop() {
            if let Some(ids) = dependants.get(&id) {
                for dependant in ids {
                    if changed.insert(*dependant) {
                        queue.push(*dependant);
                    }
                }
            }
        }
        let mut result = None;
        for id in changed {
            result = union(result, self.nodes.get(&id).and_then(|n| n.visual_bounds));
            result = union(result, next.nodes.get(&id).and_then(|n| n.visual_bounds));
        }
        if self.bounds != next.bounds {
            for [x, y, w, h] in [self.bounds, next.bounds] {
                result = union(result, Some(GRect::new(x, y, x + w, y + h)));
            }
        }
        Ok(result)
    }

    /// Builds an independent snapshot, rejecting incomplete/cyclic hierarchies.
    pub fn extract(surface: &Surface) -> Result<Self, RenderError> {
        Self::extract_cancellable(surface, &|| false)
    }

    /// Checks cancellation between nodes and glyphs while deriving a scene.
    pub fn extract_cancellable(
        surface: &Surface,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Self, RenderError> {
        if cancelled() {
            return Err(RenderError::Cancelled);
        }
        const MAX_NODES: usize = 100_000;
        if surface.objects().len() > MAX_NODES {
            return Err(RenderError::Limit("scene object count"));
        }
        let objects: HashMap<_, _> = surface.objects().iter().map(|o| (o.id, o)).collect();
        if objects.len() != surface.objects().len() {
            return Err(RenderError::Invalid("duplicate object identity".into()));
        }
        let roots: Vec<_> = surface
            .objects()
            .iter()
            .filter(|o| o.parent.is_none())
            .map(|o| o.id)
            .collect();
        let mut result = Self {
            id: surface.id,
            bounds: surface.bounds(),
            roots,
            nodes: HashMap::new(),
        };
        let mut visited = HashSet::new();
        for id in result.roots.clone() {
            extract_node(
                id,
                None,
                GAffine::IDENTITY,
                &objects,
                &mut result.nodes,
                &mut visited,
                0,
                cancelled,
                true,
            )?;
        }
        if visited.len() != objects.len() {
            return Err(RenderError::Invalid(
                "unreachable object or hierarchy cycle".into(),
            ));
        }
        Ok(result)
    }
}

/// Rebuildable immutable scene for display/export adapters.
#[derive(Clone, Debug, Default)]
pub struct RenderScene {
    surfaces: Vec<RenderSurface>,
}

impl RenderScene {
    /// Extracts every surface in document order; never changes source geometry.
    pub fn extract(document: &Document) -> Result<Self, RenderError> {
        Ok(Self {
            surfaces: document
                .surfaces()
                .iter()
                .map(RenderSurface::extract)
                .collect::<Result<_, _>>()?,
        })
    }
    /// Surfaces in document order.
    pub fn surfaces(&self) -> &[RenderSurface] {
        &self.surfaces
    }
}

fn extract_node(
    id: ObjectId,
    parent: Option<ObjectId>,
    parent_world: GAffine,
    objects: &HashMap<ObjectId, &DocumentObject>,
    nodes: &mut HashMap<ObjectId, RenderNode>,
    visited: &mut HashSet<ObjectId>,
    depth: usize,
    cancelled: &dyn Fn() -> bool,
    ancestors_visible: bool,
) -> Result<Option<GRect>, RenderError> {
    if cancelled() {
        return Err(RenderError::Cancelled);
    }
    if depth >= 128 {
        return Err(RenderError::Limit("scene hierarchy depth"));
    }
    if !visited.insert(id) {
        return Err(RenderError::Invalid("cycle or repeated child".into()));
    }
    let object = objects
        .get(&id)
        .ok_or_else(|| RenderError::Invalid(format!("missing object {id}")))?;
    if object.parent != parent {
        return Err(RenderError::Invalid(format!("inconsistent parent of {id}")));
    }
    match &object.shape {
        Some(petunia_design_document::ShapeKind::Polygon { sides }) if *sides > 100_000 => {
            return Err(RenderError::Limit("polygon sides"));
        }
        Some(petunia_design_document::ShapeKind::Star { points, .. }) if *points > 50_000 => {
            return Err(RenderError::Limit("star points"));
        }
        Some(petunia_design_document::ShapeKind::LocalPath { path, .. })
            if path.verbs.len() > 1_000_000 =>
        {
            return Err(RenderError::Limit("source path verb count"));
        }
        Some(petunia_design_document::ShapeKind::Path(path)) if path.verbs.len() > 1_000_000 => {
            return Err(RenderError::Limit("source path verb count"));
        }
        _ => {}
    }
    let world = parent_world.after(object.local_transform());
    if !world.coeffs.iter().all(|v| v.is_finite()) || world.inverse().is_none() {
        return Err(RenderError::Invalid(format!("invalid transform of {id}")));
    }
    let mut geometry = if object.shape.is_some() {
        object
            .evaluated_path_local()
            .map_err(|e| RenderError::Invalid(e.to_string()))?
    } else {
        GPath::new()
    };
    if matches!(
        object.shape,
        Some(petunia_design_document::ShapeKind::Image { .. })
    ) {
        // This is image coverage, not a vector conversion or a baked crop.
        // The original sampling frame remains the full placed-image rectangle.
        let [_, _, w, h] = object
            .bounds
            .ok_or_else(|| RenderError::Invalid("image has no local frame".into()))?;
        let rectangle = GPath::rect(GRect::new(0.0, 0.0, w, h), 0.0, 0.0);
        geometry = petunia_design_document::modifiers::evaluate_modifiers_local(
            &rectangle,
            &object.modifiers,
            [w, h],
        )
        .ok_or_else(|| RenderError::Invalid("invalid image coverage".into()))?;
    }
    let visible = ancestors_visible && object.visible;
    let geometry = if !visible {
        RenderGeometry::Path(Arc::new(geometry))
    } else if let Some(petunia_design_document::ShapeKind::Text {
        content,
        font_family,
        font_size,
        line_height,
        letter_spacing,
        on_path,
    }) = &object.shape
    {
        if on_path.is_some() {
            return Err(RenderError::Unsupported {
                object: id,
                feature: "shaped text along a path",
            });
        }
        let [_, _, w, h] = object
            .bounds
            .ok_or_else(|| RenderError::Invalid("text has no local frame".into()))?;
        let text = prepare_text(
            &TextFrameSpec {
                content: content.clone(),
                family: font_family.clone(),
                font_size: *font_size,
                line_height: *line_height,
                letter_spacing: *letter_spacing,
                width: w,
                weight: object.text_style.weight,
                italic: object.text_style.italic,
                alignment: match object.text_style.alignment {
                    petunia_design_document::TextAlignment::Start => {
                        petunia_design_text::FlowAlignment::Start
                    }
                    petunia_design_document::TextAlignment::Center => {
                        petunia_design_text::FlowAlignment::Center
                    }
                    petunia_design_document::TextAlignment::End => {
                        petunia_design_text::FlowAlignment::End
                    }
                },
                wrap: object.text_style.flow == petunia_design_document::TextFlow::Frame,
            },
            cancelled,
        )
        .map_err(|error| match error {
            TextRenderError::Cancelled => RenderError::Cancelled,
            TextRenderError::Limit(reason) => RenderError::Limit(reason),
            TextRenderError::Unsupported(feature) => RenderError::Unsupported {
                object: id,
                feature,
            },
            error => RenderError::Invalid(error.to_string()),
        })?;
        if object.modifiers.iter().any(|modifier| modifier.enabled) {
            let path = petunia_design_document::modifiers::evaluate_modifiers_local(
                text.outline(),
                &object.modifiers,
                [w, h],
            )
            .ok_or_else(|| RenderError::Invalid("invalid text coverage".into()))?;
            RenderGeometry::Path(Arc::new(path))
        } else {
            RenderGeometry::Text(text)
        }
    } else {
        RenderGeometry::Path(Arc::new(geometry))
    };
    let appearance = object.effective_appearance();
    let stroke_reach = appearance
        .strokes
        .iter()
        .filter(|s| s.visible)
        .map(|s| {
            let alignment = if s.alignment == petunia_design_document::StrokeAlignment::Center {
                0.5
            } else {
                1.0
            };
            s.width * alignment * s.miter_limit.max(1.0)
        })
        .fold(0.0_f64, f64::max);
    let mut bounds = geometry
        .transformed(world)
        .bounding_box()
        .map(|b| expand(b, stroke_reach));
    for child in &object.children {
        let child_bounds = extract_node(
            *child,
            Some(id),
            world,
            objects,
            nodes,
            visited,
            depth + 1,
            cancelled,
            visible,
        )?;
        if !objects[child].is_clip_mask {
            bounds = union(bounds, child_bounds);
        }
    }
    // Each visible effect consumes the previous result, so reaches accumulate.
    for effect in appearance.effects.iter().filter(|e| e.visible) {
        if let Some(b) = bounds {
            bounds = Some(match effect.kind {
                EffectKind::GaussianBlur { radius } => expand(b, radius * 3.0),
                EffectKind::DropShadow { offset, blur, .. } => {
                    let shadow = expand(
                        GRect::new(
                            b.x0 + offset[0],
                            b.y0 + offset[1],
                            b.x1 + offset[0],
                            b.y1 + offset[1],
                        ),
                        blur * 3.0,
                    );
                    b.union(shadow).unwrap_or(b)
                }
                _ => b,
            });
        }
    }
    if bounds.is_some_and(|b| !b.is_finite()) {
        return Err(RenderError::Invalid(format!(
            "non-finite visual bounds of {id}"
        )));
    }
    nodes.insert(
        id,
        RenderNode {
            source: (*object).clone(),
            geometry,
            world,
            visual_bounds: bounds,
        },
    );
    Ok(bounds)
}

pub(crate) fn expand(b: GRect, reach: f64) -> GRect {
    GRect::new(b.x0 - reach, b.y0 - reach, b.x1 + reach, b.y1 + reach)
}
fn union(a: Option<GRect>, b: Option<GRect>) -> Option<GRect> {
    match (a, b) {
        (Some(a), Some(b)) => a.union(b),
        (Some(a), None) | (None, Some(a)) => Some(a),
        _ => None,
    }
}
