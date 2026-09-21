//! Surface-level command builders: crop and artboard geometry (10.7, Table B).
//!
//! Pure builders (no document access). Callers submit through the command
//! lane, atomically when combined with sibling commands.

use petunia_design_foundation::{ObjectId, SurfaceId};

use super::commands::Command;

/// Minimum crop dimension in document points.
pub const MIN_CROP_SIZE: f64 = 10.0;

/// Builds crop commands clamping a drag rectangle to a surface geometry update.
/// Normalizes corner order and enforces [`MIN_CROP_SIZE`].
#[must_use]
pub fn crop_commands(
    surface: SurfaceId,
    p0: [f64; 2],
    p1: [f64; 2],
) -> Vec<Command> {
    let x = p0[0].min(p1[0]);
    let y = p0[1].min(p1[1]);
    let w = (p1[0] - p0[0]).abs().max(MIN_CROP_SIZE);
    let h = (p1[1] - p0[1]).abs().max(MIN_CROP_SIZE);
    vec![Command::SetSurfaceGeometry {
        surface,
        origin: [x, y],
        dimensions: [w, h],
    }]
}

/// Builds a move-object-to-surface command preserving world transform.
#[must_use]
pub fn move_to_surface_commands(id: ObjectId, target_surface: SurfaceId) -> Vec<Command> {
    vec![Command::MoveObjectToSurface {
        id,
        target_surface,
        preserve_world_transform: true,
    }]
}
