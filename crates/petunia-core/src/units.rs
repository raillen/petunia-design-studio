//! Canonical document units and physical unit conversions.
//!
//! Canonical geometry lives in **document units** (`Unit::Px`, 1:1).
//! Conversions between physical units (`Pt`, `Mm`, `Cm`, `Inch`, `Pica`)
//! are exact ratios anchored on the inch. Conversions involving `Px`
//! require an explicit DPI, since document units only gain a physical
//! meaning through an output density (`1 inch × DPI = output pixels`).

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// A length unit recognized by the document model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    /// Canonical document unit. Geometry is authored and stored in `Px`.
    Px,
    /// Typographic point: 1/72 inch.
    Pt,
    /// Millimeter: 1/25.4 inch.
    Mm,
    /// Centimeter: 1/2.54 inch.
    Cm,
    /// Inch: the physical anchor for all conversions.
    Inch,
    /// Pica: 12 points, 1/6 inch.
    Pica,
}

impl Unit {
    /// Exact number of inches in one of this unit, or `None` for `Px`
    /// which has no physical size without a DPI context.
    #[must_use]
    pub const fn inches_per_unit(self) -> Option<f64> {
        match self {
            Self::Px => None,
            Self::Pt => Some(1.0 / 72.0),
            Self::Mm => Some(1.0 / 25.4),
            Self::Cm => Some(1.0 / 2.54),
            Self::Inch => Some(1.0),
            Self::Pica => Some(1.0 / 6.0),
        }
    }

    /// True when the unit has an exact physical size independent of DPI.
    #[must_use]
    pub const fn is_physical(self) -> bool {
        self.inches_per_unit().is_some()
    }
}

/// A finite scalar value tagged with its unit.
///
/// `value` is canonicalized on construction: `-0.0` becomes `0.0` so
/// semantically equal values share one representation. Non-finite
/// values (NaN, ±Inf) are rejected: they never enter the document.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnitValue {
    pub value: f64,
    pub unit: Unit,
}

impl UnitValue {
    /// Build a value, rejecting non-finite input and canonicalizing `-0.0`.
    pub fn new(value: f64, unit: Unit) -> Result<Self> {
        if !value.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite unit value rejected: {value}"
            )));
        }
        Ok(Self {
            value: value + 0.0,
            unit,
        })
    }

    /// Convert between two physical units (DPI-independent).
    ///
    /// Returns `None` when either side is `Px`; use
    /// [`Self::to_document_units`] / [`Self::from_document_units`] with
    /// an explicit DPI for those cases.
    #[must_use]
    pub fn to_unit(self, target: Unit) -> Option<Self> {
        let from_inches = self.unit.inches_per_unit()?;
        let to_inches = target.inches_per_unit()?;
        // Physical ratios are exact constants; the result stays finite for
        // any finite input, so `new` cannot fail here.
        Self::new(self.value * from_inches / to_inches, target).ok()
    }

    /// Interpret a `Px` value as a physical length at the given DPI.
    ///
    /// Returns `None` when `self` is not `Px` or the DPI is not a
    /// positive finite number.
    #[must_use]
    pub fn to_physical(self, target: Unit, dpi: f64) -> Option<Self> {
        if self.unit != Unit::Px || !dpi.is_finite() || dpi <= 0.0 {
            return None;
        }
        let to_inches = target.inches_per_unit()?;
        Self::new(self.value / dpi / to_inches, target).ok()
    }

    /// Express a physical length as canonical document units (`Px`)
    /// at the given DPI. Returns `None` for `Px` input or an invalid DPI.
    #[must_use]
    pub fn from_physical(self, dpi: f64) -> Option<Self> {
        if !dpi.is_finite() || dpi <= 0.0 {
            return None;
        }
        let from_inches = self.unit.inches_per_unit()?;
        Self::new(self.value * from_inches * dpi, Unit::Px).ok()
    }

    /// Express this value in canonical document units.
    ///
    /// `Px` values pass through unchanged; physical values require a DPI.
    #[must_use]
    pub fn to_document_units(self, dpi: f64) -> Option<Self> {
        if self.unit == Unit::Px {
            return Some(self);
        }
        self.from_physical(dpi)
    }
}

