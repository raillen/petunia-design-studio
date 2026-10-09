//! Structured measurements between document points.
//!
//! The engine returns distance, angle and delta; unit conversion and
//! locale stay with the UI.

use petunia_core::{Angle, Vec2};
use serde::{Deserialize, Serialize};

/// One measurement between two document points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub distance: f64,
    pub angle: Angle,
    pub delta: Vec2,
}

/// Measure from `a` to `b` in document units.
#[must_use]
pub fn measure_points(a: petunia_core::Point, b: petunia_core::Point) -> Measurement {
    let delta = Vec2::new(b.x - a.x, b.y - a.y);
    let angle = Angle::new(delta.dy.atan2(delta.dx)).expect("atan2 is finite");
    Measurement {
        distance: delta.length(),
        angle,
        delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_reports_distance_angle_delta() {
        let measurement = measure_points(
            petunia_core::Point::new(0.0, 0.0),
            petunia_core::Point::new(3.0, 4.0),
        );
        assert_eq!(measurement.distance, 5.0);
        assert_eq!(measurement.delta, Vec2::new(3.0, 4.0));
        assert!((measurement.angle.radians() - 0.9272952180016122).abs() < 1e-12);
    }
}
