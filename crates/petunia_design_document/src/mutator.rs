//! The only writer path: UI/Shortcut/Plugin/MCP -> Action -> Command ->
//! `DocumentMutator` -> `ChangeSet`. Nothing touches storage directly.

use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};

use crate::changeset::{Change, ChangeSet};
use crate::document::Document;
use crate::document_object::DocumentObject;

fn next_local_id(ids: impl Iterator<Item = u32>) -> Result<u32, PetuniaError> {
    ids.max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| PetuniaError::invalid_input("local entry IDs exhausted"))
}

fn frame_transform_error(id: ObjectId) -> PetuniaError {
    PetuniaError::invalid_input(format!(
        "object `{id}` cannot be reframed without finite bounds"
    ))
}

fn reframe_object(
    object: &mut DocumentObject,
    transform: petunia_design_geometry::GAffine,
) -> Result<(Option<[f64; 4]>, f64), PetuniaError> {
    let [a, b, c, d, _, _] = transform.coeffs;
    let scale_x = a.hypot(b);
    let scale_y = c.hypot(d);
    let shear = (a * c + b * d).abs();
    if !transform.coeffs.iter().all(|value| value.is_finite())
        || (scale_x - 1.0).abs() > 1e-8
        || (scale_y - 1.0).abs() > 1e-8
        || shear > 1e-8
    {
        return Err(PetuniaError::invalid_input(format!(
            "object `{}` transform is not representable by TRS",
            object.id
        )));
    }
    let previous_bounds = object.bounds;
    let previous_rotation = object.rotation;
    if previous_bounds.is_none() && object.shape.is_none() && object.modifiers.is_empty() {
        object.rotation = b.atan2(a);
        if !object.rotation.is_finite() {
            return Err(PetuniaError::invalid_input(format!(
                "object `{}` produced a non-finite rotation",
                object.id
            )));
        }
        return Ok((previous_bounds, previous_rotation));
    }
    let current_bounds = previous_bounds.ok_or_else(|| frame_transform_error(object.id))?;
    if !current_bounds.iter().all(|value| value.is_finite())
        || current_bounds[2] <= 0.0
        || current_bounds[3] <= 0.0
    {
        return Err(frame_transform_error(object.id));
    }
    let width = current_bounds[2];
    let height = current_bounds[3];
    let origin = transform.apply(petunia_design_geometry::GPoint::ORIGIN);
    if !origin.is_finite() {
        return Err(PetuniaError::invalid_input(format!(
            "object `{}` produced a non-finite placement origin",
            object.id
        )));
    }
    object.bounds = Some([origin.x, origin.y, width, height]);
    object.rotation = b.atan2(a);
    if !object.rotation.is_finite() {
        return Err(PetuniaError::invalid_input(format!(
            "object `{}` produced a non-finite rotation",
            object.id
        )));
    }
    Ok((previous_bounds, previous_rotation))
}

/// Exclusive writer over a [`Document`].
#[derive(Debug)]
pub struct DocumentMutator<'doc> {
    document: &'doc mut Document,
}

impl<'doc> DocumentMutator<'doc> {
    /// Borrows a document for mutation.
    #[must_use]
    pub fn new(document: &'doc mut Document) -> Self {
        Self { document }
    }

    /// Borrows the underlying document immutably.
    #[must_use]
    pub fn document(&self) -> &Document {
        self.document
    }

