#![forbid(unsafe_code)]

//! Aubrieta geometry API: owned semantic types with adapter-contained
//! third-party numerics (Kurbo paths, i_overlay booleans).
//!
//! Public signatures never expose `kurbo` or `i_overlay` types. A dependency
//! replacement affects only the adapter modules, not the product.

mod affine;
mod boolean;
mod path;
mod point;
mod rect;
mod smooth;

pub use affine::GAffine;
pub use boolean::{boolean_op, boolean_op_with_fill, BooleanInput, BooleanOp, FillRule, GeometryTolerance};
pub use path::{GPath, PathVerb};
pub use point::GPoint;
pub use rect::GRect;
pub use smooth::{anchors_to_path, chaikin_smooth, fit_midpoint_quads, midpoint, simplify_rdp, smooth_samples};
