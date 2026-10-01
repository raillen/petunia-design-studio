//! Typed ordered live modifiers: the non-destructive EffectChain (09.31).
//!
//! Doctrine: every transformative edit stores its parameters, never its
//! result. Base geometry (`shape` + `bounds`) stays the editable source;
//! [`DocumentObject::evaluated_path`] folds the chain on read for render,
//! hit-testing, selection, booleans, and export. `Bake` (explicit user
//! operation only) commits the evaluated result back to base.
//!
//! Wire format: `modifiers` is optional (`#[serde(default)]`), so v1 files
//! without the field keep loading unchanged.

use serde::{Deserialize, Serialize};

use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{
    clip_path_to_rect, offset_path, warp_path_to_quad, GAffine, GPath, GPoint, OffsetCap,
    OffsetJoin,
};

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

/// One opacity stop of a transparency gradient.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpacityStop {
    /// Normalized position along the transparency vector in `[0.0, 1.0]`.
    pub offset: f64,
    /// Opacity multiplier at this stop in `[0.0, 1.0]`.
    #[serde(default = "default_one")]
    pub opacity: f64,
}

impl OpacityStop {
    /// Creates a clamped opacity stop.
    #[must_use]
    pub fn new(offset: f64, opacity: f64) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            opacity: opacity.clamp(0.0, 1.0),
        }
    }
}

/// One typed geometry modifier. New kinds extend this enum (never a
/// generic "destructive" operation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModifierKind {
    /// Live contour offset (expand positive, inset negative).
    ContourOffset {
        /// Offset distance in document points.
        distance: f64,
        /// Corner join style.
        #[serde(default)]
        join: OffsetJoin,
        /// End-cap style for open paths.
        #[serde(default)]
        cap: OffsetCap,
    },
    /// Live transparency gradient (second kind, 09.31 revisit check).
    /// Multiplies base opacity along a vector; geometry is untouched.
    TransparentGradient {
        /// Vector start in document points.
        start: [f64; 2],
        /// Vector end in document points.
        end: [f64; 2],
        /// Opacity stops along the vector.
        #[serde(default)]
        stops: Vec<OpacityStop>,
    },
    /// Live 4-corner perspective warp (third kind, 10.8 baseline).
    /// Maps the base outline's bounding-box corners onto `quad`
    /// (`[top-left, top-right, bottom-right, bottom-left]`). Identity quads
    /// preserve curves; genuine warps flatten at 0.25pt (F-21). Envelope
    /// meshes stay future work.
    Perspective {
        /// Target quad corners in document points.
        quad: [[f64; 2]; 4],
    },
    /// Live rectangular crop (nondestructive vector crop, 08.24).
    /// Intersects the outline with `rect` (`[x, y, w, h]`); empty results
    /// produce an empty evaluated outline while preserving the editable source.
    CropRect {
        /// Crop rectangle in document points.
        rect: [f64; 4],
    },
}

/// Parameter frame. Parent is only a legacy/input descriptor; publication
/// stores Local with the placement size at the time the parameters were set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModifierSpace {
    #[default]
    Parent,
    Local {
        reference_size: [f64; 2],
    },
}

