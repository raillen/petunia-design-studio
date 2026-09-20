//! Semantic 2D point. Plain `f64` pair; no third-party type escapes.

use serde::{Deserialize, Serialize};

/// Document-space point in user units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GPoint {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

impl GPoint {
    /// Origin point.
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// True when both coordinates are finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Euclidean distance.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_finite() {
        assert!(GPoint::ORIGIN.is_finite());
    }

    #[test]
    fn distance_pythagoras() {
        let d = GPoint::new(0.0, 0.0).distance_to(GPoint::new(3.0, 4.0));
        assert!((d - 5.0).abs() < 1e-9, "got {d}");
    }
}