#[cfg(test)]
mod tests {
    use super::{Unit, UnitValue};

    const TOLERANCE: f64 = 1e-9;

    fn physical(value: f64, unit: Unit) -> UnitValue {
        UnitValue::new(value, unit).expect("finite test value")
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= TOLERANCE
    }

    #[test]
    fn physical_ratios_match_spec_anchors() {
        // 1 inch = 25.4 mm = 2.54 cm = 72 pt = 6 pica.
        assert_eq!(
            physical(1.0, Unit::Inch).to_unit(Unit::Mm),
            Some(physical(25.4, Unit::Mm))
        );
        assert_eq!(
            physical(1.0, Unit::Inch).to_unit(Unit::Cm),
            Some(physical(2.54, Unit::Cm))
        );
        assert_eq!(
            physical(1.0, Unit::Inch).to_unit(Unit::Pt),
            Some(physical(72.0, Unit::Pt))
        );
        assert_eq!(
            physical(1.0, Unit::Inch).to_unit(Unit::Pica),
            Some(physical(6.0, Unit::Pica))
        );
        assert_eq!(
            physical(1.0, Unit::Cm).to_unit(Unit::Mm),
            Some(physical(10.0, Unit::Mm))
        );
    }

    #[test]
    fn physical_round_trip_stays_within_tolerance() {
        let start = physical(12.345678901, Unit::Mm);
        let round_tripped = start
            .to_unit(Unit::Pt)
            .and_then(|v| v.to_unit(Unit::Mm))
            .expect("physical round-trip");
        assert!(approx_eq(round_tripped.value, start.value));
    }

    #[test]
    fn px_has_no_physical_size_without_dpi() {
        assert_eq!(Unit::Px.inches_per_unit(), None);
        assert!(!Unit::Px.is_physical());
        assert!(physical(10.0, Unit::Px).to_unit(Unit::Mm).is_none());
        assert!(physical(10.0, Unit::Mm).to_unit(Unit::Px).is_none());
    }

    #[test]
    fn document_pixels_convert_through_dpi() {
        // 1 inch × 300 DPI = 300 output pixels.
        let px = physical(300.0, Unit::Px);
        assert_eq!(
            px.to_physical(Unit::Inch, 300.0),
            Some(physical(1.0, Unit::Inch))
        );
        assert_eq!(
            physical(1.0, Unit::Inch).from_physical(300.0),
            Some(physical(300.0, Unit::Px))
        );
        // Invalid DPI never produces a value.
        assert!(px.to_physical(Unit::Inch, 0.0).is_none());
        assert!(px.to_physical(Unit::Inch, f64::NAN).is_none());
        assert!(physical(1.0, Unit::Inch).from_physical(-72.0).is_none());
    }

    #[test]
    fn to_document_units_passes_px_through() {
        let px = physical(42.0, Unit::Px);
        assert_eq!(px.to_document_units(300.0), Some(px));
    }

    #[test]
    fn non_finite_values_are_rejected() {
        assert!(UnitValue::new(f64::NAN, Unit::Mm).is_err());
        assert!(UnitValue::new(f64::INFINITY, Unit::Mm).is_err());
        assert!(UnitValue::new(f64::NEG_INFINITY, Unit::Mm).is_err());
    }

    #[test]
    fn negative_zero_is_canonicalized() {
        let value = UnitValue::new(-0.0, Unit::Mm).expect("zero is finite");
        assert!(value.value.is_sign_positive());
        assert_eq!(value.value, 0.0);
    }

    #[test]
    fn serialization_round_trip_preserves_value() {
        let start = physical(12.345678901, Unit::Mm);
        let json = serde_json::to_string(&start).expect("serializable");
        let back: UnitValue = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, start);
    }
}