/// Single entry in an object's ordered modifier chain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModifierItem {
    /// Local identity within the chain.
    pub id: u32,
    /// The typed modifier definition.
    pub kind: ModifierKind,
    #[serde(default)]
    pub space: ModifierSpace,
    /// Disabled entries are skipped by evaluation (kept for re-enable).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl ModifierItem {
    /// Creates an enabled modifier entry.
    #[must_use]
    pub fn enabled(id: u32, kind: ModifierKind) -> Self {
        Self {
            id,
            kind,
            space: ModifierSpace::Parent,
            enabled: true,
        }
    }

    /// Normalizes an explicit parent-input frame once, including disabled
    /// entries. Moving/rotating/resizing thereafter never rewrites parameters.
    pub fn into_local(mut self, bounds: Option<[f64; 4]>) -> Result<Self, PetuniaError> {
        if self.space == ModifierSpace::Parent {
            let [x, y, w, h] =
                bounds.ok_or_else(|| PetuniaError::invalid_input("modifier requires bounds"))?;
            if ![x, y, w, h].iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 {
                return Err(PetuniaError::invalid_input(
                    "modifier requires finite positive bounds",
                ));
            }
            match &mut self.kind {
                ModifierKind::TransparentGradient { start, end, .. } => {
                    for point in [start, end] {
                        point[0] -= x;
                        point[1] -= y;
                    }
                }
                ModifierKind::Perspective { quad } => {
                    for point in quad {
                        point[0] -= x;
                        point[1] -= y;
                    }
                }
                ModifierKind::CropRect { rect } => {
                    rect[0] -= x;
                    rect[1] -= y;
                }
                ModifierKind::ContourOffset { .. } => {}
            }
            self.space = ModifierSpace::Local {
                reference_size: [w, h],
            };
        }
        Ok(self)
    }

    /// Current local point projection, for overlays and opacity sampling.
    pub fn project_point(&self, point: [f64; 2], size: [f64; 2]) -> Option<GPoint> {
        let ModifierSpace::Local { reference_size } = self.space else {
            return None;
        };
        if !reference_size
            .iter()
            .chain(size.iter())
            .all(|v| v.is_finite() && *v > 0.0)
        {
            return None;
        }
        let p = GPoint::new(
            point[0] * size[0] / reference_size[0],
            point[1] * size[1] / reference_size[1],
        );
        (p.x.is_finite() && p.y.is_finite()).then_some(p)
    }

    /// Explicit baking changes the source origin/size. Rebase the surviving
    /// entries while preserving their previous per-axis scale and appearance.
    pub(crate) fn rebase_local(
        mut self,
        delta: [f64; 2],
        old_size: [f64; 2],
        new_size: [f64; 2],
    ) -> Result<Self, PetuniaError> {
        let ModifierSpace::Local { reference_size } = self.space else {
            return Err(PetuniaError::invalid_input(
                "bake requires a local modifier frame",
            ));
        };
        let scale = [
            old_size[0] / reference_size[0],
            old_size[1] / reference_size[1],
        ];
        let shift = [delta[0] / scale[0], delta[1] / scale[1]];
        let reference_size = [new_size[0] / scale[0], new_size[1] / scale[1]];
        if !reference_size.iter().all(|v| v.is_finite() && *v > 0.0)
            || !shift.iter().all(|v| v.is_finite())
        {
            return Err(PetuniaError::invalid_input("modifier rebase overflow"));
        }
        match &mut self.kind {
            ModifierKind::TransparentGradient { start, end, .. } => {
                for p in [start, end] {
                    p[0] += shift[0];
                    p[1] += shift[1];
                }
            }
            ModifierKind::Perspective { quad } => {
                for p in quad {
                    p[0] += shift[0];
                    p[1] += shift[1];
                }
            }
            ModifierKind::CropRect { rect } => {
                rect[0] += shift[0];
                rect[1] += shift[1];
            }
            ModifierKind::ContourOffset { .. } => {}
        }
        self.space = ModifierSpace::Local { reference_size };
        Ok(self)
    }
}

/// Evaluates geometry in each entry's reference frame, then returns to the
/// current local frame. Nonuniform resize scales the resulting outline,
/// including contour distance in both axes, without inventing one scale.
pub fn evaluate_modifiers_local(
    base: &GPath,
    modifiers: &[ModifierItem],
    size: [f64; 2],
) -> Option<GPath> {
    let mut current = base.clone();
    for item in modifiers.iter().filter(|m| m.enabled) {
        if matches!(item.kind, ModifierKind::TransparentGradient { .. }) {
            continue;
        }
        let ModifierSpace::Local { reference_size } = item.space else {
            return None;
        };
        let sx = size[0] / reference_size[0];
        let sy = size[1] / reference_size[1];
        if ![sx, sy].iter().all(|v| v.is_finite() && *v > 0.0) {
            return None;
        }
        let input = current.transformed(GAffine::scale(1.0 / sx, 1.0 / sy));
        if !input.is_finite() {
            return None;
        }
        // Keep flattening error <= 0.25 current local points.
        let tolerance = 0.25 / sx.max(sy);
        current = apply_geometry_modifier(input, &item.kind, tolerance)
            .transformed(GAffine::scale(sx, sy));
        if !current.is_finite() {
            return None;
        }
    }
    Some(current)
}

