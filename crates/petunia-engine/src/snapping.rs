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

    // Check guides
    for guide in guides {
        match guide.orientation {
            SnapOrientation::Vertical => {
                if (point.x - guide.position).abs() <= config.threshold {
                    result_x = guide.position;
                    snapped_x = true;
                }
            }
            SnapOrientation::Horizontal => {
                if (point.y - guide.position).abs() <= config.threshold {
                    result_y = guide.position;
                    snapped_y = true;
                }
            }
        }
    }

    // Fallback to grid snapping if not snapped by guide
    if let Some(spacing) = config.grid_spacing {
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
