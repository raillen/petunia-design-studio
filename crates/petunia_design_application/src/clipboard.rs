//! Editable object fragments preserve subtrees and internal mask references.
//! Their source identities are remapped together before one atomic publication.
use petunia_design_document::{Document, DocumentObject, ShapeKind};
use petunia_design_foundation::{IdGenerator, ObjectId, PetuniaError};
use std::collections::{HashMap, HashSet};

fn invalid(reason: &str) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
/// Descendants in reverse depth-first order: deletion cannot accidentally ungroup
/// selected containers or create one history entry per child.
pub(crate) fn deletion_ids(
    document: &Document,
    selected: &[ObjectId],
) -> Result<Vec<ObjectId>, PetuniaError> {
    let objects: HashMap<_, _> = document
        .surfaces()
        .iter()
        .flat_map(|s| s.objects().iter())
        .map(|o| (o.id, o))
        .collect();
    let mut seen = HashSet::new();
    let mut stack = selected.to_vec();
    let mut order = Vec::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let object = objects
            .get(&id)
            .ok_or_else(|| invalid("missing fragment source"))?;
        if seen.len() > 10_000 {
            return Err(invalid("fragment object budget exceeded"));
        }
        order.push(id);
        stack.extend(object.children.iter().copied());
    }
    order.reverse();
    Ok(order)
}
pub(crate) fn capture(
    document: &Document,
    selected: &[ObjectId],
) -> Result<Vec<DocumentObject>, PetuniaError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let objects: HashMap<_, _> = document
        .surfaces()
        .iter()
        .flat_map(|s| s.objects().iter())
        .map(|o| (o.id, o))
        .collect();
    let mut seen = HashSet::new();
    let mut stack = selected.to_vec();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let object = objects
            .get(&id)
            .ok_or_else(|| invalid("missing fragment source"))?;
        if seen.len() > 10_000 {
            return Err(invalid("fragment object budget exceeded"));
        }
        stack.extend(object.children.iter().copied());
        if let Some(mask) = object.clip_mask_id {
            stack.push(mask)
        }
        if let Some(ShapeKind::Text {
            on_path: Some(attachment),
            ..
        }) = &object.shape
        {
            stack.push(attachment.target)
        }
    }
    // Detach only the captured descriptors. Mutating the complete source copy
    // one root at a time would temporarily break a ClipGroup's mask invariant.
    // World frames are admitted as representable TRS before publication.
    document
        .surfaces()
        .iter()
        .flat_map(|surface| surface.objects().iter())
        .filter(|object| seen.contains(&object.id))
        .map(|object| {
            let mut copy = object.clone();
            if object.parent.is_some_and(|parent| !seen.contains(&parent)) {
                let world = document
                    .world_transform_checked(object.id)
                    .map_err(|e| invalid(&e.to_string()))?;
                let [a, b, c, d, _, _] = world.coeffs;
                if (a.hypot(b) - 1.).abs() > 1e-8
                    || (c.hypot(d) - 1.).abs() > 1e-8
                    || (a * c + b * d).abs() > 1e-8
                {
                    return Err(invalid(
                        "clipboard parent transform is not representable by TRS",
                    ));
                }
                let origin = world.apply(petunia_design_geometry::GPoint::ORIGIN);
                if let Some(bounds) = &mut copy.bounds {
                    bounds[0] = origin.x;
                    bounds[1] = origin.y;
                } else if copy.role.is_some() {
                    copy.bounds = Some([origin.x, origin.y, 1., 1.]);
                } else {
                    return Err(invalid("clipboard detached object has no frame"));
                }
                copy.rotation = b.atan2(a);
                copy.parent = None;
            }
            Ok(copy)
        })
        .collect()
}
/// Internal references must resolve inside this fragment; no identity can bind
/// to unrelated artwork which happens to have the same integer in another tab.
pub(crate) fn place(
    objects: &[DocumentObject],
    ids: &mut IdGenerator,
    offset: [f64; 2],
) -> Result<(Vec<DocumentObject>, Vec<ObjectId>), PetuniaError> {
    if objects.len() > 10_000 || !offset.iter().all(|v| v.is_finite()) {
        return Err(invalid("invalid fragment placement"));
    }
    let mut mapping = HashMap::new();
    for object in objects {
        if mapping.insert(object.id, ids.next_object()).is_some() {
            return Err(invalid("duplicate fragment identity"));
        }
    }
    let mapped = |id: ObjectId| {
        mapping
            .get(&id)
            .copied()
            .ok_or_else(|| invalid("fragment contains an external identity"))
    };
    let mut placed = Vec::with_capacity(objects.len());
    let mut selected = Vec::new();
    for object in objects {
        let mut copy = object.clone();
        copy.id = mapped(object.id)?;
        copy.parent = object.parent.map(&mapped).transpose()?;
        copy.children = object
            .children
            .iter()
            .map(|id| mapped(*id))
            .collect::<Result<_, _>>()?;
        copy.clip_mask_id = object.clip_mask_id.map(&mapped).transpose()?;
        if let Some(ShapeKind::Text {
            on_path: Some(attachment),
            ..
        }) = &mut copy.shape
        {
            attachment.target = mapped(attachment.target)?;
        }
        if copy.parent.is_none() {
            if let Some(bounds) = &mut copy.bounds {
                bounds[0] += offset[0];
                bounds[1] += offset[1];
            } else if copy.role.is_some() {
                copy.bounds = Some([offset[0], offset[1], 1., 1.]);
            }
            if !copy.is_clip_mask {
                selected.push(copy.id)
            }
        }
        placed.push(copy);
    }
    Ok((placed, selected))
}
pub(crate) fn cascade(objects: &mut [DocumentObject], offset: [f64; 2]) {
    for object in objects.iter_mut().filter(|o| o.parent.is_none()) {
        if let Some(bounds) = &mut object.bounds {
            bounds[0] += offset[0];
            bounds[1] += offset[1];
        } else if object.role.is_some() {
            object.bounds = Some([offset[0], offset[1], 1., 1.]);
        }
    }
}
