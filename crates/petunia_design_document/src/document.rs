//! Document and surface: one tree, stable IDs, JSON persistence.

use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId, NATIVE_SCHEMA_VERSION};
use petunia_design_geometry::{GAffine, GPath};
use serde::{Deserialize, Serialize};

use crate::document_object::DocumentObject;

fn checked_local_transform(object: &DocumentObject) -> Result<GAffine, crate::GeometryFrameError> {
    if object.shape.is_none() && object.modifiers.is_empty() && object.bounds.is_none() {
        if !object.rotation.is_finite() {
            return Err(crate::GeometryFrameError::NonFinite(object.id));
        }
        return Ok(GAffine::rotate(object.rotation));
    }
    let bounds = object
        .bounds
        .ok_or(crate::GeometryFrameError::MissingBounds(object.id))?;
    if !bounds.iter().all(|value| value.is_finite()) || !object.rotation.is_finite() {
        return Err(crate::GeometryFrameError::NonFinite(object.id));
    }
    if bounds[2] <= 0.0 || bounds[3] <= 0.0 {
        return Err(crate::GeometryFrameError::InvalidBounds(object.id));
    }
    Ok(object.local_transform())
}

fn default_dimensions() -> [f64; 2] {
    [800.0, 600.0]
}

fn default_surface_bg() -> Option<String> {
    Some("ptnd.white".to_string())
}

fn default_true() -> bool {
    true
}

/// Drawing surface (artboard/page/export region depending on context) (10.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    /// Stable identity.
    pub id: SurfaceId,
    /// Human label. Not identity.
    pub name: String,
    /// Objects owned by this surface, in z-order. Crate-visible (A4): read via
    /// `objects()`, mutate only through `DocumentMutator`.
    pub(crate) objects: Vec<DocumentObject>,
    /// Global origin coordinate [x, y] in document pasteboard points.
    #[serde(default)]
    pub origin: [f64; 2],
    /// Width and height [w, h] in document points.
    #[serde(default = "default_dimensions")]
    pub dimensions: [f64; 2],
    /// Background color token reference (e.g. `ptnd.white`) or hex string. None = transparent.
    #[serde(default = "default_surface_bg")]
    pub background: Option<String>,
    /// Per-side bleed configuration.
    #[serde(default)]
    pub bleed: crate::surface_metadata::Bleed,
    /// Per-side inner layout margins.
    #[serde(default)]
    pub margins: crate::surface_metadata::Margins,
    /// Ordered layout guides attached to this surface.
    #[serde(default)]
    pub guides: Vec<crate::surface_metadata::Guide>,
    /// Whether this surface is included in batch export operations.
    #[serde(default = "default_true")]
    pub export_enabled: bool,
}

impl Surface {
    /// Creates an empty surface with an explicit stable ID and default 800x600 dimensions.
    #[must_use]
    pub fn new(id: SurfaceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            objects: Vec::new(),
            origin: [0.0, 0.0],
            dimensions: [800.0, 600.0],
            background: Some("ptnd.white".to_string()),
            bleed: crate::surface_metadata::Bleed::ZERO,
            margins: crate::surface_metadata::Margins::ZERO,
            guides: Vec::new(),
            export_enabled: true,
        }
    }

    /// Axis-aligned bounds `[x, y, w, h]` of the surface in global document points.
    #[must_use]
    pub fn bounds(&self) -> [f64; 4] {
        [
            self.origin[0],
            self.origin[1],
            self.dimensions[0],
            self.dimensions[1],
        ]
    }

    /// Extended bounds `[x, y, w, h]` including bleed area.
    #[must_use]
    pub fn bleed_bounds(&self) -> [f64; 4] {
        [
            self.origin[0] - self.bleed.left,
            self.origin[1] - self.bleed.top,
            self.dimensions[0] + self.bleed.left + self.bleed.right,
            self.dimensions[1] + self.bleed.top + self.bleed.bottom,
        ]
    }

    /// Objects owned by this surface in z-order (A4 read lane).
    #[must_use]
    pub fn objects(&self) -> &[DocumentObject] {
        &self.objects
    }

    /// Builds a surface pre-populated with objects in z-order (A4).
    /// Test fixtures and engine restore paths use this instead of
    /// pushing into storage directly.
    #[must_use]
    pub fn with_objects(
        id: SurfaceId,
        name: impl Into<String>,
        objects: Vec<DocumentObject>,
    ) -> Self {
        Self {
            objects,
            ..Self::new(id, name)
        }
    }
}

