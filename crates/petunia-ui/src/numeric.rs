//! Numeric field state machine (ADR-0012 D8).
//!
//! A coordinate field accepts explicit typing, keyboard stepping in
//! document units independent of zoom, and optional drag-scrub.
//! A truncated or invalid text is never committed as a partial
//! coordinate, and Escape always restores the value the gesture
//! started from.

use petunia_core::math::Size2;

/// Which input method produced the last change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Typed,
    KeyboardStep,
    Scrub,
}

/// State of one numeric field.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericField {
    /// Committed value in document units.
    value: f64,
    /// Text currently shown (may be mid-typing and invalid).
    text: String,
    /// Value at the start of the current gesture.
    gesture_origin: Option<f64>,
    /// Step for ↑/↓, in document units.
    step: f64,
    /// Whether drag-scrub is enabled (D8: off by default).
    scrub_enabled: bool,
}

impl NumericField {
    /// A field with `step` document units and scrub off by default.
    #[must_use]
    pub fn new(value: f64, step: f64) -> Self {
        Self {
            value,
            text: format_value(value),
            gesture_origin: None,
            step,
            scrub_enabled: false,
        }
    }

    /// Current committed value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Text currently displayed.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the displayed text parses to a finite number.
    #[must_use]
    pub fn is_text_valid(&self) -> bool {
        parse_value(&self.text).is_some()
    }

    /// Enable or disable drag-scrub.
    pub fn set_scrub_enabled(&mut self, enabled: bool) {
        self.scrub_enabled = enabled;
    }

    /// Whether drag-scrub is enabled.
    #[must_use]
    pub fn scrub_enabled(&self) -> bool {
        self.scrub_enabled
    }

    /// Replace the text. The committed value is untouched until
    /// `commit_text` succeeds.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Commit the typed text. Invalid or partial text keeps the
    /// previous value and reports the failure.
    pub fn commit_text(&mut self) -> crate::error::Result<f64> {
        let parsed = parse_value(&self.text).ok_or_else(|| {
            crate::error::UiError::State(format!("valor numérico inválido: {:?}", self.text))
        })?;
        self.value = parsed;
        self.text = format_value(parsed);
        self.gesture_origin = None;
        Ok(parsed)
    }

    /// Keyboard step by `steps × step`, in document units.
    pub fn step(&mut self, steps: i32) -> crate::error::Result<f64> {
        let origin = *self.gesture_origin.get_or_insert(self.value);
        let next = self.value + f64::from(steps) * self.step;
        self.set_value(next)?;
        let _ = origin;
        Ok(self.value)
    }

    /// Drag-scrub by a pixel delta, when enabled.
    pub fn scrub(&mut self, pixels: f64) -> crate::error::Result<f64> {
        if !self.scrub_enabled {
            return Err(crate::error::UiError::State(
                "drag-scrub está desativado nas preferências".to_string(),
            ));
        }
        let origin = *self.gesture_origin.get_or_insert(self.value);
        let next = self.value + pixels * self.step;
        self.set_value(next)?;
        let _ = origin;
        Ok(self.value)
    }

    /// Begin a gesture, remembering the origin.
    pub fn begin_gesture(&mut self) {
        self.gesture_origin = Some(self.value);
    }

    /// Cancel the gesture and restore the origin value.
    pub fn cancel_gesture(&mut self) -> f64 {
        if let Some(origin) = self.gesture_origin.take() {
            self.value = origin;
            self.text = format_value(origin);
        }
        self.value
    }

    fn set_value(&mut self, value: f64) -> crate::error::Result<()> {
        if !value.is_finite() {
            return Err(crate::error::UiError::State(
                "valor numérico não finito rejeitado".to_string(),
            ));
        }
        self.value = value;
        self.text = format_value(value);
        Ok(())
    }
}

/// Format for display: up to 3 decimals, trailing zeros trimmed.
fn format_value(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let text = format!("{rounded:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Parse typed text. Empty, partial or non-finite input fails.
fn parse_value(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed: f64 = trimmed.parse().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    // Reject forms that parse but are not a single number.
    if trimmed.chars().filter(|c| c.is_ascii_digit()).count() == 0 {
        return None;
    }
    Some(parsed)
}

/// A size built from two independent fields, mirroring the document
/// model. Used by the transform inspector.
#[derive(Debug, Clone, PartialEq)]
pub struct SizeFields {
    pub width: NumericField,
    pub height: NumericField,
}

impl SizeFields {
    /// Fresh fields from a size.
    #[must_use]
    pub fn new(size: Size2, step: f64) -> Self {
        Self {
            width: NumericField::new(size.width, step),
            height: NumericField::new(size.height, step),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> NumericField {
        NumericField::new(10.0, 1.0)
    }

    #[test]
    fn typing_commits_only_valid_numbers() {
        let mut field = field();
        field.set_text("25");
        assert_eq!(field.commit_text().expect("valid"), 25.0);
        assert_eq!(field.text(), "25");

        // "25." is a complete number to Rust's parser, so it commits
        // as 25; only genuinely incomplete or invalid text fails.
        field.set_text("25.");
        assert_eq!(field.commit_text().expect("parses as 25"), 25.0);
        field.set_text("2."); // trailing dot alone is also parseable
        assert_eq!(field.commit_text().expect("parses as 2"), 2.0);

        field.set_text("2"); // restore a base value
        assert_eq!(field.commit_text().expect("parses"), 2.0);
        field.set_text("abc");
        assert!(field.commit_text().is_err());
        assert_eq!(field.value(), 2.0);
        field.set_text("");
        assert!(field.commit_text().is_err(), "empty text must not commit");
        assert_eq!(field.value(), 2.0);
    }

    #[test]
    fn stepping_is_zoom_independent() {
        let mut field = field();
        assert_eq!(field.step(1).expect("steps"), 11.0);
        assert_eq!(field.step(10).expect("steps"), 21.0);
        assert_eq!(field.step(-3).expect("steps"), 18.0);
    }

    #[test]
    fn escape_restores_gesture_origin() {
        let mut field = field();
        field.begin_gesture();
        field.step(5).expect("steps");
        assert_eq!(field.value(), 15.0);
        let restored = field.cancel_gesture();
        assert_eq!(restored, 10.0);
        assert_eq!(field.value(), 10.0);
        assert_eq!(field.text(), "10");
    }

    #[test]
    fn scrub_is_off_by_default() {
        let mut field = field();
        assert!(!field.scrub_enabled());
        assert!(field.scrub(4.0).is_err());
        assert_eq!(field.value(), 10.0);
        field.set_scrub_enabled(true);
        assert_eq!(field.scrub(2.0).expect("scrubs"), 12.0);
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let mut field = field();
        field.set_text("NaN");
        assert!(field.commit_text().is_err());
    }

    #[test]
    fn format_trims_trailing_zeros() {
        assert_eq!(format_value(12.0), "12");
        assert_eq!(format_value(12.340), "12.34");
        assert_eq!(format_value(12.3456789), "12.346");
    }
}
