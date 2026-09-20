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
}
