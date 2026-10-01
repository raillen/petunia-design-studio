//! True path offsetting with joins and caps (10.3).
//!
//! Expansion follows the stroked outer edge (curves stay curves).
//! Compound closed paths and insets use the topology-aware offset engine;
//! curves flatten at 0.25pt per F-21 while their editable source is retained.

use kurbo::{BezPath, Cap, Join, PathEl, Point, Stroke};

use crate::{GPath, GPoint, PathVerb};

/// Corner join style for offsets (matches Affinity Contour Type).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OffsetJoin {
    /// Sharp pointed joins.
    Miter,
    /// Rounded joins (Affinity default).
    #[default]
    Round,
    /// Cut-off (squared) joins.
    Bevel,
}

/// End-cap style for offsets of open paths (matches Affinity Contour Caps).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OffsetCap {
    /// No cap extension (butt).
    #[default]
    None,
    /// Semicircular caps.
    Round,
    /// Squared caps.
    Square,
}

impl OffsetJoin {
    fn kurbo(self) -> Join {
        match self {
            Self::Miter => Join::Miter,
            Self::Round => Join::Round,
            Self::Bevel => Join::Bevel,
        }
    }
}

impl OffsetCap {
    fn kurbo(self) -> Cap {
        match self {
            Self::None => Cap::Butt,
            Self::Round => Cap::Round,
            Self::Square => Cap::Square,
        }
    }
}

/// Offsets `path` by `distance` document points (positive expands,
/// negative insets) with the given join/cap style.
///
/// Returns `None` when the offset collapses (empty or degenerate result);
/// callers keep the source and report instead of silently destroying it.
#[must_use]
pub fn offset_path(path: &GPath, distance: f64, join: OffsetJoin, cap: OffsetCap) -> Option<GPath> {
    if !distance.is_finite() || path.is_empty() {
        return None;
    }
    if distance.abs() < 1e-9 {
        return Some(path.clone());
    }
    let width = distance.abs() * 2.0;
    let stroke = Stroke::new(width)
        .with_join(join.kurbo())
        .with_caps(cap.kurbo());
    let bez = to_kurbo(path);
    let closed = is_closed(path);
    if closed && (distance < 0.0 || path.subpath_count() > 1) {
        return offset_closed(path, distance, join);
    }
    let band = kurbo::stroke(bez.iter(), &stroke, &kurbo::StrokeOpts::default(), 0.25);
    // A single closed loop expands to the outer boundary of its stroke.
    // Open-path stroke bands must retain every component (and winding hole).
    let contours = if closed {
        vec![pick_largest(&band)?]
    } else {
        band_contours(&band)
    };
    let mut out = GPath::new();
    for contour in contours {
        for verb in contour {
            out.push(verb).ok()?;
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// True when every subpath of `path` ends with `Close`.
fn is_closed(path: &GPath) -> bool {
    let mut has_points = false;
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_)
            | PathVerb::LineTo(_)
            | PathVerb::QuadTo(_, _)
            | PathVerb::CubicTo(_, _, _) => {
                has_points = true;
            }
            PathVerb::Close => {}
        }
    }
    // A path counts as closed when it has points and no open tail:
    // every MoveTo run terminates in Close.
    if !has_points {
        return false;
    }
    let mut open = false;
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_) => {
                if open {
                    return false;
                }
                open = true;
            }
            PathVerb::Close => open = false,
            _ => {}
        }
    }
    !open
}