/// Canonical document: owns surfaces; objects live inside surfaces.
///
/// Published objects and surfaces are read-only outside the document crate.
/// All edits must produce a ChangeSet through DocumentMutator.
///
/// ```compile_fail
/// use petunia_design_document::Document;
/// use petunia_design_foundation::ObjectId;
/// let mut doc = Document::new();
/// doc.find_object_mut(ObjectId::new(1));
/// ```
///
/// ```compile_fail
/// use petunia_design_document::Document;
/// use petunia_design_foundation::SurfaceId;
/// let mut doc = Document::new();
/// doc.surface_mut(SurfaceId::new(1));
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Schema version of this payload.
    pub(crate) schema_version: u32,
    /// Surfaces in document order. Crate-visible (A4): read via `surfaces()`,
    /// mutate only through `DocumentMutator`.
    pub(crate) surfaces: Vec<Surface>,
    /// Variable data source definitions (10.11). Crate-visible (A4): read via
    /// `data_sources()`, mutate only through `DocumentMutator`.
    #[serde(default)]
    pub(crate) data_sources: Vec<crate::variable_data::DataSourceDefinition>,
    /// Variable data property bindings (10.11). Crate-visible (A4): read via
    /// `bindings()`, mutate only through `DocumentMutator`.
    #[serde(default)]
    pub(crate) bindings: Vec<crate::variable_data::DataBinding>,
}

