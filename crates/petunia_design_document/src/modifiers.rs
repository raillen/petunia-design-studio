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

use petunia_design_geometry::{offset_path, GPath, OffsetCap, OffsetJoin};

fn default_true() -> bool {
    true
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
        }
    }
    current
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
}
