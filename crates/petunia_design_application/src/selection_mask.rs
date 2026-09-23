//! Transient raster selection mask (10.9, TOOLS_DECISIONS Batch 13).
//!
//! Photoshop-style marching-ants selection as flat polygon contours with
//! even-odd containment (the boolean engine's native hole convention).
//! Combine operations run through the tested polygon boolean engine, so
//! holes survive as sibling contours. Feather is a stored render-time
//! parameter; geometry stays sharp. Selection is session transient state
//! (like object selection): gestures never touch undo history.
//!
//! Flood/brush sampling and pixel painting stay future work: they need
//! pixel layers in the document, which do not exist yet.

use serde::{Deserialize, Serialize};

use petunia_design_geometry::{BooleanInput, BooleanOp, GPath, GPoint, boolean_op};

/// Flatten tolerance for selection booleans (F-21).
const SELECTION_TOLERANCE: f64 = 0.5;
/// Ellipse flattening segments for containment and booleans.
const ELLIPSE_SEGMENTS: usize = 64;
/// Minimum contour area to survive a combine (drops slivers).
const MIN_SELECTION_AREA: f64 = 1e-6;

/// One geometric selection primitive before flattening.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SelectionShape {
    /// Axis-aligned rectangle.
    Rect { x0: f64, y0: f64, x1: f64, y1: f64 },
    /// Axis-aligned ellipse.
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64 },
    /// Freehand polygon (implicitly closed).
    Polygon(Vec<GPoint>),
}

impl SelectionShape {
    /// Flattens the shape to closed contours (closure points not duplicated).
    #[must_use]
    pub fn contours(&self) -> Vec<Vec<GPoint>> {
        match self {
            Self::Rect { x0, y0, x1, y1 } => {
                let (x0, x1) = (x0.min(*x1), x0.max(*x1));
                let (y0, y1) = (y0.min(*y1), y0.max(*y1));
                if x1 - x0 < 1e-9 || y1 - y0 < 1e-9 {
                    return Vec::new();
                }
                vec![vec![
                    GPoint::new(x0, y0),
                    GPoint::new(x1, y0),
                    GPoint::new(x1, y1),
                    GPoint::new(x0, y1),
                ]]
            }
            Self::Ellipse { cx, cy, rx, ry } => {
                let (rx, ry) = (rx.abs(), ry.abs());
                if rx < 1e-9 || ry < 1e-9 {
                    return Vec::new();
                }
                let contour: Vec<GPoint> = (0..ELLIPSE_SEGMENTS)
                    .map(|i| {
                        let a = (i as f64) / (ELLIPSE_SEGMENTS as f64)
                            * std::f64::consts::TAU;
                        GPoint::new(cx + rx * a.cos(), cy + ry * a.sin())
                    })
                    .collect();
                vec![contour]
            }
            Self::Polygon(points) => {
                if points.len() < 3 {
                    return Vec::new();
                }
                vec![points.clone()]
            }
        }
    }

    /// True when the shape covers `pt` (even-odd over its contours).
    #[must_use]
    pub fn contains(&self, pt: GPoint) -> bool {
        self.contours().iter().any(|c| point_in_poly(pt, c))
    }
}

/// How a new shape combines with the existing mask (Photoshop modifiers:
/// Shift adds, Alt subtracts, Shift+Alt intersects).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Replace the mask (default click-drag).
    #[default]
    Replace,
    /// Union with the mask (Shift).
    Add,
    /// Carve out of the mask (Alt).
    Subtract,
    /// Keep only the overlap (Shift+Alt).
    Intersect,
}

impl SelectionMode {
    /// Resolves the mode from semantic modifiers.
    #[must_use]
    pub fn from_modifiers(additive: bool, subtractive: bool) -> Self {
        match (additive, subtractive) {
            (true, true) => Self::Intersect,
            (true, false) => Self::Add,
            (false, true) => Self::Subtract,
            (false, false) => Self::Replace,
        }
    }
}

/// Transient raster selection mask: flat contours, even-odd containment.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RasterSelection {
    /// Mask contours (holes are siblings with opposite winding).
    #[serde(default)]
    pub contours: Vec<Vec<GPoint>>,
    /// Feather radius in document points (render-time softness parameter).
    #[serde(default)]
    pub feather: f64,
}

impl RasterSelection {
    /// Creates an empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no contours survive.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }

    /// Clears the mask (Ctrl+D equivalent).
    pub fn clear(&mut self) {
        self.contours.clear();
    }

