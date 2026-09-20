//! The only writer path: UI/Shortcut/Plugin/MCP -> Action -> Command ->
//! `DocumentMutator` -> `ChangeSet`. Nothing touches storage directly.

use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

use crate::changeset::{Change, ChangeSet};
use crate::document::Document;
use crate::document_object::DocumentObject;

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

    /// Borrows the underlying document mutably.
    pub fn document_mut(&mut self) -> &mut Document {
        self.document
    }

    /// Adds a surface with an explicit stable ID.
    pub fn add_surface(
        &mut self,
        id: SurfaceId,
        name: impl Into<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        if self.document.surfaces.iter().any(|s| s.id == id) {
            return Err(AubrietaError::invalid_input(format!(
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
        object: DocumentObject,
    ) -> Result<ChangeSet, AubrietaError> {
        if self.document.find_object(object.id).is_some() {
            return Err(AubrietaError::invalid_input(format!(
                "object `{}` already exists",
                object.id
            )));
        }
        let target = self.document.surface_mut(surface)?;
        target.objects.push(object.clone());
        let mut changes = ChangeSet::empty();
        changes.push(Change::ObjectAdded { surface, object });
        Ok(changes)
    }

    /// Removes an object by stable ID, keeping it for undo.
    pub fn remove_object(&mut self, id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        for surface in &mut self.document.surfaces {
            if let Some(pos) = surface.objects.iter().position(|o| o.id == id) {
                let object = surface.objects.remove(pos);
                let mut changes = ChangeSet::empty();
                changes.push(Change::ObjectRemoved {
                    surface: surface.id,
                    object,
                });
                return Ok(changes);
            }
        }
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's semantic fill token.
    pub fn set_fill(
        &mut self,
        id: ObjectId,
        fill: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's visibility flag.
    pub fn set_visibility(
        &mut self,
        id: ObjectId,
        visible: bool,
    ) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's locked flag.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's opacity factor in [0.0, 1.0].
    pub fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's stroke token and width.
    pub fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's bounds and rotation.
    pub fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError> {
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
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's vector shape or text descriptor.
    pub fn set_shape(
        &mut self,
        id: ObjectId,
        shape: Option<crate::ShapeKind>,
    ) -> Result<ChangeSet, AubrietaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.shape.clone();
                object.shape = shape.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::ShapeChanged {
                    id,
                    previous,
                    next: shape,
                });
                return Ok(changes);
            }
        }
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Sets an object's appearance stack.
    pub fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<crate::appearance::AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError> {
        for surface in &mut self.document.surfaces {
            if let Some(object) = surface.objects.iter_mut().find(|o| o.id == id) {
                let previous = object.appearance.clone();
                if let Some(app) = &appearance {
                    if let Some(pf) = app.primary_fill() {
                        if let crate::appearance::Paint::Solid(col) = &pf.paint {
                            object.fill = Some(col.clone());
                        }
                    }
                    if let Some(ps) = app.primary_stroke() {
                        if let crate::appearance::Paint::Solid(col) = &ps.paint {
                            object.stroke = Some(col.clone());
                            object.stroke_width = ps.width;
                        }
                    }
                }
                object.appearance = appearance.clone();
                let mut changes = ChangeSet::empty();
                changes.push(Change::AppearanceChanged {
                    id,
                    previous,
                    next: appearance,
                });
                return Ok(changes);
            }
        }
        Err(AubrietaError::not_found(format!(
            "object `{id}` does not exist"
        )))
    }

    /// Reorders an object within its surface.
    pub fn reorder_object(
        &mut self,
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    ) -> Result<ChangeSet, AubrietaError> {
        let target = self.document.surface_mut(surface)?;
        let current_pos = target
            .objects
            .iter()
            .position(|o| o.id == id)
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` does not exist")))?;
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
    pub fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: crate::hierarchy::ContainerRole,
    ) -> Result<ChangeSet, AubrietaError> {
        if child_ids.is_empty() {
            return Err(AubrietaError::invalid_input("cannot create empty group"));
        }
        if self.document.find_object(group_id).is_some() {
            return Err(AubrietaError::invalid_input(format!(
                "object `{group_id}` already exists"
            )));
        }

        let surf = self.document.surface(surface)?;
        for id in &child_ids {
            if !surf.objects.iter().any(|o| o.id == *id) {
                return Err(AubrietaError::invalid_input(format!(
                    "child `{id}` not found on surface `{surface}`"
                )));
            }
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for id in &child_ids {
            if let Some(obj) = self.document.find_object(*id) {
                if let Some(b) = obj.bounds {
                    min_x = min_x.min(b[0]);
                    min_y = min_y.min(b[1]);
                    max_x = max_x.max(b[0] + b[2]);
                    max_y = max_y.max(b[1] + b[3]);
                }
            }
        }

        let group_bounds = if min_x.is_finite() && min_y.is_finite() {
            Some([
                min_x,
                min_y,
                (max_x - min_x).max(0.0),
                (max_y - min_y).max(0.0),
            ])
        } else {
            None
        };

        let mut group_obj = DocumentObject::new(
            group_id,
            match role {
                crate::hierarchy::ContainerRole::Group => "Group",
                crate::hierarchy::ContainerRole::Layer => "Layer",
                crate::hierarchy::ContainerRole::ClipGroup => "Clip Group",
            },
        );
        group_obj.role = Some(role);
        group_obj.bounds = group_bounds;
        group_obj.children = child_ids.clone();

        let mut changes = ChangeSet::empty();

        let target = self.document.surface_mut(surface)?;
        target.objects.push(group_obj.clone());
        changes.push(Change::ObjectAdded {
            surface,
            object: group_obj,
        });

        let (gx, gy) = group_bounds.map_or((0.0, 0.0), |b| (b[0], b[1]));
        for (idx, child_id) in child_ids.into_iter().enumerate() {
            if let Some(child) = self.document.find_object_mut(child_id) {
                let prev_parent = child.parent;
                let prev_bounds = child.bounds;
                let prev_rot = child.rotation;

                child.parent = Some(group_id);
                if let Some(b) = child.bounds {
                    child.bounds = Some([b[0] - gx, b[1] - gy, b[2], b[3]]);
                    changes.push(Change::BoundsChanged {
                        id: child_id,
                        previous_bounds: prev_bounds,
                        next_bounds: child.bounds,
                        previous_rotation: prev_rot,
                        next_rotation: prev_rot,
                    });
                }
                changes.push(Change::Reparented {
                    id: child_id,
                    previous_parent: prev_parent,
                    next_parent: Some(group_id),
                    previous_index: idx,
                    next_index: idx,
                });
            }
        }

        Ok(changes)
    }

    /// Ungroups a container object, moving its children to its parent container (or root).
    pub fn ungroup_objects(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        let group = self
            .document
            .find_object(group_id)
            .ok_or_else(|| AubrietaError::not_found(format!("group `{group_id}` not found")))?;
        let surface_id = self.document.find_object_surface(group_id).ok_or_else(|| {
            AubrietaError::not_found(format!("surface for `{group_id}` not found"))
        })?;

        let parent_id = group.parent;
        let (gx, gy) = group.bounds.map_or((0.0, 0.0), |b| (b[0], b[1]));
        let children = group.children.clone();

        let mut changes = ChangeSet::empty();

        for (idx, child_id) in children.iter().enumerate() {
            if let Some(child) = self.document.find_object_mut(*child_id) {
                let prev_parent = child.parent;
                let prev_bounds = child.bounds;
                let prev_rot = child.rotation;

                child.parent = parent_id;
                if let Some(b) = child.bounds {
                    child.bounds = Some([b[0] + gx, b[1] + gy, b[2], b[3]]);
                    changes.push(Change::BoundsChanged {
                        id: *child_id,
                        previous_bounds: prev_bounds,
                        next_bounds: child.bounds,
                        previous_rotation: prev_rot,
                        next_rotation: prev_rot,
                    });
                }
                changes.push(Change::Reparented {
                    id: *child_id,
                    previous_parent: prev_parent,
                    next_parent: parent_id,
                    previous_index: idx,
                    next_index: idx,
                });
            }
        }

        if let Some(pid) = parent_id {
            if let Some(parent_obj) = self.document.find_object_mut(pid) {
                let prev_ch = parent_obj.children.clone();
                if let Some(pos) = parent_obj.children.iter().position(|id| *id == group_id) {
                    parent_obj.children.remove(pos);
                    for (offset, cid) in children.iter().enumerate() {
                        parent_obj.children.insert(pos + offset, *cid);
                    }
                } else {
                    parent_obj.children.extend(children.iter().copied());
                }
                changes.push(Change::ChildrenChanged {
                    id: pid,
                    previous_children: prev_ch,
                    next_children: parent_obj.children.clone(),
                });
            }
        }

        let surf = self.document.surface_mut(surface_id)?;
        if let Some(pos) = surf.objects.iter().position(|o| o.id == group_id) {
            let removed_group = surf.objects.remove(pos);
            changes.push(Change::ObjectRemoved {
                surface: surface_id,
                object: removed_group,
            });
        }

        Ok(changes)
    }

    /// Reparents an object to a new container (or root) with cycle validation and transform preservation.
    pub fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        if let Some(np) = new_parent {
            if self.document.is_descendant(np, id) {
                return Err(AubrietaError::invalid_input(format!(
                    "cycle detected: cannot parent object `{id}` under its descendant `{np}`"
                )));
            }
            if self.document.find_object(np).is_none() {
                return Err(AubrietaError::not_found(format!(
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
                None => aubrieta_geometry::GAffine::IDENTITY,
            }
        } else {
            aubrieta_geometry::GAffine::IDENTITY
        };

        let current_parent = self
            .document
            .find_object(id)
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` not found")))?
            .parent;

        if current_parent == new_parent {
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
                previous_index: 0,
                next_index: target_index,
            });

            if let Some(w) = world_tx {
                let prev_bounds = obj.bounds;
                let prev_rot = obj.rotation;

                if let Some(inv_np) = new_parent_world.inverse() {
                    let new_local = inv_np.after(w);
                    let new_origin = new_local.apply(aubrieta_geometry::GPoint::ORIGIN);
                    obj.set_local_origin(new_origin.x, new_origin.y);
                    changes.push(Change::BoundsChanged {
                        id,
                        previous_bounds: prev_bounds,
                        next_bounds: obj.bounds,
                        previous_rotation: prev_rot,
                        next_rotation: prev_rot,
                    });
                }
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
    ) -> Result<ChangeSet, AubrietaError> {
        let mut all_ids = vec![mask_id];
        all_ids.extend(content_ids.iter().copied());

        let mut changes = self.group_objects(
            surface,
            group_id,
            all_ids,
            crate::hierarchy::ContainerRole::ClipGroup,
        )?;

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
    pub fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        let group = self
            .document
            .find_object(group_id)
            .ok_or_else(|| AubrietaError::not_found(format!("group `{group_id}` not found")))?;
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
    ) -> Result<ChangeSet, AubrietaError> {
        let surf = self.document.surface_mut(id)?;
        let prev_orig = surf.origin;
        let prev_dim = surf.dimensions;
        surf.origin = origin;
        surf.dimensions = [dimensions[0].max(1.0), dimensions[1].max(1.0)];
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
    ) -> Result<ChangeSet, AubrietaError> {
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
    ) -> Result<ChangeSet, AubrietaError> {
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
    ) -> Result<ChangeSet, AubrietaError> {
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

    /// Adds a layout guide to a surface.
    pub fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: crate::surface_metadata::Guide,
    ) -> Result<ChangeSet, AubrietaError> {
        let surf = self.document.surface_mut(surface)?;
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
    ) -> Result<ChangeSet, AubrietaError> {
        let surf = self.document.surface_mut(surface)?;
        let pos = surf
            .guides
            .iter()
            .position(|g| g.id == guide_id)
            .ok_or_else(|| AubrietaError::not_found(format!("guide `{guide_id}` not found")))?;
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
    ) -> Result<ChangeSet, AubrietaError> {
        let current_surface_id = self
            .document
            .find_object_surface(id)
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` not found")))?;

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
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` not found")))?;
        let mut obj = curr_surf.objects.remove(pos);

        let mut changes = ChangeSet::empty();
        changes.push(Change::ObjectRemoved {
            surface: current_surface_id,
            object: obj.clone(),
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
        changes.push(Change::ObjectAdded {
            surface: target_surface,
            object: obj,
        });

        Ok(changes)
    }

    /// Registers a variable data source in the document (10.11).
    pub fn add_data_source(
        &mut self,
        source: crate::variable_data::DataSourceDefinition,
    ) -> Result<ChangeSet, AubrietaError> {
        if self
            .document
            .data_sources
            .iter()
            .any(|ds| ds.id == source.id)
        {
            return Err(AubrietaError::invalid_input(format!(
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
    ) -> Result<ChangeSet, AubrietaError> {
        let pos = self
            .document
            .data_sources
            .iter()
            .position(|ds| ds.id == id)
            .ok_or_else(|| AubrietaError::not_found(format!("data source `{id}` not found")))?;
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
    ) -> Result<ChangeSet, AubrietaError> {
        if self.document.bindings.iter().any(|b| b.id == binding.id) {
            return Err(AubrietaError::invalid_input(format!(
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
            return Err(AubrietaError::not_found(format!(
                "data source `{}` not found",
                binding.source_id
            )));
        }
        // Verify target object exists
        if self.document.find_object(binding.target_object).is_none() {
            return Err(AubrietaError::not_found(format!(
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
    ) -> Result<ChangeSet, AubrietaError> {
        let pos = self
            .document
            .bindings
            .iter()
            .position(|b| b.id == id)
            .ok_or_else(|| AubrietaError::not_found(format!("data binding `{id}` not found")))?;
        let binding = self.document.bindings.remove(pos);
        let mut changes = ChangeSet::empty();
        changes.push(Change::DataBindingRemoved { binding });
        Ok(changes)
    }

    /// Materializes merged records into separate surfaces (artboards) on the canvas pasteboard (10.11).
    pub fn materialize_data_merge(
        &mut self,
        source_id: crate::variable_data::DataSourceId,
        template_surface_id: SurfaceId,
        id_gen: &mut aubrieta_foundation::IdGenerator,
    ) -> Result<ChangeSet, AubrietaError> {
        let source = self
            .document
            .data_source(source_id)
            .cloned()
            .ok_or_else(|| {
                AubrietaError::not_found(format!("data source `{source_id}` not found"))
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

    /// Reverts a change set in reverse order (undo primitive).
    pub fn revert(&mut self, changes: &ChangeSet) -> Result<(), AubrietaError> {
        for change in changes.changes.iter().rev() {
            match change.clone() {
                Change::SurfaceAdded { id, .. } => {
                    let pos = self
                        .document
                        .surfaces
                        .iter()
                        .position(|s| s.id == id)
                        .ok_or_else(|| {
                            AubrietaError::not_found(format!("surface `{id}` does not exist"))
                        })?;
                    self.document.surfaces.remove(pos);
                }
                Change::ObjectAdded { surface, object } => {
                    let target = self.document.surface_mut(surface)?;
                    let pos = target
                        .objects
                        .iter()
                        .position(|o| o.id == object.id)
                        .ok_or_else(|| {
                            AubrietaError::not_found(format!(
                                "object `{}` does not exist",
                                object.id
                            ))
                        })?;
                    target.objects.remove(pos);
                }
                Change::ObjectRemoved { surface, object } => {
                    let target = self.document.surface_mut(surface)?;
                    target.objects.push(object);
                }
                Change::FillChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.bounds = previous_bounds;
                    found.rotation = previous_rotation;
                }
                Change::ShapeChanged { id, previous, .. } => {
                    let found = self
                        .document
                        .surfaces
                        .iter_mut()
                        .flat_map(|s| s.objects.iter_mut())
                        .find(|o| o.id == id)
                        .ok_or_else(|| {
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                                AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.appearance = previous;
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
                        })?;
                    found.role = previous;
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
                            AubrietaError::not_found(format!("object `{id}` does not exist"))
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
    use aubrieta_foundation::IdGenerator;

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
            .with_fill("aubrieta.purple/500")
            .with_stroke("aubrieta.gray/900", 2.0);

        let changes = mutator.set_appearance(obj_id, Some(app)).unwrap();
        assert_eq!(changes.len(), 1);

        let obj = mutator.document.find_object(obj_id).unwrap();
        assert!(obj.appearance.is_some());
        assert_eq!(obj.fill.as_deref(), Some("aubrieta.purple/500"));
        assert_eq!(obj.stroke.as_deref(), Some("aubrieta.gray/900"));

        mutator.revert(&changes).unwrap();
        let obj_undone = mutator.document.find_object(obj_id).unwrap();
        assert!(obj_undone.appearance.is_none());
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
        let origin = world.apply(aubrieta_geometry::GPoint::ORIGIN);
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
            .set_surface_background(surface, Some("aubrieta.gray/100".to_string()))
            .unwrap();
        let c5 = mutator
            .add_surface_guide(surface, Guide::new(1, GuideOrientation::Horizontal, 200.0))
            .unwrap();

        let s_updated = mutator.document.surface(surface).unwrap();
        assert_eq!(s_updated.origin, [100.0, 50.0]);
        assert_eq!(s_updated.dimensions, [1920.0, 1080.0]);
        assert_eq!(s_updated.bleed, Bleed::uniform(9.0));
        assert_eq!(s_updated.margins, Margins::uniform(36.0));
        assert_eq!(s_updated.background.as_deref(), Some("aubrieta.gray/100"));
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
        obj.fill = Some("aubrieta.gray/200".to_string());
        mutator.add_object(surface, obj).unwrap();

        let json = r#"[
            {"name": "Alice", "color": "aubrieta.red/500"},
            {"name": "Bob", "color": "aubrieta.blue/500"}
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
        assert_eq!(obj_alice.fill.as_deref(), Some("aubrieta.red/500"));

        let s_bob = &mutator.document.surfaces[2];
        assert_eq!(s_bob.name, "Badge [record-2]");
        let obj_bob = &s_bob.objects[0];
        assert_eq!(obj_bob.name, "VIP: Bob");
        assert_eq!(obj_bob.fill.as_deref(), Some("aubrieta.blue/500"));

        // Revert materialization: removes generated surfaces
        mutator.revert(&c_merge).unwrap();
        assert_eq!(mutator.document.surfaces.len(), 1);
    }
}
