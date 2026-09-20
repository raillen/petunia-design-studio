//! Canvas viewport, camera math, snapping, and overlay descriptors (08.6, 08.27).

pub mod camera;
pub mod overlay;
pub mod snapping;

pub use camera::{ViewportCamera, MAX_ZOOM, MIN_ZOOM};
pub use overlay::{
    compute_selection_handles, CanvasOverlays, SelectionHandle, SelectionHandleKind,
};
pub use snapping::{SnapConfig, SnapEngine, SnapGuideVisual, SnapOrientation, SnapResult};
