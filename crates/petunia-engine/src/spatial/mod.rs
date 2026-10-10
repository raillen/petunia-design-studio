//! Spatial queries: indexed broad phase, exact narrow phase,
//! deterministic snapping and structured measurement.
//!
//! The index accelerates queries; it never replaces the scene graph
//! or authorial geometry. Paint order always derives from scene
//! traversal.

pub mod hit_test;
pub mod index;
pub mod measurement;
pub mod snap;

pub use hit_test::{
    entries_for_snapshot, entry_for, hit_test, hit_test_with_snapshot, hit_test_with_view,
    paint_order, paint_order_on_page, Hit, HitTestMode, HitTestRequest,
};
pub use index::{LinearIndex, RStarIndex, SpatialEntry, SpatialIndex};
pub use measurement::{measure_points, Measurement};
pub use snap::{
    affine_grid_candidates, bounds_candidates, guide_candidates, rank_candidates, solve,
    GuideVisual, MovingGeometry, SnapCandidate, SnapConstraint, SnapLatch, SnapMatch, SnapPriority,
    SnapProvider, SnapRequest, SnapResult, SnapSettings, SnapSourceId, SnapTarget, TransformDelta,
};
