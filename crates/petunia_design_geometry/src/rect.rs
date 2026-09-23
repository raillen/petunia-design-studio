//! Semantic axis-aligned rectangle with exact union/intersection math.

use serde::{Deserialize, Serialize};

use crate::GPoint;

/// Axis-aligned rectangle, normalized so `x0 <= x1` and `y0 <= y1`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GRect {
    /// Left edge.
    pub x0: f64,
    /// Top edge.
    pub y0: f64,
    /// Right edge.
    pub x1: f64,
    /// Bottom edge.
    pub y1: f64,
}

impl GRect {
    /// Creates a rect, normalizing inverted edges.
    #[must_use]
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    /// Zero-area rect at the origin.
    pub const ZERO: Self = Self {
        x0: 0.0,
        y0: 0.0,
        x1: 0.0,
        y1: 0.0,
    };

    /// Width (`x1 - x0`).
    #[must_use]
    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    /// Height (`y1 - y0`).
    #[must_use]
    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }

    /// Area. Zero when degenerate.
    #[must_use]
    pub fn area(self) -> f64 {
        (self.width().max(0.0)) * (self.height().max(0.0))
    }

    /// True when the point lies inside (edges inclusive).
    #[must_use]
    pub fn contains(self, point: GPoint) -> bool {
        point.x >= self.x0 && point.x <= self.x1 && point.y >= self.y0 && point.y <= self.y1
    }

    /// Smallest rect containing both, or `None` when either is non-finite.
    #[must_use]
    pub fn union(self, other: Self) -> Option<Self> {
        if !self.is_finite() || !other.is_finite() {
            return None;
        }
        Some(Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        })
    }

    /// Overlap region, or `None` when disjoint.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let rect = Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };
        if rect.x1 < rect.x0 || rect.y1 < rect.y0 {
            return None;
        }
        Some(rect)
    }

    /// True when all edges are finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x0.is_finite() && self.y0.is_finite() && self.x1.is_finite() && self.y1.is_finite()
    }
}

/// Toolkit-neutral resize handle (Table B).
/// The shell maps `SelectionHandleKind` onto this; rotation and node
/// affordances are not resize handles and map to nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

/// Minimum width/height in document points after a resize drag.
pub const MIN_RESIZE_SIZE: f64 = 5.0;

/// Legacy corner-radius clamp upper bound used when no bounds are known.
pub const MAX_CORNER_RADIUS_FALLBACK: f64 = 100.0;

/// Steps a corner radius by `delta`, clamped to the physical limit
/// `min(w,h)/2` when bounds are known (Table B). Non-finite inputs pass
/// `current` through; without bounds the legacy `0..100` clamp applies.
#[must_use]
pub fn step_corner_radius(current: f64, delta: f64, bounds: Option<[f64; 4]>) -> f64 {
    if !current.is_finite() || !delta.is_finite() {
        return current;
    }
    let limit = match bounds {
        Some([_, _, w, h]) if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 => {
            w.min(h) / 2.0
        }
        _ => MAX_CORNER_RADIUS_FALLBACK,
    };
    (current + delta).clamp(0.0, limit)
}

/// Resizes `[x, y, w, h]` bounds by a `(dx, dy)` drag on `handle` (Table B).
/// Dimensions clamp to [`MIN_RESIZE_SIZE`]; pass-through for callers that
/// need from-center/constrain policies (handled at the tool layer).
#[must_use]
pub fn resize_rect_from_handle(
    handle: ResizeHandle,
    initial: [f64; 4],
    dx: f64,
    dy: f64,
) -> (f64, f64, f64, f64) {
    let [ix, iy, iw, ih] = initial;
    let mut nx = ix;
    let mut ny = iy;
    let mut nw = iw;
    let mut nh = ih;

    match handle {
        ResizeHandle::TopLeft => {
            nx += dx;
            ny += dy;
            nw -= dx;
            nh -= dy;
        }
        ResizeHandle::Top => {
            ny += dy;
            nh -= dy;
        }
        ResizeHandle::TopRight => {
            ny += dy;
            nw += dx;
            nh -= dy;
        }
        ResizeHandle::Right => {
            nw += dx;
        }
        ResizeHandle::BottomRight => {
            nw += dx;
            nh += dy;
        }
        ResizeHandle::Bottom => {
            nh += dy;
        }
        ResizeHandle::BottomLeft => {
            nx += dx;
            nw -= dx;
            nh += dy;
        }
        ResizeHandle::Left => {
            nx += dx;
            nw -= dx;
        }
    }

    if nw < MIN_RESIZE_SIZE {
        nw = MIN_RESIZE_SIZE;
    }
    if nh < MIN_RESIZE_SIZE {
        nh = MIN_RESIZE_SIZE;
    }

    (nx, ny, nw, nh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_covers_both() {
        let rect = GRect::new(0.0, 0.0, 2.0, 2.0)
            .union(GRect::new(1.0, 1.0, 4.0, 3.0))
            .expect("union");
        assert_eq!(rect, GRect::new(0.0, 0.0, 4.0, 3.0));
    }

    #[test]
    fn disjoint_intersection_is_none() {
        assert!(GRect::new(0.0, 0.0, 1.0, 1.0)
            .intersection(GRect::new(2.0, 2.0, 3.0, 3.0))
            .is_none());
    }
}
