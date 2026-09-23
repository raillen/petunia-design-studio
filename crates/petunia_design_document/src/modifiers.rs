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

use petunia_design_geometry::{offset_path, GPath, GPoint, OffsetCap, OffsetJoin};

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
}

/// Single entry in an object's ordered modifier chain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModifierItem {
    /// Local identity within the chain.
    pub id: u32,
    /// The typed modifier definition.
    pub kind: ModifierKind,
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
            enabled: true,
        }
    }
}

/// Folds an ordered modifier chain over a base path.
/// Unknown/disabled entries are skipped; a collapsing step keeps the
/// previous result instead of destroying it.
#[must_use]
pub fn evaluate_modifiers(base: &GPath, modifiers: &[ModifierItem]) -> GPath {
    let mut current = base.clone();
    for item in modifiers {
        if !item.enabled {
            continue;
        }
        match &item.kind {
            ModifierKind::ContourOffset {
                distance,
                join,
                cap,
            } => {
                if let Some(offset) = offset_path(&current, *distance, *join, *cap) {
                    current = offset;
                }
            }
            // Transparency lives in the opacity domain, not geometry.
            ModifierKind::TransparentGradient { .. } => {}
        }
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
            let abx = b.x - a.x;
            let aby = b.y - a.y;
            let len2 = (abx * abx + aby * aby).max(1e-12);
            let t = (((point.x - a.x) * abx + (point.y - a.y) * aby) / len2).clamp(0.0, 1.0);
            mask *= sample_opacity_stops(stops, t);
        }
    }
    mask.clamp(0.0, 1.0)
}

/// Piecewise-linear opacity over sorted stops (empty → opaque).
fn sample_opacity_stops(stops: &[OpacityStop], t: f64) -> f64 {
    if stops.is_empty() {
        return 1.0;
    }
    let mut order: Vec<usize> = (0..stops.len()).collect();
    order.sort_by(|a, b| {
        stops[*a]
            .offset
            .partial_cmp(&stops[*b].offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if t <= stops[order[0]].offset {
        return stops[order[0]].opacity;
    }
    for w in order.windows(2) {
        let (s0, s1) = (&stops[w[0]], &stops[w[1]]);
        if t <= s1.offset {
            let range = (s1.offset - s0.offset).max(1e-9);
            let f = (t - s0.offset) / range;
            return s0.opacity + (s1.opacity - s0.opacity) * f;
        }
    }
    stops[order[order.len() - 1]].opacity
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
}
