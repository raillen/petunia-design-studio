//! Canvas viewport, camera math, snapping, and overlay descriptors (08.6, 08.27).

pub mod overlay;
pub mod snapping;

// The camera is GUI-agnostic view state, so it lives in the application layer
// (15.B) and the shell only renders it.
pub use overlay::{
    compute_selection_handles, hit_test_handle_or_border, CanvasOverlays, SelectionHandle,
    SelectionHandleKind,
};
pub use petunia_design_application::view_camera::{ViewportCamera, MAX_ZOOM, MIN_ZOOM};
pub use snapping::{SnapConfig, SnapEngine, SnapGuideVisual, SnapOrientation, SnapResult};
