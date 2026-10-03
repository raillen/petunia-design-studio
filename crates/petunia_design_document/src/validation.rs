//! Integrity checks at persistence and transaction boundaries. All graph walks
//! are iterative and use stable IDs, including on malformed input.

use std::collections::{HashMap, HashSet};

use petunia_design_foundation::PetuniaError;

use crate::{Document, DocumentObject, ModifierKind, ShapeKind};

fn invalid(message: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(message)
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn range(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

fn levels_valid(levels: &crate::ChannelLevels) -> bool {
    unit(levels.input_black)
        && unit(levels.input_white)
        && levels.input_black < levels.input_white
        && unit(levels.output_black)
        && unit(levels.output_white)
        && range(levels.gamma, 0.1, 10.0)
}

fn curve_valid(points: &[[f64; 2]]) -> bool {
    // Empty/one-point curves retain the existing identity/constant behavior.
    // Y may decrease: an inverted transfer curve is a valid user choice.
    points.iter().flatten().all(|&v| unit(v)) && points.windows(2).all(|w| w[0][0] < w[1][0])
}

fn adjustment_valid(kind: &crate::AdjustmentKind) -> bool {
    use crate::AdjustmentKind;
    match kind {
        AdjustmentKind::Levels {
            master,
            red,
            green,
            blue,
        } => {
            levels_valid(master)
                && [red, green, blue]
                    .into_iter()
                    .all(|v| v.as_ref().is_none_or(levels_valid))
        }
        AdjustmentKind::Curves {
            master_points,
            red_points,
            green_points,
            blue_points,
        } => {
            curve_valid(master_points)
                && [red_points, green_points, blue_points]
                    .into_iter()
                    .all(|v| v.as_ref().is_none_or(|points| curve_valid(points)))
        }
        AdjustmentKind::Hsl {
            hue_shift,
            saturation,
            lightness,
        } => {
            range(*hue_shift, -180.0, 180.0)
                && range(*saturation, -1.0, 1.0)
                && range(*lightness, -1.0, 1.0)
        }
        AdjustmentKind::Exposure {
            exposure,
            offset,
            gamma,
        } => range(*exposure, -5.0, 5.0) && range(*offset, -0.5, 0.5) && range(*gamma, 0.1, 5.0),
        AdjustmentKind::WhiteBalance { temperature, tint } => {
            range(*temperature, -1.0, 1.0) && range(*tint, -1.0, 1.0)
        }
    }
}

fn effect_valid(kind: &crate::EffectKind) -> bool {
    use crate::EffectKind;
    match kind {
        EffectKind::DropShadow {
            offset,
            blur,
            opacity,
            ..
        }
        | EffectKind::InnerShadow {
            offset,
            blur,
            opacity,
            ..
        } => {
            offset.iter().all(|v| v.is_finite())
                && blur.is_finite()
                && *blur >= 0.0
                && unit(*opacity)
        }
        EffectKind::GaussianBlur { radius } => radius.is_finite() && *radius >= 0.0,
        EffectKind::Sharpen { radius, amount } => {
            radius.is_finite() && *radius >= 0.0 && range(*amount, 0.0, 5.0)
        }
        EffectKind::Noise { amount, .. } => unit(*amount),
    }
}

fn unique_local_ids(mut ids: impl Iterator<Item = u32>) -> bool {
    let mut seen = HashSet::new();
    ids.all(|id| seen.insert(id))
}

fn validate_live_chain(appearance: &crate::AppearanceStack) -> Result<(), PetuniaError> {
    if !unique_local_ids(appearance.fills.iter().map(|e| e.id))
        || !unique_local_ids(appearance.strokes.iter().map(|e| e.id))
        || !unique_local_ids(appearance.effects.iter().map(|e| e.id))
        || !unique_local_ids(appearance.adjustments.iter().map(|e| e.id))
    {
        return Err(invalid("duplicate local appearance entry ID"));
    }
    if appearance.effects.iter().any(|e| !effect_valid(&e.kind))
        || appearance
            .adjustments
            .iter()
            .any(|a| !unit(a.opacity) || !adjustment_valid(&a.kind))
    {
        return Err(invalid("invalid live effect or adjustment parameters"));
    }
    Ok(())
}

pub(crate) fn validate_appearance(appearance: &crate::AppearanceStack) -> Result<(), PetuniaError> {
    validate_live_chain(appearance)?;
    if !unit(appearance.opacity)
        || appearance.fills.iter().any(|f| !unit(f.opacity))
        || appearance.strokes.iter().any(|s| {
            !s.width.is_finite()
                || s.width < 0.0
                || !unit(s.opacity)
                || !s.miter_limit.is_finite()
                || s.miter_limit < 0.0
                || !s.dash_offset.is_finite()
                || s.dash_array.iter().any(|v| !v.is_finite() || *v < 0.0)
        })
    {
        return Err(invalid(
            "appearance has invalid opacity or stroke parameters",
        ));
    }
    for paint in appearance
        .fills
        .iter()
        .map(|f| &f.paint)
        .chain(appearance.strokes.iter().map(|s| &s.paint))
    {
        let stops = match paint {
            crate::Paint::LinearGradient(g) => {
                if !g.start.iter().chain(&g.end).all(|v| v.is_finite()) {
                    return Err(invalid("invalid gradient endpoints"));
                }
                &g.stops
            }
            crate::Paint::RadialGradient(g) => {
                if !g.center.iter().all(|v| v.is_finite())
                    || !g.radius.is_finite()
                    || g.radius <= 0.0
                {
                    return Err(invalid("invalid gradient radius"));
                }
                &g.stops
            }
            _ => continue,
        };
        if stops.iter().any(|s| !unit(s.offset) || !unit(s.opacity)) {
            return Err(invalid("invalid gradient stop"));
        }
    }
    Ok(())
}

pub(crate) fn validate_modifiers(
    modifiers: &[crate::ModifierItem],
    bounds: Option<[f64; 4]>,
) -> Result<(), PetuniaError> {
    let mut modifier_ids = HashSet::new();
    for modifier in modifiers {
        let crate::ModifierSpace::Local { reference_size } = modifier.space else {
            return Err(invalid("modifier must be normalized before publication"));
        };
        if bounds.is_none() || !reference_size.iter().all(|v| v.is_finite() && *v > 0.0) {
            return Err(invalid(
                "modifier requires a finite positive local reference frame",
            ));
        }
        if !modifier_ids.insert(modifier.id) {
            return Err(invalid("duplicate modifier ID"));
        }
        let valid = match &modifier.kind {
            ModifierKind::ContourOffset { distance, .. } => distance.is_finite(),
            ModifierKind::Perspective { quad } => quad.iter().flatten().all(|v| v.is_finite()),
            ModifierKind::CropRect { rect } => {
                rect.iter().all(|v| v.is_finite()) && rect[2] > 0.0 && rect[3] > 0.0
            }
            ModifierKind::TransparentGradient { start, end, stops } => {
                start.iter().chain(end).all(|v| v.is_finite())
                    && stops.iter().all(|s| unit(s.offset) && unit(s.opacity))
            }
        };
        if !valid {
            return Err(invalid("modifier has invalid parameters"));
        }
    }
    Ok(())
}

pub(crate) fn validate_object(object: &DocumentObject) -> Result<(), PetuniaError> {
    object.text_style.validate()?;
    if !object.rotation.is_finite()
        || !unit(object.opacity)
        || !object.stroke_width.is_finite()
        || object.stroke_width < 0.0
    {
        return Err(invalid(format!(
            "object `{}` has invalid placement or appearance",
            object.id
        )));
    }
    if let Some(bounds) = object.bounds {
        if !bounds.iter().all(|v| v.is_finite()) || bounds[2] <= 0.0 || bounds[3] <= 0.0 {
            return Err(invalid(format!(
                "object `{}` has invalid bounds",
                object.id
            )));
        }
    }
    match &object.shape {
        Some(ShapeKind::Raster { layer }) => layer.validate()?,
        Some(ShapeKind::Path(_)) => {
            return Err(invalid(
                "parent-space path must be normalized before publication",
            ));
        }
        Some(ShapeKind::LocalPath {
            path,
            reference_size,
        }) => {
            if !path.is_finite() || !reference_size.iter().all(|v| v.is_finite() && *v > 0.0) {
                return Err(invalid("local path has invalid points or reference size"));
            }
        }
        Some(ShapeKind::Rectangle { corner_radii }) => {
            if !corner_radii.iter().all(|v| v.is_finite() && *v >= 0.0) {
                return Err(invalid("invalid corner radii"));
            }
        }
        Some(ShapeKind::Polygon { sides }) if !(3..=100_000).contains(sides) => {
            return Err(invalid("invalid polygon side count"))
        }
        Some(ShapeKind::Star {
            points,
            inner_ratio,
        }) if !(2..=100_000).contains(points) || !unit(*inner_ratio) => {
            return Err(invalid("invalid star parameters"))
        }
        Some(ShapeKind::Text {
            font_size,
            line_height,
            letter_spacing,
            on_path,
            ..
        }) => {
            if !font_size.is_finite()
                || *font_size <= 0.0
                || !line_height.is_finite()
                || *line_height <= 0.0
                || !letter_spacing.is_finite()
            {
                return Err(invalid("invalid text metrics"));
            }
            if on_path
                .as_ref()
                .is_some_and(|a| !unit(a.start) || !unit(a.end) || a.start >= a.end)
            {
                return Err(invalid("invalid text-on-path span"));
            }
        }
        _ => {}
    }
    if object.shape.is_some() && object.bounds.is_none() {
        return Err(invalid("shape requires a placement frame"));
    }
    validate_modifiers(&object.modifiers, object.bounds)?;
    if let Some(appearance) = &object.appearance {
        validate_appearance(appearance)?;
    }
    Ok(())
}

/// Canonical resource ceiling; render derivatives/history/toolkit allocations
/// have separate budgets. Shared buffers count once, descriptor maps count too.
pub const MAX_DOCUMENT_RESOURCE_BYTES: usize = 256 * 1024 * 1024;
fn validate_resource_budget(document: &Document) -> Result<(), PetuniaError> {
    if document.surfaces().len() > 1024 {
        return Err(invalid("document surface budget exceeded"));
    }
    let mut objects = 0usize;
    let mut bytes = 0usize;
    let mut pixels = HashSet::new();
    let mut layers = HashSet::new();
    let mut images = HashSet::new();
    let mut paths = HashSet::new();
    let mut profiles = HashSet::new();
    for surface in document.surfaces() {
        if let Some(profile) = &surface.cmyk_profile {
            if !profile.is_press_profile() {
                return Err(invalid(
                    "surface ICC press profile must be a CMYK output profile",
                ));
            }
            if profiles.insert(profile.id()) {
                bytes = bytes.saturating_add(profile.bytes().len());
            }
        }
        bytes = bytes.saturating_add(surface.name.capacity());
        if bytes > MAX_DOCUMENT_RESOURCE_BYTES {
            return Err(invalid("surface label budget exceeded"));
        }
        for object in surface.objects() {
            objects += 1;
            if objects > 100_000 {
                return Err(invalid("document object budget exceeded"));
            }
            bytes = bytes
                .saturating_add(std::mem::size_of::<DocumentObject>())
                .saturating_add(object.name.capacity());
            match &object.shape {
                Some(ShapeKind::Raster { layer }) => {
                    if let Some(profile) = layer.cmyk_profile() {
                        if profiles.insert(profile.id()) {
                            bytes = bytes.saturating_add(profile.bytes().len());
                        }
                    }
                    if layers.insert(std::sync::Arc::as_ptr(layer)) {
                        bytes = bytes.saturating_add(layer.tiles().resident_tile_count() * 96);
                        for (_, tile) in layer.tiles().tiles() {
                            if pixels.insert(std::sync::Arc::as_ptr(&tile.data)) {
                                bytes = bytes.saturating_add(tile.data.capacity());
                            }
                        }
                    }
                }
                Some(ShapeKind::Image { path, data }) => {
                    bytes = bytes.saturating_add(path.capacity());
                    if let Some(image) = data {
                        if images.insert(std::sync::Arc::as_ptr(image)) {
                            bytes = bytes.saturating_add(image.resident_bytes());
                        }
                    }
                }
                Some(ShapeKind::LocalPath { path, .. }) => {
                    if path.verbs.len() > 1_000_000 {
                        return Err(invalid("source path verb budget exceeded"));
                    }
                    if paths.insert(std::sync::Arc::as_ptr(path)) {
                        bytes = bytes.saturating_add(path.verbs.capacity().saturating_mul(
                            std::mem::size_of::<petunia_design_geometry::PathVerb>(),
                        ));
                    }
                }
                Some(ShapeKind::Text {
                    content,
                    font_family,
                    ..
                }) => {
                    if content.len() > 64 * 1024 || font_family.len() > 1024 {
                        return Err(invalid("text source budget exceeded"));
                    }
                    bytes = bytes
                        .saturating_add(content.capacity())
                        .saturating_add(font_family.capacity());
                }
                _ => {}
            }
            if bytes > MAX_DOCUMENT_RESOURCE_BYTES {
                return Err(invalid("canonical document resource byte budget exceeded"));
            }
        }
    }
    Ok(())
}

impl Document {
    /// Checks IDs, frames and reciprocal ownership, references and cycles.
    /// Loading a project never repairs or silently drops malformed artwork.
    pub fn validate(&self) -> Result<(), PetuniaError> {
        if self.schema_version != petunia_design_foundation::NATIVE_SCHEMA_VERSION {
            return Err(invalid(
                "document schema must be migrated before publication",
            ));
        }
        validate_resource_budget(self)?;
        let mut surface_ids = HashSet::new();
        let mut objects = HashMap::new();
        for surface in &self.surfaces {
            if !surface_ids.insert(surface.id) {
                return Err(invalid("duplicate surface ID"));
            }
            if !surface.origin.iter().all(|v| v.is_finite())
                || !surface.dimensions.iter().all(|v| v.is_finite() && *v > 0.0)
            {
                return Err(invalid("surface has invalid geometry"));
            }
            let b = surface.bleed;
            let m = surface.margins;
            if ![
                b.top, b.right, b.bottom, b.left, m.top, m.right, m.bottom, m.left,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
            {
                return Err(invalid("surface has invalid bleed or margins"));
            }
            let mut guides = HashSet::new();
            if surface
                .guides
                .iter()
                .any(|g| !guides.insert(g.id) || !g.position.is_finite())
            {
                return Err(invalid("invalid guide ID or position"));
            }
            for object in &surface.objects {
                validate_object(object)?;
                if objects.insert(object.id, (surface.id, object)).is_some() {
                    return Err(invalid("duplicate object ID"));
                }
            }
        }
        for (&id, &(surface, object)) in &objects {
            let same_surface = |target| {
                objects
                    .get(&target)
                    .filter(|(s, _)| *s == surface)
                    .map(|(_, o)| *o)
                    .ok_or_else(|| {
                        invalid(format!(
                            "object `{id}` has a missing or cross-surface reference"
                        ))
                    })
            };
            if let Some(parent) = object.parent {
                let parent = same_surface(parent)?;
                if !parent.children.contains(&id) {
                    return Err(invalid("parent and child references disagree"));
                }
            }
            let mut children = HashSet::new();
            for child in &object.children {
                if !children.insert(*child) || same_surface(*child)?.parent != Some(id) {
                    return Err(invalid("duplicate or nonreciprocal child reference"));
                }
            }
            if let Some(mask) = object.clip_mask_id {
                if mask == id || !same_surface(mask)?.is_clip_mask {
                    return Err(invalid("invalid clipping mask reference"));
                }
            }
            if let Some(ShapeKind::Text {
                on_path: Some(attachment),
                ..
            }) = &object.shape
            {
                if attachment.target == id
                    || !same_surface(attachment.target)?
                        .shape
                        .as_ref()
                        .is_some_and(ShapeKind::is_path)
                {
                    return Err(invalid(
                        "text attachment requires a different path on the same surface",
                    ));
                }
            }
        }
        // Each node is visited at most twice; no recursion or quadratic ancestor scan.
        let mut complete = HashSet::new();
        for &start in objects.keys() {
            let mut visiting = HashSet::new();
            let mut current = Some(start);
            while let Some(id) = current {
                if complete.contains(&id) {
                    break;
                }
                if !visiting.insert(id) {
                    return Err(invalid("hierarchy cycle"));
                }
                current = objects[&id].1.parent;
            }
            complete.extend(visiting);
        }
        let mut source_ids = HashSet::new();
        for source in &self.data_sources {
            if !source_ids.insert(source.id) {
                return Err(invalid("duplicate data source ID"));
            }
            let mut fields = HashSet::new();
            if source.schema.fields.iter().any(|f| !fields.insert(f.id))
                || source
                    .schema
                    .key_field
                    .is_some_and(|f| !fields.contains(&f))
            {
                return Err(invalid("invalid field IDs"));
            }
        }
        let mut bindings = HashSet::new();
        for binding in &self.bindings {
            let source = self
                .data_sources
                .iter()
                .find(|s| s.id == binding.source_id)
                .ok_or_else(|| invalid("binding source is missing"))?;
            if !bindings.insert(binding.id)
                || !objects.contains_key(&binding.target_object)
                || source.schema.field(binding.field_id).is_none()
            {
                return Err(invalid("invalid binding ID or target"));
            }
        }
        Ok(())
    }
}
