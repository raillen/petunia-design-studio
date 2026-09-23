//! Precision measurement primitives: distance, delta and angle (Table B).
//!
//! Pure over [`crate::GPoint`]; the shell tool keeps only gesture state.

use serde::{Deserialize, Serialize};

use crate::GPoint;

/// Measurement readout between two document points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasurementReadout {
    /// Measurement start in document points.
    pub start: GPoint,
    /// Measurement end in document points.
    pub end: GPoint,
    /// Horizontal delta.
    pub dx: f64,
    /// Vertical delta.
    pub dy: f64,
    /// Euclidean distance.
    pub distance: f64,
    /// Direction in degrees (`atan2(dy, dx)`).
    pub angle_deg: f64,
}

/// Computes the measurement readout between two document points.
#[must_use]
pub fn measure_readout(p0: GPoint, p1: GPoint) -> MeasurementReadout {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let angle_deg = dy.atan2(dx).to_degrees();
    MeasurementReadout {
        start: p0,
        end: p1,
        dx,
        dy,
        distance,
        angle_deg,
    }
}

/// Area readout of the axis-aligned rectangle spanned by two points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AreaReadout {
    /// Normalized corner: minimum x/y.
    pub min: GPoint,
    /// Normalized corner: maximum x/y.
    pub max: GPoint,
    /// Width in document points.
    pub width: f64,
    /// Height in document points.
    pub height: f64,
    /// Enclosed area in square points.
    pub area: f64,
    /// Perimeter in document points.
    pub perimeter: f64,
}

/// Computes the area readout between two document points (drag order free).
#[must_use]
pub fn area_readout(p0: GPoint, p1: GPoint) -> AreaReadout {
    let min = GPoint::new(p0.x.min(p1.x), p0.y.min(p1.y));
    let max = GPoint::new(p0.x.max(p1.x), p0.y.max(p1.y));
    let width = max.x - min.x;
    let height = max.y - min.y;
    AreaReadout {
        min,
        max,
        width,
        height,
        area: width * height,
        perimeter: 2.0 * (width + height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readout_matches_3_4_5_triangle() {
        let r = measure_readout(GPoint::ORIGIN, GPoint::new(3.0, 4.0));
        assert!((r.distance - 5.0).abs() < 1e-9);
        assert!((r.angle_deg - 53.13).abs() < 0.01);
    }

    #[test]
    fn area_readout_normalizes_drag_direction() {
        let r = area_readout(GPoint::new(30.0, 40.0), GPoint::new(10.0, 10.0));
        assert_eq!(
            (r.width, r.height, r.area, r.perimeter),
            (20.0, 30.0, 600.0, 100.0)
        );
    }
}