    /// Combines one shape into the mask under `mode`.
    pub fn combine(&mut self, shape: &SelectionShape, mode: SelectionMode) {
        let incoming = shape.contours();
        match mode {
            SelectionMode::Replace => {
                self.contours = keep_solids(&incoming);
            }
            SelectionMode::Add => {
                self.contours = union_all(&self.contours, &incoming);
            }
            SelectionMode::Subtract => {
                self.contours = difference_all(&self.contours, &incoming);
            }
            SelectionMode::Intersect => {
                self.contours = intersect_all(&self.contours, &incoming);
            }
        }
    }

    /// Inverts the mask inside `frame` (Select Inverse).
    /// Empty masks stay empty: inverting nothing selects nothing.
    pub fn invert_in(&mut self, frame: &[GPoint]) {
        if self.contours.is_empty() || frame.len() < 3 {
            return;
        }
        self.contours = difference_all(std::slice::from_ref(&frame.to_vec()), &self.contours);
    }

    /// Grows (positive) or shrinks (negative) the mask by `delta`.
    /// Implemented through the true offset engine on the contour set, so
    /// holes and islands track together.
    pub fn grow(&mut self, delta: f64) {
        if self.contours.is_empty() || !delta.is_finite() || delta.abs() < 1e-9 {
            return;
        }
        use petunia_design_geometry::{OffsetCap, OffsetJoin, offset_path};
        let path = GPath::from_polygons(&self.contours);
        // Miter joins keep rectilinear selections sharp (Round would eat corners).
        let grown = offset_path(&path, delta, OffsetJoin::Miter, OffsetCap::None);
        match grown {
            Some(path) => {
                self.contours = path
                    .to_polygons(SELECTION_TOLERANCE)
                    .into_iter()
                    .filter(|c| c.len() >= 3 && contour_area(c).abs() >= MIN_SELECTION_AREA)
                    .collect();
            }
            None => self.contours.clear(),
        }
    }

    /// Sets the feather radius (render-time parameter, never geometry).
    pub fn set_feather(&mut self, radius: f64) {
        if radius.is_finite() {
            self.feather = radius.max(0.0);
        }
    }

    /// True when `pt` lies inside the mask (even-odd over all contours).
    #[must_use]
    pub fn contains(&self, pt: GPoint) -> bool {
        let mut inside = false;
        for contour in &self.contours {
            if point_in_poly(pt, contour) {
                inside = !inside;
            }
        }
        inside
    }

    /// Total signed area (holes subtract).
    #[must_use]
    pub fn signed_area(&self) -> f64 {
        self.contours.iter().map(|c| contour_area(c)).sum()
    }
}

/// Unions two contour sets in one overlay call (hole-safe).
fn union_all(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    if a.is_empty() {
        return keep_solids(b);
    }
    if b.is_empty() {
        return keep_solids(a);
    }
    boolean_op(
        &BooleanInput::new(a.to_vec()),
        &BooleanInput::new(b.to_vec()),
        BooleanOp::Union,
    )
    .into_iter()
    .filter(|c| c.len() >= 3 && contour_area(c).abs() >= MIN_SELECTION_AREA)
    .collect()
}

/// Subtracts contour set `b` from `a` in one overlay call (hole-safe).
fn difference_all(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    if a.is_empty() {
        return Vec::new();
    }
    if b.is_empty() {
        return keep_solids(a);
    }
    boolean_op(
        &BooleanInput::new(a.to_vec()),
        &BooleanInput::new(b.to_vec()),
        BooleanOp::Difference,
    )
    .into_iter()
    .filter(|c| c.len() >= 3 && contour_area(c).abs() >= MIN_SELECTION_AREA)
    .collect()
}

/// Intersects two contour sets pairwise (distributive, hole-safe).
fn intersect_all(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for subject in a {
        for clip in b {
            out.extend(
                boolean_op(
                    &BooleanInput::single(subject.clone()),
                    &BooleanInput::single(clip.clone()),
                    BooleanOp::Intersection,
                )
                .into_iter()
                .filter(|c| c.len() >= 3 && contour_area(c).abs() >= MIN_SELECTION_AREA),
            );
        }
    }
    out
}

/// Drops degenerate contours.
fn keep_solids(contours: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    contours
        .iter()
        .filter(|c| c.len() >= 3 && contour_area(c).abs() >= MIN_SELECTION_AREA)
        .cloned()
        .collect()
}

