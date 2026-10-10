//! Snapping engine, alignment guidelines, and grid geometry.

use petunia_core::Point;

/// Kind of guideline snapping orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapOrientation {
    Horizontal,
    Vertical,
}

/// A snap guide line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapGuide {
    pub position: f64,
    pub orientation: SnapOrientation,
}

/// Configuration and threshold for snapping calculations.
#[derive(Debug, Clone, Copy)]
pub struct SnapConfig {
    pub threshold: f64,
    pub grid_spacing: Option<f64>,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            threshold: 5.0,
            grid_spacing: Some(10.0),
        }
    }
}

/// Result of snapping calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapResult {
    pub snapped_point: Point,
    pub snapped_x: bool,
    pub snapped_y: bool,
}

/// Computes snapping against a grid or guidelines.
#[must_use]
pub fn snap_point(point: Point, guides: &[SnapGuide], config: &SnapConfig) -> SnapResult {
    let mut result_x = point.x;
    let mut result_y = point.y;
    let mut snapped_x = false;
    let mut snapped_y = false;

    let valid_threshold = config.threshold.is_finite() && config.threshold >= 0.0;
    if !valid_threshold || !point.x.is_finite() || !point.y.is_finite() {
        return SnapResult {
            snapped_point: point,
            snapped_x: false,
            snapped_y: false,
        };
    }
    let mut distance_x = f64::INFINITY;
    let mut distance_y = f64::INFINITY;
    // Prefer the nearest guide; equal distances use the lower coordinate.
    for guide in guides {
        match guide.orientation {
            SnapOrientation::Vertical => {
                let distance = (point.x - guide.position).abs();
                if distance <= config.threshold
                    && (distance < distance_x
                        || (distance == distance_x && guide.position < result_x))
                {
                    distance_x = distance;
                    result_x = guide.position;
                    snapped_x = true;
                }
            }
            SnapOrientation::Horizontal => {
                let distance = (point.y - guide.position).abs();
                if distance <= config.threshold
                    && (distance < distance_y
                        || (distance == distance_y && guide.position < result_y))
                {
                    distance_y = distance;
                    result_y = guide.position;
                    snapped_y = true;
                }
            }
        }
    }

    // Fallback to grid snapping if not snapped by guide
    if let Some(spacing) = config
        .grid_spacing
        .filter(|spacing| spacing.is_finite() && *spacing > 0.0)
    {
        if !snapped_x {
            let nearest_x = (point.x / spacing).round() * spacing;
            if (point.x - nearest_x).abs() <= config.threshold {
                result_x = nearest_x;
                snapped_x = true;
            }
        }
        if !snapped_y {
            let nearest_y = (point.y / spacing).round() * spacing;
            if (point.y - nearest_y).abs() <= config.threshold {
                result_y = nearest_y;
                snapped_y = true;
            }
        }
    }

    SnapResult {
        snapped_point: Point::new(result_x, result_y),
        snapped_x,
        snapped_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapping_to_guide_and_grid() {
        let guides = vec![SnapGuide {
            position: 100.0,
            orientation: SnapOrientation::Vertical,
        }];
        let config = SnapConfig::default();

        let p1 = Point::new(102.0, 50.0);
        let res1 = snap_point(p1, &guides, &config);
        assert_eq!(res1.snapped_point.x, 100.0);
        assert!(res1.snapped_x);

        let p2 = Point::new(201.0, 302.0);
        let res2 = snap_point(p2, &[], &config);
        assert_eq!(res2.snapped_point, Point::new(200.0, 300.0));
    }
}
