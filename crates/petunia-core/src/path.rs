//! Canonical vector path geometry, contours, and nodes.

use crate::math::Point;
use serde::{Deserialize, Serialize};

/// Type of control node on a vector contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// Sharp corner without tangent continuity.
    Cusp,
    /// Collinear control handles with smooth tangent.
    Smooth,
    /// Equal-length and collinear control handles.
    Symmetric,
}

/// A single node in a vector path with optional cubic Bézier handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathNode {
    /// Anchor position of the point.
    pub point: Point,
    /// Incoming handle relative or absolute control point (leading into anchor).
    pub handle_in: Option<Point>,
    /// Outgoing handle control point (leading out of anchor).
    pub handle_out: Option<Point>,
    /// Node continuity mode.
    pub kind: NodeKind,
}

impl PathNode {
    #[must_use]
    pub const fn new(point: Point, kind: NodeKind) -> Self {
        Self {
            point,
            handle_in: None,
            handle_out: None,
            kind,
        }
    }

    #[must_use]
    pub const fn with_handles(
        point: Point,
        handle_in: Option<Point>,
        handle_out: Option<Point>,
        kind: NodeKind,
    ) -> Self {
        Self {
            point,
            handle_in,
            handle_out,
            kind,
        }
    }
}

/// A connected sequence of path nodes (can be open or closed).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Contour {
    pub nodes: Vec<PathNode>,
    pub closed: bool,
}

impl Contour {
    #[must_use]
    pub const fn new(closed: bool) -> Self {
        Self {
            nodes: Vec::new(),
            closed,
        }
    }

    pub fn push_node(&mut self, node: PathNode) {
        self.nodes.push(node);
    }
}

/// Fill winding rule for polygons and compound shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// A complete vector path composed of one or more contours.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VectorPath {
    pub contours: Vec<Contour>,
    pub fill_rule: FillRule,
}

impl VectorPath {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            contours: Vec::new(),
            fill_rule: FillRule::NonZero,
        }
    }

    pub fn push_contour(&mut self, contour: Contour) {
        self.contours.push(contour);
    }

    /// Creates a standard rectangle path.
    #[must_use]
    pub fn rect(x: f64, y: f64, width: f64, height: f64) -> Self {
        let mut contour = Contour::new(true);
        contour.push_node(PathNode::new(Point::new(x, y), NodeKind::Cusp));
        contour.push_node(PathNode::new(Point::new(x + width, y), NodeKind::Cusp));
        contour.push_node(PathNode::new(Point::new(x + width, y + height), NodeKind::Cusp));
        contour.push_node(PathNode::new(Point::new(x, y + height), NodeKind::Cusp));

        let mut path = Self::new();
        path.push_contour(contour);
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_path_creation() {
        let path = VectorPath::rect(0.0, 0.0, 100.0, 50.0);
        assert_eq!(path.contours.len(), 1);
        assert!(path.contours[0].closed);
        assert_eq!(path.contours[0].nodes.len(), 4);
    }
}