/// Shoelace area of one contour.
fn contour_area(contour: &[GPoint]) -> f64 {
    if contour.len() < 3 {
        return 0.0;
    }
    let mut sum: f64 = contour
        .windows(2)
        .map(|w| w[0].x * w[1].y - w[1].x * w[0].y)
        .sum();
    let (first, last) = (contour[0], contour[contour.len() - 1]);
    sum += last.x * first.y - first.x * last.y;
    sum / 2.0
}

/// Even-odd point-in-polygon.
fn point_in_poly(pt: GPoint, poly: &[GPoint]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        if (pi.y > pt.y) != (pj.y > pt.y)
            && pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) + pi.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> SelectionShape {
        SelectionShape::Rect { x0, y0, x1, y1 }
    }

    #[test]
    fn rect_contains_inside_not_outside() {
        let shape = rect(0.0, 0.0, 10.0, 10.0);
        assert!(shape.contains(GPoint::new(5.0, 5.0)));
        assert!(!shape.contains(GPoint::new(15.0, 5.0)));
    }

    #[test]
    fn ellipse_contains_center_not_corner() {
        let shape = SelectionShape::Ellipse {
            cx: 0.0,
            cy: 0.0,
            rx: 10.0,
            ry: 5.0,
        };
        assert!(shape.contains(GPoint::new(0.0, 0.0)));
        assert!(!shape.contains(GPoint::new(9.0, 4.0)));
    }

    #[test]
    fn add_merges_overlapping_areas() {
        let mut sel = RasterSelection::new();
        sel.combine(&rect(0.0, 0.0, 10.0, 10.0), SelectionMode::Replace);
        sel.combine(&rect(5.0, 5.0, 15.0, 15.0), SelectionMode::Add);
        assert!((sel.signed_area().abs() - 175.0).abs() < 1.0, "got {}", sel.signed_area());
        assert!(sel.contains(GPoint::new(12.0, 12.0)));
    }

    #[test]
    fn subtract_carves_hole_and_contains_respects_it() {
        let mut sel = RasterSelection::new();
        sel.combine(&rect(0.0, 0.0, 10.0, 10.0), SelectionMode::Replace);
        sel.combine(&rect(2.0, 2.0, 8.0, 8.0), SelectionMode::Subtract);
        assert!((sel.signed_area().abs() - 64.0).abs() < 1.0, "got {}", sel.signed_area());
        assert!(sel.contains(GPoint::new(1.0, 1.0)));
        assert!(!sel.contains(GPoint::new(5.0, 5.0)));
    }

    #[test]
    fn intersect_keeps_overlap_only() {
        let mut sel = RasterSelection::new();
        sel.combine(&rect(0.0, 0.0, 10.0, 10.0), SelectionMode::Replace);
        sel.combine(&rect(5.0, 5.0, 15.0, 15.0), SelectionMode::Intersect);
        assert!((sel.signed_area().abs() - 25.0).abs() < 1.0, "got {}", sel.signed_area());
        assert!(!sel.contains(GPoint::new(2.0, 2.0)));
        assert!(sel.contains(GPoint::new(7.0, 7.0)));
    }

    #[test]
    fn invert_inside_frame_selects_complement() {
        let mut sel = RasterSelection::new();
        sel.combine(&rect(0.0, 0.0, 10.0, 10.0), SelectionMode::Replace);
        let frame = rect(-50.0, -50.0, 50.0, 50.0).contours();
        sel.invert_in(&frame[0]);
        assert!(!sel.contains(GPoint::new(5.0, 5.0)));
        assert!(sel.contains(GPoint::new(30.0, 30.0)));
    }

    #[test]
    fn grow_expands_and_shrink_restores() {
        let mut sel = RasterSelection::new();
        sel.combine(&rect(0.0, 0.0, 10.0, 10.0), SelectionMode::Replace);
        sel.grow(5.0);
        assert!((sel.signed_area().abs() - 400.0).abs() < 8.0, "got {}", sel.signed_area());
        sel.grow(-5.0);
        assert!((sel.signed_area().abs() - 100.0).abs() < 8.0, "got {}", sel.signed_area());
    }

    #[test]
    fn mode_from_modifiers_matches_photoshop() {
        assert_eq!(SelectionMode::from_modifiers(false, false), SelectionMode::Replace);
        assert_eq!(SelectionMode::from_modifiers(true, false), SelectionMode::Add);
        assert_eq!(SelectionMode::from_modifiers(false, true), SelectionMode::Subtract);
        assert_eq!(SelectionMode::from_modifiers(true, true), SelectionMode::Intersect);
    }
}
