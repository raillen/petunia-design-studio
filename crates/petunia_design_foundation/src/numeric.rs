//! Validated numeric input parsing with strict diagnostics (Dossier V1 §13, Ledger M15).
//!
//! Rejects invalid, empty, or non-finite inputs with descriptive bilingual
//! errors instead of silently coercing them to `0.0`.

use serde::{Deserialize, Serialize};

/// Semantic domain and boundary constraints for numeric field inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericFieldKind {
    /// Arbitrary coordinate or displacement in points (finite, signed).
    DistancePt,
    /// Non-negative dimension or stroke width in points (`>= 0.0`, finite).
    NonNegativeDimensionPt,
    /// Strictly positive dimension in points (`> 0.0`, finite, e.g. width, height, font size).
    PositiveDimensionPt,
    /// Opacity or percentage (`0.0..=100.0` or factor `0.0..=1.0`).
    Percentage,
    /// Rotation angle in degrees (finite).
    AngleDeg,
    /// Integer count with an inclusive minimum and maximum.
    Count { min: u32, max: u32 },
}

/// Structured validation failure for typed numeric inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericParseError {
    /// Input was empty or contained only whitespace.
    Empty,
    /// Input could not be parsed as a floating-point or integer number.
    NotANumber,
    /// Input parsed into NaN or positive/negative infinity.
    NonFinite,
    /// Negative numbers are forbidden for this field.
    NegativeNotAllowed,
    /// Strictly positive value was required (e.g. width/height cannot be zero).
    ZeroNotAllowed,
    /// Value is strictly below the required minimum bound.
    BelowMinimum(String),
    /// Value is strictly above the required maximum bound.
    AboveMaximum(String),
}

impl NumericParseError {
    /// Human-readable message in en-US.
    #[must_use]
    pub fn message_en_us(&self) -> &'static str {
        match self {
            Self::Empty => "Value cannot be empty",
            Self::NotANumber => "Invalid value: enter a valid number",
            Self::NonFinite => "Value must be a finite number",
            Self::NegativeNotAllowed => "Negative values are not allowed",
            Self::ZeroNotAllowed => "Value must be strictly greater than zero",
            Self::BelowMinimum(_) => "Value is below the allowed minimum",
            Self::AboveMaximum(_) => "Value exceeds the allowed maximum",
        }
    }

    /// Human-readable message in pt-BR.
    #[must_use]
    pub fn message_pt_br(&self) -> &'static str {
        match self {
            Self::Empty => "O valor não pode estar vazio",
            Self::NotANumber => "Valor inválido: digite um número válido",
            Self::NonFinite => "O valor deve ser um número finito",
            Self::NegativeNotAllowed => "Valores negativos não são permitidos",
            Self::ZeroNotAllowed => "O valor deve ser estritamente maior que zero",
            Self::BelowMinimum(_) => "O valor está abaixo do mínimo permitido",
            Self::AboveMaximum(_) => "O valor excede o limite máximo permitido",
        }
    }

    /// Bilingual message for UI toasts and error tooltips.
    #[must_use]
    pub fn message_bilingual(&self) -> String {
        format!("{} / {}", self.message_pt_br(), self.message_en_us())
    }
}