/// Picks the largest-area contour of a stroke band as verbs.
/// Used for expansion, where the outer edge is always the wanted boundary.
fn pick_largest(band: &BezPath) -> Option<Vec<PathVerb>> {
    let contours = band_contours(band);
    if contours.is_empty() {
        return None;
    }
    let area = |verbs: &[PathVerb]| contour_area(verbs).abs();
    let picked = contours.into_iter().max_by(|a, b| {
        area(a)
            .partial_cmp(&area(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    if area(&picked) < 1e-6 {
        return None;
    }
    Some(picked)
}

/// Splits a stroke band into per-subpath verb contours.
fn band_contours(band: &BezPath) -> Vec<Vec<PathVerb>> {
    let mut contours: Vec<Vec<PathVerb>> = Vec::new();
    let mut current: Vec<PathVerb> = Vec::new();
    for el in band.iter() {
        match el {
            PathEl::MoveTo(p) => {
                if !current.is_empty() {
                    contours.push(std::mem::take(&mut current));
                }
                current.push(PathVerb::MoveTo(GPoint::new(p.x, p.y)));
            }
            PathEl::LineTo(p) => current.push(PathVerb::LineTo(GPoint::new(p.x, p.y))),
            PathEl::QuadTo(c, p) => current.push(PathVerb::QuadTo(
                GPoint::new(c.x, c.y),
                GPoint::new(p.x, p.y),
            )),
            PathEl::CurveTo(c1, c2, p) => current.push(PathVerb::CubicTo(
                GPoint::new(c1.x, c1.y),
                GPoint::new(c2.x, c2.y),
                GPoint::new(p.x, p.y),
            )),
            PathEl::ClosePath => current.push(PathVerb::Close),
        }
    }
    if !current.is_empty() {
        contours.push(current);
    }
    contours
}

/// Offsets all closed contours together; never concatenate disconnected loops
/// into one polygon. Orientation carries outer/hole identity into the engine.
fn offset_closed(path: &GPath, distance: f64, join: OffsetJoin) -> Option<GPath> {
    let flat: Vec<Vec<[f64; 2]>> = path
        .to_polygons(0.25)
        .into_iter()
        .filter(|contour| contour.len() >= 3)
        .map(|contour| contour.into_iter().map(|p| [p.x, p.y]).collect())
        .collect();
    if flat.is_empty() {
        return None;
    }
    let contours = io_offset_adapter::offset_contours(&flat, distance, join);
    if contours.is_empty() {
        return None;
    }
    let points: Vec<Vec<GPoint>> = contours
        .into_iter()
        .map(|c| c.into_iter().map(|p| GPoint::new(p[0], p[1])).collect())
        .collect();
    let out = GPath::from_polygons(&points);
    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// Shoelace area of one contour (endpoints only; curves approximated).
fn contour_area(verbs: &[PathVerb]) -> f64 {
    let pts: Vec<Point> = verbs
        .iter()
        .filter_map(|v| match v {
            PathVerb::MoveTo(p) | PathVerb::LineTo(p) => Some(Point::new(p.x, p.y)),
            PathVerb::QuadTo(_, p) | PathVerb::CubicTo(_, _, p) => Some(Point::new(p.x, p.y)),
            PathVerb::Close => None,
        })
        .collect();
    if pts.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for w in pts.windows(2) {
        sum += w[0].x * w[1].y - w[1].x * w[0].y;
    }
    sum += pts[pts.len() - 1].x * pts[0].y - pts[0].x * pts[pts.len() - 1].y;
    sum / 2.0
}

fn to_kurbo_point(point: GPoint) -> Point {
    Point::new(point.x, point.y)
}

/// i_overlay offset adapter. Alongside `boolean.rs`, the only module allowed
/// to name `i_overlay` types: third-party numerics stay contained here.
mod io_offset_adapter {
    use i_overlay::mesh::outline::offset::OutlineOffset;
    use i_overlay::mesh::style::{LineJoin, OutlineStyle};

    use super::OffsetJoin;

    /// True contour offset of one flattened loop. Positive expands,
    /// negative erodes (despiked). Joins follow `OffsetJoin`.
    pub(super) fn offset_contours(
        flat: &[Vec<[f64; 2]>],
        distance: f64,
        join: OffsetJoin,
    ) -> Vec<Vec<[f64; 2]>> {
        let style = OutlineStyle {
            outer_offset: distance,
            inner_offset: distance,
            join: match join {
                OffsetJoin::Miter => LineJoin::Miter(0.35),
                OffsetJoin::Round => LineJoin::Round(0.5),
                OffsetJoin::Bevel => LineJoin::Bevel,
            },
        };
        flat.outline_as::<i32>(&style)
            .into_iter()
            .flat_map(|shape| shape.into_iter())
            .map(|contour| contour.into_iter().map(|p| [p[0], p[1]]).collect())
            .collect()
    }
}

fn to_kurbo(path: &GPath) -> BezPath {
    let mut bez = BezPath::new();
    for verb in &path.verbs {
        match *verb {
            PathVerb::MoveTo(p) => bez.move_to(to_kurbo_point(p)),
            PathVerb::LineTo(p) => bez.line_to(to_kurbo_point(p)),
            PathVerb::QuadTo(c, p) => bez.quad_to(to_kurbo_point(c), to_kurbo_point(p)),
            PathVerb::CubicTo(c1, c2, p) => {
                bez.curve_to(to_kurbo_point(c1), to_kurbo_point(c2), to_kurbo_point(p));
            }
            PathVerb::Close => bez.close_path(),
        }
    }
    bez
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GRect;

    fn rect_path() -> GPath {
        GPath::rect(GRect::new(0.0, 0.0, 100.0, 60.0), 0.0, 0.0)
    }

    #[test]
    fn expand_grows_bounds_symmetrically() {
        let out =
            offset_path(&rect_path(), 10.0, OffsetJoin::Miter, OffsetCap::None).expect("offset");
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.x0 - -10.0).abs() < 0.5, "got {bounds:?}");
        assert!((bounds.y0 - -10.0).abs() < 0.5, "got {bounds:?}");
        assert!((bounds.width() - 120.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.height() - 80.0).abs() < 1.0, "got {bounds:?}");
    }

    #[test]
    fn inset_shrinks_bounds_symmetrically() {
        let out =
            offset_path(&rect_path(), -10.0, OffsetJoin::Miter, OffsetCap::None).expect("offset");
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.x0 - 10.0).abs() < 0.5, "got {bounds:?}");
        assert!((bounds.width() - 80.0).abs() < 1.0, "got {bounds:?}");
    }

    #[test]
    fn zero_distance_clones() {
        let out =
            offset_path(&rect_path(), 0.0, OffsetJoin::Round, OffsetCap::None).expect("offset");
        assert_eq!(out.verbs.len(), rect_path().verbs.len());
    }

    #[test]
    fn collapse_returns_none() {
        assert!(offset_path(&rect_path(), -1000.0, OffsetJoin::Round, OffsetCap::None).is_none());
        assert!(offset_path(&GPath::new(), 5.0, OffsetJoin::Round, OffsetCap::None).is_none());
    }

    #[test]
    fn open_path_offsets_to_closed_band() {
        let mut line = GPath::new();
        line.push(PathVerb::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
        line.push(PathVerb::LineTo(GPoint::new(100.0, 0.0)))
            .unwrap();
        let out = offset_path(&line, 5.0, OffsetJoin::Round, OffsetCap::Round).expect("offset");
        let bounds = out.bounding_box().expect("bounds");
        assert!(bounds.height() >= 9.0, "got {bounds:?}");
        assert!(bounds.width() >= 100.0, "got {bounds:?}");
    }
}