impl Document {
    /// Converts a tool's world-space parameter input to the selected object's
    /// current local frame, including rotation and ancestor placement.
    pub fn modifier_from_world(
        &self,
        id: ObjectId,
        mut item: crate::ModifierItem,
    ) -> Result<crate::ModifierItem, PetuniaError> {
        if item.space != crate::ModifierSpace::Parent {
            return Err(PetuniaError::invalid_input(
                "world input must not carry an already-local frame",
            ));
        }
        let object = self
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found("modifier target missing"))?;
        let [_, _, w, h] = object
            .bounds
            .ok_or_else(|| PetuniaError::invalid_input("modifier requires bounds"))?;
        let inverse = self
            .world_transform_checked(id)
            .map_err(|e| PetuniaError::invalid_input(e.to_string()))?
            .inverse()
            .ok_or_else(|| PetuniaError::invalid_input("modifier transform is singular"))?;
        let project = |p: &mut [f64; 2]| {
            let local = inverse.apply(petunia_design_geometry::GPoint::new(p[0], p[1]));
            *p = [local.x, local.y];
        };
        match &mut item.kind {
            crate::ModifierKind::TransparentGradient { start, end, .. } => {
                project(start);
                project(end);
            }
            crate::ModifierKind::Perspective { quad } => {
                for p in quad {
                    project(p);
                }
            }
            crate::ModifierKind::CropRect {
                rect: [x, y, cw, ch],
            } => {
                if ![*x, *y, *cw, *ch].iter().all(|v| v.is_finite()) || *cw <= 0.0 || *ch <= 0.0 {
                    return Err(PetuniaError::invalid_input(
                        "world crop requires finite positive dimensions",
                    ));
                }
                // A world rectangle must remain an axis-aligned rectangle in
                // local space. An enclosing AABB would silently crop extra art.
                let corners = [
                    [*x, *y],
                    [*x + *cw, *y],
                    [*x + *cw, *y + *ch],
                    [*x, *y + *ch],
                ];
                let local = corners.map(|mut p| {
                    project(&mut p);
                    p
                });
                let axis_aligned = local.windows(2).all(|pair| {
                    (pair[0][0] - pair[1][0]).abs() < 1e-8 || (pair[0][1] - pair[1][1]).abs() < 1e-8
                });
                if !axis_aligned {
                    return Err(PetuniaError::invalid_input(
                        "rotated world crop requires a polygon mask",
                    ));
                }
                *x = local.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
                *y = local.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
                *cw = local.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max) - *x;
                *ch = local.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max) - *y;
            }
            crate::ModifierKind::ContourOffset { .. } => {}
        }
        item.space = crate::ModifierSpace::Local {
            reference_size: [w, h],
        };
        Ok(item)
    }

    /// Object opacity at a world-space point. Ancestor/group compositing is
    /// separate; this evaluates the object's editable transparency chain.
    pub fn opacity_at_world(
        &self,
        id: ObjectId,
        point: petunia_design_geometry::GPoint,
    ) -> Result<f64, crate::GeometryFrameError> {
        let object = self
            .find_object(id)
            .ok_or(crate::GeometryFrameError::MissingObject(id))?;
        let inverse = self
            .world_transform_checked(id)?
            .inverse()
            .ok_or(crate::GeometryFrameError::NonFinite(id))?;
        object.opacity_at_local(inverse.apply(point))
    }

    /// Version of the normalized native payload; only migration changes it.
    #[must_use]
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Creates an empty document at the current native schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: NATIVE_SCHEMA_VERSION,
            surfaces: Vec::new(),
            data_sources: Vec::new(),
            bindings: Vec::new(),
        }
    }

    /// Finds a data source by stable ID.
    #[must_use]
    pub fn data_source(
        &self,
        id: crate::variable_data::DataSourceId,
    ) -> Option<&crate::variable_data::DataSourceDefinition> {
        self.data_sources.iter().find(|ds| ds.id == id)
    }

    /// Finds a data binding by stable ID.
    #[must_use]
    pub fn binding(
        &self,
        id: crate::variable_data::BindingId,
    ) -> Option<&crate::variable_data::DataBinding> {
        self.bindings.iter().find(|b| b.id == id)
    }

    /// All surfaces in document order (A4 read lane).
    #[must_use]
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    /// All variable data source definitions (A4 read lane).
    #[must_use]
    pub fn data_sources(&self) -> &[crate::variable_data::DataSourceDefinition] {
        &self.data_sources
    }

    /// All variable data property bindings (A4 read lane).
    #[must_use]
    pub fn bindings(&self) -> &[crate::variable_data::DataBinding] {
        &self.bindings
    }

    /// Finds a surface by stable ID.
    pub fn surface(&self, id: SurfaceId) -> Result<&Surface, PetuniaError> {
        self.surfaces
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("surface `{id}` does not exist")))
    }

    /// Finds a mutable surface by stable ID.
    pub(crate) fn surface_mut(&mut self, id: SurfaceId) -> Result<&mut Surface, PetuniaError> {
        self.surfaces
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("surface `{id}` does not exist")))
    }

    /// Finds an object anywhere in the document by stable ID.
    #[must_use]
    pub fn find_object(&self, id: ObjectId) -> Option<&DocumentObject> {
        self.surfaces
            .iter()
            .flat_map(|s| s.objects.iter())
            .find(|o| o.id == id)
    }

    /// Finds a mutable object anywhere in the document by stable ID.
    pub(crate) fn find_object_mut(&mut self, id: ObjectId) -> Option<&mut DocumentObject> {
        self.surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
    }

    /// Finds the SurfaceId containing an object.
    #[must_use]
    pub fn find_object_surface(&self, id: ObjectId) -> Option<SurfaceId> {
        for s in &self.surfaces {
            if s.objects.iter().any(|o| o.id == id) {
                return Some(s.id);
            }
        }
        None
    }

    /// Checks if `candidate` is a descendant of `ancestor` (or is the ancestor itself).
    #[must_use]
    pub fn is_descendant(&self, candidate: ObjectId, ancestor: ObjectId) -> bool {
        if candidate == ancestor {
            return true;
        }
        let mut curr = Some(candidate);
        let mut visited = std::collections::HashSet::new();
        while let Some(c) = curr {
            if !visited.insert(c) {
                break;
            }
            if let Some(obj) = self.find_object(c) {
                if let Some(p) = obj.parent {
                    if p == ancestor {
                        return true;
                    }
                    curr = Some(p);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        false
    }

    /// Computes the accumulated world affine transform with explicit frame validation.
    pub fn world_transform_checked(
        &self,
        id: ObjectId,
    ) -> Result<GAffine, crate::GeometryFrameError> {
        let mut chain: Vec<(ObjectId, GAffine)> = Vec::new();
        let mut current = Some(id);
        while let Some(object_id) = current {
            if chain.iter().any(|(visited, _)| *visited == object_id) {
                return Err(crate::GeometryFrameError::HierarchyCycle(object_id));
            }
            let object = self
                .find_object(object_id)
                .ok_or(crate::GeometryFrameError::MissingObject(object_id))?;
            let transform = checked_local_transform(object)?;
            chain.push((object_id, transform));
            current = object.parent;
        }

        let mut accumulated = GAffine::IDENTITY;
        for (_, local) in chain.into_iter().rev() {
            accumulated = accumulated.after(local);
            if !accumulated.coeffs.iter().all(|value| value.is_finite()) {
                return Err(crate::GeometryFrameError::NonFinite(id));
            }
        }
        Ok(accumulated)
    }

    /// Computes the accumulated world affine transform from root to this object.
    ///
    /// This compatibility wrapper preserves the existing `PetuniaError` API while
    /// delegating validation to [`Self::world_transform_checked`].
    pub fn world_transform(&self, id: ObjectId) -> Result<GAffine, PetuniaError> {
        self.world_transform_checked(id)
            .map_err(|error| PetuniaError::invalid_input(error.to_string()))
    }

    /// Returns the object's base path projected into world/pasteboard space.
    pub fn base_path_world(&self, id: ObjectId) -> Result<GPath, crate::GeometryFrameError> {
        let object = self
            .find_object(id)
            .ok_or(crate::GeometryFrameError::MissingObject(id))?;
        let transform = self.world_transform_checked(id)?;
        let path = object.base_path_local()?.transformed(transform);
        if path.is_finite() {
            Ok(path)
        } else {
            Err(crate::GeometryFrameError::NonFinite(id))
        }
    }

    /// Returns the object's evaluated path projected into world/pasteboard space.
    pub fn evaluated_path_world(&self, id: ObjectId) -> Result<GPath, crate::GeometryFrameError> {
        let object = self
            .find_object(id)
            .ok_or(crate::GeometryFrameError::MissingObject(id))?;
        let transform = self.world_transform_checked(id)?;
        let path = object.evaluated_path_local()?.transformed(transform);
        if path.is_finite() {
            Ok(path)
        } else {
            Err(crate::GeometryFrameError::NonFinite(id))
        }
    }

    /// Returns the nominal placement frame projected into world space.
    pub fn frame_bounds_world(&self, id: ObjectId) -> Result<[f64; 4], crate::GeometryFrameError> {
        let object = self
            .find_object(id)
            .ok_or(crate::GeometryFrameError::MissingObject(id))?;
        let bounds = match object.bounds {
            Some(bounds) => bounds,
            None if object.shape.is_none() && object.modifiers.is_empty() => {
                return Ok([0.0, 0.0, 0.0, 0.0]);
            }
            None => return Err(crate::GeometryFrameError::MissingBounds(id)),
        };
        if !bounds.iter().all(|value| value.is_finite()) {
            return Err(crate::GeometryFrameError::NonFinite(id));
        }
        if bounds[2] <= 0.0 || bounds[3] <= 0.0 {
            return Err(crate::GeometryFrameError::InvalidBounds(id));
        }
        let transform = self.world_transform_checked(id)?;
        let points = [
            petunia_design_geometry::GPoint::new(0.0, 0.0),
            petunia_design_geometry::GPoint::new(bounds[2], 0.0),
            petunia_design_geometry::GPoint::new(bounds[2], bounds[3]),
            petunia_design_geometry::GPoint::new(0.0, bounds[3]),
        ]
        .map(|point| transform.apply(point));
        let min_x = points
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = points
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = points
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = points
            .iter()
            .map(|point| point.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return Err(crate::GeometryFrameError::NonFinite(id));
        }
        Ok([min_x, min_y, max_x - min_x, max_y - min_y])
    }

    /// Returns the evaluated world-space bounds, or `None` for an empty path.
    pub fn evaluated_bounds_world(
        &self,
        id: ObjectId,
    ) -> Result<Option<[f64; 4]>, crate::GeometryFrameError> {
        Ok(self.evaluated_path_world(id)?.bounding_box().map(|rect| {
            [
                rect.x0,
                rect.y0,
                rect.width().max(1.0),
                rect.height().max(1.0),
            ]
        }))
    }

    /// Alias for the world-space bounds used by spatial queries and culling.
    pub fn world_aabb(&self, id: ObjectId) -> Result<Option<[f64; 4]>, crate::GeometryFrameError> {
        self.evaluated_bounds_world(id)
    }

    /// Serializes the document to canonical JSON.
    pub fn to_json(&self) -> Result<String, PetuniaError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PetuniaError::io(format!("serialize document: {e}")))
    }

    /// Parses a document from canonical JSON, rejecting unknown schemas.
    /// Legacy `aubrieta.*` tokens are normalized to `ptnd.*` on read (15.A):
    /// old projects open without data loss and are re-emitted in the current
    /// namespace.
    pub fn from_json(text: &str) -> Result<Self, PetuniaError> {
        let mut doc: Self = serde_json::from_str(text)
            .map_err(|e| PetuniaError::io(format!("parse document: {e}")))?;
        if !(1..=NATIVE_SCHEMA_VERSION).contains(&doc.schema_version) {
            return Err(PetuniaError::invalid_input(format!(
                "unsupported schema {}, expected {}",
                doc.schema_version, NATIVE_SCHEMA_VERSION
            )));
        }
        for surface in &mut doc.surfaces {
            for object in &mut surface.objects {
                if doc.schema_version >= 3
                    && object
                        .modifiers
                        .iter()
                        .any(|m| m.space == crate::ModifierSpace::Parent)
                {
                    return Err(PetuniaError::invalid_input(
                        "schema 3 requires explicit local modifier frames",
                    ));
                }
                if let Some(shape) = object.shape.take() {
                    object.shape = Some(shape.into_local(object.bounds)?);
                }
                object.modifiers = std::mem::take(&mut object.modifiers)
                    .into_iter()
                    .map(|modifier| modifier.into_local(object.bounds))
                    .collect::<Result<_, _>>()?;
            }
        }
        doc.schema_version = NATIVE_SCHEMA_VERSION;
        doc.normalize_legacy_namespaces();
        doc.validate()?;
        Ok(doc)
    }

    /// Rewrites every persisted legacy `aubrieta.*` token to `ptnd.*` (15.A).
    /// Idempotent; safe to call on an already-current document.
    fn normalize_legacy_namespaces(&mut self) {
        use petunia_design_foundation::normalized;

        let fix = |slot: &mut Option<String>| {
            if let Some(value) = slot.as_mut() {
                *value = normalized(value);
            }
        };

        for surface in &mut self.surfaces {
            fix(&mut surface.background);
            for object in &mut surface.objects {
                fix(&mut object.fill);
                fix(&mut object.stroke);
                if let Some(appearance) = object.appearance.as_mut() {
                    for fill in &mut appearance.fills {
                        normalize_paint(&mut fill.paint, &normalized);
                    }
                    for stroke in &mut appearance.strokes {
                        normalize_paint(&mut stroke.paint, &normalized);
                    }
                }
            }
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Rewrites one paint's persisted color tokens to the current namespace.
fn normalize_paint(paint: &mut crate::appearance::Paint, normalized: &impl Fn(&str) -> String) {
    use crate::appearance::Paint;
    match paint {
        Paint::None => {}
        Paint::Solid(token) => *token = normalized(token),
        Paint::LinearGradient(gradient) => {
            for stop in &mut gradient.stops {
                stop.color = normalized(&stop.color);
            }
        }
        Paint::RadialGradient(gradient) => {
            for stop in &mut gradient.stops {
                stop.color = normalized(&stop.color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_foundation::IdGenerator;

    #[test]
    fn save_reopen_roundtrip_preserves_ids() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface_id = gen.next_surface();
        let mut surface = Surface::new(surface_id, "Page 1");
        surface
            .objects
            .push(DocumentObject::new(gen.next_object(), "Rect"));
        doc.surfaces.push(surface);

        let json = doc.to_json().expect("serialize");
        let reopened = Document::from_json(&json).expect("reopen");
        assert_eq!(doc, reopened);
    }

    #[test]
    fn unknown_schema_is_rejected() {
        let bad = r#"{"schema_version":999,"surfaces":[]}"#;
        assert!(Document::from_json(bad).is_err());
    }

    fn shape_object(
        id: ObjectId,
        bounds: [f64; 4],
        rotation: f64,
        shape: crate::ShapeKind,
    ) -> DocumentObject {
        let mut object = DocumentObject::new(id, "shape");
        object.bounds = Some(bounds);
        object.rotation = rotation;
        object.shape = Some(shape);
        object
    }

    #[test]
    fn local_path_excludes_placement() {
        let object = shape_object(
            ObjectId::new(1),
            [100.0, 200.0, 80.0, 40.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        let local = object.base_path_local().expect("local path");
        let bounds = local.bounding_box().expect("local bounds");
        assert_eq!(
            [bounds.x0, bounds.y0, bounds.width(), bounds.height()],
            [0.0, 0.0, 80.0, 40.0]
        );
    }

    #[test]
    fn root_world_path_applies_placement_once() {
        let mut document = Document::new();
        let surface = Surface::with_objects(
            petunia_design_foundation::SurfaceId::new(1),
            "Page",
            vec![shape_object(
                ObjectId::new(2),
                [100.0, 200.0, 80.0, 40.0],
                0.0,
                crate::ShapeKind::Rectangle {
                    corner_radii: [0.0; 4],
                },
            )],
        );
        document.surfaces.push(surface);
        let world = document
            .base_path_world(ObjectId::new(2))
            .expect("world path");
        let bounds = world.bounding_box().expect("world bounds");
        assert_eq!(
            [bounds.x0, bounds.y0, bounds.width(), bounds.height()],
            [100.0, 200.0, 80.0, 40.0]
        );
    }

    #[test]
    fn child_world_transform_composes_parent_placement() {
        let mut document = Document::new();
        let mut parent = shape_object(
            ObjectId::new(2),
            [100.0, 200.0, 200.0, 200.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        let mut child = shape_object(
            ObjectId::new(3),
            [25.0, 30.0, 50.0, 40.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        child.parent = Some(parent.id);
        parent.children.push(child.id);
        document.surfaces.push(Surface::with_objects(
            petunia_design_foundation::SurfaceId::new(1),
            "Page",
            vec![parent, child],
        ));
        let world = document
            .base_path_world(ObjectId::new(3))
            .expect("world path");
        let bounds = world.bounding_box().expect("world bounds");
        assert_eq!(
            [bounds.x0, bounds.y0, bounds.width(), bounds.height()],
            [125.0, 230.0, 50.0, 40.0]
        );
    }

    #[test]
    fn hierarchy_cycle_is_diagnosed() {
        let mut document = Document::new();
        let mut parent = shape_object(
            ObjectId::new(2),
            [0.0, 0.0, 100.0, 100.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        let mut child = shape_object(
            ObjectId::new(3),
            [10.0, 10.0, 20.0, 20.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        child.parent = Some(parent.id);
        parent.parent = Some(child.id);
        parent.children.push(child.id);
        child.children.push(parent.id);
        document.surfaces.push(Surface::with_objects(
            petunia_design_foundation::SurfaceId::new(1),
            "Page",
            vec![parent, child],
        ));
        assert!(matches!(
            document.world_transform_checked(ObjectId::new(3)),
            Err(crate::GeometryFrameError::HierarchyCycle(_))
        ));
    }

    #[test]
    fn ambiguous_legacy_path_requires_migration() {
        let object = shape_object(
            ObjectId::new(2),
            [0.0, 0.0, 100.0, 100.0],
            0.0,
            crate::ShapeKind::Path(petunia_design_geometry::GPath::rect(
                petunia_design_geometry::GRect::new(0.0, 0.0, 10.0, 10.0),
                0.0,
                0.0,
            )),
        );
        assert!(matches!(
            object.base_path_local(),
            Err(crate::GeometryFrameError::AmbiguousPath(_))
        ));
    }

    #[test]
    fn parent_space_modifier_is_reframed_for_local_evaluation() {
        let mut object = shape_object(
            ObjectId::new(2),
            [0.0, 0.0, 100.0, 100.0],
            0.0,
            crate::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
        );
        object.modifiers.push(crate::ModifierItem::enabled(
            1,
            crate::ModifierKind::Perspective {
                quad: [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]],
            },
        ));
        object.modifiers = object
            .modifiers
            .into_iter()
            .map(|m| m.into_local(object.bounds).unwrap())
            .collect();
        assert_eq!(
            object.evaluated_path_local().unwrap(),
            object.base_path_local().unwrap()
        );
        if let crate::ModifierKind::Perspective { quad } = &mut object.modifiers[0].kind {
            quad[0][0] = f64::NAN;
        }
        assert!(matches!(
            object.evaluated_path_local(),
            Err(crate::GeometryFrameError::NonFinite(_))
        ));
    }
}
