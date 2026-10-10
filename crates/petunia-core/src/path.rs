//! Canonical vector path geometry, contours, and nodes.

use crate::id::{ContourId, NodeId};
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

/// Canonical segment between a node and the next one.
///
/// `outgoing` belongs to the start node. In a closed contour, the last
/// node connects implicitly to the first; no duplicated anchor marks
/// closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentKind {
    /// Straight segment; handles do not alter it.
    Line,
    /// Cubic Bézier segment derived from endpoint handles.
    Cubic,
}

/// A single node in a vector path with optional cubic Bézier handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathNode {
    /// Stable identity of this anchor across moves, handle edits and undo.
    pub id: NodeId,
    /// Anchor position of the point.
    pub point: Point,
    /// Incoming handle relative or absolute control point (leading into anchor).
    pub handle_in: Option<Point>,
    /// Outgoing handle control point (leading out of anchor).
    pub handle_out: Option<Point>,
    /// Node continuity mode.
    pub kind: NodeKind,
    /// Segment from this node to the next one.
    pub outgoing: SegmentKind,
}

impl PathNode {
    #[must_use]
    pub fn new(point: Point, kind: NodeKind) -> Self {
        Self {
            id: NodeId::new_v4(),
            point,
            handle_in: None,
            handle_out: None,
            kind,
            outgoing: SegmentKind::Cubic,
        }
    }

    #[must_use]
    pub fn line(point: Point, kind: NodeKind) -> Self {
        Self {
            id: NodeId::new_v4(),
            point,
            handle_in: None,
            handle_out: None,
            kind,
            outgoing: SegmentKind::Line,
        }
    }

    #[must_use]
    pub fn with_handles(
        point: Point,
        handle_in: Option<Point>,
        handle_out: Option<Point>,
        kind: NodeKind,
    ) -> Self {
        Self {
            id: NodeId::new_v4(),
            point,
            handle_in,
            handle_out,
            kind,
            outgoing: SegmentKind::Cubic,
        }
    }
}

/// A connected sequence of path nodes (can be open or closed).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Contour {
    /// Stable identity of this contour inside its path.
    pub id: ContourId,
    pub nodes: Vec<PathNode>,
    pub closed: bool,
}

impl Contour {
    #[must_use]
    pub fn new(closed: bool) -> Self {
        Self {
            id: ContourId::new_v4(),
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
    /// Validate persisted identity and finite geometry independently of renderability.
    pub fn validate(&self) -> crate::error::Result<()> {
        let mut contours = std::collections::HashSet::new();
        let mut nodes = std::collections::HashSet::new();
        for contour in &self.contours {
            if !contours.insert(contour.id) {
                return Err(crate::CoreError::InvalidPath(format!(
                    "duplicate contour {}",
                    contour.id
                )));
            }
            for node in &contour.nodes {
                if !nodes.insert(node.id) {
                    return Err(crate::CoreError::InvalidPath(format!(
                        "duplicate node {}",
                        node.id
                    )));
                }
                for point in std::iter::once(node.point)
                    .chain(node.handle_in)
                    .chain(node.handle_out)
                {
                    if !point.x.is_finite() || !point.y.is_finite() {
                        return Err(crate::CoreError::InvalidPath(format!(
                            "non-finite coordinate on {}",
                            node.id
                        )));
                    }
                }
            }
        }
        Ok(())
    }
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
        contour.push_node(PathNode::line(Point::new(x, y), NodeKind::Cusp));
        contour.push_node(PathNode::line(Point::new(x + width, y), NodeKind::Cusp));
        contour.push_node(PathNode::line(
            Point::new(x + width, y + height),
            NodeKind::Cusp,
        ));
        contour.push_node(PathNode::line(Point::new(x, y + height), NodeKind::Cusp));

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
        assert!(path.contours[0]
            .nodes
            .iter()
            .all(|node| node.outgoing == SegmentKind::Line));
    }

    #[test]
    fn test_contour_and_node_ids_are_stable_through_serialization() {
        let mut contour = Contour::new(true);
        contour.push_node(PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp));
        contour.push_node(PathNode::line(Point::new(10.0, 0.0), NodeKind::Cusp));
        assert_ne!(contour.nodes[0].id, contour.nodes[1].id);

        let json = serde_json::to_string(&contour).expect("serializable");
        let back: Contour = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, contour);
        assert_eq!(back.id, contour.id);
        assert_eq!(back.nodes[0].id, contour.nodes[0].id);
        assert_eq!(back.nodes[1].outgoing, SegmentKind::Line);
    }
}