fn apply_geometry_modifier(current: GPath, kind: &ModifierKind, tolerance: f64) -> GPath {
    match kind {
        ModifierKind::ContourOffset {
            distance,
            join,
            cap,
        } => offset_path(&current, *distance, *join, *cap).unwrap_or(current),
        ModifierKind::TransparentGradient { .. } => current,
        ModifierKind::Perspective { quad } => {
            warp_path_to_quad(&current, quad.map(|[x, y]| GPoint::new(x, y)), tolerance)
                .unwrap_or(current)
        }
        ModifierKind::CropRect { rect: [x, y, w, h] } => clip_path_to_rect(
            &current,
            petunia_design_geometry::GRect::new(*x, *y, x + w, y + h),
            tolerance,
        )
        .unwrap_or_default(),
    }
}

/// Samples in current local coordinates. No allocation or stop sorting per
/// sample; stored source stop order is preserved for subsequent editing.
pub fn evaluate_opacity_local(
    modifiers: &[ModifierItem],
    point: GPoint,
    size: [f64; 2],
) -> Option<f64> {
    let mut mask = 1.0;
    for item in modifiers.iter().filter(|m| m.enabled) {
        if let ModifierKind::TransparentGradient { start, end, stops } = &item.kind {
            let ModifierSpace::Local { reference_size } = item.space else {
                return None;
            };
            if !reference_size
                .iter()
                .chain(size.iter())
                .all(|v| v.is_finite() && *v > 0.0)
            {
                return None;
            }
            // Pull the sample back, rather than projecting onto a resized
            // vector: Euclidean projection does not commute with anisotropic scale.
            let p = GPoint::new(
                point.x / size[0] * reference_size[0],
                point.y / size[1] * reference_size[1],
            );
            if !p.x.is_finite() || !p.y.is_finite() {
                return None;
            }
            mask *= sample_vector(
                GPoint::new(start[0], start[1]),
                GPoint::new(end[0], end[1]),
                stops,
                p,
            );
        }
    }
    mask.is_finite().then(|| mask.clamp(0.0, 1.0))
}

fn sample_vector(a: GPoint, b: GPoint, stops: &[OpacityStop], point: GPoint) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = dx.hypot(dy);
    if len <= 1e-12 {
        return sample_opacity_stops(stops, 0.0);
    }
    let t = (((point.x - a.x) / len) * (dx / len) + ((point.y - a.y) / len) * (dy / len))
        .clamp(0.0, 1.0);
    sample_opacity_stops(stops, t)
}

/// Folds an ordered modifier chain over a base path.
/// Unknown/disabled entries are skipped; a collapsing step keeps the
/// previous result for invalid warps/offsets; an empty crop remains empty.
#[must_use]
pub fn evaluate_modifiers(base: &GPath, modifiers: &[ModifierItem]) -> GPath {
    let mut current = base.clone();
    for item in modifiers {
        if !item.enabled {
            continue;
        }
        current = apply_geometry_modifier(current, &item.kind, 0.25);
    }
    current
}

/// Samples the transparency mask at a document point: product of every
/// enabled gradient entry evaluated along its vector. No entries → 1.0.
#[must_use]
pub fn evaluate_opacity_at(modifiers: &[ModifierItem], point: GPoint) -> f64 {
    let mut mask = 1.0;
    for item in modifiers {
        if !item.enabled {
            continue;
        }
        if let ModifierKind::TransparentGradient { start, end, stops } = &item.kind {
            let a = GPoint::new(start[0], start[1]);
            let b = GPoint::new(end[0], end[1]);
            mask *= sample_vector(a, b, stops, point);
        }
    }
    mask.clamp(0.0, 1.0)
}