    /// Adds a surface with an explicit stable ID.
    pub fn add_surface(
        &mut self,
        id: SurfaceId,
        name: impl Into<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.document.surfaces.iter().any(|s| s.id == id) {
            return Err(PetuniaError::invalid_input(format!(
                "surface `{id}` already exists"
            )));
        }
        let name = name.into();
        self.document
            .surfaces
            .push(crate::document::Surface::new(id, name.clone()));
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceAdded { id, name });
        Ok(changes)
    }

    /// Adds an object to a surface.
    pub fn add_object(
        &mut self,
        surface: SurfaceId,
        mut object: DocumentObject,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.document.find_object(object.id).is_some() {
            return Err(PetuniaError::invalid_input(format!(
                "object `{}` already exists",
                object.id
            )));
        }
        if let Some(shape) = object.shape.take() {
            object.shape = Some(shape.into_local(object.bounds)?);
        }
        object.modifiers = std::mem::take(&mut object.modifiers)
            .into_iter()
            .map(|modifier| modifier.into_local(object.bounds))
            .collect::<Result<_, _>>()?;
        crate::validation::validate_object(&object)?;
        let target = self.document.surface_mut(surface)?;
        target.objects.push(object.clone());
        let index = target.objects.len() - 1;
        let mut changes = ChangeSet::empty();
        changes.push(Change::ObjectAdded {
            surface,
            object,
            index,
        });
        Ok(changes)
    }

    /// Adds a whole detached graph in one staged append and validation walk.
    pub fn add_objects_bulk(
        &mut self,
        surface: SurfaceId,
        objects: Vec<DocumentObject>,
    ) -> Result<ChangeSet, PetuniaError> {
        self.document.surface(surface)?;
        let mut ids: std::collections::HashSet<_> = self
            .document
            .surfaces
            .iter()
            .flat_map(|s| s.objects.iter())
            .map(|o| o.id)
            .collect();
        let mut normalized = Vec::with_capacity(objects.len());
        for mut object in objects {
            if !ids.insert(object.id) {
                return Err(PetuniaError::invalid_input(
                    "duplicate bulk object identity",
                ));
            }
            if let Some(shape) = object.shape.take() {
                object.shape = Some(shape.into_local(object.bounds)?);
            }
            object.modifiers = object
                .modifiers
                .into_iter()
                .map(|m| m.into_local(object.bounds))
                .collect::<Result<_, _>>()?;
            crate::validation::validate_object(&object)?;
            normalized.push(object);
        }
        let mut staged = self.document.clone();
        let target = staged.surface_mut(surface)?;
        let first = target.objects.len();
        target.objects.extend(normalized.iter().cloned());
        staged.validate()?;
        *self.document = staged;
        let mut changes = ChangeSet::empty();
        for (i, object) in normalized.into_iter().enumerate() {
            changes.push(Change::ObjectAdded {
                surface,
                object,
                index: first + i,
            });
        }
        Ok(changes)
    }

    /// Removes an object by stable ID, keeping it for undo (F-09/F-10).
    /// Cleans `parent.children` references and `clip_mask_id` pointers so no
    /// dangling IDs remain. Records the original z-index for order-preserving
    /// undo plus `ChildrenChanged`/`ClipMaskChanged` for full revert.
    pub fn remove_object(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(pos) = surface.objects.iter().position(|o| o.id == id) {
                let object = surface.objects.remove(pos);
                let mut changes = ChangeSet::empty();
                // Detach from parent container.
                if let Some(parent_id) = object.parent {
                    if let Some(parent) = surface.objects.iter_mut().find(|o| o.id == parent_id) {
                        if let Some(p) = parent.children.iter().position(|c| *c == id) {
                            let prev = parent.children.clone();
                            parent.children.remove(p);
                            changes.push(Change::ChildrenChanged {
                                id: parent_id,
                                previous_children: prev,
                                next_children: parent.children.clone(),
                            });
                        }
                    }
                }
                // Clear clip pointers referencing the removed object.
                for other in surface.objects.iter_mut() {
                    if let Some(crate::ShapeKind::Text {
                        on_path: Some(attachment),
                        ..
                    }) = &other.shape
                    {
                        if attachment.target == id {
                            let previous = other.shape.clone();
                            if let Some(crate::ShapeKind::Text { on_path, .. }) = &mut other.shape {
                                *on_path = None;
                            }
                            changes.push(Change::ShapeChanged {
                                id: other.id,
                                previous,
                                next: other.shape.clone(),
                            });
                        }
                    }
                    if other.clip_mask_id == Some(id) {
                        let prev_mask = other.clip_mask_id;
                        let prev_is = other.is_clip_mask;
                        other.clip_mask_id = None;
                        changes.push(Change::ClipMaskChanged {
                            id: other.id,
                            previous_mask: prev_mask,
                            next_mask: None,
                            previous_is_mask: prev_is,
                            next_is_mask: prev_is,
                        });
                    }
                }
                // Children of a removed container become root-level (parent=None).
                for child_id in object.children.clone() {
                    if let Some(child) = surface.objects.iter_mut().find(|o| o.id == child_id) {
                        let prev_parent = child.parent;
                        child.parent = None;
                        changes.push(Change::Reparented {
                            id: child_id,
                            previous_parent: prev_parent,
                            next_parent: None,
                            previous_index: 0,
                            next_index: 0,
                        });
                    }
                }
                self.document.bindings.retain(|binding| {
                    if binding.target_object == id {
                        changes.push(Change::DataBindingRemoved {
                            binding: binding.clone(),
                        });
                        false
                    } else {
                        true
                    }
                });
                changes.push(Change::ObjectRemoved {
                    surface: surface.id,
                    object,
                    index: pos,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Renames an object by stable ID.
    pub fn rename_object(
        &mut self,
        id: ObjectId,
        new_name: impl Into<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        let name = new_name.into();
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.name.clone();
                if previous == name {
                    return Ok(ChangeSet::empty());
                }
                object.name = name.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::NameChanged {
                    id,
                    previous,
                    next: name,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's semantic fill token.
    pub fn set_fill(
        &mut self,
        id: ObjectId,
        fill: Option<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.fill.clone();
                object.fill = fill.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::FillChanged {
                    id,
                    previous,
                    next: fill,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's visibility flag.
    pub fn set_visibility(
        &mut self,
        id: ObjectId,
        visible: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.visible;
                object.visible = visible;
                let mut changes = ChangeSet::empty();
                changes.push(Change::VisibilityChanged {
                    id,
                    previous,
                    next: visible,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's locked flag.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.locked;
                object.locked = locked;
                let mut changes = ChangeSet::empty();
                changes.push(Change::LockChanged {
                    id,
                    previous,
                    next: locked,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's opacity factor in [0.0, 1.0].
    pub fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, PetuniaError> {
        if !opacity.is_finite() {
            return Err(PetuniaError::invalid_input("opacity must be finite"));
        }
        let clamped = opacity.clamp(0.0, 1.0);
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.opacity;
                object.opacity = clamped;
                let mut changes = ChangeSet::empty();
                changes.push(Change::OpacityChanged {
                    id,
                    previous,
                    next: clamped,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's stroke token and width.
    pub fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        if !width.is_finite() {
            return Err(PetuniaError::invalid_input("stroke width must be finite"));
        }
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous_stroke = object.stroke.clone();
                let previous_width = object.stroke_width;
                object.stroke = stroke.clone();
                object.stroke_width = width.max(0.0);
                let mut changes = ChangeSet::empty();
                changes.push(Change::StrokeChanged {
                    id,
                    previous_stroke,
                    next_stroke: stroke,
                    previous_width,
                    next_width: object.stroke_width,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's bounds and rotation.
    pub fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        if !rotation.is_finite()
            || bounds
                .is_some_and(|b| !b.iter().all(|v| v.is_finite()) || b[2] <= 0.0 || b[3] <= 0.0)
        {
            return Err(PetuniaError::invalid_input(
                "bounds must be finite with positive dimensions and finite rotation",
            ));
        }
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous_bounds = object.bounds;
                let previous_rotation = object.rotation;
                object.bounds = bounds;
                object.rotation = rotation;
                let mut changes = ChangeSet::empty();
                changes.push(Change::BoundsChanged {
                    id,
                    previous_bounds,
                    next_bounds: bounds,
                    previous_rotation,
                    next_rotation: rotation,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's vector shape or text descriptor.
    pub fn set_shape(
        &mut self,
        id: ObjectId,
        shape: Option<crate::ShapeKind>,
    ) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                // Build and validate everything before publishing either frame
                // or shape. This makes the primitive safe for in-place staging.
                let mut candidate = object.clone();
                if candidate.bounds.is_none() {
                    if let Some(crate::ShapeKind::Path(path)) = &shape {
                        if !path.is_finite() {
                            return Err(PetuniaError::invalid_input("path must be finite"));
                        }
                        let rect = path.bounding_box().ok_or_else(|| {
                            PetuniaError::invalid_input("empty path requires an explicit frame")
                        })?;
                        candidate.bounds = Some([
                            rect.x0,
                            rect.y0,
                            rect.width().max(1.0),
                            rect.height().max(1.0),
                        ]);
                    }
                }
                candidate.shape = shape.map(|s| s.into_local(candidate.bounds)).transpose()?;
                crate::validation::validate_object(&candidate)?;
                let mut changes = ChangeSet::empty();
                if candidate.bounds != object.bounds {
                    changes.push(Change::BoundsChanged {
                        id,
                        previous_bounds: object.bounds,
                        next_bounds: candidate.bounds,
                        previous_rotation: object.rotation,
                        next_rotation: candidate.rotation,
                    });
                }
                changes.push(Change::ShapeChanged {
                    id,
                    previous: object.shape.clone(),
                    next: candidate.shape.clone(),
                });
                *object = candidate;
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Replaces geometry and frame together, validating before changing either.
    pub fn set_path(
        &mut self,
        id: ObjectId,
        path: petunia_design_geometry::GPath,
        bounds: [f64; 4],
        rotation: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        if !rotation.is_finite() {
            return Err(PetuniaError::invalid_input("path rotation must be finite"));
        }
        let shape = crate::ShapeKind::Path(path).into_local(Some(bounds))?;
        if self.document.find_object(id).is_none() {
            return Err(PetuniaError::not_found(format!(
                "object `{id}` does not exist"
            )));
        }
        let mut changes = self.set_bounds(id, Some(bounds), rotation)?;
        changes.extend(self.set_shape(id, Some(shape))?);
        Ok(changes)
    }

    /// Divides two objects into non-overlapping pieces (10.3, F-21).
    /// Computes subject-only (`Difference`), clip-only (`Difference`
    /// reversed) and intersection pieces with the explicit default
    /// tolerance, then replaces the inputs atomically. Empty pieces are
    /// skipped. Result objects inherit each input's appearance provenance.
    pub fn divide_objects(
        &mut self,
        surface: SurfaceId,
        subject_id: ObjectId,
        clip_id: ObjectId,
        subject_only_id: ObjectId,
        clip_only_id: ObjectId,
        intersection_id: ObjectId,
    ) -> Result<ChangeSet, PetuniaError> {
        if subject_id == clip_id {
            return Err(PetuniaError::invalid_input(
                "cannot divide an object by itself",
            ));
        }
        for id in [subject_only_id, clip_only_id, intersection_id] {
            if self.document.find_object(id).is_some() {
                return Err(PetuniaError::invalid_input(format!(
                    "result object `{id}` already exists"
                )));
            }
        }
        let subject = self
            .document
            .find_object(subject_id)
            .ok_or_else(|| PetuniaError::not_found(format!("subject `{subject_id}` not found")))?
            .clone();
        let clip = self
            .document
            .find_object(clip_id)
            .ok_or_else(|| PetuniaError::not_found(format!("clip `{clip_id}` not found")))?
            .clone();

        if [&subject, &clip].iter().any(|object| {
            matches!(
                object.shape,
                Some(crate::ShapeKind::Raster { .. })
                    | Some(crate::ShapeKind::Image { .. })
                    | Some(crate::ShapeKind::Text { .. })
            )
        }) {
            return Err(PetuniaError::capability_unavailable(
                "vector division requires vector operands",
            ));
        }
        let tolerance = petunia_design_geometry::GeometryTolerance::default_tolerance().clamped();
        let subj_input = petunia_design_geometry::BooleanInput::new(
            subject.evaluated_path().to_polygons(tolerance.flatten),
        );
        let clip_input = petunia_design_geometry::BooleanInput::new(
            clip.evaluated_path().to_polygons(tolerance.flatten),
        );

        let pieces = [
            (
                subject_only_id,
                "Divided Subject",
                petunia_design_geometry::boolean_op(
                    &subj_input,
                    &clip_input,
                    petunia_design_geometry::BooleanOp::Difference,
                ),
                &subject,
            ),
            (
                clip_only_id,
                "Divided Clip",
                petunia_design_geometry::boolean_op(
                    &clip_input,
                    &subj_input,
                    petunia_design_geometry::BooleanOp::Difference,
                ),
                &clip,
            ),
            (
                intersection_id,
                "Divided Intersection",
                petunia_design_geometry::boolean_op(
                    &subj_input,
                    &clip_input,
                    petunia_design_geometry::BooleanOp::Intersection,
                ),
                &subject,
            ),
        ];

        // Build all results before mutating: failure leaves inputs intact.
        let mut results = Vec::new();
        for (id, name, contours, donor) in pieces {
            if contours.iter().any(|c| c.len() >= 3) {
                let path = petunia_design_geometry::GPath::from_polygons(&contours);
                let bounds = path
                    .bounding_box()
                    .map(|r| [r.x0, r.y0, r.width(), r.height()]);
                let mut obj = DocumentObject::new(id, name);
                obj.shape = Some(crate::ShapeKind::Path(path));
                obj.bounds = bounds;
                obj.appearance = donor.appearance.clone();
                obj.fill = donor.fill.clone();
                obj.stroke = donor.stroke.clone();
                obj.stroke_width = donor.stroke_width;
                obj.opacity = donor.opacity;
                results.push(obj);
            }
        }
        if results.is_empty() {
            return Err(PetuniaError::invalid_input(
                "divide produced no non-empty pieces",
            ));
        }

        let mut changes = ChangeSet::empty();
        let c1 = self.remove_object(subject_id)?;
        changes.extend(c1);
        let c2 = self.remove_object(clip_id)?;
        changes.extend(c2);
        for obj in results {
            let c = self.add_object(surface, obj)?;
            changes.extend(c);
        }
        Ok(changes)
    }

    /// Atomically replaces descriptors in a single graph walk. Native resource
    /// resolution uses this lane instead of one O(objects) lookup per tile layer.
    pub fn set_shapes_bulk(
        &mut self,
        shapes: Vec<(ObjectId, crate::ShapeKind)>,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut requested = std::collections::HashMap::new();
        for (id, shape) in shapes {
            if requested.insert(id, shape).is_some() {
                return Err(PetuniaError::invalid_input("duplicate bulk shape target"));
            }
        }
        let mut next_document = self.document.clone();
        let mut changes = ChangeSet::empty();
        for object in next_document
            .surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
        {
            if let Some(shape) = requested.remove(&object.id) {
                let next = Some(shape.into_local(object.bounds)?);
                let previous = std::mem::replace(&mut object.shape, next.clone());
                changes.push(Change::ShapeChanged {
                    id: object.id,
                    previous,
                    next,
                });
            }
        }
        if !requested.is_empty() {
            return Err(PetuniaError::not_found("unknown bulk shape target"));
        }
        next_document.validate()?;
        *self.document = next_document;
        Ok(changes)
    }

    /// Validates before publication; only a text object accepts typography edits.
    pub fn set_text_style(
        &mut self,
        id: ObjectId,
        style: crate::TextStyle,
    ) -> Result<ChangeSet, PetuniaError> {
        style.validate()?;
        let object = self
            .document
            .find_object_mut(id)
            .ok_or_else(|| PetuniaError::not_found("unknown text object"))?;
        if !matches!(object.shape, Some(crate::ShapeKind::Text { .. })) {
            return Err(PetuniaError::invalid_input(
                "typography requires a text object",
            ));
        }
        if object.text_style == style {
            return Ok(ChangeSet::empty());
        }
        let previous = std::mem::replace(&mut object.text_style, style);
        let mut changes = ChangeSet::empty();
        changes.push(Change::TextStyleChanged {
            id,
            previous,
            next: style,
        });
        Ok(changes)
    }

    /// Explicit pixel/vector mask mode; the descriptor and its undo record agree.
    pub fn set_mask_mode(
        &mut self,
        id: ObjectId,
        mode: crate::MaskMode,
    ) -> Result<ChangeSet, PetuniaError> {
        let object = self
            .document
            .find_object_mut(id)
            .ok_or_else(|| PetuniaError::not_found("unknown mask object"))?;
        if object.mask_mode == mode {
            return Ok(ChangeSet::empty());
        }
        let previous = object.mask_mode;
        object.mask_mode = mode;
        let mut changes = ChangeSet::empty();
        changes.push(Change::MaskModeChanged {
            id,
            previous,
            next: mode,
        });
        Ok(changes)
    }

    /// Converts a parametric shape or text object to an editable vector path (10.3, 10.6).
    /// Text has no vector outline without font shaping: rejects explicitly
    /// (F-20) instead of silently substituting a rectangle.
    pub fn convert_to_curves(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                if matches!(
                    object.shape,
                    Some(crate::ShapeKind::Text { .. })
                        | Some(crate::ShapeKind::Image { .. })
                        | Some(crate::ShapeKind::Raster { .. })
                ) {
                    return Err(PetuniaError::invalid_input(format!(
                        "object `{id}` cannot be converted to curves without vectorization"
                    )));
                }
                let path = object.to_path();
                if path.is_empty() {
                    return Err(PetuniaError::invalid_input(format!(
                        "object `{id}` has no convertible outline"
                    )));
                }
                let previous = object.shape.clone();
                let next = Some(crate::ShapeKind::Path(path).into_local(object.bounds)?);
                object.shape = next.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::ShapeChanged { id, previous, next });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Bakes corner geometry into an explicit vector path (10.2, 10.3) (F-08).
    /// Rectangles with non-zero radii, polygons and stars become explicit
    /// paths. Objects already holding a `Path` are a NoOp (empty changeset)
    /// so callers do not pollute undo.
    pub fn bake_corners(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let is_path = self
            .document
            .find_object(id)
            .map(|o| o.shape.as_ref().is_some_and(crate::ShapeKind::is_path))
            .unwrap_or(false);
        if is_path {
            return Ok(ChangeSet::empty());
        }
        self.convert_to_curves(id)
    }

    /// Sets an object's live contour offset, non-destructively (09.31).
    /// Upserts the single `ContourOffset` entry (chain order preserved),
    /// keeping join/cap when only the distance changes. Zero distance
    /// removes the entry. Base geometry is never touched.
    pub fn set_contour_offset(
        &mut self,
        id: ObjectId,
        distance: f64,
        join: petunia_design_geometry::OffsetJoin,
        cap: petunia_design_geometry::OffsetCap,
    ) -> Result<ChangeSet, PetuniaError> {
        if !distance.is_finite() {
            return Err(PetuniaError::invalid_input(
                "offset distance must be finite",
            ));
        }
        let next = if distance.abs() < 1e-9 {
            self.modifiers_without_contour(id)?
        } else {
            self.modifiers_with_contour(id, distance, join, cap)?
        };
        self.set_modifiers(id, next)
    }

    /// Replaces an object's full modifier chain (one undo entry).
    pub fn set_modifiers(
        &mut self,
        id: ObjectId,
        modifiers: Vec<crate::modifiers::ModifierItem>,
    ) -> Result<ChangeSet, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let modifiers: Vec<_> = modifiers
                    .into_iter()
                    .map(|m| m.into_local(object.bounds))
                    .collect::<Result<_, _>>()?;
                crate::validation::validate_modifiers(&modifiers, object.bounds)?;
                let previous = object.modifiers.clone();
                if previous == modifiers {
                    return Ok(ChangeSet::empty());
                }
                object.modifiers = modifiers.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::ModifiersChanged {
                    id,
                    previous,
                    next: modifiers,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Bakes live contour offsets into base geometry (explicit user op, 09.31).
    /// The evaluated outline becomes the base `Path`, contour entries clear,
    /// bounds follow. Other modifier kinds survive.
    pub fn bake_contour(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let object = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let Some(last) = object.modifiers.iter().rposition(|m| {
            m.enabled && matches!(m.kind, crate::ModifierKind::ContourOffset { .. })
        }) else {
            return Ok(ChangeSet::empty());
        };
        if object.modifiers[..=last].iter().any(|m| {
            m.enabled
                && Self::is_geometry_modifier(&m.kind)
                && !matches!(m.kind, crate::ModifierKind::ContourOffset { .. })
        }) {
            return Err(PetuniaError::invalid_input(
                "contour after another geometry modifier requires BakeGeometry",
            ));
        }
        self.bake_geometry_prefix(id, last + 1)
    }

    /// Chain with the contour entry upserted (joins/caps preserved on update).
    fn modifiers_with_contour(
        &self,
        id: ObjectId,
        distance: f64,
        join: petunia_design_geometry::OffsetJoin,
        cap: petunia_design_geometry::OffsetCap,
    ) -> Result<Vec<crate::modifiers::ModifierItem>, PetuniaError> {
        let obj = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let mut next = obj.modifiers.clone();
        if let Some(entry) = next
            .iter_mut()
            .find(|m| matches!(m.kind, crate::modifiers::ModifierKind::ContourOffset { .. }))
        {
            if let crate::modifiers::ModifierKind::ContourOffset { distance: d, .. } =
                &mut entry.kind
            {
                *d = distance;
            }
            entry.space = crate::ModifierSpace::Parent;
            let _ = (join, cap);
        } else {
            let nid = next_local_id(next.iter().map(|m| m.id))?;
            next.push(crate::modifiers::ModifierItem::enabled(
                nid,
                crate::modifiers::ModifierKind::ContourOffset {
                    distance,
                    join,
                    cap,
                },
            ));
        }
        Ok(next)
    }

    /// Bakes live transparency gradients into base opacity (explicit user op).
    /// Documented approximation: the center sample flattens the mask, like
    /// export. Transparent entries clear; other modifiers survive.
    pub fn bake_transparency(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let (sampled, has_transparency) = {
            let obj = self
                .document
                .find_object(id)
                .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
            let has = obj.modifiers.iter().any(|m| {
                m.enabled
                    && matches!(
                        m.kind,
                        crate::modifiers::ModifierKind::TransparentGradient { .. }
                    )
            });
            (obj.sampled_opacity(), has)
        };
        if !has_transparency {
            return Ok(ChangeSet::empty());
        }
        let mut changes = ChangeSet::empty();
        changes.extend(self.set_opacity(id, sampled)?);
        let remaining: Vec<crate::modifiers::ModifierItem> = self
            .document
            .find_object(id)
            .map(|o| {
                o.modifiers
                    .iter()
                    .filter(|m| {
                        !(m.enabled
                            && matches!(
                                m.kind,
                                crate::modifiers::ModifierKind::TransparentGradient { .. }
                            ))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        changes.extend(self.set_modifiers(id, remaining)?);
        Ok(changes)
    }

    /// Chain with the contour entry removed.
    fn modifiers_without_contour(
        &self,
        id: ObjectId,
    ) -> Result<Vec<crate::modifiers::ModifierItem>, PetuniaError> {
        let obj = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        Ok(obj
            .modifiers
            .iter()
            .filter(|m| !matches!(m.kind, crate::modifiers::ModifierKind::ContourOffset { .. }))
            .cloned()
            .collect())
    }

    /// Sets a live perspective quad, non-destructively (09.31, 10.8).
    /// Upserts the single `Perspective` entry (chain order preserved).
    /// Degenerate quads are stored but decline at evaluation (warp math).
    pub fn set_perspective(
        &mut self,
        id: ObjectId,
        quad: [[f64; 2]; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        if !quad.iter().flatten().all(|v| v.is_finite()) {
            return Err(PetuniaError::invalid_input(
                "perspective quad must be finite",
            ));
        }
        let obj = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let mut next = obj.modifiers.clone();
        if let Some(entry) = next
            .iter_mut()
            .find(|m| matches!(m.kind, crate::modifiers::ModifierKind::Perspective { .. }))
        {
            if let crate::modifiers::ModifierKind::Perspective { quad: q } = &mut entry.kind {
                *q = quad;
            }
            entry.space = crate::ModifierSpace::Parent;
        } else {
            let nid = next_local_id(next.iter().map(|m| m.id))?;
            next.push(crate::modifiers::ModifierItem::enabled(
                nid,
                crate::modifiers::ModifierKind::Perspective { quad },
            ));
        }
        self.set_modifiers(id, next)
    }

    /// Sets a live rectangular crop, non-destructively (09.31, 08.24).
    /// Upserts the single `CropRect` entry (chain order preserved).
    pub fn set_crop_rect(
        &mut self,
        id: ObjectId,
        rect: [f64; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        if !rect.iter().all(|v| v.is_finite()) || rect[2] < 1.0 || rect[3] < 1.0 {
            return Err(PetuniaError::invalid_input(
                "crop rect must be finite with positive size",
            ));
        }
        let obj = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let mut next = obj.modifiers.clone();
        if let Some(entry) = next
            .iter_mut()
            .find(|m| matches!(m.kind, crate::modifiers::ModifierKind::CropRect { .. }))
        {
            if let crate::modifiers::ModifierKind::CropRect { rect: r } = &mut entry.kind {
                *r = rect;
            }
            entry.space = crate::ModifierSpace::Parent;
        } else {
            let nid = next_local_id(next.iter().map(|m| m.id))?;
            next.push(crate::modifiers::ModifierItem::enabled(
                nid,
                crate::modifiers::ModifierKind::CropRect { rect },
            ));
        }
        self.set_modifiers(id, next)
    }

    /// True for geometry-domain modifiers (baked by [`Self::bake_geometry`]).
    fn is_geometry_modifier(kind: &crate::modifiers::ModifierKind) -> bool {
        matches!(
            kind,
            crate::modifiers::ModifierKind::ContourOffset { .. }
                | crate::modifiers::ModifierKind::Perspective { .. }
                | crate::modifiers::ModifierKind::CropRect { .. }
        )
    }

    /// Bakes all live geometry-domain modifiers into base geometry
    /// (explicit user op, 09.31). The evaluated outline becomes the base
    /// `Path`, geometry entries clear, bounds follow. Transparency survives.
    pub fn bake_geometry(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let object = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        if !object
            .modifiers
            .iter()
            .any(|m| m.enabled && Self::is_geometry_modifier(&m.kind))
        {
            return Ok(ChangeSet::empty());
        }
        self.bake_geometry_prefix(id, object.modifiers.len())
    }

    fn bake_geometry_prefix(
        &mut self,
        id: ObjectId,
        end: usize,
    ) -> Result<ChangeSet, PetuniaError> {
        let object = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        if matches!(
            object.shape,
            Some(crate::ShapeKind::Raster { .. })
                | Some(crate::ShapeKind::Image { .. })
                | Some(crate::ShapeKind::Text { .. })
        ) {
            return Err(PetuniaError::capability_unavailable("this source requires explicit rasterization/shaped outline conversion before geometry bake"));
        }
        let bounds = object.bounds.ok_or_else(|| frame_transform_error(id))?;
        let base = object
            .base_path_local()
            .map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
        let evaluated = crate::modifiers::evaluate_modifiers_local(
            &base,
            &object.modifiers[..end],
            [bounds[2], bounds[3]],
        )
        .ok_or_else(|| PetuniaError::invalid_input("modifier evaluation failed"))?;
        let (origin, size) = evaluated
            .bounding_box()
            .map_or(([0.0, 0.0], [bounds[2], bounds[3]]), |b| {
                ([b.x0, b.y0], [b.width().max(1.0), b.height().max(1.0)])
            });
        // Placement rotates about the local top-left. Moving the source origin
        // requires a rotated translation to keep every world point unchanged.
        let placed = object
            .local_transform()
            .apply(petunia_design_geometry::GPoint::new(origin[0], origin[1]));
        let next_bounds = Some([placed.x, placed.y, size[0], size[1]]);
        let next_shape = Some(crate::ShapeKind::LocalPath {
            path: std::sync::Arc::new(evaluated.transformed(
                petunia_design_geometry::GAffine::translate(-origin[0], -origin[1]),
            )),
            reference_size: size,
        });
        let next_modifiers = object
            .modifiers
            .iter()
            .enumerate()
            .filter(|(i, m)| !(*i < end && m.enabled && Self::is_geometry_modifier(&m.kind)))
            .map(|(_, m)| {
                m.clone()
                    .rebase_local([-origin[0], -origin[1]], [bounds[2], bounds[3]], size)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut candidate = object.clone();
        candidate.bounds = next_bounds;
        candidate.shape = next_shape.clone();
        candidate.modifiers = next_modifiers.clone();
        crate::validation::validate_object(&candidate)?;
        let mut changes = ChangeSet::empty();
        if object.bounds != next_bounds {
            changes.push(Change::BoundsChanged {
                id,
                previous_bounds: object.bounds,
                next_bounds,
                previous_rotation: object.rotation,
                next_rotation: object.rotation,
            });
        }
        changes.push(Change::ShapeChanged {
            id,
            previous: object.shape.clone(),
            next: next_shape,
        });
        changes.push(Change::ModifiersChanged {
            id,
            previous: object.modifiers.clone(),
            next: next_modifiers,
        });
        *self
            .document
            .find_object_mut(id)
            .expect("object was checked") = candidate;
        Ok(changes)
    }

    /// Offsets a path outline, non-destructively (09.31, 10.3).
    /// Legacy entry point: upserts the live `ContourOffset` modifier instead
    /// of rewriting geometry. Use [`Self::bake_contour`] to commit.
    pub fn offset_path(&mut self, id: ObjectId, delta: f64) -> Result<ChangeSet, PetuniaError> {
        if !delta.is_finite() {
            return Err(PetuniaError::invalid_input("offset delta must be finite"));
        }
        self.set_contour_offset(
            id,
            delta,
            petunia_design_geometry::OffsetJoin::Round,
            petunia_design_geometry::OffsetCap::None,
        )
    }
    /// Slices a path object at a specific point (10.2) (F-08: Break Path).
    /// Splits the nearest contour into two contours at the projection of
    /// `point`, preserving Bézier verbs by re-emitting the flattened split as
    /// line segments plus the original verbs' structure via `from_polygons`.
    /// Returns the shape change; bounds are recomputed from the result.
    pub fn slice_path(&mut self, id: ObjectId, point: [f64; 2]) -> Result<ChangeSet, PetuniaError> {
        if !point[0].is_finite() || !point[1].is_finite() {
            return Err(PetuniaError::invalid_input("slice point must be finite"));
        }
        // Snapshot shape+bounds without holding a borrow across mutation.
        let (prev_shape, prev_bounds, prev_rot) = {
            let obj = self
                .document
                .find_object(id)
                .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
            (obj.shape.clone(), obj.bounds, obj.rotation)
        };
        let path = match prev_shape.as_ref() {
            Some(shape) if shape.is_path() => self
                .document
                .find_object(id)
                .expect("object checked above")
                .to_path(),
            _ => {
                return Err(PetuniaError::invalid_input(format!(
                    "object `{id}` is not an editable path: convert to curves first"
                )))
            }
        };
        let target = petunia_design_geometry::GPoint::new(point[0], point[1]);
        let contours = path.to_polygons(0.5);
        if contours.is_empty() {
            return Err(PetuniaError::invalid_input(format!(
                "object `{id}` path is empty"
            )));
        }
        // Find nearest segment across all contours.
        let mut best: Option<(usize, usize, f64)> = None;
        for (ci, contour) in contours.iter().enumerate() {
            if contour.len() < 2 {
                continue;
            }
            for si in 0..contour.len() {
                let a = contour[si];
                let b_pt = contour[(si + 1) % contour.len()];
                let abx = b_pt.x - a.x;
                let aby = b_pt.y - a.y;
                let len2 = abx * abx + aby * aby;
                if len2 < 1e-12 {
                    continue;
                }
                let t = ((target.x - a.x) * abx + (target.y - a.y) * aby) / len2;
                let t = t.clamp(0.0, 1.0);
                let proj = petunia_design_geometry::GPoint::new(a.x + abx * t, a.y + aby * t);
                let d = proj.distance_to(target);
                if best.is_none_or(|(_, _, bd)| d < bd) {
                    best = Some((ci, si, d));
                }
            }
        }
        let (ci, si, _) = best.ok_or_else(|| {
            PetuniaError::invalid_input(format!("object `{id}` has no splittable segment"))
        })?;
        // Break contour `ci` after segment `si` into an open contour starting
        // at the projection point: [proj, si+1, ..., si] (two ends at proj).
        let contour = &contours[ci];
        let n = contour.len();
        let a = contour[si];
        let b_pt = contour[(si + 1) % n];
        let abx = b_pt.x - a.x;
        let aby = b_pt.y - a.y;
        let len2 = (abx * abx + aby * aby).max(1e-12);
        let t = (((target.x - a.x) * abx + (target.y - a.y) * aby) / len2).clamp(0.0, 1.0);
        let proj = petunia_design_geometry::GPoint::new(a.x + abx * t, a.y + aby * t);
        let mut broken: Vec<petunia_design_geometry::GPoint> = Vec::with_capacity(n + 2);
        broken.push(proj);
        for k in 1..=n {
            broken.push(contour[(si + k) % n]);
        }
        broken.push(proj);
        let mut new_contours = contours.clone();
        new_contours[ci] = broken;
        let new_path = petunia_design_geometry::GPath::from_polygons(&new_contours);
        let new_bounds = new_path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width(), r.height()]);
        if let Some(obj) = self.document.find_object_mut(id) {
            let next_shape =
                Some(crate::ShapeKind::Path(new_path).into_local(new_bounds.or(prev_bounds))?);
            obj.shape = next_shape.clone();
            if new_bounds.is_some() {
                obj.bounds = new_bounds;
            }
            let mut changes = ChangeSet::empty();
            changes.push(Change::ShapeChanged {
                id,
                previous: prev_shape,
                next: next_shape,
            });
            if new_bounds != prev_bounds {
                changes.push(Change::BoundsChanged {
                    id,
                    previous_bounds: prev_bounds,
                    next_bounds: new_bounds,
                    previous_rotation: prev_rot,
                    next_rotation: prev_rot,
                });
            }
            return Ok(changes);
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Aligns multiple objects relative to their collective bounding box (10.1).
    pub fn align_objects(
        &mut self,
        surface: SurfaceId,
        ids: &[ObjectId],
        mode: crate::AlignmentMode,
    ) -> Result<ChangeSet, PetuniaError> {
        let surf = self.document.surface(surface)?;
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for id in ids {
            if let Some(obj) = surf.objects.iter().find(|o| o.id == *id) {
                if let Some(b) = obj.bounds {
                    min_x = min_x.min(b[0]);
                    max_x = max_x.max(b[0] + b[2]);
                    min_y = min_y.min(b[1]);
                    max_y = max_y.max(b[1] + b[3]);
                }
            }
        }

        if min_x.is_infinite() {
            return Ok(ChangeSet::empty());
        }

        let mut changes = ChangeSet::empty();
        for id in ids {
            if let Some(c) = self.align_single_object(*id, mode, min_x, max_x, min_y, max_y)? {
                changes.extend(c);
            }
        }
        Ok(changes)
    }

    fn align_single_object(
        &mut self,
        id: ObjectId,
        mode: crate::AlignmentMode,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Result<Option<ChangeSet>, PetuniaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                if let Some(b) = object.bounds {
                    let mut new_x = b[0];
                    let mut new_y = b[1];
                    match mode {
                        crate::AlignmentMode::Left => new_x = min_x,
                        crate::AlignmentMode::Center => new_x = (min_x + max_x) / 2.0 - b[2] / 2.0,
                        crate::AlignmentMode::Right => new_x = max_x - b[2],
                        crate::AlignmentMode::Top => new_y = min_y,
                        crate::AlignmentMode::Middle => new_y = (min_y + max_y) / 2.0 - b[3] / 2.0,
                        crate::AlignmentMode::Bottom => new_y = max_y - b[3],
                    }
                    let previous_bounds = object.bounds;
                    let previous_rotation = object.rotation;
                    object.bounds = Some([new_x, new_y, b[2], b[3]]);
                    let mut changes = ChangeSet::empty();
                    changes.push(Change::BoundsChanged {
                        id,
                        previous_bounds,
                        next_bounds: object.bounds,
                        previous_rotation,
                        next_rotation: previous_rotation,
                    });
                    return Ok(Some(changes));
                }
            }
        }
        Ok(None)
    }

    /// Distributes objects with equal gaps along an axis (10.1) (F-16).
    /// First and last (by leading edge) stay fixed; middle objects are placed
    /// so gaps between consecutive bounds are equal. Rotation is preserved
    /// (F-07): previous code forced `rotation: 0.0`.
    pub fn distribute_objects(
        &mut self,
        surface: SurfaceId,
        ids: &[ObjectId],
        axis: crate::DistributionAxis,
    ) -> Result<ChangeSet, PetuniaError> {
        if ids.len() < 3 {
            return Ok(ChangeSet::empty());
        }
        let surf = self.document.surface(surface)?;
        // Snapshot bounds+rotation without holding borrows across set_bounds.
        let mut items: Vec<(ObjectId, [f64; 4], f64)> = Vec::new();
        for id in ids {
            if let Some(obj) = surf.objects.iter().find(|o| o.id == *id) {
                if let Some(b) = obj.bounds {
                    items.push((*id, b, obj.rotation));
                }
            }
        }
        if items.len() < 3 {
            return Ok(ChangeSet::empty());
        }

        match axis {
            crate::DistributionAxis::Horizontal => {
                items.sort_by(|a, b| {
                    a.1[0]
                        .partial_cmp(&b.1[0])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let first_lead = items.first().unwrap().1[0];
                let last_trail = items.last().unwrap().1[0] + items.last().unwrap().1[2];
                let total_width: f64 = items.iter().map(|(_, b, _)| b[2]).sum();
                let span = last_trail - first_lead;
                let gap = (span - total_width) / ((items.len() - 1) as f64);
                if !gap.is_finite() {
                    return Ok(ChangeSet::empty());
                }
                let mut changes = ChangeSet::empty();
                let mut cursor = first_lead;
                for (id, b, rot) in items {
                    let c = self.set_bounds(id, Some([cursor, b[1], b[2], b[3]]), rot)?;
                    changes.extend(c);
                    cursor += b[2] + gap;
                }
                Ok(changes)
            }
            crate::DistributionAxis::Vertical => {
                items.sort_by(|a, b| {
                    a.1[1]
                        .partial_cmp(&b.1[1])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let first_lead = items.first().unwrap().1[1];
                let last_trail = items.last().unwrap().1[1] + items.last().unwrap().1[3];
                let total_height: f64 = items.iter().map(|(_, b, _)| b[3]).sum();
                let span = last_trail - first_lead;
                let gap = (span - total_height) / ((items.len() - 1) as f64);
                if !gap.is_finite() {
                    return Ok(ChangeSet::empty());
                }
                let mut changes = ChangeSet::empty();
                let mut cursor = first_lead;
                for (id, b, rot) in items {
                    let c = self.set_bounds(id, Some([b[0], cursor, b[2], b[3]]), rot)?;
                    changes.extend(c);
                    cursor += b[3] + gap;
                }
                Ok(changes)
            }
        }
    }

    /// Syncs legacy `fill/stroke/stroke_width/opacity` mirrors from a stack.
    /// Solid primaries mirror their token; non-solid primaries clear the stale
    /// legacy token so PDF/SVG/pixel paths (which read the stack first) never
    /// show a stale color (F-06).
    fn sync_legacy_from_stack(
        object: &mut DocumentObject,
        stack: Option<&crate::appearance::AppearanceStack>,
    ) {
        match stack {
            None => {}
            Some(app) => {
                match app.primary_fill() {
                    Some(pf) => match &pf.paint {
                        crate::appearance::Paint::Solid(col) => {
                            object.fill = Some(col.clone());
                        }
                        _ => {
                            object.fill = None;
                        }
                    },
                    None => {
                        object.fill = None;
                    }
                }
                match app.primary_stroke() {
                    Some(ps) => match &ps.paint {
                        crate::appearance::Paint::Solid(col) => {
                            object.stroke = Some(col.clone());
                            object.stroke_width = ps.width;
                        }
                        _ => {
                            object.stroke = None;
                        }
                    },
                    None => {
                        object.stroke = None;
                    }
                }
                object.opacity = app.opacity.clamp(0.0, 1.0);
            }
        }
    }

    /// Sets an object's appearance stack (F-06).
    /// Emits `AppearanceChanged` plus mirrored `FillChanged`/`StrokeChanged`/
    /// `OpacityChanged` so undo reverts both the stack and the legacy mirrors
    /// in one atomic `ChangeSet`.
    pub fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<crate::appearance::AppearanceStack>,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(stack) = &appearance {
            crate::validation::validate_appearance(stack)?;
        }
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.appearance.clone();
                let prev_fill = object.fill.clone();
                let prev_stroke = object.stroke.clone();
                let prev_width = object.stroke_width;
                let prev_opacity = object.opacity;
                if previous == appearance {
                    return Ok(ChangeSet::empty());
                }
                Self::sync_legacy_from_stack(object, appearance.as_ref());
                object.appearance = appearance.clone();
                let mut changes = ChangeSet::empty();
                if prev_fill != object.fill {
                    changes.push(Change::FillChanged {
                        id,
                        previous: prev_fill,
                        next: object.fill.clone(),
                    });
                }
                if prev_stroke != object.stroke || prev_width != object.stroke_width {
                    changes.push(Change::StrokeChanged {
                        id,
                        previous_stroke: prev_stroke,
                        next_stroke: object.stroke.clone(),
                        previous_width: prev_width,
                        next_width: object.stroke_width,
                    });
                }
                if (prev_opacity - object.opacity).abs() > f64::EPSILON {
                    changes.push(Change::OpacityChanged {
                        id,
                        previous: prev_opacity,
                        next: object.opacity,
                    });
                }
                changes.push(Change::AppearanceChanged {
                    id,
                    previous,
                    next: appearance,
                });
                return Ok(changes);
            }
        }
        Err(PetuniaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Loads the editable appearance stack for granular commands (F-18).
    /// Returns `(object_id, stack)`: the effective stack when no explicit
    /// stack exists, so simple UI targets the primary entry without ever
    /// silently deleting secondary entries (10.4).
    fn editable_stack(
        &self,
        id: ObjectId,
    ) -> Result<crate::appearance::AppearanceStack, PetuniaError> {
        self.document
            .find_object(id)
            .map(|o| o.effective_appearance())
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))
    }

    /// Appends a fill entry, reassigning `id` on collision (F-18).
    pub fn add_fill(
        &mut self,
        id: ObjectId,
        mut fill: crate::appearance::FillItem,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.fills.iter().any(|f| f.id == fill.id) {
            fill.id = next_local_id(stack.fills.iter().map(|f| f.id))?;
        }
        stack.add_fill(fill);
        self.set_appearance(id, Some(stack))
    }

    /// Removes a fill entry by local id (F-18). Missing ids are an error;
    /// removing the last entry is allowed (object keeps stack opacity/blend).
    pub fn remove_fill(&mut self, id: ObjectId, fill_id: u32) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if !stack.remove_fill(fill_id) {
            return Err(PetuniaError::not_found(format!(
                "fill `{fill_id}` not found on object `{id}`"
            )));
        }
        self.set_appearance(id, Some(stack))
    }

    /// Sets one fill entry's opacity (F-18). NoOp when unchanged.
    pub fn set_fill_item_opacity(
        &mut self,
        id: ObjectId,
        fill_id: u32,
        opacity: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let entry = stack
            .fills
            .iter_mut()
            .find(|f| f.id == fill_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("fill `{fill_id}` not found on object `{id}`"))
            })?;
        let clamped = opacity.clamp(0.0, 1.0);
        if (entry.opacity - clamped).abs() <= f64::EPSILON {
            return Ok(ChangeSet::empty());
        }
        entry.opacity = clamped;
        self.set_appearance(id, Some(stack))
    }

    /// Sets one fill entry's blend mode (F-18). NoOp when unchanged.
    pub fn set_fill_item_blend(
        &mut self,
        id: ObjectId,
        fill_id: u32,
        blend_mode: crate::appearance::BlendMode,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let entry = stack
            .fills
            .iter_mut()
            .find(|f| f.id == fill_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("fill `{fill_id}` not found on object `{id}`"))
            })?;
        if entry.blend_mode == blend_mode {
            return Ok(ChangeSet::empty());
        }
        entry.blend_mode = blend_mode;
        self.set_appearance(id, Some(stack))
    }

    /// Moves a fill entry to a new position in stack order (F-18).
    pub fn reorder_fill(
        &mut self,
        id: ObjectId,
        fill_id: u32,
        new_index: usize,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let pos = stack
            .fills
            .iter()
            .position(|f| f.id == fill_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("fill `{fill_id}` not found on object `{id}`"))
            })?;
        let dest = new_index.min(stack.fills.len().saturating_sub(1));
        if pos == dest {
            return Ok(ChangeSet::empty());
        }
        let item = stack.fills.remove(pos);
        stack.fills.insert(dest, item);
        self.set_appearance(id, Some(stack))
    }

    /// Appends a stroke entry, reassigning `id` on collision (F-18).
    pub fn add_stroke(
        &mut self,
        id: ObjectId,
        mut stroke: crate::appearance::StrokeItem,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.strokes.iter().any(|s| s.id == stroke.id) {
            stroke.id = next_local_id(stack.strokes.iter().map(|s| s.id))?;
        }
        stack.add_stroke(stroke);
        self.set_appearance(id, Some(stack))
    }

    /// Removes a stroke entry by local id (F-18).
    pub fn remove_stroke(
        &mut self,
        id: ObjectId,
        stroke_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if !stack.remove_stroke(stroke_id) {
            return Err(PetuniaError::not_found(format!(
                "stroke `{stroke_id}` not found on object `{id}`"
            )));
        }
        self.set_appearance(id, Some(stack))
    }

    /// Sets one stroke entry's width (F-18). Negative widths are rejected.
    pub fn set_stroke_item_width(
        &mut self,
        id: ObjectId,
        stroke_id: u32,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        if !width.is_finite() || width < 0.0 {
            return Err(PetuniaError::invalid_input(
                "stroke width must be finite and >= 0",
            ));
        }
        let mut stack = self.editable_stack(id)?;
        let entry = stack
            .strokes
            .iter_mut()
            .find(|s| s.id == stroke_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("stroke `{stroke_id}` not found on object `{id}`"))
            })?;
        if (entry.width - width).abs() <= f64::EPSILON {
            return Ok(ChangeSet::empty());
        }
        entry.width = width;
        self.set_appearance(id, Some(stack))
    }

    /// Moves a stroke entry to a new position in stack order (F-18).
    pub fn reorder_stroke(
        &mut self,
        id: ObjectId,
        stroke_id: u32,
        new_index: usize,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let pos = stack
            .strokes
            .iter()
            .position(|s| s.id == stroke_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("stroke `{stroke_id}` not found on object `{id}`"))
            })?;
        let dest = new_index.min(stack.strokes.len().saturating_sub(1));
        if pos == dest {
            return Ok(ChangeSet::empty());
        }
        let item = stack.strokes.remove(pos);
        stack.strokes.insert(dest, item);
        self.set_appearance(id, Some(stack))
    }

    /// Appends an effect entry, reassigning `id` on collision (F-18).
    pub fn add_effect(
        &mut self,
        id: ObjectId,
        mut effect: crate::appearance::EffectItem,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.effects.iter().any(|e| e.id == effect.id) {
            effect.id = next_local_id(stack.effects.iter().map(|e| e.id))?;
        }
        stack.add_effect(effect);
        self.set_appearance(id, Some(stack))
    }

    /// Removes an effect entry by local id (F-18).
    pub fn remove_effect(
        &mut self,
        id: ObjectId,
        effect_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if !stack.remove_effect(effect_id) {
            return Err(PetuniaError::not_found(format!(
                "effect `{effect_id}` not found on object `{id}`"
            )));
        }
        self.set_appearance(id, Some(stack))
    }

    /// Enables/disables an effect entry without deleting it (F-18, 10.4).
    /// NoOp when the flag already matches.
    pub fn toggle_effect(
        &mut self,
        id: ObjectId,
        effect_id: u32,
        visible: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let entry = stack
            .effects
            .iter_mut()
            .find(|e| e.id == effect_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("effect `{effect_id}` not found on object `{id}`"))
            })?;
        if entry.visible == visible {
            return Ok(ChangeSet::empty());
        }
        entry.visible = visible;
        self.set_appearance(id, Some(stack))
    }

    /// Appends a tonal adjustment entry, reassigning `id` on collision (Spec 10.10).
    pub fn add_adjustment(
        &mut self,
        id: ObjectId,
        mut adjustment: crate::adjustments::AdjustmentItem,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.adjustments.iter().any(|a| a.id == adjustment.id) {
            adjustment.id = next_local_id(stack.adjustments.iter().map(|a| a.id))?;
        }
        stack.add_adjustment(adjustment);
        self.set_appearance(id, Some(stack))
    }

    /// Removes a tonal adjustment entry by local id (Spec 10.10).
    pub fn remove_adjustment(
        &mut self,
        id: ObjectId,
        adjustment_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if !stack.remove_adjustment(adjustment_id) {
            return Err(PetuniaError::not_found(format!(
                "adjustment `{adjustment_id}` not found on object `{id}`"
            )));
        }
        self.set_appearance(id, Some(stack))
    }

    /// Updates an existing tonal adjustment entry (Spec 10.10).
    pub fn set_adjustment(
        &mut self,
        id: ObjectId,
        adjustment: crate::adjustments::AdjustmentItem,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if let Some(entry) = stack.adjustments.iter_mut().find(|a| a.id == adjustment.id) {
            *entry = adjustment;
            self.set_appearance(id, Some(stack))
        } else {
            Err(PetuniaError::not_found(format!(
                "adjustment `{}` not found on object `{id}`",
                adjustment.id
            )))
        }
    }

    /// Sets the whole-stack opacity (F-18). Mirrors to legacy opacity.
    pub fn set_stack_opacity(
        &mut self,
        id: ObjectId,
        opacity: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        let clamped = opacity.clamp(0.0, 1.0);
        if (stack.opacity - clamped).abs() <= f64::EPSILON {
            return Ok(ChangeSet::empty());
        }
        stack.opacity = clamped;
        self.set_appearance(id, Some(stack))
    }

    /// Sets the whole-stack blend mode (F-18). NoOp when unchanged.
    pub fn set_stack_blend(
        &mut self,
        id: ObjectId,
        blend_mode: crate::appearance::BlendMode,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.blend_mode == blend_mode {
            return Ok(ChangeSet::empty());
        }
        stack.blend_mode = blend_mode;
        self.set_appearance(id, Some(stack))
    }

    /// Copies an appearance stack from one object to another (F-18, 10.4
    /// Copy/Paste Style). Destination legacy mirrors are synced by
    /// `set_appearance`.
    pub fn paste_appearance(
        &mut self,
        source_id: ObjectId,
        dest_id: ObjectId,
    ) -> Result<ChangeSet, PetuniaError> {
        let stack = self
            .document
            .find_object(source_id)
            .ok_or_else(|| {
                PetuniaError::not_found(format!("source object `{source_id}` not found"))
            })?
            .effective_appearance();
        self.set_appearance(dest_id, Some(stack))
    }

    /// Clears all effects, keeping fills/strokes (F-18). NoOp when empty.
    pub fn clear_effects(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let mut stack = self.editable_stack(id)?;
        if stack.effects.is_empty() {
            return Ok(ChangeSet::empty());
        }
        stack.effects.clear();
        self.set_appearance(id, Some(stack))
    }

    /// Moves an object one step or to an edge of surface z-order (10.1, F-16).
    /// Implemented via [`Self::reorder_object`] so undo restores the exact
    /// previous index. Edge positions are a NoOp (empty set, no history).
    pub fn arrange_object(
        &mut self,
        surface: SurfaceId,
        id: ObjectId,
        position: crate::ArrangePosition,
    ) -> Result<ChangeSet, PetuniaError> {
        let target = self.document.surface(surface)?;
        let len = target.objects.len();
        let current = target
            .objects
            .iter()
            .position(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let dest = match position {
            crate::ArrangePosition::Front => len.saturating_sub(1),
            crate::ArrangePosition::Back => 0,
            crate::ArrangePosition::Forward => (current + 1).min(len.saturating_sub(1)),
            crate::ArrangePosition::Backward => current.saturating_sub(1),
        };
        if dest == current {
            return Ok(ChangeSet::empty());
        }
        self.reorder_object(surface, id, dest)
    }

    /// Reorders an object within its surface.
    pub fn reorder_object(
        &mut self,
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    ) -> Result<ChangeSet, PetuniaError> {
        let target = self.document.surface_mut(surface)?;
        let current_pos = target
            .objects
            .iter()
            .position(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        let dest = new_index.min(target.objects.len() - 1);
        if current_pos == dest {
            return Ok(ChangeSet::empty());
        }
        let item = target.objects.remove(current_pos);
        target.objects.insert(dest, item);
        let mut changes = ChangeSet::empty();
        changes.push(Change::ObjectReordered {
            surface,
            id,
            previous_index: current_pos,
            next_index: dest,
        });
        Ok(changes)
    }

    /// Groups multiple objects under a new container object (10.5 One-Tree).
    ///
    /// The operation validates the common parent, captures each child's world
    /// transform, and derives the group frame in that parent's local space.
    /// Paths and document-anchored modifiers without an explicit frame are
    /// rejected until their migration is available.
    pub fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: crate::hierarchy::ContainerRole,
    ) -> Result<ChangeSet, PetuniaError> {
        if child_ids.is_empty() {
            return Err(PetuniaError::invalid_input("cannot create empty group"));
        }
        if self.document.find_object(group_id).is_some() {
            return Err(PetuniaError::invalid_input(format!(
                "object `{group_id}` already exists"
            )));
        }
        if child_ids.contains(&group_id) {
            return Err(PetuniaError::invalid_input(
                "cannot group object into itself",
            ));
        }

        let first_child = self.document.find_object(child_ids[0]).ok_or_else(|| {
            PetuniaError::not_found(format!("child `{}` not found", child_ids[0]))
        })?;
        let common_parent = first_child.parent;
        let mut child_snapshots = Vec::with_capacity(child_ids.len());
        for child_id in child_ids.iter().copied() {
            let child = self
                .document
                .find_object(child_id)
                .ok_or_else(|| PetuniaError::not_found(format!("child `{child_id}` not found")))?;
            if child.parent != common_parent {
                return Err(PetuniaError::invalid_input(
                    "grouping requires all children to share the same parent",
                ));
            }
            if child.locked {
                return Err(PetuniaError::invalid_input(format!(
                    "child `{child_id}` is locked"
                )));
            }
            child
                .evaluated_path_local()
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            let world = self
                .document
                .world_transform_checked(child_id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            let world_bounds = self
                .document
                .frame_bounds_world(child_id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            let previous_index = if let Some(parent_id) = common_parent {
                self.document
                    .find_object(parent_id)
                    .and_then(|parent| parent.children.iter().position(|id| *id == child_id))
                    .unwrap_or(0)
            } else {
                self.document
                    .surface(surface)
                    .ok()
                    .and_then(|surface| {
                        surface
                            .objects
                            .iter()
                            .position(|object| object.id == child_id)
                    })
                    .unwrap_or(0)
            };
            child_snapshots.push((child_id, child.clone(), previous_index, world, world_bounds));
        }

        let parent_world = match common_parent {
            Some(parent_id) => self
                .document
                .world_transform_checked(parent_id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?,
            None => petunia_design_geometry::GAffine::IDENTITY,
        };
        let inverse_parent = parent_world.inverse().ok_or_else(|| {
            PetuniaError::invalid_input("group parent transform is not invertible")
        })?;
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for (_, _, _, _, [x, y, width, height]) in &child_snapshots {
            let corners = [
                petunia_design_geometry::GPoint::new(*x, *y),
                petunia_design_geometry::GPoint::new(*x + *width, *y),
                petunia_design_geometry::GPoint::new(*x + *width, *y + *height),
                petunia_design_geometry::GPoint::new(*x, *y + *height),
            ];
            for corner in corners {
                let local = inverse_parent.apply(corner);
                min_x = min_x.min(local.x);
                min_y = min_y.min(local.y);
                max_x = max_x.max(local.x);
                max_y = max_y.max(local.y);
            }
        }
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return Err(PetuniaError::invalid_input(
                "group children do not have finite world frames",
            ));
        }
        let group_bounds = [
            min_x,
            min_y,
            (max_x - min_x).max(1.0),
            (max_y - min_y).max(1.0),
        ];
        let group_local =
            petunia_design_geometry::GAffine::translate(group_bounds[0], group_bounds[1]);
        let group_world = parent_world.after(group_local);
        let inverse_group = group_world
            .inverse()
            .ok_or_else(|| PetuniaError::invalid_input("group transform is not invertible"))?;
        let group_insert_index = child_snapshots
            .iter()
            .map(|(_, _, previous_index, _, _)| *previous_index)
            .min()
            .unwrap_or(0);
        let mut candidates = Vec::with_capacity(child_snapshots.len());
        for (child_id, child, previous_index, world, _) in child_snapshots {
            let mut candidate = child.clone();
            candidate.parent = Some(group_id);
            let local = inverse_group.after(world);
            reframe_object(&mut candidate, local)?;
            candidates.push((child_id, child, candidate, previous_index));
        }

        let mut group_object = DocumentObject::new(
            group_id,
            match role {
                crate::hierarchy::ContainerRole::Group => "Group",
                crate::hierarchy::ContainerRole::Layer => "Layer",
                crate::hierarchy::ContainerRole::ClipGroup => "Clip Group",
            },
        );
        group_object.role = Some(role);
        group_object.parent = common_parent;
        group_object.bounds = Some(group_bounds);
        group_object.children = child_ids.clone();

        let mut changes = ChangeSet::empty();
        let group_index = {
            let target = self.document.surface_mut(surface)?;
            target.objects.push(group_object.clone());
            target.objects.len() - 1
        };
        changes.push(Change::ObjectAdded {
            surface,
            object: group_object,
            index: group_index,
        });

        if let Some(parent_id) = common_parent {
            if let Some(parent) = self.document.find_object_mut(parent_id) {
                let previous_children = parent.children.clone();
                let insert_index = group_insert_index;
                parent.children.retain(|id| !child_ids.contains(id));
                parent
                    .children
                    .insert(insert_index.min(parent.children.len()), group_id);
                changes.push(Change::ChildrenChanged {
                    id: parent_id,
                    previous_children,
                    next_children: parent.children.clone(),
                });
            }
        }

        for (child_id, previous, candidate, previous_index) in candidates {
            if let Some(child) = self.document.find_object_mut(child_id) {
                let previous_bounds = previous.bounds;
                let previous_rotation = previous.rotation;
                child.bounds = candidate.bounds;
                child.rotation = candidate.rotation;
                child.parent = Some(group_id);
                changes.push(Change::BoundsChanged {
                    id: child_id,
                    previous_bounds,
                    next_bounds: child.bounds,
                    previous_rotation,
                    next_rotation: child.rotation,
                });
                changes.push(Change::Reparented {
                    id: child_id,
                    previous_parent: previous.parent,
                    next_parent: Some(group_id),
                    previous_index,
                    next_index: 0,
                });
            }
        }
        Ok(changes)
    }

    /// Ungroups a container object while preserving each child's world transform.
    pub fn ungroup_objects(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let group = self
            .document
            .find_object(group_id)
            .cloned()
            .ok_or_else(|| PetuniaError::not_found(format!("group `{group_id}` not found")))?;
        let surface_id = self.document.find_object_surface(group_id).ok_or_else(|| {
            PetuniaError::not_found(format!("surface for `{group_id}` not found"))
        })?;
        let parent_id = group.parent;
        self.document
            .world_transform_checked(group_id)
            .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
        let parent_world = match parent_id {
            Some(parent_id) => self
                .document
                .world_transform_checked(parent_id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?,
            None => petunia_design_geometry::GAffine::IDENTITY,
        };
        let inverse_parent = parent_world.inverse().ok_or_else(|| {
            PetuniaError::invalid_input("group parent transform is not invertible")
        })?;
        let mut candidates = Vec::with_capacity(group.children.len());
        for (index, child_id) in group.children.iter().copied().enumerate() {
            let child = self
                .document
                .find_object(child_id)
                .cloned()
                .ok_or_else(|| PetuniaError::not_found(format!("child `{child_id}` not found")))?;
            child
                .evaluated_path_local()
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            let world = self
                .document
                .world_transform_checked(child_id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            let mut candidate = child.clone();
            candidate.parent = parent_id;
            reframe_object(&mut candidate, inverse_parent.after(world))?;
            candidates.push((index, child, candidate));
        }

        let mut changes = ChangeSet::empty();
        for (index, previous, candidate) in candidates {
            if let Some(child) = self.document.find_object_mut(candidate.id) {
                let previous_bounds = previous.bounds;
                let previous_rotation = previous.rotation;
                child.bounds = candidate.bounds;
                child.rotation = candidate.rotation;
                child.parent = parent_id;
                changes.push(Change::BoundsChanged {
                    id: candidate.id,
                    previous_bounds,
                    next_bounds: child.bounds,
                    previous_rotation,
                    next_rotation: child.rotation,
                });
                changes.push(Change::Reparented {
                    id: candidate.id,
                    previous_parent: previous.parent,
                    next_parent: parent_id,
                    previous_index: index,
                    next_index: index,
                });
            }
        }

        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.document.find_object_mut(parent_id) {
                let previous_children = parent.children.clone();
                let insert_index = previous_children
                    .iter()
                    .position(|id| *id == group_id)
                    .unwrap_or(parent.children.len());
                parent.children.retain(|id| *id != group_id);
                for (offset, child_id) in group.children.iter().enumerate() {
                    parent.children.insert(insert_index + offset, *child_id);
                }
                changes.push(Change::ChildrenChanged {
                    id: parent_id,
                    previous_children,
                    next_children: parent.children.clone(),
                });
            }
        }

        let surface = self.document.surface_mut(surface_id)?;
        if let Some(index) = surface
            .objects
            .iter()
            .position(|object| object.id == group_id)
        {
            let removed_group = surface.objects.remove(index);
            changes.push(Change::ObjectRemoved {
                surface: surface_id,
                object: removed_group,
                index,
            });
        }
        Ok(changes)
    }

    /// Reparents an object to a new container (or root) with cycle validation and transform preservation (F-09).
    /// Records the real previous index. Reordering inside the same parent is
    /// supported via `target_index` instead of returning an empty set.
    /// When `preserve_world_transform` is set, both translation and rotation
    /// are recomposed from the world affine (previous code dropped rotation).
    pub fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(np) = new_parent {
            if self.document.is_descendant(np, id) {
                return Err(PetuniaError::invalid_input(format!(
                    "cycle detected: cannot parent object `{id}` under its descendant `{np}`"
                )));
            }
            if self.document.find_object(np).is_none() {
                return Err(PetuniaError::not_found(format!(
                    "target parent `{np}` not found"
                )));
            }
        }

        let world_tx = if preserve_world_transform {
            Some(self.document.world_transform(id)?)
        } else {
            None
        };

        let new_parent_world = if preserve_world_transform {
            match new_parent {
                Some(np_id) => self.document.world_transform(np_id)?,
                None => petunia_design_geometry::GAffine::IDENTITY,
            }
        } else {
            petunia_design_geometry::GAffine::IDENTITY
        };
        let reframed = if let Some(world) = world_tx {
            let object = self
                .document
                .find_object(id)
                .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?;
            let inverse_parent = new_parent_world.inverse().ok_or_else(|| {
                PetuniaError::invalid_input("new parent transform is not invertible")
            })?;
            let mut candidate = object.clone();
            reframe_object(&mut candidate, inverse_parent.after(world))?;
            Some((candidate.bounds, candidate.rotation))
        } else {
            None
        };

        let current_parent = self
            .document
            .find_object(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?
            .parent;

        // Real previous index: position inside old parent's children, or
        // z-order position on the surface when parent is None.
        let previous_index = match current_parent {
            Some(cp_id) => self
                .document
                .find_object(cp_id)
                .and_then(|p| p.children.iter().position(|c| *c == id))
                .unwrap_or(0),
            None => {
                let surf_id = self.document.find_object_surface(id).ok_or_else(|| {
                    PetuniaError::not_found(format!("surface for `{id}` not found"))
                })?;
                self.document
                    .surface(surf_id)
                    .ok()
                    .and_then(|s| s.objects.iter().position(|o| o.id == id))
                    .unwrap_or(0)
            }
        };

        // Same-parent reorder: move within children (or NoOp when equal).
        if current_parent == new_parent {
            if let Some(cp_id) = current_parent {
                let (prev_ch, next_ch, moved) = {
                    let parent = self.document.find_object_mut(cp_id).ok_or_else(|| {
                        PetuniaError::not_found(format!("parent `{cp_id}` not found"))
                    })?;
                    let prev_ch = parent.children.clone();
                    let old_pos = parent.children.iter().position(|c| *c == id);
                    match old_pos {
                        None => (prev_ch, parent.children.clone(), false),
                        Some(old) => {
                            let mut next = parent.children.clone();
                            next.remove(old);
                            let dest = target_index.min(next.len());
                            if old == dest {
                                (prev_ch, parent.children.clone(), false)
                            } else {
                                next.insert(dest, id);
                                parent.children = next.clone();
                                (prev_ch, next, true)
                            }
                        }
                    }
                };
                if !moved {
                    return Ok(ChangeSet::empty());
                }
                let mut changes = ChangeSet::empty();
                changes.push(Change::ChildrenChanged {
                    id: cp_id,
                    previous_children: prev_ch,
                    next_children: next_ch,
                });
                changes.push(Change::Reparented {
                    id,
                    previous_parent: current_parent,
                    next_parent: new_parent,
                    previous_index,
                    next_index: target_index,
                });
                return Ok(changes);
            }
            return Ok(ChangeSet::empty());
        }

        let mut changes = ChangeSet::empty();

        if let Some(cp_id) = current_parent {
            if let Some(cp) = self.document.find_object_mut(cp_id) {
                let prev_ch = cp.children.clone();
                if let Some(pos) = cp.children.iter().position(|c| *c == id) {
                    cp.children.remove(pos);
                }
                changes.push(Change::ChildrenChanged {
                    id: cp_id,
                    previous_children: prev_ch,
                    next_children: cp.children.clone(),
                });
            }
        }

        if let Some(np_id) = new_parent {
            if let Some(np) = self.document.find_object_mut(np_id) {
                let prev_ch = np.children.clone();
                let insert_pos = target_index.min(np.children.len());
                np.children.insert(insert_pos, id);
                changes.push(Change::ChildrenChanged {
                    id: np_id,
                    previous_children: prev_ch,
                    next_children: np.children.clone(),
                });
            }
        }

        if let Some(obj) = self.document.find_object_mut(id) {
            let prev_parent = obj.parent;
            obj.parent = new_parent;
            changes.push(Change::Reparented {
                id,
                previous_parent: prev_parent,
                next_parent: new_parent,
                previous_index,
                next_index: target_index,
            });

            if let Some((next_bounds, next_rotation)) = reframed {
                let prev_bounds = obj.bounds;
                let prev_rot = obj.rotation;
                obj.bounds = next_bounds;
                obj.rotation = next_rotation;
                changes.push(Change::BoundsChanged {
                    id,
                    previous_bounds: prev_bounds,
                    next_bounds: obj.bounds,
                    previous_rotation: prev_rot,
                    next_rotation: obj.rotation,
                });
            }
        }

        Ok(changes)
    }

    /// Creates a clipping mask group containing a mask object and masked content items.
    pub fn create_clip_group(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut all_ids = vec![mask_id];
        all_ids.extend(content_ids.iter().copied());

        let mut changes = self.group_objects(
            surface,
            group_id,
            all_ids,
            crate::hierarchy::ContainerRole::ClipGroup,
        )?;

        if self.document.find_object(mask_id).is_some_and(|mask| {
            matches!(
                mask.shape,
                Some(crate::ShapeKind::Raster { .. }) | Some(crate::ShapeKind::Image { .. })
            )
        }) {
            changes.extend(self.set_mask_mode(mask_id, crate::MaskMode::Alpha)?);
        }
        if let Some(mask) = self.document.find_object_mut(mask_id) {
            let prev_is_mask = mask.is_clip_mask;
            let prev_mask_id = mask.clip_mask_id;
            mask.is_clip_mask = true;
            changes.push(Change::ClipMaskChanged {
                id: mask_id,
                previous_mask: prev_mask_id,
                next_mask: None,
                previous_is_mask: prev_is_mask,
                next_is_mask: true,
            });
        }

        for cid in content_ids {
            if let Some(child) = self.document.find_object_mut(cid) {
                let prev_mask = child.clip_mask_id;
                let prev_is_mask = child.is_clip_mask;
                child.clip_mask_id = Some(mask_id);
                changes.push(Change::ClipMaskChanged {
                    id: cid,
                    previous_mask: prev_mask,
                    next_mask: Some(mask_id),
                    previous_is_mask: prev_is_mask,
                    next_is_mask: false,
                });
            }
        }

        Ok(changes)
    }

    /// Releases a clipping group, freeing the mask boundary and content.
    pub fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let group = self
            .document
            .find_object(group_id)
            .ok_or_else(|| PetuniaError::not_found(format!("group `{group_id}` not found")))?;
        let children = group.children.clone();

        let mut changes = ChangeSet::empty();
        for cid in &children {
            if let Some(child) = self.document.find_object_mut(*cid) {
                let prev_mask = child.clip_mask_id;
                let prev_is_mask = child.is_clip_mask;
                child.is_clip_mask = false;
                child.clip_mask_id = None;
                changes.push(Change::ClipMaskChanged {
                    id: *cid,
                    previous_mask: prev_mask,
                    next_mask: None,
                    previous_is_mask: prev_is_mask,
                    next_is_mask: false,
                });
            }
        }

        let ungroup_changes = self.ungroup_objects(group_id)?;
        for c in ungroup_changes.changes {
            changes.push(c);
        }

        Ok(changes)
    }

    /// Sets a surface's origin and dimensions (10.7).
    pub fn set_surface_geometry(
        &mut self,
        id: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        if !origin.iter().all(|v| v.is_finite())
            || !dimensions.iter().all(|v| v.is_finite() && *v > 0.0)
        {
            return Err(PetuniaError::invalid_input("invalid surface geometry"));
        }
        let surf = self.document.surface_mut(id)?;
        let prev_orig = surf.origin;
        let prev_dim = surf.dimensions;
        surf.origin = origin;
        surf.dimensions = dimensions;
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceGeometryChanged {
            id,
            previous_origin: prev_orig,
            next_origin: surf.origin,
            previous_dimensions: prev_dim,
            next_dimensions: surf.dimensions,
        });
        Ok(changes)
    }

    /// Sets a surface's bleed configuration.
    pub fn set_surface_bleed(
        &mut self,
        id: SurfaceId,
        bleed: crate::surface_metadata::Bleed,
    ) -> Result<ChangeSet, PetuniaError> {
        if ![bleed.top, bleed.right, bleed.bottom, bleed.left]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
        {
            return Err(PetuniaError::invalid_input("invalid surface bleed"));
        }
        let surf = self.document.surface_mut(id)?;
        let prev = surf.bleed;
        surf.bleed = bleed;
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceBleedChanged {
            id,
            previous: prev,
            next: bleed,
        });
        Ok(changes)
    }

    /// Sets a surface's margins configuration.
    pub fn set_surface_margins(
        &mut self,
        id: SurfaceId,
        margins: crate::surface_metadata::Margins,
    ) -> Result<ChangeSet, PetuniaError> {
        if ![margins.top, margins.right, margins.bottom, margins.left]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
        {
            return Err(PetuniaError::invalid_input("invalid surface margins"));
        }
        let surf = self.document.surface_mut(id)?;
        let prev = surf.margins;
        surf.margins = margins;
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceMarginsChanged {
            id,
            previous: prev,
            next: margins,
        });
        Ok(changes)
    }

    /// Sets a surface's background color token or hex.
    pub fn set_surface_background(
        &mut self,
        id: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        let surf = self.document.surface_mut(id)?;
        let prev = surf.background.clone();
        surf.background = background.clone();
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceBackgroundChanged {
            id,
            previous: prev,
            next: background,
        });
        Ok(changes)
    }

    /// Assigns an actual CMYK profile without changing any ink numbers.
    pub fn set_surface_cmyk_profile(
        &mut self,
        id: SurfaceId,
        profile: Option<petunia_design_color::IccProfile>,
    ) -> Result<ChangeSet, PetuniaError> {
        if profile.as_ref().is_some_and(|p| !p.is_press_profile()) {
            return Err(PetuniaError::invalid_input(
                "surface press profile must be an ICC CMYK output profile",
            ));
        }
        let surface = self.document.surface_mut(id)?;
        let previous = surface.cmyk_profile.clone();
        if previous == profile {
            return Ok(ChangeSet::empty());
        }
        surface.cmyk_profile = profile.clone();
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceCmykProfileChanged {
            id,
            previous,
            next: profile,
        });
        Ok(changes)
    }

    /// Sets inclusion in batch export, with reversible history.
    pub fn set_surface_export_enabled(
        &mut self,
        id: SurfaceId,
        enabled: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        let surface = self.document.surface_mut(id)?;
        let previous = surface.export_enabled;
        if previous == enabled {
            return Ok(ChangeSet::empty());
        }
        surface.export_enabled = enabled;
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceExportEnabledChanged {
            id,
            previous,
            next: enabled,
        });
        Ok(changes)
    }

    /// Adds a layout guide to a surface.
    pub fn create_surface_guide(
        &mut self,
        surface: SurfaceId,
        orientation: crate::GuideOrientation,
        position: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        let id = next_local_id(self.document.surface(surface)?.guides.iter().map(|g| g.id))?;
        self.add_surface_guide(surface, crate::Guide::new(id, orientation, position))
    }

    /// Adds a guide with an explicit identity (restore/import/replay).
    pub fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: crate::surface_metadata::Guide,
    ) -> Result<ChangeSet, PetuniaError> {
        let surf = self.document.surface_mut(surface)?;
        if !guide.position.is_finite() || surf.guides.iter().any(|g| g.id == guide.id) {
            return Err(PetuniaError::invalid_input(
                "invalid or duplicate surface guide",
            ));
        }
        surf.guides.push(guide.clone());
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceGuideAdded { surface, guide });
        Ok(changes)
    }

    /// Removes a layout guide from a surface by local guide ID.
    pub fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let surf = self.document.surface_mut(surface)?;
        let pos = surf
            .guides
            .iter()
            .position(|g| g.id == guide_id)
            .ok_or_else(|| PetuniaError::not_found(format!("guide `{guide_id}` not found")))?;
        let removed = surf.guides.remove(pos);
        let mut changes = ChangeSet::empty();
        changes.push(Change::SurfaceGuideRemoved {
            surface,
            guide: removed,
        });
        Ok(changes)
    }

    /// Moves an object from its current surface to a target surface (10.7).
    pub fn move_object_between_surfaces(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        let current_surface_id = self
            .document
            .find_object_surface(id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?;

        if current_surface_id == target_surface {
            return Ok(ChangeSet::empty());
        }

        let curr_orig = self.document.surface(current_surface_id)?.origin;
        let target_orig = self.document.surface(target_surface)?.origin;

        let curr_surf = self.document.surface_mut(current_surface_id)?;
        let pos = curr_surf
            .objects
            .iter()
            .position(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?;
        let mut obj = curr_surf.objects.remove(pos);

        let mut changes = ChangeSet::empty();
        changes.push(Change::ObjectRemoved {
            surface: current_surface_id,
            object: obj.clone(),
            index: pos,
        });

        if preserve_world_transform {
            if let Some(b) = obj.bounds {
                let dx = curr_orig[0] - target_orig[0];
                let dy = curr_orig[1] - target_orig[1];
                obj.bounds = Some([b[0] + dx, b[1] + dy, b[2], b[3]]);
            }
        }

        let tgt_surf = self.document.surface_mut(target_surface)?;
        tgt_surf.objects.push(obj.clone());
        let added_index = tgt_surf.objects.len() - 1;
        changes.push(Change::ObjectAdded {
            surface: target_surface,
            object: obj,
            index: added_index,
        });

        Ok(changes)
    }

    /// Registers a variable data source in the document (10.11).
    pub fn add_data_source(
        &mut self,
        source: crate::variable_data::DataSourceDefinition,
    ) -> Result<ChangeSet, PetuniaError> {
        if self
            .document
            .data_sources
            .iter()
            .any(|ds| ds.id == source.id)
        {
            return Err(PetuniaError::invalid_input(format!(
                "data source `{}` already exists",
                source.id
            )));
        }
        self.document.data_sources.push(source.clone());
        let mut changes = ChangeSet::empty();
        changes.push(Change::DataSourceAdded { source });
        Ok(changes)
    }

    /// Removes a variable data source and its associated bindings (10.11).
    pub fn remove_data_source(
        &mut self,
        id: crate::variable_data::DataSourceId,
    ) -> Result<ChangeSet, PetuniaError> {
        let pos = self
            .document
            .data_sources
            .iter()
            .position(|ds| ds.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("data source `{id}` not found")))?;
        let source = self.document.data_sources.remove(pos);

        let mut changes = ChangeSet::empty();
        // Remove cascading bindings
        let removed_bindings: Vec<crate::variable_data::DataBinding> = self
            .document
            .bindings
            .iter()
            .filter(|b| b.source_id == id)
            .cloned()
            .collect();
        for b in removed_bindings {
            if let Some(b_pos) = self.document.bindings.iter().position(|x| x.id == b.id) {
                self.document.bindings.remove(b_pos);
                changes.push(Change::DataBindingRemoved { binding: b });
            }
        }

        changes.push(Change::DataSourceRemoved { source });
        Ok(changes)
    }

    /// Adds a data binding between a field and a document object property (10.11).
    pub fn add_data_binding(
        &mut self,
        binding: crate::variable_data::DataBinding,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.document.bindings.iter().any(|b| b.id == binding.id) {
            return Err(PetuniaError::invalid_input(format!(
                "data binding `{}` already exists",
                binding.id
            )));
        }
        // Verify source exists
        if !self
            .document
            .data_sources
            .iter()
            .any(|ds| ds.id == binding.source_id)
        {
            return Err(PetuniaError::not_found(format!(
                "data source `{}` not found",
                binding.source_id
            )));
        }
        // Verify target object exists
        if self.document.find_object(binding.target_object).is_none() {
            return Err(PetuniaError::not_found(format!(
                "target object `{}` not found",
                binding.target_object
            )));
        }

        self.document.bindings.push(binding.clone());
        let mut changes = ChangeSet::empty();
        changes.push(Change::DataBindingAdded { binding });
        Ok(changes)
    }

    /// Removes a data binding by ID (10.11).
    pub fn remove_data_binding(
        &mut self,
        id: crate::variable_data::BindingId,
    ) -> Result<ChangeSet, PetuniaError> {
        let pos = self
            .document
            .bindings
            .iter()
            .position(|b| b.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("data binding `{id}` not found")))?;
        let binding = self.document.bindings.remove(pos);
        let mut changes = ChangeSet::empty();
        changes.push(Change::DataBindingRemoved { binding });
        Ok(changes)
    }

    /// Force-sets an object's parent without validation or change records (A5).
    /// Reserved for history redo (`Replayer`), which re-applies an already
    /// validated `ChangeSet`; all other writers use `reparent_object`.
    pub fn force_parent(
        &mut self,
        id: ObjectId,
        parent: Option<ObjectId>,
    ) -> Result<(), PetuniaError> {
        let found = self
            .document
            .surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        found.parent = parent;
        Ok(())
    }

    /// Force-sets a container's children list (A5). See [`Self::force_parent`].
    pub fn force_children(
        &mut self,
        id: ObjectId,
        children: Vec<ObjectId>,
    ) -> Result<(), PetuniaError> {
        let found = self
            .document
            .surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        found.children = children;
        Ok(())
    }

    /// Force-sets a container's structural role (A5). See [`Self::force_parent`].
    pub fn force_role(
        &mut self,
        id: ObjectId,
        role: Option<crate::hierarchy::ContainerRole>,
    ) -> Result<(), PetuniaError> {
        let found = self
            .document
            .surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        found.role = role;
        Ok(())
    }

    /// Force-sets an object's clip-mask relationship (A5). See [`Self::force_parent`].
    pub fn force_clip_mask(
        &mut self,
        id: ObjectId,
        mask: Option<ObjectId>,
        is_mask: bool,
    ) -> Result<(), PetuniaError> {
        let found = self
            .document
            .surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` does not exist")))?;
        found.clip_mask_id = mask;
        found.is_clip_mask = is_mask;
        Ok(())
    }

    /// Materializes merged records into separate surfaces (artboards) on the canvas pasteboard (10.11).
    pub fn materialize_data_merge(
        &mut self,
        source_id: crate::variable_data::DataSourceId,
        template_surface_id: SurfaceId,
        id_gen: &mut petunia_design_foundation::IdGenerator,
    ) -> Result<ChangeSet, PetuniaError> {
        let source = self
            .document
            .data_source(source_id)
            .cloned()
            .ok_or_else(|| {
                PetuniaError::not_found(format!("data source `{source_id}` not found"))
            })?;
        let template_surface = self.document.surface(template_surface_id)?.clone();

        let bindings: Vec<crate::variable_data::DataBinding> = self
            .document
            .bindings
            .iter()
            .filter(|b| b.source_id == source_id)
            .cloned()
            .collect();

        let mut created_surfaces = Vec::new();
        let [orig_x, orig_y] = template_surface.origin;
        let [dim_w, _dim_h] = template_surface.dimensions;

        for (idx, record) in source.records.iter().enumerate() {
            let new_surf_id = id_gen.next_surface();
            let mut new_surface = template_surface.clone();
            new_surface.id = new_surf_id;
            new_surface.name = format!("{} [{}]", template_surface.name, record.key);
            // Arrange newly generated artboards horizontally with a 100pt gap
            let offset_x =
                orig_x + (f64::from(u32::try_from(idx + 1).unwrap_or(1))) * (dim_w + 100.0);
            new_surface.origin = [offset_x, orig_y];

            // Re-key objects to preserve uniqueness and apply record bindings
            let mut id_map = std::collections::HashMap::new();
            for obj in &new_surface.objects {
                id_map.insert(obj.id, id_gen.next_object());
            }

            for obj in &mut new_surface.objects {
                let original_id = obj.id;

                // Apply bindings meant for original_id before re-keying
                let obj_bindings: Vec<&crate::variable_data::DataBinding> = bindings
                    .iter()
                    .filter(|b| b.target_object == original_id)
                    .collect();
                crate::variable_data::DataMergeEvaluator::apply_to_object(
                    obj,
                    record,
                    &obj_bindings,
                )?;

                obj.id = id_map[&original_id];
                if let Some(parent_id) = obj.parent {
                    obj.parent = id_map.get(&parent_id).copied();
                }
                obj.children = obj
                    .children
                    .iter()
                    .filter_map(|c| id_map.get(c).copied())
                    .collect();
            }

            self.document.surfaces.push(new_surface.clone());
            created_surfaces.push(new_surface);
        }

        let mut changes = ChangeSet::empty();
        changes.push(Change::BatchSurfacesAdded {
            surfaces: created_surfaces,
        });
        Ok(changes)
    }

    /// Attaches a fully-formed surface (A4/A5).
    /// Reserved for history redo of `BatchSurfacesAdded` and engine-level
    /// restore paths; interactive code uses `add_surface`.
    pub fn attach_surface(&mut self, surface: crate::document::Surface) {
        if let Some(pos) = self
            .document
            .surfaces
            .iter()
            .position(|s| s.id == surface.id)
        {
            self.document.surfaces[pos] = surface;
        } else {
            self.document.surfaces.push(surface);
        }
    }

    /// Reverts a change set in reverse order (undo primitive).
    pub fn revert(&mut self, changes: &ChangeSet) -> Result<(), PetuniaError> {
        for change in changes.changes.iter().rev() {
            match change.clone() {
                Change::SurfaceAdded { id, .. } => {
                    let pos = self
                        .document
                        .surfaces
                        .iter()
                        .position(|s| s.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("surface `{id}` does not exist"))
                        })?;
                    self.document.surfaces.remove(pos);
                }
                Change::ObjectAdded {
                    surface, object, ..
                } => {
                    let target = self.document.surface_mut(surface)?;
                    let pos = target
                        .objects
                        .iter()
                        .position(|o| o.id == object.id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!(
                                "object `{}` does not exist",
                                object.id
                            ))
                        })?;
                    target.objects.remove(pos);
                }
                Change::ObjectRemoved {
                    surface,
                    object,
                    index,
                } => {
                    let target = self.document.surface_mut(surface)?;
                    // Restore at the original z-index (F-09); clamp for safety
                    // when the surface shrank since removal.
                    let dest = index.min(target.objects.len());
                    // Avoid duplicating when redo/undo interleave unexpectedly.
                    if target.objects.iter().any(|o| o.id == object.id) {
                        continue;
                    }
                    target.objects.insert(dest, object);
                }
                Change::NameChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.name = previous;
                }
                Change::FillChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.fill = previous;
                }
                Change::VisibilityChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.visible = previous;
                }
                Change::LockChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.locked = previous;
                }
                Change::OpacityChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.opacity = previous;
                }
                Change::StrokeChanged {
                    id,
                    previous_stroke,
                    previous_width,
                    ..
                } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.stroke = previous_stroke;
                    found.stroke_width = previous_width;
                }
                Change::BoundsChanged {
                    id,
                    previous_bounds,
                    previous_rotation,
                    ..
                } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.bounds = previous_bounds;
                    found.rotation = previous_rotation;
                }
                Change::TextStyleChanged { id, previous, .. } => {
                    self.set_text_style(id, previous)?;
                }
                Change::ShapeChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.shape = previous;
                }
                Change::ObjectReordered {
                    surface,
                    id,
                    previous_index,
                    ..
                } => {
                    let target = self.document.surface_mut(surface)?;
                    let current_pos =
                        target
                            .objects
                            .iter()
                            .position(|o| o.id == id)
                            .ok_or_else(|| {
                                PetuniaError::not_found(format!("object `{id}` does not exist"))
                            })?;
                    let item = target.objects.remove(current_pos);
                    let dest = previous_index.min(target.objects.len());
                    target.objects.insert(dest, item);
                }
                Change::AppearanceChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.appearance = previous.clone();
                    // Re-mirror legacy tokens so post-undo state matches
                    // pre-change state even for old changesets that lack the
                    // explicit Fill/Stroke/Opacity entries (F-06).
                    Self::sync_legacy_from_stack(found, previous.as_ref());
                    // When previous is None (no stack), legacy mirrors keep
                    // whatever Fill/Stroke/Opacity reverts already restored
                    // via their own Change entries (reverse order).
                }
                Change::ModifiersChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.modifiers = previous.clone();
                }
                Change::Reparented {
                    id,
                    previous_parent,
                    ..
                } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.parent = previous_parent;
                }
                Change::ChildrenChanged {
                    id,
                    previous_children,
                    ..
                } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.children = previous_children;
                }
                Change::ContainerRoleChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.role = previous;
                }
                Change::MaskModeChanged { id, previous, .. } => {
                    self.set_mask_mode(id, previous)?;
                }
                Change::ClipMaskChanged {
                    id,
                    previous_mask,
                    previous_is_mask,
                    ..
                } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            PetuniaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.clip_mask_id = previous_mask;
                    found.is_clip_mask = previous_is_mask;
                }
                Change::SurfaceGeometryChanged {
                    id,
                    previous_origin,
                    previous_dimensions,
                    ..
                } => {
                    let surf = self.document.surface_mut(id)?;
                    surf.origin = previous_origin;
                    surf.dimensions = previous_dimensions;
                }
                Change::SurfaceBleedChanged { id, previous, .. } => {
                    let surf = self.document.surface_mut(id)?;
                    surf.bleed = previous;
                }
                Change::SurfaceMarginsChanged { id, previous, .. } => {
                    let surf = self.document.surface_mut(id)?;
                    surf.margins = previous;
                }
                Change::SurfaceBackgroundChanged { id, previous, .. } => {
                    let surf = self.document.surface_mut(id)?;
                    surf.background = previous;
                }
                Change::SurfaceCmykProfileChanged { id, previous, .. } => {
                    self.set_surface_cmyk_profile(id, previous)?;
                }
                Change::SurfaceExportEnabledChanged { id, previous, .. } => {
                    self.document.surface_mut(id)?.export_enabled = previous;
                }
                Change::SurfaceGuideAdded { surface, guide } => {
                    let surf = self.document.surface_mut(surface)?;
                    if let Some(pos) = surf.guides.iter().position(|g| g.id == guide.id) {
                        surf.guides.remove(pos);
                    }
                }
                Change::SurfaceGuideRemoved { surface, guide } => {
                    let surf = self.document.surface_mut(surface)?;
                    surf.guides.push(guide);
                }
                Change::DataSourceAdded { source } => {
                    if let Some(pos) = self
                        .document
                        .data_sources
                        .iter()
                        .position(|ds| ds.id == source.id)
                    {
                        self.document.data_sources.remove(pos);
                    }
                }
                Change::DataSourceRemoved { source } => {
                    self.document.data_sources.push(source);
                }
                Change::DataBindingAdded { binding } => {
                    if let Some(pos) = self
                        .document
                        .bindings
                        .iter()
                        .position(|b| b.id == binding.id)
                    {
                        self.document.bindings.remove(pos);
                    }
                }
                Change::DataBindingRemoved { binding } => {
                    self.document.bindings.push(binding);
                }
                Change::BatchSurfacesAdded { surfaces } => {
                    for surf in surfaces {
                        if let Some(pos) =
                            self.document.surfaces.iter().position(|s| s.id == surf.id)
                        {
                            self.document.surfaces.remove(pos);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_foundation::IdGenerator;

    #[test]
    fn add_and_undo_object_roundtrip() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        {
            let mut mutator = DocumentMutator::new(&mut doc);
            mutator.add_surface(surface, "Page").expect("surface");
            let object = DocumentObject::new(gen.next_object(), "Rect");
            let changes = mutator.add_object(surface, object).expect("add");
            assert_eq!(changes.len(), 1);
            mutator.revert(&changes).expect("undo");
        }
        assert!(doc.surface(surface).expect("surface").objects.is_empty());
    }

    #[test]
    fn appearance_roundtrip_undo_redo() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let obj_id = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Canvas").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(obj_id, "Box"))
            .unwrap();

        let app = crate::appearance::AppearanceStack::new()
            .with_fill("ptnd.purple/500")
            .with_stroke("ptnd.gray/900", 2.0);

        let changes = mutator.set_appearance(obj_id, Some(app)).unwrap();
        // F-06: one atomic set emits mirrored Fill/Stroke entries plus the
        // stack entry so undo reverts both together.
        assert_eq!(changes.len(), 3);

        let obj = mutator.document.find_object(obj_id).unwrap();
        assert!(obj.appearance.is_some());
        assert_eq!(obj.fill.as_deref(), Some("ptnd.purple/500"));
        assert_eq!(obj.stroke.as_deref(), Some("ptnd.gray/900"));

        mutator.revert(&changes).unwrap();
        let obj_undone = mutator.document.find_object(obj_id).unwrap();
        assert!(obj_undone.appearance.is_none());
        assert!(obj_undone.fill.is_none());
        assert!(obj_undone.stroke.is_none());
    }

    #[test]
    fn group_and_ungroup_roundtrip_undo_redo() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let c1 = gen.next_object();
        let c2 = gen.next_object();
        let grp = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Canvas").unwrap();

        let mut o1 = DocumentObject::new(c1, "Item1");
        o1.bounds = Some([10.0, 10.0, 50.0, 50.0]);
        let mut o2 = DocumentObject::new(c2, "Item2");
        o2.bounds = Some([70.0, 70.0, 40.0, 40.0]);

        mutator.add_object(surface, o1).unwrap();
        mutator.add_object(surface, o2).unwrap();

        // 1. Group objects
        let grp_changes = mutator
            .group_objects(
                surface,
                grp,
                vec![c1, c2],
                crate::hierarchy::ContainerRole::Group,
            )
            .unwrap();

        let group_obj = mutator.document.find_object(grp).unwrap();
        assert_eq!(group_obj.children, vec![c1, c2]);
        assert_eq!(group_obj.role, Some(crate::hierarchy::ContainerRole::Group));
        assert_eq!(group_obj.bounds, Some([10.0, 10.0, 100.0, 100.0]));

        let child1 = mutator.document.find_object(c1).unwrap();
        assert_eq!(child1.parent, Some(grp));
        // local offset compensated: 10.0 - 10.0 = 0.0
        assert_eq!(child1.bounds, Some([0.0, 0.0, 50.0, 50.0]));

        // 2. Undo grouping
        mutator.revert(&grp_changes).unwrap();
        assert!(mutator.document.find_object(grp).is_none());
        let c1_restored = mutator.document.find_object(c1).unwrap();
        assert_eq!(c1_restored.parent, None);
        assert_eq!(c1_restored.bounds, Some([10.0, 10.0, 50.0, 50.0]));
    }

    #[test]
    fn reparent_cycle_detection_prevents_loops() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let parent = gen.next_object();
        let child = gen.next_object();
        let grandchild = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Canvas").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(parent, "Parent"))
            .unwrap();
        mutator
            .add_object(surface, DocumentObject::new(child, "Child"))
            .unwrap();
        mutator
            .add_object(surface, DocumentObject::new(grandchild, "Grandchild"))
            .unwrap();

        // Build hierarchy: Parent -> Child -> Grandchild
        mutator
            .reparent_object(child, Some(parent), 0, false)
            .unwrap();
        mutator
            .reparent_object(grandchild, Some(child), 0, false)
            .unwrap();

        // Try to reparent Parent under Grandchild (should fail with cycle detection!)
        let err = mutator.reparent_object(parent, Some(grandchild), 0, false);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("cycle detected"));
    }

    #[test]
    fn reparent_world_transform_preservation() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let parent = gen.next_object();
        let item = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Canvas").unwrap();

        let mut p_obj = DocumentObject::new(parent, "Group");
        p_obj.bounds = Some([100.0, 200.0, 300.0, 300.0]);
        mutator.add_object(surface, p_obj).unwrap();

        let mut item_obj = DocumentObject::new(item, "Item");
        item_obj.bounds = Some([150.0, 250.0, 20.0, 20.0]);
        mutator.add_object(surface, item_obj).unwrap();

        // Reparent item under parent with world-transform preservation
        mutator
            .reparent_object(item, Some(parent), 0, true)
            .unwrap();

        // World transform of item should still place it at (150, 250)
        let world = mutator.document.world_transform(item).unwrap();
        let origin = world.apply(petunia_design_geometry::GPoint::ORIGIN);
        assert!((origin.x - 150.0).abs() < 1e-10);
        assert!((origin.y - 250.0).abs() < 1e-10);

        // Local origin under parent should be (50, 50)
        let item_ref = mutator.document.find_object(item).unwrap();
        assert_eq!(item_ref.bounds, Some([50.0, 50.0, 20.0, 20.0]));
    }

    #[test]
    fn create_and_release_clipping_group() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let mask = gen.next_object();
        let content = gen.next_object();
        let clip_grp = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Canvas").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(mask, "CircleMask"))
            .unwrap();
        mutator
            .add_object(surface, DocumentObject::new(content, "Photo"))
            .unwrap();

        mutator
            .create_clip_group(surface, clip_grp, mask, vec![content])
            .unwrap();

        let mask_obj = mutator.document.find_object(mask).unwrap();
        assert!(mask_obj.is_clip_mask);
        let content_obj = mutator.document.find_object(content).unwrap();
        assert_eq!(content_obj.clip_mask_id, Some(mask));

        // Release clipping group
        mutator.release_clip_group(clip_grp).unwrap();
        let mask_released = mutator.document.find_object(mask).unwrap();
        assert!(!mask_released.is_clip_mask);
        let content_released = mutator.document.find_object(content).unwrap();
        assert_eq!(content_released.clip_mask_id, None);
    }

    #[test]
    fn surface_geometry_bleed_margins_and_guides_undo_redo() {
        use crate::surface_metadata::{Bleed, Guide, GuideOrientation, Margins};

        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Artboard 1").unwrap();

        // Check defaults: 800x600, origin 0,0
        let s = mutator.document.surface(surface).unwrap();
        assert_eq!(s.dimensions, [800.0, 600.0]);
        assert_eq!(s.origin, [0.0, 0.0]);
        assert_eq!(s.bleed, Bleed::ZERO);

        // Update geometry
        let c1 = mutator
            .set_surface_geometry(surface, [100.0, 50.0], [1920.0, 1080.0])
            .unwrap();
        let c2 = mutator
            .set_surface_bleed(surface, Bleed::uniform(9.0))
            .unwrap();
        let c3 = mutator
            .set_surface_margins(surface, Margins::uniform(36.0))
            .unwrap();
        let c4 = mutator
            .set_surface_background(surface, Some("ptnd.gray/100".to_string()))
            .unwrap();
        let c5 = mutator
            .add_surface_guide(surface, Guide::new(1, GuideOrientation::Horizontal, 200.0))
            .unwrap();

        let s_updated = mutator.document.surface(surface).unwrap();
        assert_eq!(s_updated.origin, [100.0, 50.0]);
        assert_eq!(s_updated.dimensions, [1920.0, 1080.0]);
        assert_eq!(s_updated.bleed, Bleed::uniform(9.0));
        assert_eq!(s_updated.margins, Margins::uniform(36.0));
        assert_eq!(s_updated.background.as_deref(), Some("ptnd.gray/100"));
        assert_eq!(s_updated.guides.len(), 1);
        assert_eq!(
            s_updated.bleed_bounds(),
            [100.0 - 9.0, 50.0 - 9.0, 1920.0 + 18.0, 1080.0 + 18.0]
        );

        // Revert all
        mutator.revert(&c5).unwrap();
        mutator.revert(&c4).unwrap();
        mutator.revert(&c3).unwrap();
        mutator.revert(&c2).unwrap();
        mutator.revert(&c1).unwrap();

        let s_restored = mutator.document.surface(surface).unwrap();
        assert_eq!(s_restored.origin, [0.0, 0.0]);
        assert_eq!(s_restored.dimensions, [800.0, 600.0]);
        assert_eq!(s_restored.bleed, Bleed::ZERO);
        assert!(s_restored.guides.is_empty());
    }

    #[test]
    fn move_object_between_surfaces_preserves_world_transform() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let s1 = gen.next_surface();
        let s2 = gen.next_surface();
        let obj_id = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(s1, "Page 1").unwrap();
        mutator.add_surface(s2, "Page 2").unwrap();

        // Surface 1 at (0, 0), Surface 2 at (1000, 0)
        mutator
            .set_surface_geometry(s1, [0.0, 0.0], [800.0, 600.0])
            .unwrap();
        mutator
            .set_surface_geometry(s2, [1000.0, 0.0], [800.0, 600.0])
            .unwrap();

        let mut obj = DocumentObject::new(obj_id, "Box");
        // Placed at (1050, 50) on Surface 1 -> in pasteboard coords it is (1050, 50)
        obj.bounds = Some([1050.0, 50.0, 100.0, 100.0]);
        mutator.add_object(s1, obj).unwrap();

        // Move to Surface 2 with world transform preservation
        let changes = mutator
            .move_object_between_surfaces(obj_id, s2, true)
            .unwrap();

        // On Surface 2 (origin 1000, 0), the local coords should be (50, 50)
        let s2_ref = mutator.document.surface(s2).unwrap();
        let moved = s2_ref.objects.iter().find(|o| o.id == obj_id).unwrap();
        assert_eq!(moved.bounds, Some([50.0, 50.0, 100.0, 100.0]));

        // Revert moves object back to Surface 1 at (1050, 50)
        mutator.revert(&changes).unwrap();
        let s1_ref = mutator.document.surface(s1).unwrap();
        let back = s1_ref.objects.iter().find(|o| o.id == obj_id).unwrap();
        assert_eq!(back.bounds, Some([1050.0, 50.0, 100.0, 100.0]));
    }

    #[test]
    fn variable_data_source_and_bindings_undo_redo() {
        use crate::variable_data::*;

        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let obj_id = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Card").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(obj_id, "Title"))
            .unwrap();

        // 1. Add Data Source
        let csv = "Name,Role\nAlice,Manager\nBob,Engineer";
        let ds =
            DataSourceParser::parse_delimited(DataSourceId::new(1), "staff.csv", csv, ',').unwrap();
        let c_ds = mutator.add_data_source(ds).unwrap();
        assert_eq!(mutator.document.data_sources.len(), 1);

        // 2. Add Data Binding
        let binding = DataBinding {
            id: BindingId::new(1),
            source_id: DataSourceId::new(1),
            field_id: FieldId::new(1),
            target_object: obj_id,
            target_property: TargetProperty::TextContent,
            formatter: ValueFormatter::Uppercase,
            missing_policy: MissingValuePolicy::Skip,
        };
        let c_bind = mutator.add_data_binding(binding).unwrap();
        assert_eq!(mutator.document.bindings.len(), 1);

        // 3. Revert binding and source
        mutator.revert(&c_bind).unwrap();
        assert!(mutator.document.bindings.is_empty());
        mutator.revert(&c_ds).unwrap();
        assert!(mutator.document.data_sources.is_empty());
    }

    #[test]
    fn variable_data_materialize_data_merge_roundtrip() {
        use crate::variable_data::*;

        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let obj_id = gen.next_object();

        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Badge").unwrap();
        let mut obj = DocumentObject::new(obj_id, "Placeholder");
        obj.fill = Some("ptnd.gray/200".to_string());
        mutator.add_object(surface, obj).unwrap();

        let json = r#"[
            {"name": "Alice", "color": "ptnd.red/500"},
            {"name": "Bob", "color": "ptnd.blue/500"}
        ]"#;
        let ds = DataSourceParser::parse_json(DataSourceId::new(10), "badges.json", json).unwrap();
        let field_name_id = ds.schema.field_by_name("name").unwrap().id;
        let field_color_id = ds.schema.field_by_name("color").unwrap().id;
        mutator.add_data_source(ds).unwrap();

        let bind_name = DataBinding {
            id: BindingId::new(10),
            source_id: DataSourceId::new(10),
            field_id: field_name_id,
            target_object: obj_id,
            target_property: TargetProperty::TextContent,
            formatter: ValueFormatter::Prefix("VIP: ".to_string()),
            missing_policy: MissingValuePolicy::Skip,
        };
        let bind_color = DataBinding {
            id: BindingId::new(11),
            source_id: DataSourceId::new(10),
            field_id: field_color_id,
            target_object: obj_id,
            target_property: TargetProperty::FillColor,
            formatter: ValueFormatter::None,
            missing_policy: MissingValuePolicy::Skip,
        };
        mutator.add_data_binding(bind_name).unwrap();
        mutator.add_data_binding(bind_color).unwrap();

        // Materialize data merge (generates 2 new artboards)
        let c_merge = mutator
            .materialize_data_merge(DataSourceId::new(10), surface, &mut gen)
            .unwrap();

        assert_eq!(mutator.document.surfaces.len(), 3); // Template + 2 generated
        let s_alice = &mutator.document.surfaces[1];
        assert_eq!(s_alice.name, "Badge [record-1]");
        let obj_alice = &s_alice.objects[0];
        assert_eq!(obj_alice.name, "VIP: Alice");
        assert_eq!(obj_alice.fill.as_deref(), Some("ptnd.red/500"));

        let s_bob = &mutator.document.surfaces[2];
        assert_eq!(s_bob.name, "Badge [record-2]");
        let obj_bob = &s_bob.objects[0];
        assert_eq!(obj_bob.name, "VIP: Bob");
        assert_eq!(obj_bob.fill.as_deref(), Some("ptnd.blue/500"));

        // Revert materialization: removes generated surfaces
        mutator.revert(&c_merge).unwrap();
        assert_eq!(mutator.document.surfaces.len(), 1);
    }
}
