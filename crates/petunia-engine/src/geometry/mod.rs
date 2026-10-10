//! Geometry kernels: Bézier, bounds, intersections, booleans,
//! offsetting and simplification.
//!
//! Everything here consumes flattened or analytic Petunia geometry
//! and returns owned values; no external engine type crosses the
//! module boundary except inside the boolean adapter.

pub mod bezier;
pub mod boolean;
pub mod bounds;
pub mod fit;
pub mod intersections;
pub mod offset;
pub mod shape_builder;
pub mod simplify;
pub mod smart_delete;

pub use bezier::{flatten_contour, segment_bezier, CubicBez};
pub use boolean::{boolean_paths, boolean_rings, ring_bounds, BooleanOp};
pub use bounds::{point_in_polygon, Bounds};
pub use fit::{
    end_tangent, fit_cubics, g1_continuous, is_straight, refit_contour, sample_uniform,
    start_tangent,
};
pub use intersections::{
    orient, polyline_intersections, segments_intersect, Orientation, SegmentIntersection,
};
pub use offset::offset_ring;
pub use shape_builder::{BuilderShape, BuiltRegion, ShapeBuilder};
pub use simplify::{simplify_closed, simplify_open};
pub use smart_delete::{delete_node, SmartDeleteMode, SmartDeleteOutcome, SmartDeleteRefusal};
