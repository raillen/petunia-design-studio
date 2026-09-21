#![forbid(unsafe_code)]

//! Petunia geometry API: owned semantic types with adapter-contained
//! third-party numerics (Kurbo paths, i_overlay booleans).
//!
//! Public signatures never expose `kurbo` or `i_overlay` types. A dependency
//! replacement affects only the adapter modules, not the product.

mod affine;
mod boolean;
pub mod measure;
mod node_edit;
mod path;
mod point;
mod rect;
mod smooth;

pub use affine::{pivot_angle_delta, rotate_point_around, scale_bounds_about, scale_factor_around, GAffine};
pub use boolean::{boolean_op, boolean_op_with_fill, BooleanInput, BooleanOp, FillRule, GeometryTolerance};
pub use measure::{measure_readout, MeasurementReadout};
pub use node_edit::{move_verb, move_verb_to};
pub use path::{GPath, PathVerb};
pub use point::GPoint;
pub use rect::{resize_rect_from_handle, step_corner_radius, MIN_RESIZE_SIZE, ResizeHandle, GRect};
pub use smooth::{anchors_to_path, chaikin_smooth, fit_midpoint_quads, midpoint, simplify_rdp, smooth_samples};
