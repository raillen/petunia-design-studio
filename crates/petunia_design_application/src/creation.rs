//! Object-creation command builders shared by tools, panels and MCP (F-01).
//!
//! Every creation gesture produces the same ordered command list
//! (`CreateObject` + `SetBounds` + `SetShape` + appearance), submitted
//! atomically via `DocumentSession::transact` so one gesture is exactly one
//! undo entry.

use petunia_design_document::ShapeKind;
use petunia_design_foundation::{ObjectId, SurfaceId};

use super::commands::Command;

/// Builds the canonical command list for creating a shaped object.
#[must_use]
pub fn create_shape_commands(
    surface: SurfaceId,
    id: ObjectId,
    name: impl Into<String>,
    shape: ShapeKind,
    bounds: Option<[f64; 4]>,
    fill: Option<String>,
    stroke: Option<(String, f64)>,
) -> Vec<Command> {
    let mut cmds = Vec::with_capacity(4);
    cmds.push(Command::CreateObject {
        surface,
        id,
        name: name.into(),
    });
    cmds.push(Command::SetBounds {
        id,
        bounds,
        rotation: 0.0,
    });
    cmds.push(Command::SetShape {
        id,
        shape: Some(shape),
    });
    cmds.push(Command::SetFill { id, fill });
    if let Some((stroke, width)) = stroke {
        cmds.push(Command::SetStroke {
            id,
            stroke: Some(stroke),
            width,
        });
    }
    cmds
}

/// Builds the canonical command list for creating an artboard surface.
#[must_use]
pub fn create_artboard_commands(
    surface: SurfaceId,
    origin: [f64; 2],
    dimensions: [f64; 2],
) -> Vec<Command> {
    vec![
        Command::CreateSurface {
            id: surface,
            name: format!("Artboard {}", surface.raw()),
        },
        Command::SetSurfaceGeometry {
            surface,
            origin,
            dimensions,
        },
    ]
}