/// Parses and validates a raw user string against a `NumericFieldKind` contract.
///
/// Strips optional trailing unit suffixes (`pt`, `px`, `%`, `deg`, `°`),
/// normalizes decimal comma `,` to `.`, and rejects invalid inputs with
/// an explicit `NumericParseError` instead of falling back to `0.0`.
pub fn parse_numeric_input(raw: &str, kind: NumericFieldKind) -> Result<f64, NumericParseError> {
    let mut trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(NumericParseError::Empty);
    }

    // Strip unit suffixes case-insensitively
    let lower = trimmed.to_ascii_lowercase();
    for suffix in &["pt", "px", "%", "deg", "°"] {
        if lower.ends_with(suffix) {
            trimmed = trimmed[..trimmed.len() - suffix.len()].trim();
            break;
        }
    }

    if trimmed.is_empty() {
        return Err(NumericParseError::Empty);
    }

    let normalized = trimmed.replace(',', ".");
    let val = normalized
        .parse::<f64>()
        .map_err(|_| NumericParseError::NotANumber)?;

    if !val.is_finite() {
        return Err(NumericParseError::NonFinite);
    }

    match kind {
        NumericFieldKind::DistancePt | NumericFieldKind::AngleDeg => Ok(val),
        NumericFieldKind::NonNegativeDimensionPt => {
            if val < 0.0 {
                Err(NumericParseError::NegativeNotAllowed)
            } else {
                Ok(val)
            }
        }
        NumericFieldKind::PositiveDimensionPt => {
            if val < 0.0 {
                Err(NumericParseError::NegativeNotAllowed)
            } else if val <= 0.0 {
                Err(NumericParseError::ZeroNotAllowed)
            } else {
                Ok(val)
            }
        }
        NumericFieldKind::Percentage => {
            if val < 0.0 {
                Err(NumericParseError::BelowMinimum("0%".to_string()))
            } else if val > 100.0 {
                Err(NumericParseError::AboveMaximum("100%".to_string()))
            } else {
                Ok(val)
            }
        }
        NumericFieldKind::Count { min, max } => {
            let int_val = val.round() as i64;
            if (val - int_val as f64).abs() > 1e-6 {
                return Err(NumericParseError::NotANumber);
            }
            if int_val < min as i64 {
                Err(NumericParseError::BelowMinimum(min.to_string()))
            } else if int_val > max as i64 {
                Err(NumericParseError::AboveMaximum(max.to_string()))
            } else {
                Ok(int_val as f64)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numeric_valid_distance_and_units() {
        assert_eq!(
            parse_numeric_input("12.5 pt", NumericFieldKind::DistancePt).unwrap(),
            12.5
        );
        assert_eq!(
            parse_numeric_input("-4,2px", NumericFieldKind::DistancePt).unwrap(),
            -4.2
        );
        assert_eq!(
            parse_numeric_input("  0  ", NumericFieldKind::DistancePt).unwrap(),
            0.0
        );
    }

    #[test]
    fn parse_numeric_strictly_rejects_empty_and_garbage_never_zero() {
        assert_eq!(
            parse_numeric_input("", NumericFieldKind::DistancePt),
            Err(NumericParseError::Empty)
        );
        assert_eq!(
            parse_numeric_input("   ", NumericFieldKind::DistancePt),
            Err(NumericParseError::Empty)
        );
        assert_eq!(
            parse_numeric_input("pt", NumericFieldKind::DistancePt),
            Err(NumericParseError::Empty)
        );
        assert_eq!(
            parse_numeric_input("abc", NumericFieldKind::DistancePt),
            Err(NumericParseError::NotANumber)
        );
        assert_eq!(
            parse_numeric_input("NaN", NumericFieldKind::DistancePt),
            Err(NumericParseError::NonFinite)
        );
        assert_eq!(
            parse_numeric_input("inf", NumericFieldKind::DistancePt),
            Err(NumericParseError::NonFinite)
        );
    }

    #[test]
    fn parse_numeric_boundary_checks() {
        assert_eq!(
            parse_numeric_input("-1.0", NumericFieldKind::NonNegativeDimensionPt),
            Err(NumericParseError::NegativeNotAllowed)
        );
        assert_eq!(
            parse_numeric_input("0.0", NumericFieldKind::NonNegativeDimensionPt).unwrap(),
            0.0
        );

        assert_eq!(
            parse_numeric_input("0.0", NumericFieldKind::PositiveDimensionPt),
            Err(NumericParseError::ZeroNotAllowed)
        );
        assert_eq!(
            parse_numeric_input("14.0 pt", NumericFieldKind::PositiveDimensionPt).unwrap(),
            14.0
        );

        assert_eq!(
            parse_numeric_input("105 %", NumericFieldKind::Percentage),
            Err(NumericParseError::AboveMaximum("100%".to_string()))
        );
        assert_eq!(
            parse_numeric_input("-5%", NumericFieldKind::Percentage),
            Err(NumericParseError::BelowMinimum("0%".to_string()))
        );
        assert_eq!(
            parse_numeric_input("75.5%", NumericFieldKind::Percentage).unwrap(),
            75.5
        );

        let sides_contract = NumericFieldKind::Count { min: 3, max: 64 };
        assert_eq!(
            parse_numeric_input("2", sides_contract),
            Err(NumericParseError::BelowMinimum("3".to_string()))
        );
        assert_eq!(
            parse_numeric_input("65", sides_contract),
            Err(NumericParseError::AboveMaximum("64".to_string()))
        );
        assert_eq!(parse_numeric_input("5", sides_contract).unwrap(), 5.0);
    }
}