/// Piecewise-linear opacity over sorted stops (empty → opaque).
fn sample_opacity_stops(stops: &[OpacityStop], t: f64) -> f64 {
    if stops.is_empty() {
        return 1.0;
    }
    let mut left: Option<&OpacityStop> = None;
    let mut right: Option<&OpacityStop> = None;
    for stop in stops {
        if stop.offset == t {
            return stop.opacity;
        }
        if stop.offset < t && left.is_none_or(|prev| stop.offset >= prev.offset) {
            left = Some(stop);
        }
        if stop.offset > t && right.is_none_or(|next| stop.offset < next.offset) {
            right = Some(stop);
        }
    }
    match (left, right) {
        (Some(a), Some(b)) => {
            a.opacity + (b.opacity - a.opacity) * ((t - a.offset) / (b.offset - a.offset))
        }
        (Some(a), None) => a.opacity,
        (None, Some(b)) => b.opacity,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> GPath {
        GPath::rect(
            petunia_design_geometry::GRect::new(0.0, 0.0, 100.0, 60.0),
            0.0,
            0.0,
        )
    }

    #[test]
    fn empty_chain_returns_base() {
        assert_eq!(
            evaluate_modifiers(&rect(), &[]).verbs.len(),
            rect().verbs.len()
        );
    }

    #[test]
    fn contour_expand_grows() {
        let out = evaluate_modifiers(
            &rect(),
            &[ModifierItem::enabled(
                1,
                ModifierKind::ContourOffset {
                    distance: 10.0,
                    join: OffsetJoin::Miter,
                    cap: OffsetCap::None,
                },
            )],
        );
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.width() - 120.0).abs() < 1.0, "got {bounds:?}");
    }

    #[test]
    fn contour_inset_shrinks() {
        let out = evaluate_modifiers(
            &rect(),
            &[ModifierItem::enabled(
                1,
                ModifierKind::ContourOffset {
                    distance: -10.0,
                    join: OffsetJoin::Miter,
                    cap: OffsetCap::None,
                },
            )],
        );
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.width() - 80.0).abs() < 1.0, "got {bounds:?}");
    }

    #[test]
    fn disabled_entries_are_skipped() {
        let item = ModifierItem {
            id: 1,
            kind: ModifierKind::ContourOffset {
                distance: 50.0,
                join: OffsetJoin::Round,
                cap: OffsetCap::None,
            },
            enabled: false,
            space: ModifierSpace::Parent,
        };
        assert_eq!(
            evaluate_modifiers(&rect(), &[item]).verbs.len(),
            rect().verbs.len()
        );
    }

    #[test]
    fn collapsing_step_keeps_previous() {
        let out = evaluate_modifiers(
            &rect(),
            &[ModifierItem::enabled(
                1,
                ModifierKind::ContourOffset {
                    distance: -1000.0,
                    join: OffsetJoin::Round,
                    cap: OffsetCap::None,
                },
            )],
        );
        assert_eq!(out.verbs.len(), rect().verbs.len());
    }

    #[test]
    fn chain_applies_in_order() {
        let doubled = ModifierItem::enabled(
            1,
            ModifierKind::ContourOffset {
                distance: 10.0,
                join: OffsetJoin::Miter,
                cap: OffsetCap::None,
            },
        );
        let out = evaluate_modifiers(&rect(), &[doubled.clone(), doubled]);
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.width() - 140.0).abs() < 2.0, "got {bounds:?}");
    }

    #[test]
    fn wire_format_roundtrips_and_defaults_enabled() {
        let item = ModifierItem::enabled(
            7,
            ModifierKind::ContourOffset {
                distance: 5.0,
                join: OffsetJoin::Round,
                cap: OffsetCap::None,
            },
        );
        let back: ModifierItem =
            serde_json::from_value(serde_json::to_value(&item).unwrap()).unwrap();
        assert_eq!(back, item);
        // `enabled` defaults to true when the field is absent (v1 files).
        let legacy: ModifierItem = serde_json::from_value(serde_json::json!({
            "id": 7,
            "kind": { "type": "ContourOffset", "distance": 5.0 },
        }))
        .unwrap();
        assert!(legacy.enabled);
        assert_eq!(legacy, item);
    }

    fn transparency(start: [f64; 2], end: [f64; 2]) -> ModifierItem {
        ModifierItem::enabled(
            1,
            ModifierKind::TransparentGradient {
                start,
                end,
                stops: vec![OpacityStop::new(0.0, 1.0), OpacityStop::new(1.0, 0.0)],
            },
        )
    }

    #[test]
    fn opacity_samples_endpoints_and_midpoint() {
        use petunia_design_geometry::GPoint;
        let chain = vec![transparency([0.0, 0.0], [100.0, 0.0])];
        assert!((evaluate_opacity_at(&chain, GPoint::new(0.0, 0.0)) - 1.0).abs() < 1e-9);
        assert!((evaluate_opacity_at(&chain, GPoint::new(100.0, 0.0)) - 0.0).abs() < 1e-9);
        assert!((evaluate_opacity_at(&chain, GPoint::new(50.0, 0.0)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn opacity_clamps_beyond_vector() {
        use petunia_design_geometry::GPoint;
        let chain = vec![transparency([0.0, 0.0], [100.0, 0.0])];
        assert!((evaluate_opacity_at(&chain, GPoint::new(-50.0, 0.0)) - 1.0).abs() < 1e-9);
        assert!((evaluate_opacity_at(&chain, GPoint::new(500.0, 0.0)) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn opacity_multiplies_stacked_entries() {
        use petunia_design_geometry::GPoint;
        let chain = vec![
            transparency([0.0, 0.0], [100.0, 0.0]),
            transparency([0.0, 0.0], [100.0, 0.0]),
        ];
        assert!((evaluate_opacity_at(&chain, GPoint::new(50.0, 0.0)) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn disabled_entries_do_not_mask() {
        use petunia_design_geometry::GPoint;
        let mut item = transparency([0.0, 0.0], [100.0, 0.0]);
        item.enabled = false;
        assert!((evaluate_opacity_at(&[item], GPoint::new(100.0, 0.0)) - 1.0).abs() < 1e-9);
    }

    fn trapezoid() -> ModifierItem {
        ModifierItem::enabled(
            1,
            ModifierKind::Perspective {
                quad: [[0.0, 0.0], [100.0, 25.0], [100.0, 75.0], [0.0, 100.0]],
            },
        )
    }

    #[test]
    fn perspective_warps_bounds_toward_quad() {
        let out = evaluate_modifiers(&rect(), &[trapezoid()]);
        let bounds = out.bounding_box().expect("bounds");
        // Right edge pinches to half height; left edge spans full height.
        assert!((bounds.x0 - 0.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.width() - 100.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.height() - 100.0).abs() < 30.0, "got {bounds:?}");
    }

    #[test]
    fn crop_rect_clips_to_window() {
        let item = ModifierItem::enabled(
            1,
            ModifierKind::CropRect {
                rect: [25.0, 10.0, 50.0, 40.0],
            },
        );
        let out = evaluate_modifiers(&rect(), &[item]);
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.x0 - 25.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.width() - 50.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.height() - 40.0).abs() < 1.0, "got {bounds:?}");
    }

    #[test]
    fn crop_outside_produces_empty_outline() {
        let item = ModifierItem::enabled(
            1,
            ModifierKind::CropRect {
                rect: [500.0, 500.0, 10.0, 10.0],
            },
        );
        let out = evaluate_modifiers(&rect(), &[item]);
        assert!(out.is_empty());
    }

    #[test]
    fn warp_and_crop_chain_in_order() {
        let crop = ModifierItem::enabled(
            2,
            ModifierKind::CropRect {
                rect: [0.0, 0.0, 60.0, 60.0],
            },
        );
        let out = evaluate_modifiers(&rect(), &[trapezoid(), crop]);
        let bounds = out.bounding_box().expect("bounds");
        assert!(bounds.width() <= 61.0, "got {bounds:?}");
    }
}
